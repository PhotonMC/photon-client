//! `voxel play`: join a server and walk around in it.

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::VulkanBackend;
use voxel_render::biome_tint::BiomeTints;
use voxel_render::window::{DisplayData, WindowSurface};
use voxel_render::world_mesh::BlockRenderer;

use crate::animated_atlas::AnimatedAtlas;
use crate::assets::FolderAssets;
use crate::controls::Controls;
use crate::gui::Gui;
use crate::item_layers::ItemLayerCache;
use crate::overlays::{self, FrameView, cracks, hands, hud_fov, outline, packed_light, tick_hands};
use crate::ppm;
use crate::profile::FrameProfile;
use crate::screens::ScreenAssets;
use crate::script::{HeldKeys, Script};
use crate::sections::SectionMeshes;
use crate::sky::{Sky, SkyCamera};
use crate::view::{CLOUD_RANGE, CameraType, camera_position, eye_position, pick, player_camera};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The client ticks twenty times a second.
const TICK_LENGTH: Duration = Duration::from_millis(50);

/// Join a server and walk around in it.
///
/// The player is the ported `LocalPlayer`: its tick runs twenty times a
/// second against the world the server streams, and what it sends is what
/// the reference client would send. WASD moves, space jumps, left shift
/// sneaks, left control sprints, the mouse looks, and escape frees the
/// cursor (a second escape closes the window).
pub fn run_play(mut args: impl Iterator<Item = String>) -> Result<()> {
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
    Play::start(server, (width, height))?.run()
}

/// What a drawn frame leaves the loop to do.
enum Flow {
    Next,
    Stop,
}

/// Where a frame is seen from.
struct FrameCamera {
    partial: f64,
    /// The player's eyes.
    eye: Vec3,
    /// The camera, which backs away from the eyes when detached.
    camera: Vec3,
    /// The camera relative to the mesh origin.
    relative: [f32; 3],
    /// The camera's yaw and pitch.
    rotation: (f32, f32),
    /// The view and projection at the camera, and at the origin for the sky.
    view_projection: [[f32; 4]; 4],
    sky_view_projection: [[f32; 4]; 4],
    render_distance: i32,
    eye_in_water: bool,
    /// `Camera.getCullFrustum`.
    frustum: voxel_render::visibility::Frustum,
}

/// A joined session with its window, renderer and everything the frames
/// draw.
struct Play {
    session: PlaySession,
    surface: WindowSurface,
    backend: VulkanBackend,
    script: Option<Script>,
    block_renderer: Arc<BlockRenderer>,
    biome_tints: Arc<BiomeTints>,
    animated_atlas: AnimatedAtlas,
    /// Meshes are measured from a fixed point near where the player started.
    origin: [i32; 3],
    sections: SectionMeshes,
    sky: Sky,
    player_model: voxel_render::player_model::PlayerModel,
    gui: Gui,
    screen_assets: ScreenAssets,
    /// `ItemInHandRenderer`: the stacks the hands show and how high they are.
    item_in_hand: voxel_game::item_in_hand::ItemInHand,
    /// Each item's first-person layers, by item and hand.
    held_item_layers: [ItemLayerCache; 2],
    camera: voxel_render::camera::Camera,
    controls: Controls,
    /// `VOXEL_CAPTURE_FRAMES`: how many frames after the script ends are
    /// captured, and how many have been.
    capture_frames: u32,
    captured_frames: u32,
    aspect: f32,
    sized_for: (u32, u32),
    accumulated: Duration,
    last_frame: Instant,
    ticks: u64,
    frames: u64,
    profile: Option<FrameProfile>,
}

impl Play {
    /// Join, open the window, and load everything the frames draw.
    fn start(server: SocketAddr, (width, height): (i32, i32)) -> Result<Self> {
        let root =
            env::var("VOXEL_ASSETS").map_err(|_| "VOXEL_ASSETS must name the asset folder")?;
        let username = env::var("VOXEL_USER").unwrap_or_else(|_| "VoxelClient".to_owned());
        let auto_jump = env::var("VOXEL_AUTO_JUMP").is_ok_and(|value| value == "1");
        let script = env::var("VOXEL_SCRIPT")
            .ok()
            .map(|text| Script::parse(&text))
            .transpose()?;

        let mut session = join(server, &username, auto_jump)?;
        let position = report_spawn(&session)?;
        let (mut surface, mut backend, size) = open_window(width, height)?;
        // The vanilla data pack the server runs, which sits next to the
        // assets: the biomes' colours and the sky's data come from it.
        let data_root = env::var("VOXEL_DATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::Path::new(&root).with_file_name("data"));
        let biome_tints = Arc::new(
            BiomeTints::load(
                &data_root,
                std::path::Path::new(&root),
                session.registry_names("minecraft:worldgen/biome"),
                session.hashed_seed(),
            )
            .ok_or("the biome colours could not be read")?,
        );
        let (block_renderer, animated_atlas) = bake_blocks(&root, &mut backend)?;
        let sky_sprites = sky_sprites(&block_renderer);
        let origin = [
            position.x.floor() as i32,
            position.y.floor() as i32,
            position.z.floor() as i32,
        ];
        // The chunk fade option, 0.75 seconds by default and
        // `VOXEL_CHUNK_FADE` seconds if given; a scripted run shows every
        // section at once unless asked, so its captures do not catch one
        // mid-fade.
        let chunk_fade = match env::var("VOXEL_CHUNK_FADE") {
            Ok(seconds) => Duration::from_secs_f64(seconds.parse::<f64>()?.clamp(0.0, 2.0)),
            Err(_) if script.is_some() => Duration::ZERO,
            Err(_) => Duration::from_millis(750),
        };
        let sections = SectionMeshes::new(
            &mut session,
            Arc::clone(&block_renderer),
            Arc::clone(&biome_tints),
            chunk_fade,
        );
        let sky = Sky::load(&data_root, &root, &session, sky_sprites)?;
        backend.load_weather_textures(
            &texture(&root, "environment/rain", "rain")?,
            &texture(&root, "environment/snow", "snow")?,
        )?;
        backend.load_crack_texture(&destroy_stages(&root)?)?;
        // The player's default skin, picked by its profile id as the
        // reference picks one when the profile has no skin of its own.
        let skin = voxel_render::player_renderer::default_skin(
            session
                .profile_id()
                .unwrap_or(voxel_core::binary::Uuid { most: 0, least: 0 }),
        );
        backend.load_entity_texture(&texture(&root, skin.texture, "skin")?)?;
        // The registries a tooltip reads, from the data pack, named as the
        // server synchronised them.
        let registries = voxel_game::registries::Registries::load(&data_root, |registry| {
            session.registry_names(registry).to_vec()
        });
        let gui = Gui::load(std::path::Path::new(&root), registries, &mut backend)?;
        let capture_frames = env::var("VOXEL_CAPTURE_FRAMES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        // `VOXEL_CAMERA=back` or `front` starts detached.
        let camera_type = match env::var("VOXEL_CAMERA").as_deref() {
            Ok("back") => CameraType::ThirdPersonBack,
            Ok("front") => CameraType::ThirdPersonFront,
            _ => CameraType::FirstPerson,
        };
        let controls = Controls::new(camera_type, script.is_none());
        surface.set_cursor_captured(controls.captured);
        Ok(Self {
            screen_assets: gui.screen_assets(),
            session,
            surface,
            backend,
            script,
            block_renderer,
            biome_tints,
            animated_atlas,
            origin,
            sections,
            sky,
            player_model: voxel_render::player_model::PlayerModel::new(skin.slim),
            gui,
            item_in_hand: voxel_game::item_in_hand::ItemInHand::default(),
            held_item_layers: [
                ItemLayerCache::new(voxel_render::model::DisplayContext::FirstPersonRightHand),
                ItemLayerCache::new(voxel_render::model::DisplayContext::FirstPersonLeftHand),
            ],
            camera: voxel_render::camera::Camera::new(),
            controls,
            capture_frames,
            captured_frames: 0,
            aspect: size.0 as f32 / size.1 as f32,
            sized_for: size,
            accumulated: Duration::ZERO,
            last_frame: Instant::now(),
            ticks: 0,
            frames: 0,
            profile: FrameProfile::from_env(),
        })
    }

    /// Frames until the window closes or a script ends.
    fn run(mut self) -> Result<()> {
        while !self.surface.should_close() {
            let frame_start = Instant::now();
            for event in self.surface.poll() {
                self.controls.handle(
                    event,
                    &mut self.session,
                    &mut self.surface,
                    &self.screen_assets,
                )?;
            }
            self.fit_swapchain()?;
            let now = Instant::now();
            let frame_ticks = (now - self.last_frame).as_secs_f32() / TICK_LENGTH.as_secs_f32();
            self.accumulated += now - self.last_frame;
            self.last_frame = now;
            self.session.drain()?;
            self.controls.receive_suggestions(&mut self.session);
            let hud = self.session.drain_hud();
            if !hud.is_empty() {
                self.gui.receive_hud(hud);
            }
            let chat = self.session.drain_chat();
            if !chat.is_empty() {
                self.gui.receive_chat(
                    chat,
                    &mut self.controls.chat_log,
                    self.controls.chat.is_some(),
                );
            }
            self.controls.container_tick(
                &mut self.session,
                &mut self.surface,
                &self.screen_assets,
            )?;
            // A respawn or a change of dimension: nothing of the old level stays.
            if self.session.take_level_reset() {
                self.backend.clear_sections()?;
                self.sections.clear();
                self.sky.reset();
            }
            let script_done = self.run_ticks()?;
            let ticked = Instant::now();
            if let Flow::Stop = self.draw([frame_start, ticked], frame_ticks, script_done)? {
                break;
            }
        }
        println!("play_frames={} play_ticks={}", self.frames, self.ticks);
        Ok(())
    }

    /// A resized or fullscreen window gets a swapchain of its new size.
    fn fit_swapchain(&mut self) -> Result<()> {
        let size = self.surface.scale();
        let wanted = (
            size.framebuffer_width() as u32,
            size.framebuffer_height() as u32,
        );
        if (self.backend.out_of_date() || wanted != self.sized_for) && wanted.0 > 0 && wanted.1 > 0
        {
            self.sized_for = wanted;
            self.backend.resize(wanted.0, wanted.1)?;
            let (width, height) = self.backend.extent();
            self.aspect = width as f32 / height as f32;
        }
        Ok(())
    }

    /// The ticks the time since the last frame owes, each with the keys a
    /// person or the script holds. Whether the script has ended.
    fn run_ticks(&mut self) -> Result<bool> {
        let mut script_done = false;
        while self.accumulated >= TICK_LENGTH {
            self.accumulated -= TICK_LENGTH;
            let held = match self.script.as_mut() {
                Some(script) => match script.next() {
                    Some((held, turn, pitch)) => {
                        if (turn != 0.0 || pitch != 0.0)
                            && let Some(player) = self.session.player_mut()
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
                None => self.controls.keys,
            };
            // Once the script ends the keys stay as its last step left them,
            // so a capture can show a block mid-break.
            if self.script.is_some() && !script_done {
                self.controls.script_step(
                    held,
                    &mut self.session,
                    &mut self.surface,
                    &self.screen_assets,
                )?;
            }
            self.tick(held)?;
        }
        Ok(script_done)
    }

    /// One client tick.
    fn tick(&mut self, held: HeldKeys) -> Result<()> {
        let session = &mut self.session;
        // `Minecraft.handleKeybinds`: the attack key is spent before the
        // player moves, aiming the way `GameRenderer.pick` aims.
        let aim = session
            .player()
            .map_or(voxel_game::game_mode::Aim::Miss, |player| {
                pick(session.world(), player, eye_position(player, 1.0))
            });
        let grabbed = self.controls.captured || self.script.is_some();
        session.keys(
            voxel_game::game_mode::Keys {
                attack_clicks: self.controls.attack_key.take_clicks(),
                attack_held: self.controls.attack_key.is_down() && grabbed,
                use_clicks: self.controls.use_key.take_clicks(),
                use_held: self.controls.use_key.is_down(),
            },
            aim,
        )?;
        for hand in session.take_items_used() {
            self.item_in_hand.item_used(hand);
        }
        // After the keys and before the player ticks.
        tick_hands(&mut self.item_in_hand, session, &self.block_renderer);
        session.tick(held.input())?;
        self.animated_atlas.tick();
        if let Some(player) = session.player() {
            self.camera.tick(player);
        }
        self.gui.tick();
        self.controls.chat_log.tick();
        self.sky.tick(session);
        self.ticks += 1;
        Ok(())
    }

    /// `Camera.alignWithEntity` and the projection: the front view turns
    /// round, and a detached camera backs away from the eyes until a block
    /// is in the way.
    fn frame_camera(&self, player: &voxel_world::local_player::LocalPlayer) -> FrameCamera {
        let partial = self.accumulated.as_secs_f64() / TICK_LENGTH.as_secs_f64();
        let feet = player.interpolated_position(partial);
        let (player_y_rot, player_x_rot) = player.rotation();
        let camera_type = self.controls.camera_type;
        let (y_rot, x_rot) = if camera_type == CameraType::ThirdPersonFront {
            (player_y_rot + 180.0, -player_x_rot)
        } else {
            (player_y_rot, player_x_rot)
        };
        let eye = Vec3::new(
            feet.x,
            feet.y + f64::from(self.camera.eye_height(partial as f32)),
            feet.z,
        );
        let camera = camera_position(self.session.world(), camera_type, eye, (y_rot, x_rot));
        let relative = [
            (camera.x - f64::from(self.origin[0])) as f32,
            (camera.y - f64::from(self.origin[1])) as f32,
            (camera.z - f64::from(self.origin[2])) as f32,
        ];
        let eye_in_water = player.is_eye_in_water();
        let fov = self.camera.fov(partial as f32, eye_in_water);
        // The camera's far plane: four times the render distance, or the
        // cloud range if that is farther.
        let render_distance = self.session.view_distance().clamp(2, 12);
        let far = ((render_distance * 16 * 4) as f32).max((CLOUD_RANGE * 16) as f32);
        // The view bob option's default is on.
        let (walked, bob) = player.view_bob(partial as f32);
        let bob = voxel_render::camera::view_bob(walked, bob);
        FrameCamera {
            partial,
            eye,
            camera,
            relative,
            rotation: (y_rot, x_rot),
            view_projection: player_camera(self.aspect, fov, far, relative, y_rot, x_rot, bob),
            sky_view_projection: player_camera(self.aspect, fov, far, [0.0; 3], y_rot, x_rot, bob),
            render_distance,
            eye_in_water,
            // The field of view never culls narrower than the option.
            frustum: voxel_render::visibility::Frustum::new(
                &crate::view::view_rotation(y_rot, x_rot),
                &voxel_render::visibility::culling_projection(
                    fov.max(voxel_render::camera::DEFAULT_FOV),
                    self.aspect,
                    far,
                ),
                [camera.x, camera.y, camera.z],
            ),
        }
    }

    /// What is drawn over the blocks from the player's side: the hands in
    /// first person unless the player only watches, the aimed block's
    /// outline, the cracks, and the player's own model once the camera
    /// leaves its eyes.
    fn draw_overlays(&mut self, frame: &FrameCamera) -> Result<()> {
        let player = self.session.player().ok_or("no local player")?;
        let view = FrameView {
            partial: frame.partial,
            light: packed_light(self.session.world(), frame.eye),
            origin: self.origin,
        };
        let first_person = self.controls.camera_type == CameraType::FirstPerson;
        if first_person && !player.spectating() {
            let (hand, held) = hands(
                player,
                &self.item_in_hand,
                &mut self.held_item_layers,
                (&self.block_renderer, &self.biome_tints),
                &mut self.player_model,
                view,
            );
            let hand_projection = voxel_render::scene::perspective(
                hud_fov(player).to_radians(),
                self.aspect,
                0.05,
                100.0,
            );
            self.backend.set_hand(&hand, &held, hand_projection)?;
        } else {
            self.backend.set_hand(&[], &[], [[0.0; 4]; 4])?;
        }
        let line_width = voxel_render::outline::appropriate_line_width(self.backend.extent().0);
        self.backend.set_outline(&outline(
            &self.session,
            player,
            view,
            frame.relative,
            line_width,
        ))?;
        self.backend.set_breaking(&cracks(
            &self.session,
            &self.block_renderer,
            frame.camera,
            view,
        ))?;
        if first_person {
            self.backend.set_entities(&[])?;
        } else {
            let model = overlays::player_model(player, &mut self.player_model, view);
            self.backend.set_entities(&model)?;
        }
        Ok(())
    }

    /// One frame: the overlays, the sections, the atlas, the sky, then the
    /// GUI and the scene. A finished script captures its frames and stops.
    fn draw(
        &mut self,
        [frame_start, ticked]: [Instant; 2],
        frame_ticks: f32,
        script_done: bool,
    ) -> Result<Flow> {
        // While a respawn waits for its position there is nothing to draw.
        let Some(player) = self.session.player().filter(|_| self.session.is_loaded()) else {
            std::thread::sleep(Duration::from_millis(5));
            return Ok(Flow::Next);
        };
        let frame = self.frame_camera(player);
        self.draw_overlays(&frame)?;
        // Drop the sections whose chunk went away, then mesh the dirty ones.
        self.sections
            .forget_chunks(self.session.take_forgotten_chunks(), &mut self.backend)?;
        self.sections.update(
            &mut self.session,
            &mut self.backend,
            &crate::sections::SectionView {
                camera: [frame.camera.x, frame.camera.y, frame.camera.z],
                frustum: frame.frustum,
                view_distance: frame.render_distance,
                origin: self.origin,
            },
            script_done,
        )?;
        let meshed = Instant::now();
        self.backend.set_sort_origin(frame.relative);
        self.backend
            .update_atlas(&self.animated_atlas.changed_regions())?;
        let colour = self.sky.draw(
            &mut self.backend,
            &self.session,
            &self.biome_tints,
            &SkyCamera {
                eye: frame.eye,
                rotation: frame.rotation,
                view_projection: frame.sky_view_projection,
                render_distance: frame.render_distance,
                eye_in_water: frame.eye_in_water,
                partial: frame.partial,
                frame_ticks,
                origin: self.origin,
            },
        )?;
        if script_done {
            return self.finish_script(colour, &frame);
        }
        self.gui.draw(
            &mut self.backend,
            &self.surface,
            &mut self.session,
            &mut self.controls,
            (&self.block_renderer, &self.biome_tints),
            &mut self.player_model,
        )?;
        let built = Instant::now();
        self.backend.draw_scene(colour, frame.view_projection)?;
        self.frames += 1;
        if let Some(profile) = self.profile.as_mut() {
            let drawn = Instant::now();
            profile.record(
                [frame_start, ticked, meshed, built, drawn],
                &mut self.sections,
            );
        }
        Ok(Flow::Next)
    }

    /// With a frame count, the frames after the script ends are captured
    /// one per tick into numbered files; then the run reports where the
    /// player ended, captures `VOXEL_CAPTURE_PPM` and stops.
    fn finish_script(&mut self, colour: [f32; 4], frame: &FrameCamera) -> Result<Flow> {
        let capture = env::var("VOXEL_CAPTURE_PPM");
        if self.captured_frames < self.capture_frames {
            if let Ok(path) = capture {
                let captured = self.backend.capture_scene(colour, frame.view_projection)?;
                let numbered = format!("{path}.{:03}.ppm", self.captured_frames);
                ppm::write(&numbered, captured.width, captured.height, &captured.rgba)?;
                println!("captured_frame={numbered}");
            }
            self.captured_frames += 1;
            return Ok(Flow::Next);
        }
        report_end(&self.session, self.ticks, frame.rotation)?;
        if let Ok(path) = capture {
            let captured = self.backend.capture_scene(colour, frame.view_projection)?;
            ppm::write(&path, captured.width, captured.height, &captured.rgba)?;
            println!("captured_frame={path}");
        }
        Ok(Flow::Stop)
    }
}

/// Where the player spawned, reported with the chunks loaded and the biome
/// there.
fn report_spawn(session: &PlaySession) -> Result<Vec3> {
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
    Ok(position)
}

/// The game's window and a Vulkan backend on it, with the framebuffer size
/// the backend was made for; `VOXEL_FULLSCREEN` then starts fullscreen, as
/// F11 would switch.
fn open_window(width: i32, height: i32) -> Result<(WindowSurface, VulkanBackend, (u32, u32))> {
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
    let size = (
        scale.framebuffer_width() as u32,
        scale.framebuffer_height() as u32,
    );
    let backend = unsafe { VulkanBackend::create(display_handle, window_handle, size.0, size.1) }?;
    if env::var_os("VOXEL_FULLSCREEN").is_some() {
        surface.toggle_fullscreen();
    }
    Ok((surface, backend, size))
}

/// Every block's models and one atlas for all of them (sent to the GPU),
/// as the reference bakes them at resource load; the world is then meshed a
/// 16-block section at a time as its chunks arrive and change.
fn bake_blocks(
    root: &str,
    backend: &mut VulkanBackend,
) -> Result<(Arc<BlockRenderer>, AnimatedAtlas)> {
    let source = FolderAssets {
        root: std::path::PathBuf::from(root),
    };
    let requested_mips: i32 = env::var("VOXEL_MIP_LEVELS")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(4);
    let baking = Instant::now();
    let mut block_renderer = BlockRenderer::new(&source, requested_mips)
        .map_err(|error| format!("block models: {error:?}"))?;
    println!(
        "play_atlas={}x{} models={} skipped={} in {:?}",
        block_renderer.atlas.width,
        block_renderer.atlas.height,
        block_renderer.model_count(),
        block_renderer.skipped.len(),
        baking.elapsed()
    );
    backend.load_atlas(&block_renderer.atlas.levels)?;
    // The atlas now lives on the GPU; the renderer only keeps where the
    // sprites are, and the animations take their frames with them.
    let animated = AnimatedAtlas::new(&mut block_renderer.atlas);
    block_renderer.atlas.levels = Vec::new();
    Ok((Arc::new(block_renderer), animated))
}

/// The sun and the moon's phases in the block atlas.
fn sky_sprites(block_renderer: &BlockRenderer) -> voxel_render::sky::SkySprites {
    let sprite = |name: &str| {
        block_renderer
            .atlas
            .sprites
            .get(name)
            .copied()
            .unwrap_or([0.0; 4])
    };
    let moon_names: Vec<String> = voxel_render::sky::moon_sprites().collect();
    voxel_render::sky::SkySprites {
        sun: sprite(voxel_render::sky::SUN_SPRITE),
        moon: std::array::from_fn(|phase| sprite(&moon_names[phase])),
    }
}

/// Join a server and wait until its level has loaded around the player,
/// then a little longer for the chunks near them.
fn join(server: SocketAddr, username: &str, auto_jump: bool) -> Result<PlaySession> {
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
pub fn texture(root: &str, name: &str, label: &str) -> Result<voxel_render::DecodedImage> {
    let path = std::path::Path::new(root).join(format!("minecraft/textures/{name}.png"));
    Ok(voxel_render::png::decode_rgba(&std::fs::read(path)?)
        .map_err(|error| format!("{label}: {error:?}"))?)
}

/// `ModelBakery.DESTROY_STAGES`, stacked into one image.
fn destroy_stages(root: &str) -> Result<voxel_render::DecodedImage> {
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

/// What a finished script leaves the player as.
fn report_end(session: &PlaySession, ticks: u64, (y_rot, x_rot): (f32, f32)) -> Result<()> {
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
    Ok(())
}
