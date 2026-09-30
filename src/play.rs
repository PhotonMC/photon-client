//! `voxel-client play`: join a server and walk around in it.

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::PlaySession;
use voxel_render::VulkanBackend;
use voxel_render::gui::screen::creative::CreativeAssets;
use voxel_render::gui::screen::{KeyMods, KeyResult};
use voxel_render::window::{DisplayData, WindowSurface};

use crate::assets::FolderAssets;
use crate::atmosphere::{rain_fog_target, weather};
use crate::hud_input::{attack_speed, hud_input};
use crate::item_layers::ItemLayerCache;
use crate::ppm;
use crate::profile::FrameProfile;
use crate::screens::{
    OpenScreen, ScreenAssets, WindowClipboard, close_inventory, open_inventory, run_actions,
};
use crate::script::{HeldKeys, Script};
use crate::sections::SectionMeshes;
use crate::view::{CLOUD_RANGE, CameraType, eye_position, max_camera_zoom, pick, player_camera};

/// Join a server and walk around in it.
///
/// The player is the ported `LocalPlayer`: its tick runs twenty times a
/// second against the world the server streams, and what it sends is what
/// the reference client would send. WASD moves, space jumps, left shift
/// sneaks, left control sprints, the mouse looks, and escape frees the
/// cursor (a second escape closes the window).
pub fn run_play(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    let server: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25565".to_owned())
        .parse()?;
    let width: i32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(1280);
    let height: i32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(720);
    let root = env::var("VOXEL_ASSETS").map_err(|_| "VOXEL_ASSETS must name the asset folder")?;
    let username = env::var("VOXEL_USER").unwrap_or_else(|_| "VoxelClient".to_owned());
    let auto_jump = env::var("VOXEL_AUTO_JUMP").is_ok_and(|value| value == "1");
    let mut script = env::var("VOXEL_SCRIPT")
        .ok()
        .map(|text| Script::parse(&text))
        .transpose()?;

    let mut session = join(server, &username, auto_jump)?;
    let position = session.player().ok_or("no local player")?.position();
    println!(
        "play_spawn={:.3},{:.3},{:.3}",
        position.x, position.y, position.z
    );
    println!("play_chunks={}", session.world().loaded_chunk_count());
    let spawn_biome = session
        .world()
        .biome_at(
            position.x.floor() as i32,
            position.y.floor() as i32,
            position.z.floor() as i32,
        )
        .and_then(|id| {
            session
                .registry_names("minecraft:worldgen/biome")
                .get(usize::try_from(id).ok()?)
        })
        .cloned()
        .unwrap_or_default();
    println!("play_biome={spawn_biome}");

    let display = DisplayData {
        width,
        height,
        fullscreen: false,
    };
    let mut surface = WindowSurface::create(display, "Minecraft")?;
    let scale = surface.scale();
    let (display_handle, window_handle) = surface
        .raw_handles()
        .map_err(|error| format!("window handles unavailable: {error}"))?;
    let mut backend = unsafe {
        VulkanBackend::create(
            display_handle,
            window_handle,
            scale.framebuffer_width() as u32,
            scale.framebuffer_height() as u32,
        )
    }?;
    // `VOXEL_FULLSCREEN` starts fullscreen, as F11 would switch.
    if env::var_os("VOXEL_FULLSCREEN").is_some() {
        surface.toggle_fullscreen();
    }
    // The vanilla data pack the server runs, which sits next to the assets:
    // the biomes' colours and the sky's data come from it.
    let data_root = env::var("VOXEL_DATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::Path::new(&root).with_file_name("data"));
    let biome_tints = voxel_render::biome_tint::BiomeTints::load(
        &data_root,
        std::path::Path::new(&root),
        session.registry_names("minecraft:worldgen/biome"),
        session.hashed_seed(),
    )
    .ok_or("the biome colours could not be read")?;
    let biome_tints = std::sync::Arc::new(biome_tints);
    // Every block's models and one atlas for all of them, as the reference
    // bakes them at resource load; the world is then meshed a 16-block
    // section at a time as its chunks arrive and change.
    let source = FolderAssets {
        root: std::path::PathBuf::from(&root),
    };
    let requested_mips: i32 = env::var("VOXEL_MIP_LEVELS")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(4);
    let baking = std::time::Instant::now();
    let block_renderer = voxel_render::world_mesh::BlockRenderer::new(&source, requested_mips)
        .map_err(|error| format!("block models: {error:?}"))?;
    let block_renderer = std::sync::Arc::new(block_renderer);
    println!(
        "play_atlas={}x{} models={} skipped={} in {:?}",
        block_renderer.atlas.width,
        block_renderer.atlas.height,
        block_renderer.model_count(),
        block_renderer.skipped.len(),
        baking.elapsed()
    );
    backend.load_atlas(&block_renderer.atlas.levels)?;
    // Animated sprites tick with the client and redraw their slots of a
    // copy of the atlas, which then goes to the GPU.
    let mut atlas_levels = block_renderer.atlas.levels.clone();
    let mut animations = block_renderer.atlas.animations.clone();
    let mut animation_redraw = vec![false; animations.len()];
    let sprite = |name: &str| {
        block_renderer
            .atlas
            .sprites
            .get(name)
            .copied()
            .unwrap_or([0.0; 4])
    };
    let moon_names: Vec<String> = voxel_render::sky::moon_sprites().collect();
    let sky_sprites = voxel_render::sky::SkySprites {
        sun: sprite(voxel_render::sky::SUN_SPRITE),
        moon: std::array::from_fn(|phase| sprite(&moon_names[phase])),
    };
    // Meshes are measured from a fixed point near where the player started.
    let origin = [
        position.x.floor() as i32,
        position.y.floor() as i32,
        position.z.floor() as i32,
    ];
    // The chunk fade option, 0.75 seconds by default and `VOXEL_CHUNK_FADE`
    // seconds if given; a scripted run shows every section at once unless
    // asked, so its captures do not catch one mid-fade.
    let chunk_fade = match env::var("VOXEL_CHUNK_FADE") {
        Ok(seconds) => Duration::from_secs_f64(seconds.parse::<f64>()?.clamp(0.0, 2.0)),
        Err(_) if script.is_some() => Duration::ZERO,
        Err(_) => Duration::from_millis(750),
    };
    let mut sections = SectionMeshes::new(
        &mut session,
        std::sync::Arc::clone(&block_renderer),
        std::sync::Arc::clone(&biome_tints),
        chunk_fade,
    );

    let environment = voxel_render::sky::Environment::load(
        &data_root,
        session.dimension_type().unwrap_or("minecraft:overworld"),
        session.registry_names("minecraft:worldgen/biome"),
    )
    .ok_or("the dimension's sky data could not be read")?;
    let stars = voxel_render::sky::build_stars();
    let dimension = session.world().dimension();
    let probe = |session: &PlaySession, eye: [f64; 3]| {
        environment.evaluate(
            eye,
            // The dimension the session joined in, even after a respawn.
            |x, y, z| {
                let y = dimension.clamp_quart_y(y);
                session.world().biome_at(x * 4, y * 4, z * 4)
            },
            |clock| session.clock_ticks(clock).unwrap_or(0),
            voxel_render::sky::Weather {
                rain: session.rain_level(),
                thunder: session.thunder_level(),
            },
        )
    };
    let eye_of = |session: &PlaySession| {
        let player = session.player().expect("a loaded session has a player");
        let feet = player.position();
        [feet.x, feet.y + f64::from(player.eye_height()), feet.z]
    };
    let mut attributes_last = probe(&session, eye_of(&session));
    let cloud_image = texture(&root, "environment/clouds", "clouds")?;
    let cloud_cells = voxel_render::clouds::CloudCells::from_image(&cloud_image);
    backend.load_weather_textures(
        &texture(&root, "environment/rain", "rain")?,
        &texture(&root, "environment/snow", "snow")?,
    )?;
    backend.load_crack_texture(&destroy_stages(&root)?)?;
    // The player's default skin, picked by its profile id as the reference
    // picks one when the profile has no skin of its own.
    let skin = voxel_render::player_renderer::default_skin(
        session
            .profile_id()
            .unwrap_or(voxel_core::binary::Uuid { most: 0, least: 0 }),
    );
    backend.load_entity_texture(&texture(&root, skin.texture, "skin")?)?;
    let mut player_model = voxel_render::player_model::PlayerModel::new(skin.slim);
    // The HUD: its sprites, its own state, and the wheel's partial steps.
    let font = std::sync::Arc::new(
        voxel_render::gui::font::Font::load(std::path::Path::new(&root))
            .ok_or("the default font could not be read")?,
    );
    let gui_atlas = voxel_render::gui::GuiAtlas::load(std::path::Path::new(&root), &font)
        .ok_or("the HUD sprites could not be read")?;
    backend.load_gui_texture(&gui_atlas.image)?;
    let mut hud = voxel_render::gui::Hud::default();
    // Each item's GUI layers, built the first time a slot shows it.
    let mut gui_item_layers = ItemLayerCache::default();
    let mut scroll_wheel = voxel_game::hotbar::ScrollWheel::default();
    let started_at = std::time::Instant::now();
    // `ItemInHandRenderer`: the stacks the hands show and how high they are.
    let mut item_in_hand = voxel_game::item_in_hand::ItemInHand::default();
    // Each item's first-person layers, by item and hand.
    let mut held_item_layers = [ItemLayerCache::default(), ItemLayerCache::default()];
    // Attack clicks waiting for the next tick, and `Minecraft.missTime`:
    // ticks a missed attack keeps the next one waiting.
    let mut attack_key = voxel_game::attack::AttackKey::default();
    // The use key keeps its clicks and held state the same way.
    let mut use_key = voxel_game::attack::AttackKey::default();
    let mut cloud_key = None;
    // The reference seeds this source uniquely per run.
    let mut flicker_random = voxel_core::random::RandomSource::create(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos() as i64)
            .unwrap_or(0),
    );
    let mut block_light_flicker = 0.0_f32;
    let mut rain_fog = 0.0_f32;
    let capture_frames: u32 = env::var("VOXEL_CAPTURE_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let mut captured_frames = 0;
    let mut attributes_next = attributes_last.clone();

    let mut aspect = scale.framebuffer_width() as f32 / scale.framebuffer_height() as f32;
    let mut sized_for = (
        scale.framebuffer_width() as u32,
        scale.framebuffer_height() as u32,
    );
    let mut keys = HeldKeys::default();
    let mut camera = voxel_render::camera::Camera::new();
    // F5 cycles first person, third person from behind and from the front.
    // `VOXEL_CAMERA=back` or `front` starts detached.
    let mut camera_type = match env::var("VOXEL_CAMERA").as_deref() {
        Ok("back") => CameraType::ThirdPersonBack,
        Ok("front") => CameraType::ThirdPersonFront,
        _ => CameraType::FirstPerson,
    };
    let mut captured = script.is_none();
    surface.set_cursor_captured(captured);
    // The open inventory screen, the mouse in GUI units, and the button a
    // drag is held with (`MouseHandler.activeButton`).
    let mut inventory_screen: OpenScreen = None;
    let mut creative_tab = voxel_game::creative_menu::default_tab();
    let mut gui_mouse = (0.0_f64, 0.0_f64);
    let mut active_button: Option<i32> = None;
    let lang = std::sync::Arc::new(voxel_game::text::Lang::load(std::path::Path::new(&root)));
    let screen_assets = ScreenAssets {
        creative: CreativeAssets {
            trees: std::sync::Arc::new(voxel_game::creative_menu::SearchTrees::new(lang.clone())),
            font: font.clone(),
            titles: voxel_data::item::creative_tabs::CREATIVE_TABS
                .iter()
                .map(|tab| lang.text(tab.title))
                .collect(),
        },
        inventory_title: lang.text("container.crafting"),
    };
    let mut last_cursor: Option<(f64, f64)> = None;
    let tick_length = Duration::from_millis(50);
    let mut accumulated = Duration::ZERO;
    let mut last_frame = std::time::Instant::now();
    let mut ticks = 0_u64;
    let mut frames = 0_u64;
    let mut profile = FrameProfile::from_env();

    while !surface.should_close() {
        let frame_start = std::time::Instant::now();
        for event in surface.poll() {
            if let Some(screen) = inventory_screen.as_mut() {
                let size = surface.scale();
                screen.resize(size.gui_scaled_width(), size.gui_scaled_height());
            }
            match event {
                // An open screen takes the keys and the mouse.
                glfw::WindowEvent::Key(
                    key,
                    _,
                    glfw::Action::Press | glfw::Action::Repeat,
                    modifiers,
                ) if inventory_screen.is_some() => {
                    let menu = session.inventory_menu().clone();
                    let screen = inventory_screen.as_mut().expect("checked above");
                    let mods = KeyMods {
                        control: modifiers.contains(glfw::Modifiers::Control),
                        shift: modifiers.contains(glfw::Modifiers::Shift),
                        command: modifiers.contains(glfw::Modifiers::Super),
                    };
                    match screen.key_pressed(key, mods, &menu, &mut WindowClipboard(&mut surface)) {
                        KeyResult::Close => {
                            close_inventory(
                                &mut session,
                                &mut surface,
                                &mut inventory_screen,
                                &mut creative_tab,
                            )?;
                            captured = true;
                            last_cursor = None;
                        }
                        KeyResult::Actions(actions) => {
                            run_actions(&mut session, &mut inventory_screen, actions)?;
                        }
                    }
                }
                glfw::WindowEvent::Key(..) if inventory_screen.is_some() => {
                    inventory_screen
                        .as_mut()
                        .expect("checked above")
                        .key_released();
                }
                glfw::WindowEvent::Char(character) if inventory_screen.is_some() => {
                    inventory_screen
                        .as_mut()
                        .expect("checked above")
                        .char_typed(character);
                }
                glfw::WindowEvent::MouseButton(button, action, modifiers)
                    if inventory_screen.is_some() =>
                {
                    let button = button as i32;
                    let shift = modifiers.contains(glfw::Modifiers::Shift);
                    let menu = session.inventory_menu().clone();
                    let screen = inventory_screen.as_mut().expect("checked above");
                    let actions = if action == glfw::Action::Press {
                        active_button = Some(button);
                        let now = std::time::Instant::now();
                        screen.mouse_clicked(gui_mouse, button, shift, now, &menu)
                    } else {
                        active_button = None;
                        screen.mouse_released(gui_mouse, button, shift, &menu)
                    };
                    run_actions(&mut session, &mut inventory_screen, actions)?;
                }
                glfw::WindowEvent::Scroll(_, y) if inventory_screen.is_some() => {
                    inventory_screen
                        .as_mut()
                        .expect("checked above")
                        .mouse_scrolled(y);
                }
                glfw::WindowEvent::Key(glfw::Key::E, _, glfw::Action::Press, _)
                    if captured && !session.player().is_some_and(|player| player.spectating()) =>
                {
                    open_inventory(
                        &mut surface,
                        &mut inventory_screen,
                        session.game_mode() == voxel_world::ClientPlayerGameMode::Creative,
                        creative_tab,
                        &screen_assets,
                    );
                    captured = false;
                    keys = HeldKeys::default();
                    attack_key.release();
                    use_key.release();
                }
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    if captured {
                        captured = false;
                        surface.set_cursor_captured(false);
                        keys = HeldKeys::default();
                        attack_key.release();
                        use_key.release();
                    } else {
                        surface.request_close();
                    }
                }
                glfw::WindowEvent::Key(glfw::Key::F5, _, glfw::Action::Press, _) => {
                    camera_type = camera_type.next();
                }
                glfw::WindowEvent::Key(glfw::Key::F11, _, glfw::Action::Press, _) => {
                    surface.toggle_fullscreen();
                }
                glfw::WindowEvent::Scroll(x, y) if captured => {
                    let spectating = session.player().is_some_and(|player| player.spectating());
                    if !spectating
                        && let Some(slot) =
                            scroll_wheel.select(x, y, session.status().selected_slot)
                    {
                        session.select_slot(slot);
                    }
                }
                glfw::WindowEvent::Key(key, _, glfw::Action::Press, _)
                    if captured && hotbar_key(key).is_some() =>
                {
                    let spectating = session.player().is_some_and(|player| player.spectating());
                    if !spectating && let Some(slot) = hotbar_key(key) {
                        session.select_slot(slot);
                    }
                }
                glfw::WindowEvent::Key(key, _, action, _) if captured => {
                    keys.set(key, action != glfw::Action::Release);
                }
                glfw::WindowEvent::MouseButton(
                    glfw::MouseButton::Button1,
                    glfw::Action::Press,
                    _,
                ) if captured => {
                    attack_key.press();
                }
                glfw::WindowEvent::MouseButton(
                    glfw::MouseButton::Button1,
                    glfw::Action::Release,
                    _,
                ) => {
                    attack_key.release();
                }
                glfw::WindowEvent::MouseButton(
                    glfw::MouseButton::Button2,
                    glfw::Action::Press,
                    _,
                ) if captured => {
                    use_key.press();
                }
                glfw::WindowEvent::MouseButton(
                    glfw::MouseButton::Button2,
                    glfw::Action::Release,
                    _,
                ) => {
                    use_key.release();
                }
                glfw::WindowEvent::MouseButton(_, glfw::Action::Press, _) if !captured => {
                    captured = true;
                    last_cursor = None;
                    surface.set_cursor_captured(true);
                }
                glfw::WindowEvent::CursorPos(x, y) if inventory_screen.is_some() => {
                    // `MouseHandler.onMove`: screen coordinates to GUI units.
                    let size = surface.scale();
                    let (width, height) = surface.window_size();
                    gui_mouse = (
                        x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
                        y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
                    );
                    if active_button.is_some() {
                        let menu = session.inventory_menu().clone();
                        let screen = inventory_screen.as_mut().expect("checked above");
                        screen.mouse_dragged(gui_mouse, &menu);
                    }
                    last_cursor = None;
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    if captured
                        && let Some((last_x, last_y)) = last_cursor
                        && let Some(player) = session.player_mut()
                    {
                        // The default sensitivity, 0.5, makes the scale exactly one.
                        let sensitivity = 0.5 * 0.6 + 0.2_f64;
                        let scaled = sensitivity * sensitivity * sensitivity * 8.0;
                        player.turn((x - last_x) * scaled, (y - last_y) * scaled);
                    }
                    last_cursor = Some((x, y));
                }
                _ => {}
            }
        }

        // A resized or fullscreen window gets a swapchain of its new size.
        let size = surface.scale();
        let wanted = (
            size.framebuffer_width() as u32,
            size.framebuffer_height() as u32,
        );
        if (backend.out_of_date() || wanted != sized_for) && wanted.0 > 0 && wanted.1 > 0 {
            sized_for = wanted;
            backend.resize(wanted.0, wanted.1)?;
            let (width, height) = backend.extent();
            aspect = width as f32 / height as f32;
        }

        let now = std::time::Instant::now();
        let frame_ticks = (now - last_frame).as_secs_f32() / tick_length.as_secs_f32();
        accumulated += now - last_frame;
        last_frame = now;
        session.drain()?;
        // `containerTick`: each inventory screen hands over to the other when
        // the game mode changes under it. Neither closes the container.
        if let Some(open) = inventory_screen.as_mut() {
            let creative = session.game_mode() == voxel_world::ClientPlayerGameMode::Creative;
            let size = surface.scale();
            let (width, height) = (size.gui_scaled_width(), size.gui_scaled_height());
            let open_tab = open.as_creative_mut().map(|open| open.tab());
            if open_tab.is_some() != creative {
                creative_tab = open_tab.unwrap_or(creative_tab);
                inventory_screen =
                    Some(screen_assets.screen(creative, width, height, creative_tab));
            }
        }
        // A respawn or a change of dimension: nothing of the old level stays.
        if session.take_level_reset() {
            backend.clear_sections()?;
            sections.clear();
            cloud_key = None;
        }
        let mut script_done = false;
        while accumulated >= tick_length {
            accumulated -= tick_length;
            let held = match script.as_mut() {
                Some(script) => match script.next() {
                    Some((held, turn, pitch)) => {
                        if (turn != 0.0 || pitch != 0.0)
                            && let Some(player) = session.player_mut()
                        {
                            let (y_rot, x_rot) = player.rotation();
                            player.set_rotation(y_rot + turn as f32, x_rot + pitch as f32);
                        }
                        held
                    }
                    None => {
                        script_done = true;
                        HeldKeys::default()
                    }
                },
                None => keys,
            };
            // A scripted `attack` is a click on the tick; `mine` holds the key.
            // Once the script ends the key stays as its last step left it,
            // so a capture can show a block mid-break.
            if script.is_some() && !script_done {
                if held.inventory {
                    if inventory_screen.is_some() {
                        close_inventory(
                            &mut session,
                            &mut surface,
                            &mut inventory_screen,
                            &mut creative_tab,
                        )?;
                        surface.set_cursor_captured(false);
                    } else {
                        open_inventory(
                            &mut surface,
                            &mut inventory_screen,
                            session.game_mode() == voxel_world::ClientPlayerGameMode::Creative,
                            creative_tab,
                            &screen_assets,
                        );
                    }
                }
                if let Some((slot, button, kind)) = held.click {
                    let kind = [
                        voxel_game::inventory_menu::ClickType::Pickup,
                        voxel_game::inventory_menu::ClickType::QuickMove,
                        voxel_game::inventory_menu::ClickType::Swap,
                        voxel_game::inventory_menu::ClickType::Clone,
                        voxel_game::inventory_menu::ClickType::Throw,
                        voxel_game::inventory_menu::ClickType::QuickCraft,
                        voxel_game::inventory_menu::ClickType::PickupAll,
                    ][kind.clamp(0, 6) as usize];
                    match inventory_screen.as_mut() {
                        Some(screen) => {
                            let action = screen.scripted_click(slot, button, kind);
                            run_actions(&mut session, &mut inventory_screen, vec![action])?;
                        }
                        None => session.container_click(slot, button, kind)?,
                    }
                }
                if let Some(mouse) = held.mouse {
                    gui_mouse = mouse;
                }
                if let (Some(text), Some(screen)) = (held.typed, inventory_screen.as_mut()) {
                    for character in text.chars() {
                        screen.char_typed(character);
                    }
                }
                if let Some(screen) = inventory_screen
                    .as_mut()
                    .and_then(|screen| screen.as_creative_mut())
                {
                    if let Some(tab) = held.tab {
                        screen.select_tab_index(tab);
                    }
                    if let Some(scroll) = held.scroll {
                        screen.menu_mut().set_scroll(scroll as f32 / 100.0);
                    }
                }
                if held.attack {
                    attack_key.press();
                }
                if !held.mine {
                    attack_key.release();
                } else if !attack_key.is_down() {
                    attack_key.press();
                    attack_key.take_clicks();
                }
                // `use` is a click on the tick; `hold_use` holds the key.
                if held.use_click {
                    use_key.press();
                }
                if !held.hold_use {
                    use_key.release();
                } else if !use_key.is_down() {
                    use_key.press();
                    use_key.take_clicks();
                }
            }
            // `Minecraft.handleKeybinds`: the attack key is spent before the
            // player moves, aiming the way `GameRenderer.pick` aims.
            let aim = session
                .player()
                .map_or(voxel_game::game_mode::Aim::Miss, |player| {
                    pick(session.world(), player, eye_position(player, 1.0))
                });
            let grabbed = captured || script.is_some();
            session.keys(
                voxel_game::game_mode::Keys {
                    attack_clicks: attack_key.take_clicks(),
                    attack_held: attack_key.is_down() && grabbed,
                    use_clicks: use_key.take_clicks(),
                    use_held: use_key.is_down(),
                },
                aim,
            )?;
            for hand in session.take_items_used() {
                item_in_hand.item_used(hand);
            }
            // `ItemInHandRenderer.tick`, after the keys and before the
            // player ticks: the hands follow the stacks and rise back as the
            // attack strength recovers.
            {
                let world = session.world();
                let local = world.local_player();
                let attack_speed = attack_speed(local);
                let swap = session
                    .player()
                    .map_or(1.0, |player| player.item_swap_scale(1.0, attack_speed));
                let stack = |slot: i16| {
                    local
                        .and_then(|state| state.inventory_slot(slot))
                        .filter(|stack| stack.item_count() > 0)
                };
                let selected = session.status().selected_slot as i16;
                item_in_hand.tick(stack(36 + selected), stack(45), swap, false, &|item| {
                    block_renderer.item_swap(item)
                });
            }
            session.tick(held.input())?;
            // `LightmapRenderStateExtractor.tick`: the block light's flicker
            // wanders by a small random step and decays.
            block_light_flicker += (flicker_random.next_float() - flicker_random.next_float())
                * flicker_random.next_float()
                * flicker_random.next_float()
                * 0.1;
            block_light_flicker *= 0.9;
            for (animation, redraw) in animations.iter_mut().zip(&mut animation_redraw) {
                animation.tick();
                *redraw |= animation.needs_to_draw();
            }
            if let Some(player) = session.player() {
                camera.tick(player);
            }
            hud.tick();
            // The camera's attribute probe steps once a tick.
            if session.player().is_some() {
                attributes_last =
                    std::mem::replace(&mut attributes_next, probe(&session, eye_of(&session)));
            }
            ticks += 1;
        }

        let ticked = std::time::Instant::now();
        // While a respawn waits for its position there is nothing to draw.
        let Some(player) = session.player().filter(|_| session.is_loaded()) else {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        };
        let partial = accumulated.as_secs_f64() / tick_length.as_secs_f64();
        let feet = player.interpolated_position(partial);
        let (player_y_rot, player_x_rot) = player.rotation();
        // `Camera.alignWithEntity`: the front view turns round, and a
        // detached camera backs away from the eye until a block is in the
        // way.
        let (y_rot, x_rot) = if camera_type == CameraType::ThirdPersonFront {
            (player_y_rot + 180.0, -player_x_rot)
        } else {
            (player_y_rot, player_x_rot)
        };
        let eye_world = voxel_core::math::Vec3::new(
            feet.x,
            feet.y + f64::from(camera.eye_height(partial as f32)),
            feet.z,
        );
        let camera_world = if camera_type == CameraType::FirstPerson {
            eye_world
        } else {
            let forwards = voxel_core::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
            let zoom = max_camera_zoom(session.world(), eye_world, forwards, 4.0);
            voxel_core::math::Vec3::new(
                eye_world.x - forwards.x * f64::from(zoom),
                eye_world.y - forwards.y * f64::from(zoom),
                eye_world.z - forwards.z * f64::from(zoom),
            )
        };
        let eye = [
            (camera_world.x - f64::from(origin[0])) as f32,
            (camera_world.y - f64::from(origin[1])) as f32,
            (camera_world.z - f64::from(origin[2])) as f32,
        ];
        // Packed block and sky light at the eyes, which the player's model
        // and hand are lit by.
        let light = {
            let world = session.world();
            let [x, y, z] =
                [eye_world.x, eye_world.y, eye_world.z].map(|value| value.floor() as i32);
            (i32::from(world.block_brightness(x, y, z)) << 4)
                | (i32::from(world.sky_brightness(x, y, z)) << 20)
        };
        // The player's own model shows once the camera leaves its eyes; in
        // them, the empty hand does, unless the player only watches.
        if camera_type == CameraType::FirstPerson && !player.spectating() {
            let (walked, bob) = player.view_bob(partial as f32);
            let (x_bob, y_bob) = player.hand_bob(partial as f32);
            // `submitHandsWithItems`: the swing is the main hand's, since
            // which arm swings is not tracked yet.
            let swap = |item: i32| block_renderer.item_swap(item);
            let main_height = item_in_hand.inverse_arm_height(
                voxel_game::game_mode::MAIN_HAND,
                partial as f32,
                &swap,
            );
            let off_height = item_in_hand.inverse_arm_height(
                voxel_game::game_mode::OFF_HAND,
                partial as f32,
                &swap,
            );
            let view = voxel_render::player_renderer::HandView {
                bob_walked: walked,
                bob,
                view_x_rot: player_x_rot,
                view_y_rot: player_y_rot,
                x_bob,
                y_bob,
                inverse_arm_height: main_height,
                attack: player.attack_anim(partial as f32),
                light,
            };
            let main_item = item_in_hand.item(voxel_game::game_mode::MAIN_HAND);
            let off_item = item_in_hand.item(voxel_game::game_mode::OFF_HAND);
            // An empty main hand shows the arm; each stack is drawn in its
            // hand, the main hand on the right.
            let hand = if main_item.is_none() {
                voxel_render::player_renderer::hand_vertices(&mut player_model, &view)
            } else {
                Vec::new()
            };
            let mut held = Vec::new();
            for (stack, right, attack, height) in [
                (main_item, true, view.attack, main_height),
                (off_item, false, 0.0, off_height),
            ] {
                let Some(stack) = stack else {
                    continue;
                };
                let layers = held_item_layers[usize::from(!right)].get(stack, || {
                    block_renderer.item_layers(
                        stack.item_registry_id(),
                        if right {
                            voxel_render::model::DisplayContext::FirstPersonRightHand
                        } else {
                            voxel_render::model::DisplayContext::FirstPersonLeftHand
                        },
                        &|temperature, downfall| {
                            biome_tints.grass_at_climate(temperature, downfall)
                        },
                        stack.patch_bytes(),
                    )
                });
                held.extend(voxel_render::player_renderer::held_item_vertices(
                    &view, layers, right, attack, height,
                ));
            }
            // `Camera.calculateHudFov`: the hand's field of view is the
            // default, narrowed in water, and not widened by speed.
            let hud_fov = if player.is_eye_in_water() {
                70.0 * 0.857_142_87
            } else {
                70.0
            };
            let hand_projection =
                voxel_render::scene::perspective(f32::to_radians(hud_fov), aspect, 0.05, 100.0);
            backend.set_hand(&hand, &held, hand_projection)?;
        } else {
            backend.set_hand(&[], &[], [[0.0; 4]; 4])?;
        }
        // `extractBlockOutline`: the block aimed at this frame, unless the
        // player cannot build (adventure and spectator, whose exceptions are
        // not ported).
        let mut outline = Vec::new();
        if !matches!(
            session.game_mode(),
            voxel_world::ClientPlayerGameMode::Adventure
                | voxel_world::ClientPlayerGameMode::Spectator
        ) && let voxel_game::game_mode::Aim::Block { x, y, z, .. } =
            pick(session.world(), player, eye_position(player, partial))
            && let Some(state) = session.world().block_state_at(x, y, z)
        {
            let shape = voxel_world::outline_shape(state as u32, [x, y, z]);
            outline = voxel_render::outline::outline_vertices(
                &shape,
                [
                    (x - origin[0]) as f32,
                    (y - origin[1]) as f32,
                    (z - origin[2]) as f32,
                ],
                eye,
                voxel_render::outline::OUTLINE_COLOUR,
                voxel_render::outline::appropriate_line_width(backend.extent().0),
            );
        }
        backend.set_outline(&outline)?;
        // `extractBlockDestroyAnimation`: each cracked block near the
        // camera, at the furthest stage anyone has it.
        let mut cracked: std::collections::BTreeMap<(i32, i32, i32), i32> =
            std::collections::BTreeMap::new();
        for progress in session.world().destroying_blocks().values() {
            let centre = [progress.x, progress.y, progress.z].map(|v| f64::from(v) + 0.5);
            let distance = (centre[0] - camera_world.x).powi(2)
                + (centre[1] - camera_world.y).powi(2)
                + (centre[2] - camera_world.z).powi(2);
            if distance <= 1024.0 {
                let stage = cracked
                    .entry((progress.x, progress.y, progress.z))
                    .or_insert(-1);
                *stage = (*stage).max(progress.stage);
            }
        }
        let mut breaking = Vec::new();
        for (&(x, y, z), &stage) in &cracked {
            let Some(state) = session.world().block_state_at(x, y, z) else {
                continue;
            };
            let (quads, offset) = block_renderer.block_quads(state as u32, [x, y, z]);
            breaking.extend(voxel_render::breaking::breaking_vertices(
                &quads,
                offset,
                [
                    (x - origin[0]) as f32,
                    (y - origin[1]) as f32,
                    (z - origin[2]) as f32,
                ],
                stage,
            ));
        }
        backend.set_breaking(&breaking)?;
        if camera_type == CameraType::FirstPerson {
            backend.set_entities(&[])?;
        } else {
            let model_pose = player.model_pose(partial as f32);
            player_model.setup_anim(&voxel_render::player_model::PlayerRenderState {
                head_y_rot: voxel_core::math::wrap_degrees_f32(
                    model_pose.head_rot - model_pose.body_rot,
                ),
                x_rot: player_x_rot,
                walk_position: model_pose.walk_position,
                walk_speed: model_pose.walk_speed,
                age_in_ticks: player.tick_count() as f32 + partial as f32,
                swim_amount: model_pose.swim_amount,
                crouching: model_pose.crouching,
                spectator: player.spectating(),
                attack_time: player.attack_anim(partial as f32),
                ..Default::default()
            });
            let visually_swimming = model_pose.pose == voxel_world::ClientEntityPose::Swimming;
            backend.set_entities(&voxel_render::player_renderer::player_vertices(
                &player_model,
                &voxel_render::player_renderer::Placement {
                    feet: [
                        (feet.x - f64::from(origin[0])) as f32,
                        (feet.y - f64::from(origin[1])) as f32,
                        (feet.z - f64::from(origin[2])) as f32,
                    ],
                    body_rot: model_pose.body_rot,
                    x_rot: player_x_rot,
                    swim_amount: model_pose.swim_amount,
                    in_water: player.in_water(),
                    visually_swimming,
                    crouching: model_pose.crouching,
                    light,
                },
            ))?;
        }
        let fov = camera.fov(partial as f32, player.is_eye_in_water());
        let eye_in_water = player.is_eye_in_water();
        // The camera's far plane: four times the render distance, or the
        // cloud range if that is farther.
        let render_distance = session.view_distance().clamp(2, 12);
        let far = ((render_distance * 16 * 4) as f32).max((CLOUD_RANGE * 16) as f32);
        // The view bob option's default is on.
        let (walked, bob) = player.view_bob(partial as f32);
        let bob = voxel_render::camera::view_bob(walked, bob);
        let view_projection = player_camera(aspect, fov, far, eye, y_rot, x_rot, bob);

        // Drop the sections whose chunk went away, then mesh the dirty ones.
        sections.forget_chunks(session.take_forgotten_chunks(), &mut backend)?;
        let camera_block =
            [eye_world.x, eye_world.y, eye_world.z].map(|value| value.floor() as i64);
        sections.update(
            &mut session,
            &mut backend,
            feet,
            camera_block,
            script_done,
            origin,
        )?;
        let meshed = std::time::Instant::now();
        backend.set_sort_origin(eye);
        let mut regions = Vec::new();
        for (animation, redraw) in animations.iter().zip(&mut animation_redraw) {
            if !std::mem::take(redraw) {
                continue;
            }
            for (level, target) in atlas_levels.iter_mut().enumerate() {
                let (x, y, width, height) = animation.draw(level, target);
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for row in y..y + height {
                    let start = ((row * target.width + x) * 4) as usize;
                    rgba.extend_from_slice(&target.rgba[start..start + width as usize * 4]);
                }
                regions.push(voxel_render::vulkan::AtlasRegion {
                    level: level as u32,
                    x,
                    y,
                    width,
                    height,
                    rgba,
                });
            }
        }
        backend.update_atlas(&regions)?;

        let attributes = attributes_last.lerp(&attributes_next, partial as f32);
        // `AtmosphericFogEnvironment.updateRainFogState`: rain pulls the fog
        // in where the camera sees the sky, half as much in a biome without
        // precipitation, easing a fifth of the way there per tick.
        {
            let target = rain_fog_target(&session, &biome_tints, &environment, eye_world);
            rain_fog += (target - rain_fog) * frame_ticks * 0.2;
        }
        let weather = weather(&session, &biome_tints, eye_world, origin, partial as f32);
        backend.set_weather(&weather.rain, &weather.snow)?;
        let look = voxel_core::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
        let sky = voxel_render::sky::build_frame(
            &attributes,
            &voxel_render::sky::SkyView {
                forward: [look.x as f32, look.y as f32, look.z as f32],
                eye_above_horizon: eye_world.y - 63.0,
                eye_in_water,
                render_distance,
                cloud_range: CLOUD_RANGE,
                weather: voxel_render::sky::Weather {
                    rain: session.rain_level(),
                    thunder: session.thunder_level(),
                },
                rain_fog,
            },
            &stars,
            &sky_sprites,
            environment.skybox(),
        );
        // The lightmap is extracted once per tick, from the probe's values
        // at the end of the tick rather than blended within it.
        let lightmap = attributes_next.lightmap();
        backend.set_lightmap(
            lightmap.sky_colour,
            lightmap.sky_factor,
            lightmap.block_tint,
            1.4 + block_light_flicker,
            lightmap.ambient,
            // The brightness option's default.
            0.5,
        );
        let sky_view_projection = player_camera(aspect, fov, far, [0.0; 3], y_rot, x_rot, bob);
        // Clouds, when their colour is not fully transparent; their faces
        // are rebuilt when the camera enters another cell or crosses the
        // layer.
        if sky.cloud_colour[3] > 0.0 {
            let camera_position = [eye_world.x, eye_world.y, eye_world.z];
            let placement = voxel_render::clouds::place(
                &cloud_cells,
                sky.cloud_height,
                camera_position,
                session.game_time(),
                partial as f32,
            );
            let key = (placement.cell, placement.relative);
            if cloud_key != Some(key) {
                cloud_key = Some(key);
                let faces = cloud_cells.build(
                    placement.relative,
                    placement.cell.0,
                    placement.cell.1,
                    voxel_render::clouds::radius_cells(CLOUD_RANGE),
                );
                backend.set_cloud_faces(&faces)?;
            }
            backend.set_cloud_frame(Some(voxel_render::vulkan::CloudFrame {
                view_projection: sky_view_projection,
                offset: placement.offset,
                fog_end: sky.cloud_fog_end,
                colour: sky.cloud_colour,
            }));
        } else {
            backend.set_cloud_frame(None);
        }
        backend.set_fog(sky.fog_colour, sky.terrain_fog);
        backend.set_sky(
            &sky.vertices,
            &sky.batches,
            sky_view_projection,
            sky.fog_colour,
            sky.sky_fog_end,
        )?;
        let colour = sky.clear_colour;

        // With a frame count, the frames after the script ends are captured
        // one per tick into numbered files before the run stops.
        if script_done && captured_frames < capture_frames {
            if let Ok(path) = env::var("VOXEL_CAPTURE_PPM") {
                let captured = backend.capture_scene(colour, view_projection)?;
                let numbered = format!("{path}.{captured_frames:03}.ppm");
                ppm::write(&numbered, captured.width, captured.height, &captured.rgba)?;
                println!("captured_frame={numbered}");
            }
            captured_frames += 1;
            continue;
        }
        if script_done {
            let player = session.player().ok_or("no local player")?;
            let position = player.position();
            println!(
                "play_end ticks={ticks} position={:.4},{:.4},{:.4} rotation={y_rot},{x_rot} on_ground={}",
                position.x,
                position.y,
                position.z,
                player.on_ground()
            );
            println!("play_teleports={}", session.teleports());
            println!("play_swings={}", session.swings());
            if let Some(warning) = session.warning() {
                println!("play_warning={warning}");
            }
            if let Ok(path) = env::var("VOXEL_CAPTURE_PPM") {
                let captured = backend.capture_scene(colour, view_projection)?;
                ppm::write(&path, captured.width, captured.height, &captured.rgba)?;
                println!("captured_frame={path}");
            }
            break;
        }
        // The HUD, in GUI units.
        {
            let size = surface.scale();
            let screen_menu = inventory_screen
                .is_some()
                .then(|| session.inventory_menu().clone());
            let hud_player = session.player();
            let mut draw_item =
                |stack: &voxel_world::ClientItemStackState,
                 x: f32,
                 y: f32,
                 out: &mut Vec<voxel_render::gui::items::GuiItemVertex>| {
                    let layers = gui_item_layers.get(stack, || {
                        block_renderer.item_layers(
                            stack.item_registry_id(),
                            voxel_render::model::DisplayContext::Gui,
                            &|temperature, downfall| {
                                biome_tints.grass_at_climate(temperature, downfall)
                            },
                            stack.patch_bytes(),
                        )
                    });
                    voxel_render::gui::items::item_vertices(layers, x, y, out);
                };
            let frame = hud.build(
                &gui_atlas,
                &font,
                size.gui_scaled_width(),
                size.gui_scaled_height(),
                &hud_input(&session, camera_type == CameraType::FirstPerson),
                started_at.elapsed().as_millis(),
                &mut draw_item,
            );
            // The open screen over the HUD.
            let mut frame = frame;
            if let (Some(screen), Some(menu)) = (inventory_screen.as_mut(), screen_menu) {
                // The player in the screen, posed as the entity stands now.
                let mut draw_player =
                    |area: (i32, i32, i32, i32), size: f32, offset_y: f32, mouse: (f32, f32)| {
                        let Some(player) = hud_player else {
                            return Vec::new();
                        };
                        let pose = player.model_pose(1.0);
                        let state = voxel_render::player_model::PlayerRenderState {
                            walk_position: pose.walk_position,
                            walk_speed: pose.walk_speed,
                            age_in_ticks: player.tick_count() as f32 + 1.0,
                            swim_amount: pose.swim_amount,
                            crouching: pose.crouching,
                            spectator: player.spectating(),
                            attack_time: player.attack_anim(1.0),
                            ..Default::default()
                        };
                        voxel_render::player_renderer::gui_player_vertices(
                            &mut player_model,
                            &state,
                            area,
                            size,
                            offset_y,
                            mouse,
                        )
                    };
                // A stack's tooltip in words.
                let mut tooltip_lines = |stack: &voxel_world::ClientItemStackState,
                                         creative: bool| {
                    voxel_game::tooltip::lines(stack, true, voxel_game::tooltip::Flags { creative })
                        .iter()
                        .map(|line| lang.runs(line, &voxel_game::text::Style::default()))
                        .collect::<Vec<_>>()
                };
                let mut draw = voxel_render::gui::screen::ScreenDraw {
                    frame: &mut frame,
                    atlas: &gui_atlas,
                    font: &font,
                    item: &mut draw_item,
                    player: &mut draw_player,
                    tooltip: &mut tooltip_lines,
                };
                screen.build(&mut draw, &menu, gui_mouse);
            }
            let scale = size.gui_scale() as f32;
            backend.set_gui(
                &frame,
                [
                    size.framebuffer_width() as f32 / scale,
                    size.framebuffer_height() as f32 / scale,
                ],
            )?;
        }
        let built = std::time::Instant::now();
        backend.draw_scene(colour, view_projection)?;
        frames += 1;
        if let Some(profile) = profile.as_mut() {
            let drawn = std::time::Instant::now();
            profile.record([frame_start, ticked, meshed, built, drawn], &mut sections);
        }
    }
    println!("play_frames={frames} play_ticks={ticks}");
    Ok(())
}

/// Join a server and wait until its level has loaded around the player,
/// then a little longer for the chunks near them.
fn join(
    server: SocketAddr,
    username: &str,
    auto_jump: bool,
) -> Result<PlaySession, Box<dyn std::error::Error>> {
    println!("play_connecting={server} user={username}");
    let mut session = PlaySession::connect(server, username, Duration::from_secs(30), auto_jump)?;
    println!("play_joined, waiting for the level");
    let started = std::time::Instant::now();
    while !session.is_loaded() {
        session.wait(Duration::from_millis(100))?;
        if started.elapsed() > Duration::from_secs(30) {
            let world = session.world();
            let cache = world.client_chunk_cache_snapshot();
            return Err(format!(
                "the level did not load within thirty seconds: player at {:?}, {} chunks, cache centre {:?}",
                session.player().map(|player| player.position()),
                world.loaded_chunk_count(),
                cache.map(|cache| (cache.view_center_x, cache.view_center_z, cache.storage_radius)),
            )
            .into());
        }
    }
    // Let the chunks around the player arrive before the first mesh.
    let settle = std::time::Instant::now();
    while settle.elapsed() < Duration::from_millis(2000) {
        session.wait(Duration::from_millis(50))?;
        session.tick(voxel_world::input::Input::EMPTY)?;
    }
    Ok(session)
}

/// A texture of the game's own, `minecraft/textures/<name>.png`, with a
/// label for its errors.
fn texture(
    root: &str,
    name: &str,
    label: &str,
) -> Result<voxel_render::DecodedImage, Box<dyn std::error::Error>> {
    let path = std::path::Path::new(root).join(format!("minecraft/textures/{name}.png"));
    Ok(voxel_render::png::decode_rgba(&std::fs::read(path)?)
        .map_err(|error| format!("{label}: {error:?}"))?)
}

/// `ModelBakery.DESTROY_STAGES`, stacked into one image.
fn destroy_stages(root: &str) -> Result<voxel_render::DecodedImage, Box<dyn std::error::Error>> {
    let mut cracks = voxel_render::DecodedImage {
        width: 0,
        height: 0,
        rgba: Vec::new(),
    };
    for stage in 0..10 {
        let label = format!("destroy_stage_{stage}");
        let image = texture(root, &format!("block/{label}"), &label)?;
        if stage > 0 && (image.width, image.height) != (cracks.width, cracks.height / stage) {
            return Err(format!("{label}: stages differ in size").into());
        }
        cracks.width = image.width;
        cracks.height += image.height;
        cracks.rgba.extend_from_slice(&image.rgba);
    }
    Ok(cracks)
}

/// The number key for a hotbar slot, 1 to 9.
fn hotbar_key(key: glfw::Key) -> Option<i32> {
    use glfw::Key;
    [
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
    ]
    .iter()
    .position(|candidate| *candidate == key)
    .map(|slot| slot as i32)
}
