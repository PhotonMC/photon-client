use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::{PlaySession, connect_and_probe};
use voxel_client::render::vulkan::SectionFade;
use voxel_client::render::{self, VulkanBackend};
use voxel_client::window::{DisplayData, WindowSurface};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    if env::args().nth(1).as_deref() == Some("window") {
        return run_window(env::args().skip(2));
    }
    if env::args().nth(1).as_deref() == Some("play") {
        return run_play(env::args().skip(2));
    }
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "VoxelClient".to_owned());
    let ticks: u32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(20);
    let timeout_seconds: u64 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(20);
    if timeout_seconds == 0 {
        return Err("timeout_seconds must be greater than zero".into());
    }

    let mut trace = connect_and_probe(address, &username, Duration::from_secs(timeout_seconds))?;
    for _ in 0..ticks {
        trace.world.tick();
    }

    println!("client_connection=ok");
    println!("play_login={}", trace.play_login);
    println!("world_constructed=true");
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!(
        "update_tags_packet_count={}",
        trace.update_tags_packet_count
    );
    println!(
        "block_tag_vine_climbable={}",
        trace.world.block_state_registry().is_climbable_state(8389)
    );
    println!(
        "block_tag_vine_glide_through={}",
        trace
            .world
            .block_state_registry()
            .is_glide_through_state(8389)
    );
    println!("ticks_run={ticks}");
    println!("world_game_time={}", trace.world.game_time());
    let local_player_previous_position = trace
        .world
        .local_player()
        .map(|player| player.previous_position())
        .unwrap_or((0.0, 0.0, 0.0));
    let local_player_interpolated_position = trace
        .world
        .local_player()
        .map(|player| player.interpolated_position(0.5))
        .unwrap_or((0.0, 0.0, 0.0));
    println!("local_player_previous_position={local_player_previous_position:?}");
    println!("local_player_interpolated_position_half={local_player_interpolated_position:?}");
    println!(
        "local_player_tick_count={}",
        trace
            .world
            .local_player()
            .map(|player| player.tick_count())
            .unwrap_or(0)
    );
    Ok(())
}

/// Open the client window and report the sizing the reference rules produce.
///
/// This is the surface only: nothing is drawn yet, so the window stays blank
/// until the Vulkan backend lands. `frames` bounds the loop so the probe can be
/// run non-interactively; `0` keeps it open until the window is closed.
fn run_window(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    let width: i32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(854);
    let height: i32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(480);
    let frames: u64 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(0);

    let display = DisplayData {
        width,
        height,
        fullscreen: false,
    };
    let mut surface = WindowSurface::create(display, "Minecraft")?;

    let scale = surface.scale();
    println!("window_created=true");
    println!("requested_size={width}x{height}");
    println!(
        "framebuffer_size={}x{}",
        scale.framebuffer_width(),
        scale.framebuffer_height()
    );
    let (window_x, window_y) = surface.position();
    println!("window_position={window_x},{window_y}");
    println!("gui_scale={}", scale.gui_scale());
    println!(
        "gui_scaled_size={}x{}",
        scale.gui_scaled_width(),
        scale.gui_scaled_height()
    );
    // Bring the Vulkan backend up against the live surface. Until a frame is
    // presented a Wayland compositor never maps the window, so this is also
    // what makes it appear at all.
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
    let described = backend.description().clone();
    println!("vulkan_device={}", described.device_name);
    println!("vulkan_discrete={}", described.discrete);
    println!(
        "vulkan_api={}.{}.{}",
        vk_version_major(described.api_version),
        vk_version_minor(described.api_version),
        vk_version_patch(described.api_version)
    );
    println!("swapchain_format={:?}", described.surface_format);
    println!("swapchain_images={}", described.image_count);
    println!(
        "swapchain_extent={}x{}",
        described.extent.width, described.extent.height
    );

    // With a server address the probe joins it, waits for the chunks around
    // the player, and draws a box of the real world instead of one block.
    let from_server = match (env::var("VOXEL_SERVER"), env::var("VOXEL_ASSETS")) {
        (Ok(server), Ok(root)) => Some(load_world(&server, &root)?),
        _ => None,
    };

    // Assets come from a folder the user names; nothing is bundled. With a
    // folder and a model name the probe draws that model; with neither it
    // falls back to its own cube and checkerboard, and says which it used.
    //
    // A block name takes the longer road: its blockstate file is read, the
    // variant its properties select is taken, and the model and rotation come
    // from there rather than from the environment.
    let chosen = match (env::var("VOXEL_BLOCK"), env::var("VOXEL_ASSETS")) {
        (Ok(block), Ok(root)) => {
            let source = FolderAssets {
                root: std::path::PathBuf::from(&root),
            };
            let name = voxel_client::identifier::Identifier::parse(&block)
                .map_err(|error| format!("{block}: {error:?}"))?;
            let text = render::AssetSource::block_state(&source, &name)
                .ok_or_else(|| format!("{block}: no blockstate file"))?;
            let file = render::parse_block_state(&text)
                .ok_or_else(|| format!("{block}: blockstate file was refused"))?;
            // The registry says which properties the block has and what they
            // may hold, so a typo is caught here rather than silently picking
            // the wrong variant.
            let definition = voxel_client::world::block_registry::block(&name.as_string())
                .ok_or_else(|| format!("{block}: not in the block registry"))?;
            let mut properties =
                render::blockstate::parse_properties(&env::var("VOXEL_STATE").unwrap_or_default());
            for (key, value) in &properties {
                let property = definition
                    .properties
                    .iter()
                    .map(|index| &voxel_client::world::block_registry::PROPERTIES[*index as usize])
                    .find(|property| property.name == key)
                    .ok_or_else(|| format!("{block}: no property named {key}"))?;
                if !property.values.contains(&value.as_str()) {
                    return Err(format!("{block}: {key} cannot be {value}").into());
                }
            }
            // A property the caller left out keeps the default state's value,
            // which is what a blockstate selector expects to find.
            for (slot, index) in definition.properties.iter().enumerate() {
                let property = &voxel_client::world::block_registry::PROPERTIES[*index as usize];
                properties
                    .entry(property.name.to_owned())
                    .or_insert_with(|| {
                        property.values[usize::from(definition.default[slot])].to_owned()
                    });
            }
            println!("block_state_id={:?}", definition.state_id(&properties));
            let variants = file.variants_for(&properties);
            let variant = variants
                .first()
                .ok_or_else(|| format!("{block}: no variant matches those properties"))?;
            println!("block={block}");
            println!("block_parts={}", variants.len());
            println!(
                "block_variant={} x{} y{} z{} uvlock={}",
                variant.model.as_string(),
                variant.x.degrees(),
                variant.y.degrees(),
                variant.z.degrees(),
                variant.uvlock
            );
            Some((
                root,
                variant.model.as_string(),
                variant.x.degrees(),
                variant.y.degrees(),
                variant.z.degrees(),
                variant.uvlock,
                name.as_string(),
            ))
        }
        _ => None,
    };
    let (assets, model_name) = match &chosen {
        Some((root, model, ..)) => (Ok(root.clone()), Ok(model.clone())),
        None => (env::var("VOXEL_ASSETS"), env::var("VOXEL_MODEL")),
    };
    let (mesh, texture_levels) = if let Some(loaded) = from_server {
        loaded
    } else {
        match (assets, model_name) {
            (Ok(root), Ok(name)) => {
                let source = FolderAssets {
                    root: std::path::PathBuf::from(&root),
                };
                let identifier = voxel_client::identifier::Identifier::parse(&name)
                    .map_err(|error| format!("{name}: {error:?}"))?;
                let model = render::resolve_model(&source, &identifier)
                    .map_err(|error| format!("{name}: {error:?}"))?;
                // The reference's own default is four levels, lowered when a
                // sprite in the set is too small or not a clean multiple.
                let requested: i32 = env::var("VOXEL_MIP_LEVELS")
                    .ok()
                    .map(|value| value.parse())
                    .transpose()?
                    .unwrap_or(4);
                let atlas = render::build_atlas(&source, &model, 2048, requested)
                    .map_err(|error| format!("{name}: {error:?}"))?;
                // A blockstate variant turns its model in quarter turns. When a
                // block was named the turns come from the variant it selected;
                // otherwise the probe takes them directly.
                let angle = |name: &str, from_variant: Option<i32>| -> Result<i32, String> {
                    match from_variant {
                        Some(degrees) => Ok(degrees),
                        None => env::var(name)
                            .ok()
                            .map(|value| value.parse::<i32>())
                            .transpose()
                            .map_err(|error| format!("{name}: {error}"))
                            .map(|parsed| parsed.unwrap_or(0)),
                    }
                };
                let rotate_x = angle("VOXEL_ROTATE_X", chosen.as_ref().map(|c| c.2))?;
                let rotate_y = angle("VOXEL_ROTATE_Y", chosen.as_ref().map(|c| c.3))?;
                let rotate_z = angle("VOXEL_ROTATE_Z", chosen.as_ref().map(|c| c.4))?;
                let state = voxel_client::render::rotation::model_state_transform(
                    rotate_x, rotate_y, rotate_z,
                )
                .ok_or("variant rotation must be a multiple of ninety")?;
                let identity = rotate_x % 360 == 0 && rotate_y % 360 == 0 && rotate_z % 360 == 0;
                // Outside a world a block's tints are fixed per block, so they
                // come from the registry rather than from the world. A model drawn
                // by name rather than by block has none.
                let tints: Vec<u32> = chosen
                    .as_ref()
                    .map(|c| render::constant_tints(&c.6).to_vec())
                    .unwrap_or_default();
                // A variant may lock its UVs, which turns each face's texture
                // coordinate back by as much as the model turned.
                let uv_lock = chosen.as_ref().is_some_and(|c| c.5);
                let uv_transform = |facing| {
                    voxel_client::render::rotation::inverse_face_transform(
                        rotate_x, rotate_y, rotate_z, facing, uv_lock,
                    )
                    .unwrap_or(voxel_client::render::rotation::FLAT_IDENTITY)
                };
                // A grid repeats the block, so the faces between two of them can
                // be dropped. That is the smallest scene where culling does any
                // work at all.
                let grid: u32 = env::var("VOXEL_GRID")
                    .ok()
                    .map(|value| value.parse())
                    .transpose()?
                    .unwrap_or(1);
                let mut mesh = render::Mesh::default();
                for cell_x in 0..grid {
                    for cell_y in 0..grid {
                        for cell_z in 0..grid {
                            let cell = [cell_x, cell_y, cell_z];
                            let occluded = |facing: voxel_client::direction::Direction| {
                                let (step_x, step_y, step_z) = facing.step();
                                [step_x, step_y, step_z].into_iter().enumerate().all(
                                    |(axis, step)| {
                                        let moved = cell[axis] as i32 + step;
                                        moved >= 0 && moved < grid as i32
                                    },
                                )
                            };
                            let part = render::build_mesh(
                                &model,
                                &atlas,
                                (!identity).then_some(&state),
                                &tints,
                                uv_transform,
                                occluded,
                            )
                            .map_err(|error| format!("{name}: {error:?}"))?;
                            let centre = (grid as f32 - 1.0) / 2.0;
                            let offset = [
                                cell_x as f32 - centre,
                                cell_y as f32 - centre,
                                cell_z as f32 - centre,
                            ];
                            let base = mesh.vertices.len() as u32;
                            for mut vertex in part.vertices {
                                for (position, shift) in vertex.position.iter_mut().zip(offset) {
                                    *position += shift;
                                }
                                mesh.vertices.push(vertex);
                            }
                            mesh.indices
                                .extend(part.indices.into_iter().map(|index| index + base));
                        }
                    }
                }
                println!("model_uvlock={uv_lock}");
                println!("model_grid={grid}");
                println!("model_tints={}", tints.len());
                println!("model_rotation={rotate_x},{rotate_y},{rotate_z}");
                println!("model_source={name}");
                println!("model_elements={}", model.elements.len());
                println!("model_sprites={}", atlas.sprites.len());
                println!("model_ambient_occlusion={}", model.ambient_occlusion);
                println!("atlas_size={}x{}", atlas.width, atlas.height);
                println!("atlas_mip_levels={}", atlas.levels.len());
                println!("atlas_mip_requested={requested}");
                // Each level can be written out, which is how the padding fill
                // and the halving get looked at rather than assumed.
                if let Ok(prefix) = env::var("VOXEL_ATLAS_DUMP") {
                    for (level, image) in atlas.levels.iter().enumerate() {
                        let path = format!("{prefix}{level}.ppm");
                        write_image_ppm(&path, image)?;
                        println!("atlas_level_{level}={}x{}", image.width, image.height);
                    }
                }
                (mesh, atlas.levels)
            }
            _ => {
                println!("model_source=built-in cube and checkerboard");
                let (rgba, size) = checkerboard();
                println!("model_rotation=0,0,0");
                let mesh = render::full_cube(render::SpriteRect {
                    min_u: 0.0,
                    min_v: 0.0,
                    max_u: 1.0,
                    max_v: 1.0,
                });
                (
                    mesh,
                    vec![render::DecodedImage {
                        width: size,
                        height: size,
                        rgba,
                    }],
                )
            }
        }
    };
    println!(
        "texture_size={}x{}",
        texture_levels[0].width, texture_levels[0].height
    );
    backend.load_scene(&mesh, &texture_levels)?;
    println!("mesh_vertices={}", mesh.vertices.len());
    println!("mesh_indices={}", mesh.indices.len());

    let colour = clear_colour();
    let aspect = scale.framebuffer_width() as f32 / scale.framebuffer_height() as f32;
    let view_projection = camera(aspect, 0.0);
    let captured = backend.capture_scene(colour, view_projection)?;
    let mut rendered = 1_u64;
    if let Ok(path) = env::var("VOXEL_CAPTURE_PPM") {
        write_ppm(&path, &captured)?;
        println!("captured_frame={path}");
        println!("captured_size={}x{}", captured.width, captured.height);
    }

    let mut seen = 0_u64;
    while !surface.should_close() {
        for event in surface.wait() {
            if matches!(
                event,
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _)
            ) {
                surface.request_close();
            }
            if let glfw::WindowEvent::FramebufferSize(_, _) = event {
                let scale = surface.scale();
                println!(
                    "resized framebuffer={}x{} gui_scale={} gui_scaled={}x{}",
                    scale.framebuffer_width(),
                    scale.framebuffer_height(),
                    scale.gui_scale(),
                    scale.gui_scaled_width(),
                    scale.gui_scaled_height()
                );
            }
        }
        backend.draw_scene(colour, camera(aspect, rendered as f32 * 0.02))?;
        rendered += 1;
        seen += 1;
        if frames != 0 && seen >= frames {
            break;
        }
    }
    println!("rendered_frames={rendered}");
    println!("window_closed=true");
    Ok(())
}

/// Placeholder clear colour. This is not a reference value: no Java frame has
/// been captured or compared yet, so nothing about these pixels is parity.
const CLEAR_COLOUR: [f32; 4] = [0.1, 0.12, 0.16, 1.0];

/// Read the clear colour from `VOXEL_CLEAR_COLOUR` as `r,g,b` in 0..1.
///
/// Only scaffolding: it lets a run be identified on screen while the real
/// pipeline does not exist yet.
fn clear_colour() -> [f32; 4] {
    let Ok(raw) = env::var("VOXEL_CLEAR_COLOUR") else {
        return CLEAR_COLOUR;
    };
    let parts: Vec<f32> = raw
        .split(',')
        .filter_map(|part| part.trim().parse().ok())
        .collect();
    match parts[..] {
        [red, green, blue] => [red, green, blue, 1.0],
        _ => CLEAR_COLOUR,
    }
}

fn vk_version_major(version: u32) -> u32 {
    version >> 22
}

fn vk_version_minor(version: u32) -> u32 {
    (version >> 12) & 0x3ff
}

fn vk_version_patch(version: u32) -> u32 {
    version & 0xfff
}

/// Write a captured frame as binary PPM.
///
/// PPM keeps the readback dependency-free while the frame comparison format is
/// still undecided; P15.a will pin what captures are actually stored as.
fn write_ppm(
    path: &str,
    frame: &voxel_client::render::CapturedFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(out, "P6\n{} {}\n255\n", frame.width, frame.height)?;
    for pixel in frame.rgba.as_chunks::<4>().0 {
        out.write_all(&pixel[..3])?;
    }
    out.flush()?;
    Ok(())
}

fn write_image_ppm(
    path: &str,
    image: &render::DecodedImage,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(out, "P6\n{} {}\n255\n", image.width, image.height)?;
    for pixel in image.rgba.as_chunks::<4>().0 {
        out.write_all(&pixel[..3])?;
    }
    out.flush()?;
    Ok(())
}

/// View-projection for the probe's camera, orbiting the block.
///
/// None of this matches the reference: the client's own camera arrives with
/// P14, and until then these numbers exist only to make the cube visible.
fn camera(aspect: f32, angle: f32) -> [[f32; 4]; 4] {
    // Inside a world the eye is the player's and it turns on the spot; with a
    // single block or a grid there is no player, so the eye orbits it instead.
    if let Some(eye) = CAMERA_EYE.get().copied() {
        let look = [eye[0] + angle.cos(), eye[1] - 0.25, eye[2] + angle.sin()];
        let view = render::scene::look_at(eye, look, [0.0, 1.0, 0.0]);
        let projection = render::scene::perspective(70.0_f32.to_radians(), aspect, 0.05, 512.0);
        return render::scene::multiply(projection, view);
    }
    let grid = env::var("VOXEL_GRID")
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(1.0)
        .max(1.0);
    let radius = 2.4_f32 * grid;
    let eye = [
        0.5 + radius * angle.cos(),
        1.4 * grid,
        0.5 + radius * angle.sin(),
    ];
    let view = render::scene::look_at(eye, [0.5, 0.5, 0.5], [0.0, 1.0, 0.0]);
    let projection = render::scene::perspective(70.0_f32.to_radians(), aspect, 0.05, 100.0);
    render::scene::multiply(projection, view)
}

/// Where the player stands inside a drawn world box, eyes included.
///
/// Set by the world loader rather than read from the environment, because it
/// depends on where the server put the player. The eye height is the
/// reference's standing value.
static CAMERA_EYE: std::sync::OnceLock<[f32; 3]> = std::sync::OnceLock::new();

/// A fallback texture so the probe runs with no assets present.
fn checkerboard() -> (Vec<u8>, u32) {
    const SIZE: u32 = 16;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let light = (x / 2 + y / 2) % 2 == 0;
            let shade = if light { 200 } else { 120 };
            rgba.extend_from_slice(&[shade, shade / 2 + 60, 90, 255]);
        }
    }
    (rgba, SIZE)
}

/// Read models and textures from a folder laid out like the game's assets.
struct FolderAssets {
    root: std::path::PathBuf,
}

impl FolderAssets {
    /// The layout the game's asset tree uses: namespace, kind, then the name.
    fn path(
        &self,
        name: &voxel_client::identifier::Identifier,
        kind: &str,
        extension: &str,
    ) -> std::path::PathBuf {
        self.root.join(name.namespace()).join(kind).join(format!(
            "{}{extension}",
            name.path().replace('/', std::path::MAIN_SEPARATOR_STR)
        ))
    }
}

impl render::AssetSource for FolderAssets {
    fn model(&self, name: &voxel_client::identifier::Identifier) -> Option<String> {
        std::fs::read_to_string(self.path(name, "models", ".json")).ok()
    }

    fn texture(&self, name: &voxel_client::identifier::Identifier) -> Option<Vec<u8>> {
        std::fs::read(self.path(name, "textures", ".png")).ok()
    }

    fn block_state(&self, name: &voxel_client::identifier::Identifier) -> Option<String> {
        // A blockstate file sits in its own directory, named after the block
        // rather than after a model.
        std::fs::read_to_string(self.path(name, "blockstates", ".json")).ok()
    }

    fn texture_metadata(
        &self,
        name: &voxel_client::identifier::Identifier,
    ) -> Option<render::TextureMetadata> {
        // A sprite's metadata file sits next to its texture with a second
        // extension, and the mip strategy is one section inside it.
        let text = std::fs::read_to_string(self.path(name, "textures", ".png.mcmeta")).ok()?;
        match voxel_client::json::parse(&text).ok()? {
            voxel_client::json::JsonValue::Object(object) => {
                render::TextureMetadata::parse(object.object("texture")?)
            }
            _ => None,
        }
    }

    fn texture_animation(
        &self,
        name: &voxel_client::identifier::Identifier,
    ) -> Option<render::AnimationFrames> {
        let text = std::fs::read_to_string(self.path(name, "textures", ".png.mcmeta")).ok()?;
        match voxel_client::json::parse(&text).ok()? {
            voxel_client::json::JsonValue::Object(object) => {
                render::AnimationFrames::parse(object.object("animation")?)
            }
            _ => None,
        }
    }
}

/// Join a server, wait for the chunks around the player, and build one mesh
/// for the box of world in front of them.
///
/// The box is small on purpose: this is the first time the network side and
/// the renderer meet, so it draws what fits in one look rather than a render
/// distance.
fn load_world(
    server: &str,
    root: &str,
) -> Result<(render::Mesh, Vec<render::DecodedImage>), Box<dyn std::error::Error>> {
    let address: SocketAddr = server.parse()?;
    let username = env::var("VOXEL_USER").unwrap_or_else(|_| "VoxelClient".to_owned());
    let seconds: u64 = env::var("VOXEL_TIMEOUT")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(30);
    let trace = connect_and_probe(address, &username, Duration::from_secs(seconds))?;
    println!("server={server}");
    println!("server_login={}", trace.play_login);
    println!("server_chunks={}", trace.world.loaded_chunk_count());
    println!("server_sections={}", trace.world.loaded_section_count());

    // The box hangs around the spawn position the server put the player at,
    // so what is drawn is what the player would be looking at.
    let centre = trace
        .world
        .local_player()
        .map(|player| player.previous_position())
        .ok_or("the server sent no player position")?;
    let reach: i32 = env::var("VOXEL_REACH")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(12);
    let height: i32 = env::var("VOXEL_HEIGHT")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(12);
    // Where the server drops the player is not always where the interesting
    // part of the world is, so the box's floor can be aimed by hand.
    let floor: i32 = match env::var("VOXEL_FLOOR") {
        Ok(value) => value.parse()?,
        Err(_) => centre.1.floor() as i32 - height / 2,
    };
    let origin = [
        centre.0.floor() as i32 - reach,
        floor,
        centre.2.floor() as i32 - reach,
    ];
    let size = [reach * 2 + 1, height, reach * 2 + 1];
    println!(
        "world_box origin={},{},{} size={}x{}x{}",
        origin[0], origin[1], origin[2], size[0], size[1], size[2]
    );

    let source = FolderAssets {
        root: std::path::PathBuf::from(root),
    };
    let requested: i32 = env::var("VOXEL_MIP_LEVELS")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(4);
    let built = render::build_world_mesh(
        &source,
        |x, y, z| trace.world.block_state_at(x, y, z),
        |x, y, z| {
            (
                trace.world.block_brightness(x, y, z),
                trace.world.sky_brightness(x, y, z),
            )
        },
        // This older snapshot path draws every biome as plains.
        |kind, _| match kind {
            render::biome_tint::TintKind::Grass => 0xFF91_BD59,
            render::biome_tint::TintKind::Foliage => 0xFF77_AB2F,
            render::biome_tint::TintKind::DryFoliage => 0xFFA0_7545,
            render::biome_tint::TintKind::Water => 0xFF3F_76E4,
        },
        origin,
        size,
        requested,
    )
    .map_err(|error| format!("world mesh: {error:?}"))?;

    // The mesh sits with the box's near corner at the origin, so the player
    // stands where the server put them, measured from that corner. The eye
    // goes there rather than orbiting outside, which is the view the player
    // would actually have.
    let eye_height: f32 = env::var("VOXEL_EYE")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(1.62);
    let _ = CAMERA_EYE.set([
        (centre.0 - f64::from(origin[0])) as f32,
        // The eye is measured from the box's floor when the floor was aimed by
        // hand, because the player may be nowhere near it.
        if env::var("VOXEL_FLOOR").is_ok() {
            eye_height
        } else {
            (centre.1 - f64::from(origin[1])) as f32 + eye_height
        },
        (centre.2 - f64::from(origin[2])) as f32,
    ]);
    println!("world_solid_blocks={}", built.solid_count);
    println!("world_models={}", built.model_count);
    println!("atlas_size={}x{}", built.atlas.width, built.atlas.height);
    println!("atlas_mip_levels={}", built.atlas.levels.len());
    // A world always holds blocks whose models need machinery that is not
    // built yet; saying which is better than drawing a hole in silence.
    for (block, reason) in &built.skipped {
        println!("world_skipped={block} ({reason})");
    }
    Ok((built.mesh, built.atlas.levels))
}

/// Held movement keys, as the key mappings see them.
#[derive(Default, Clone, Copy)]
struct HeldKeys {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    jump: bool,
    sneak: bool,
    sprint: bool,
}

impl HeldKeys {
    fn input(self) -> voxel_client::input::Input {
        voxel_client::input::Input::new(
            self.forward,
            self.backward,
            self.left,
            self.right,
            self.jump,
            self.sneak,
            self.sprint,
        )
    }

    fn set(&mut self, key: glfw::Key, down: bool) {
        match key {
            glfw::Key::W => self.forward = down,
            glfw::Key::S => self.backward = down,
            glfw::Key::A => self.left = down,
            glfw::Key::D => self.right = down,
            glfw::Key::Space => self.jump = down,
            glfw::Key::LeftShift => self.sneak = down,
            glfw::Key::LeftControl => self.sprint = down,
            _ => {}
        }
    }

    /// One step of a `VOXEL_SCRIPT`, such as `forward,sprint`.
    fn parse(step: &str) -> Result<Self, String> {
        let mut keys = Self::default();
        for name in step
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            match name {
                "forward" => keys.forward = true,
                "backward" => keys.backward = true,
                "left" => keys.left = true,
                "right" => keys.right = true,
                "jump" => keys.jump = true,
                "sneak" => keys.sneak = true,
                "sprint" => keys.sprint = true,
                "none" => {}
                other => return Err(format!("unknown key {other} in VOXEL_SCRIPT")),
            }
        }
        Ok(keys)
    }
}

/// A scripted run: keys held for so many ticks, and turns applied between.
struct Script {
    steps: Vec<(HeldKeys, u32, f64, f64)>,
    index: usize,
    ticks_in_step: u32,
}

impl Script {
    /// `forward=40;forward,jump=10;turn:90;none=20` holds the keys for the
    /// counted ticks, and `turn:D` turns the view by D degrees.
    fn parse(text: &str) -> Result<Self, String> {
        let mut steps = Vec::new();
        for step in text
            .split(';')
            .map(str::trim)
            .filter(|step| !step.is_empty())
        {
            if let Some(degrees) = step.strip_prefix("turn:") {
                let degrees: f64 = degrees.parse().map_err(|_| format!("bad turn {degrees}"))?;
                steps.push((HeldKeys::default(), 0, degrees, 0.0));
                continue;
            }
            if let Some(degrees) = step.strip_prefix("pitch:") {
                let degrees: f64 = degrees
                    .parse()
                    .map_err(|_| format!("bad pitch {degrees}"))?;
                steps.push((HeldKeys::default(), 0, 0.0, degrees));
                continue;
            }
            let (keys, count) = step
                .split_once('=')
                .ok_or(format!("{step}: expected keys=ticks"))?;
            let count: u32 = count
                .parse()
                .map_err(|_| format!("bad tick count {count}"))?;
            steps.push((HeldKeys::parse(keys)?, count, 0.0, 0.0));
        }
        Ok(Self {
            steps,
            index: 0,
            ticks_in_step: 0,
        })
    }

    /// The keys for the next tick and a turn and pitch to apply first, or
    /// nothing once done.
    fn next(&mut self) -> Option<(HeldKeys, f64, f64)> {
        let mut turn = 0.0;
        let mut pitch = 0.0;
        loop {
            let (keys, count, degrees, tilt) = *self.steps.get(self.index)?;
            if count == 0 {
                turn += degrees;
                pitch += tilt;
                self.index += 1;
                continue;
            }
            self.ticks_in_step += 1;
            if self.ticks_in_step >= count {
                self.index += 1;
                self.ticks_in_step = 0;
            }
            return Some((keys, turn, pitch));
        }
    }
}

/// The cloud range option's default, in chunks.
const CLOUD_RANGE: i32 = 128;

/// View-projection from the player's eye, measured from the mesh origin.
fn player_camera(
    aspect: f32,
    fov: f32,
    far: f32,
    eye: [f32; 3],
    y_rot: f32,
    x_rot: f32,
    bob: [[f32; 4]; 4],
) -> [[f32; 4]; 4] {
    let look = voxel_client::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
    let target = [
        eye[0] + look.x as f32,
        eye[1] + look.y as f32,
        eye[2] + look.z as f32,
    ];
    let view = render::scene::look_at(eye, target, [0.0, 1.0, 0.0]);
    let projection = render::scene::multiply(
        render::scene::perspective(fov.to_radians(), aspect, 0.05, far),
        bob,
    );
    render::scene::multiply(projection, view)
}

/// Join a server and walk around in it.
///
/// The player is the ported `LocalPlayer`: its tick runs twenty times a
/// second against the world the server streams, and what it sends is what
/// the reference client would send. WASD moves, space jumps, left shift
/// sneaks, left control sprints, the mouse looks, and escape frees the
/// cursor (a second escape closes the window).
fn run_play(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
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

    println!("play_connecting={server} user={username}");
    let mut session = PlaySession::connect(server, &username, Duration::from_secs(30), auto_jump)?;
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
        session.tick(voxel_client::input::Input::EMPTY)?;
    }
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
    let biome_tints = render::biome_tint::BiomeTints::load(
        &env::var("VOXEL_DATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::Path::new(&root).with_file_name("data")),
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
    let block_renderer = render::world_mesh::BlockRenderer::new(&source, requested_mips)
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
    let moon_names: Vec<String> = render::sky::moon_sprites().collect();
    let sky_sprites = render::sky::SkySprites {
        sun: sprite(render::sky::SUN_SPRITE),
        moon: std::array::from_fn(|phase| sprite(&moon_names[phase])),
    };
    // Meshes are measured from a fixed point near where the player started.
    let origin = [
        position.x.floor() as i32,
        position.y.floor() as i32,
        position.z.floor() as i32,
    ];
    // Dirty sections, with when each was last dirtied.
    let mut pending: std::collections::HashMap<(i32, i32, i32), std::time::Instant> = session
        .take_dirty_sections()
        .into_iter()
        .map(|key| (key, std::time::Instant::now()))
        .collect();
    // Sections are meshed on worker threads, leaving one core for the game.
    let worker_count = std::thread::available_parallelism()
        .map_or(2, |count| count.get().saturating_sub(1))
        .clamp(1, 8);
    let mut workers = render::section_workers::SectionWorkers::new(
        worker_count,
        std::sync::Arc::clone(&block_renderer),
        std::sync::Arc::clone(&biome_tints),
    );
    // The generation of each section's newest hand-out; a mesh from an
    // older one is stale.
    let mut latest: std::collections::HashMap<(i32, i32, i32), u64> =
        std::collections::HashMap::new();
    let mut next_generation = 0_u64;
    const SETTLE: Duration = Duration::from_secs(2);
    let mut fades: std::collections::HashMap<(i32, i32, i32), SectionFade> =
        std::collections::HashMap::new();
    // The chunk fade option, 0.75 seconds by default and `VOXEL_CHUNK_FADE`
    // seconds if given; a scripted run shows every section at once unless
    // asked, so its captures do not catch one mid-fade.
    let chunk_fade = match env::var("VOXEL_CHUNK_FADE") {
        Ok(seconds) => Duration::from_secs_f64(seconds.parse::<f64>()?.clamp(0.0, 2.0)),
        Err(_) if script.is_some() => Duration::ZERO,
        Err(_) => Duration::from_millis(750),
    };
    let section_low = session.world().dimension().min_y.div_euclid(16);
    let section_high = section_low + session.world().dimension().height / 16;

    // The sky's data comes from the same vanilla data pack the server runs,
    // which sits next to the assets.
    let data_root = env::var("VOXEL_DATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::Path::new(&root).with_file_name("data"));
    let environment = render::sky::Environment::load(
        &data_root,
        session.dimension_type().unwrap_or("minecraft:overworld"),
        session.registry_names("minecraft:worldgen/biome"),
    )
    .ok_or("the dimension's sky data could not be read")?;
    let stars = render::sky::build_stars();
    let dimension = session.world().dimension();
    let probe = |session: &PlaySession, eye: [f64; 3]| {
        let quart_min = dimension.min_y.div_euclid(4);
        let quart_max = quart_min + dimension.height / 4 - 1;
        environment.evaluate(
            eye,
            |x, y, z| {
                let y = y.clamp(quart_min, quart_max);
                session.world().biome_at(x * 4, y * 4, z * 4)
            },
            |clock| session.clock_ticks(clock).unwrap_or(0),
            render::sky::Weather {
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
    let cloud_image = render::png::decode_rgba(&std::fs::read(
        std::path::Path::new(&root).join("minecraft/textures/environment/clouds.png"),
    )?)
    .map_err(|error| format!("clouds: {error:?}"))?;
    let cloud_cells = render::clouds::CloudCells::from_image(&cloud_image);
    let environment_texture =
        |name: &str| -> Result<render::DecodedImage, Box<dyn std::error::Error>> {
            let path = std::path::Path::new(&root)
                .join(format!("minecraft/textures/environment/{name}.png"));
            Ok(render::png::decode_rgba(&std::fs::read(path)?)
                .map_err(|error| format!("{name}: {error:?}"))?)
        };
    backend.load_weather_textures(&environment_texture("rain")?, &environment_texture("snow")?)?;
    let mut cloud_key = None;
    // The reference seeds this source uniquely per run.
    let mut flicker_random = voxel_client::random::RandomSource::create(
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
    let mut camera = voxel_client::camera::Camera::new();
    let mut captured = script.is_none();
    surface.set_cursor_captured(captured);
    let mut last_cursor: Option<(f64, f64)> = None;
    let tick_length = Duration::from_millis(50);
    let mut accumulated = Duration::ZERO;
    let mut last_frame = std::time::Instant::now();
    let mut ticks = 0_u64;
    let mut frames = 0_u64;
    // `VOXEL_PROFILE` prints where the frame time went, every two seconds.
    let profile = env::var_os("VOXEL_PROFILE").is_some();
    let mut profile_spans = [Duration::ZERO; 5];
    let mut profile_frames = 0_u32;
    let mut profile_started = std::time::Instant::now();
    let mut mesh_stats = (0_u32, Duration::ZERO);
    let mut mesh_counts: std::collections::HashMap<(i32, i32, i32), u32> =
        std::collections::HashMap::new();

    while !surface.should_close() {
        let frame_start = std::time::Instant::now();
        for event in surface.poll() {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    if captured {
                        captured = false;
                        surface.set_cursor_captured(false);
                        keys = HeldKeys::default();
                    } else {
                        surface.request_close();
                    }
                }
                glfw::WindowEvent::Key(glfw::Key::F11, _, glfw::Action::Press, _) => {
                    surface.toggle_fullscreen();
                }
                glfw::WindowEvent::Key(key, _, action, _) if captured => {
                    keys.set(key, action != glfw::Action::Release);
                }
                glfw::WindowEvent::MouseButton(_, glfw::Action::Press, _) if !captured => {
                    captured = true;
                    last_cursor = None;
                    surface.set_cursor_captured(true);
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
        // A respawn or a change of dimension: nothing of the old level stays.
        if session.take_level_reset() {
            backend.clear_sections()?;
            pending.clear();
            latest.clear();
            fades.clear();
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
        let (y_rot, x_rot) = player.rotation();
        let eye = [
            (feet.x - f64::from(origin[0])) as f32,
            (feet.y + f64::from(camera.eye_height(partial as f32)) - f64::from(origin[1])) as f32,
            (feet.z - f64::from(origin[2])) as f32,
        ];
        let fov = camera.fov(partial as f32, player.is_eye_in_water());
        let eye_in_water = player.is_eye_in_water();
        // The camera's far plane: four times the render distance, or the
        // cloud range if that is farther.
        let render_distance = session.view_distance().clamp(2, 12);
        let far = ((render_distance * 16 * 4) as f32).max((CLOUD_RANGE * 16) as f32);
        // The view bob option's default is on.
        let (walked, bob) = player.view_bob(partial as f32);
        let bob = voxel_client::camera::view_bob(walked, bob);
        let view_projection = player_camera(aspect, fov, far, eye, y_rot, x_rot, bob);

        // Mesh the dirty sections nearest the player first on the workers,
        // and drop the ones whose chunk went away.
        for (chunk_x, chunk_z) in session.take_forgotten_chunks() {
            for section_y in section_low..section_high {
                let key = (chunk_x, section_y, chunk_z);
                pending.remove(&key);
                latest.remove(&key);
                fades.remove(&key);
                backend.set_section(
                    key,
                    &render::Mesh::default(),
                    [0.0; 3],
                    SectionFade::none(),
                )?;
            }
        }
        // `LevelRenderer.compileSections`: a section fades in from the fog
        // the first time it shows, unless it is near the camera; its later
        // rebuilds keep that first time.
        let camera_block = [
            feet.x.floor() as i64,
            (feet.y + f64::from(camera.eye_height(partial as f32))).floor() as i64,
            feet.z.floor() as i64,
        ];
        let mut fade_for = |key: (i32, i32, i32)| {
            *fades.entry(key).or_insert_with(|| {
                let centre = [key.0, key.1, key.2].map(|value| i64::from(value) * 16 + 8);
                let distance: i64 = (0..3)
                    .map(|axis| (centre[axis] - camera_block[axis]).pow(2))
                    .sum();
                SectionFade {
                    since: std::time::Instant::now(),
                    length: if distance < 768 {
                        Duration::ZERO
                    } else {
                        chunk_fade
                    },
                }
            })
        };
        let dirtied = std::time::Instant::now();
        pending.extend(
            session
                .take_dirty_sections()
                .into_iter()
                .map(|key| (key, dirtied)),
        );
        loop {
            for built in workers.collect() {
                mesh_stats.0 += 1;
                mesh_stats.1 += built.took;
                *mesh_counts.entry(built.key).or_insert(0_u32) += 1;
                if latest.get(&built.key) == Some(&built.generation) {
                    latest.remove(&built.key);
                    let fade = fade_for(built.key);
                    backend.set_section(built.key, &built.mesh, built.centre, fade)?;
                }
            }
            let world = session.world();
            let player_section = (
                feet.x.floor() as i32 >> 4,
                feet.y.floor() as i32 >> 4,
                feet.z.floor() as i32 >> 4,
            );
            let view = session.view_distance().clamp(2, 12);
            // A section away from the camera waits until the chunks around it
            // have arrived, so it is built once rather than again for every
            // neighbour (or, at the edge of what the server sends, until it
            // has waited long enough); one near the camera, where the player
            // changes blocks, is built at once.
            let now = std::time::Instant::now();
            let mut ordered: Vec<(i32, i32, i32)> = pending
                .iter()
                .filter(|&(&(x, y, z), &dirtied)| {
                    let near = [x, y, z]
                        .iter()
                        .zip(camera_block)
                        .map(|(&section, block)| (i64::from(section) * 16 + 8 - block).pow(2))
                        .sum::<i64>()
                        < 768;
                    (x - player_section.0).abs() <= view
                        && (z - player_section.2).abs() <= view
                        && (near
                            || script_done
                            || now - dirtied >= SETTLE
                            || (-1..=1)
                                .all(|dx| (-1..=1).all(|dz| world.contains_chunk(x + dx, z + dz))))
                })
                .map(|(&key, _)| key)
                .collect();
            ordered.sort_by_key(|&(x, y, z)| {
                (x - player_section.0).pow(2)
                    + (y - player_section.1).pow(2)
                    + (z - player_section.2).pow(2)
            });
            // Keep every worker busy with a little queued behind it, and
            // spend at most a few milliseconds of the frame copying.
            let budget = std::time::Instant::now();
            let nothing_to_hand_out = ordered.is_empty();
            for key in ordered {
                if workers.in_flight() >= worker_count * 4
                    || budget.elapsed() > Duration::from_millis(4)
                {
                    break;
                }
                pending.remove(&key);
                if world.section_is_empty(key.0, key.1, key.2) {
                    latest.remove(&key);
                    // An empty section has shown too, so what is built in it
                    // later does not fade.
                    let fade = fade_for(key);
                    backend.set_section(key, &render::Mesh::default(), [0.0; 3], fade)?;
                    continue;
                }
                let (min, size, quart_min, quart_size) =
                    render::section_workers::SectionWorkers::region(key);
                let copy = world.copy_region(min, size, quart_min, quart_size);
                next_generation += 1;
                latest.insert(key, next_generation);
                workers.submit(key, next_generation, copy, origin);
            }
            // A scripted run finishes every section before its capture.
            if !script_done || (nothing_to_hand_out && workers.in_flight() == 0) {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
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
                regions.push(render::vulkan::AtlasRegion {
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
            let world = session.world();
            let block = [
                feet.x.floor() as i32,
                (feet.y + f64::from(camera.eye_height(partial as f32))).floor() as i32,
                feet.z.floor() as i32,
            ];
            let dimension = world.dimension();
            let quart_min = dimension.min_y.div_euclid(4);
            let quart_max = quart_min + dimension.height / 4 - 1;
            let noise_biome = |x: i32, y: i32, z: i32| {
                world.biome_at(x * 4, y.clamp(quart_min, quart_max) * 4, z * 4)
            };
            let biome = biome_tints.biome_at(block[0], block[1], block[2], &noise_biome);
            let precipitation = environment.has_precipitation(i32::try_from(biome).ok());
            let sky_light = f32::from(world.sky_brightness(block[0], block[1], block[2]));
            let target = session.rain_level()
                * ((sky_light - 8.0) / 7.0).clamp(0.0, 1.0)
                * if precipitation { 1.0 } else { 0.5 };
            rain_fog += (target - rain_fog) * frame_ticks * 0.2;
        }
        // Rain and snow around the camera.
        {
            let world = session.world();
            let dimension = world.dimension();
            let quart_min = dimension.min_y.div_euclid(4);
            let quart_max = quart_min + dimension.height / 4 - 1;
            let noise_biome = |x: i32, y: i32, z: i32| {
                world.biome_at(x * 4, y.clamp(quart_min, quart_max) * 4, z * 4)
            };
            let sea_level = world.sea_level();
            let height = |x: i32, z: i32| world.motion_blocking_height(x, z);
            let precipitation = |x: i32, y: i32, z: i32| {
                if !world.contains_chunk(x.div_euclid(16), z.div_euclid(16)) {
                    return render::biome_tint::Precipitation::None;
                }
                biome_tints.precipitation_at([x, y, z], &noise_biome, sea_level)
            };
            let lighter = render::block_light::Lighter {
                state: |[x, y, z]: [i32; 3]| {
                    world
                        .block_state_at(x, y, z)
                        .and_then(|state| u32::try_from(state).ok())
                },
                light: |[x, y, z]: [i32; 3]| {
                    (
                        world.block_brightness(x, y, z),
                        world.sky_brightness(x, y, z),
                    )
                },
            };
            let light_coords = |x: i32, y: i32, z: i32| {
                lighter.light_coords((lighter.state)([x, y, z]), [x, y, z])
            };
            let weather = render::weather::build(
                &render::weather::WeatherWorld {
                    height: &height,
                    precipitation: &precipitation,
                    light_coords: &light_coords,
                },
                [
                    feet.x,
                    feet.y + f64::from(camera.eye_height(partial as f32)),
                    feet.z,
                ],
                origin,
                session.game_time(),
                partial as f32,
                session.rain_level(),
            );
            backend.set_weather(&weather.rain, &weather.snow)?;
        }
        let look = voxel_client::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
        let sky = render::sky::build_frame(
            &attributes,
            &render::sky::SkyView {
                forward: [look.x as f32, look.y as f32, look.z as f32],
                eye_above_horizon: feet.y + f64::from(camera.eye_height(partial as f32)) - 63.0,
                eye_in_water,
                render_distance,
                cloud_range: CLOUD_RANGE,
                weather: render::sky::Weather {
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
            let camera_position = [
                feet.x,
                feet.y + f64::from(camera.eye_height(partial as f32)),
                feet.z,
            ];
            let placement = render::clouds::place(
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
                    render::clouds::radius_cells(CLOUD_RANGE),
                );
                backend.set_cloud_faces(&faces)?;
            }
            backend.set_cloud_frame(Some(render::vulkan::CloudFrame {
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
                write_ppm(&numbered, &captured)?;
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
            if let Some(warning) = session.warning() {
                println!("play_warning={warning}");
            }
            if let Ok(path) = env::var("VOXEL_CAPTURE_PPM") {
                let captured = backend.capture_scene(colour, view_projection)?;
                write_ppm(&path, &captured)?;
                println!("captured_frame={path}");
            }
            break;
        }
        let built = std::time::Instant::now();
        backend.draw_scene(colour, view_projection)?;
        frames += 1;
        if profile {
            let drawn = std::time::Instant::now();
            for (span, (from, to)) in profile_spans.iter_mut().zip([
                (frame_start, ticked),
                (ticked, meshed),
                (meshed, built),
                (built, drawn),
                (frame_start, drawn),
            ]) {
                *span += to - from;
            }
            profile_frames += 1;
            if profile_started.elapsed() >= Duration::from_secs(2) {
                let per_frame =
                    |span: Duration| span.as_secs_f64() * 1000.0 / f64::from(profile_frames);
                println!(
                    "profile fps={:.0} ms/frame: tick={:.2} mesh={:.2} build={:.2} draw={:.2} total={:.2} pending={}",
                    f64::from(profile_frames) / profile_started.elapsed().as_secs_f64(),
                    per_frame(profile_spans[0]),
                    per_frame(profile_spans[1]),
                    per_frame(profile_spans[2]),
                    per_frame(profile_spans[3]),
                    per_frame(profile_spans[4]),
                    pending.len(),
                );
                if mesh_stats.0 > 0 {
                    let repeated = mesh_counts.values().filter(|count| **count > 1).count();
                    println!(
                        "profile meshes={} avg_ms={:.2} worker_ms={:.0} sections={} rebuilt={}",
                        mesh_stats.0,
                        mesh_stats.1.as_secs_f64() * 1000.0 / f64::from(mesh_stats.0),
                        mesh_stats.1.as_secs_f64() * 1000.0,
                        mesh_counts.len(),
                        repeated,
                    );
                }
                mesh_stats = (0, Duration::ZERO);
                profile_spans = [Duration::ZERO; 5];
                profile_frames = 0;
                profile_started = std::time::Instant::now();
            }
        }
    }
    println!("play_frames={frames} play_ticks={ticks}");
    Ok(())
}
