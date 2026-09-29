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

    fn item(&self, name: &voxel_client::identifier::Identifier) -> Option<String> {
        std::fs::read_to_string(self.path(name, "items", ".json")).ok()
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
    /// A scripted attack click on this tick.
    attack: bool,
    /// The attack key held down on this tick.
    mine: bool,
    /// A scripted use click on this tick.
    use_click: bool,
    /// The use key held down on this tick.
    hold_use: bool,
    /// A scripted press of the inventory key on this tick.
    inventory: bool,
    /// A scripted click in the open inventory: slot, button, click type.
    click: Option<(i32, i32, i32)>,
    /// A scripted tab choice on the open creative screen, by table index.
    tab: Option<usize>,
    /// A scripted scroll of the creative screen, in hundredths.
    scroll: Option<i32>,
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
                "attack" => keys.attack = true,
                "mine" => keys.mine = true,
                "use" => keys.use_click = true,
                "hold_use" => keys.hold_use = true,
                "inventory" => keys.inventory = true,
                "none" => {}
                other => {
                    // `click/SLOT/BUTTON/TYPE`, the type by its id.
                    let parts: Vec<&str> = other.split('/').collect();
                    match parts.as_slice() {
                        ["click", slot, button, kind] => {
                            let number = |text: &str| {
                                text.parse::<i32>()
                                    .map_err(|_| format!("bad click {other} in VOXEL_SCRIPT"))
                            };
                            keys.click = Some((number(slot)?, number(button)?, number(kind)?));
                        }
                        ["tab", tab] => {
                            keys.tab = Some(
                                tab.parse()
                                    .map_err(|_| format!("bad tab {other} in VOXEL_SCRIPT"))?,
                            );
                        }
                        ["scroll", hundredths] => {
                            keys.scroll = Some(
                                hundredths
                                    .parse()
                                    .map_err(|_| format!("bad scroll {other} in VOXEL_SCRIPT"))?,
                            );
                        }
                        _ => return Err(format!("unknown key {other} in VOXEL_SCRIPT")),
                    }
                }
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
/// `GameRenderer.pick` for blocks: the outline seen from `eye` along the
/// view within reach (4.5 blocks, 5 in creative), or a miss. Entities are
/// not picked.
fn pick(
    world: &voxel_client::world::ClientWorld,
    player: &voxel_client::world::local_player::LocalPlayer,
    eye: voxel_client::math::Vec3,
) -> voxel_client::game_mode::Aim {
    let reach = if player.abilities().instabuild() {
        5.0
    } else {
        4.5
    };
    let (y_rot, x_rot) = player.rotation();
    let look = voxel_client::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
    let to = voxel_client::math::Vec3::new(
        eye.x + look.x * reach,
        eye.y + look.y * reach,
        eye.z + look.z * reach,
    );
    match world.clip_outlines(eye, to) {
        // `filterHitResult`: a hit past the reach is a miss.
        Some(hit) if hit.location.distance_to_sqr(eye) < reach * reach => {
            voxel_client::game_mode::Aim::Block {
                x: hit.block_pos.x,
                y: hit.block_pos.y,
                z: hit.block_pos.z,
                face: hit.direction.data3d() as u8,
                location: [hit.location.x, hit.location.y, hit.location.z],
                inside: hit.inside,
            }
        }
        _ => voxel_client::game_mode::Aim::Miss,
    }
}

/// `Entity.getEyePosition` between ticks.
fn eye_position(
    player: &voxel_client::world::local_player::LocalPlayer,
    partial: f64,
) -> voxel_client::math::Vec3 {
    let feet = player.interpolated_position(partial);
    voxel_client::math::Vec3::new(feet.x, feet.y + f64::from(player.eye_height()), feet.z)
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

/// Where the camera looks from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CameraType {
    FirstPerson,
    ThirdPersonBack,
    ThirdPersonFront,
}

impl CameraType {
    /// `CameraType.cycle`.
    fn next(self) -> Self {
        match self {
            Self::FirstPerson => Self::ThirdPersonBack,
            Self::ThirdPersonBack => Self::ThirdPersonFront,
            Self::ThirdPersonFront => Self::FirstPerson,
        }
    }
}

/// `Camera.getMaxZoom`: how far the camera can back away from `eye` along
/// `forwards` before a block is in the way, tried from the eight corners of
/// a small box around the eye.
fn max_camera_zoom(
    world: &voxel_client::world::ClientWorld,
    eye: voxel_client::math::Vec3,
    forwards: voxel_client::math::Vec3,
    mut distance: f32,
) -> f32 {
    use voxel_client::math::Vec3;
    for corner in 0..8 {
        let offset = |bit: i32| (((corner >> bit) & 1) * 2 - 1) as f32 * 0.1;
        let from = Vec3::new(
            eye.x + f64::from(offset(0)),
            eye.y + f64::from(offset(1)),
            eye.z + f64::from(offset(2)),
        );
        let reach = -f64::from(distance);
        let to = Vec3::new(
            from.x + forwards.x * reach,
            from.y + forwards.y * reach,
            from.z + forwards.z * reach,
        );
        if let Some(hit) = world.clip_blocks(from, to) {
            let distance_squared = hit.distance_to_sqr(eye) as f32;
            if distance_squared < distance * distance {
                distance = f64::from(distance_squared).sqrt() as f32;
            }
        }
    }
    distance
}

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
    let forward = [look.x as f32, look.y as f32, look.z as f32];
    // `Camera.setRotation`: the camera's side vector turns with the yaw
    // alone, so looking straight up or down keeps a well-defined view.
    let yaw = y_rot.to_radians();
    let right = [-yaw.cos(), 0.0, -yaw.sin()];
    let up = [
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    ];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let view = [
        [right[0], up[0], -forward[0], 0.0],
        [right[1], up[1], -forward[1], 0.0],
        [right[2], up[2], -forward[2], 0.0],
        [-dot(right, eye), -dot(up, eye), dot(forward, eye), 1.0],
    ];
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
/// The open container screen: the survival inventory or the creative one.
enum Screen {
    Survival(render::inventory_screen::InventoryScreen),
    Creative(Box<render::creative_screen::CreativeScreen>),
}

/// A click a screen asks for.
enum Action {
    Container(render::inventory_screen::SlotClick),
    Creative(voxel_client::creative_menu::Click),
}

/// What a key did to a screen.
enum ScreenKey {
    Close,
    Actions(Vec<Action>),
}

fn container_actions(clicks: Vec<render::inventory_screen::SlotClick>) -> Vec<Action> {
    clicks.into_iter().map(Action::Container).collect()
}

fn creative_actions(clicks: Vec<voxel_client::creative_menu::Click>) -> Vec<Action> {
    clicks.into_iter().map(Action::Creative).collect()
}

impl Screen {
    fn resize(&mut self, width: i32, height: i32) {
        match self {
            Self::Survival(screen) => screen.resize(width, height),
            Self::Creative(screen) => screen.resize(width, height),
        }
    }

    fn key_pressed(&mut self, key: glfw::Key, control: bool, menu: &InventoryMenu) -> ScreenKey {
        match self {
            Self::Survival(screen) => match screen.key_pressed(key, control, menu) {
                render::inventory_screen::KeyResult::Close => ScreenKey::Close,
                render::inventory_screen::KeyResult::Clicks(clicks) => {
                    ScreenKey::Actions(container_actions(clicks))
                }
            },
            Self::Creative(screen) => match screen.key_pressed(key, control, menu) {
                render::creative_screen::KeyResult::Close => ScreenKey::Close,
                render::creative_screen::KeyResult::Clicks(clicks) => {
                    ScreenKey::Actions(creative_actions(clicks))
                }
            },
        }
    }

    fn mouse_clicked(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        menu: &InventoryMenu,
    ) -> Vec<Action> {
        let now = std::time::Instant::now();
        match self {
            Self::Survival(screen) => {
                container_actions(screen.mouse_clicked(mouse.0, mouse.1, button, shift, now, menu))
            }
            Self::Creative(screen) => {
                creative_actions(screen.mouse_clicked(mouse.0, mouse.1, button, shift, now, menu))
            }
        }
    }

    fn mouse_released(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        menu: &InventoryMenu,
    ) -> Vec<Action> {
        match self {
            Self::Survival(screen) => {
                container_actions(screen.mouse_released(mouse.0, mouse.1, button, shift, menu))
            }
            Self::Creative(screen) => {
                creative_actions(screen.mouse_released(mouse.0, mouse.1, button, shift, menu))
            }
        }
    }

    fn mouse_dragged(&mut self, mouse: (f64, f64), menu: &InventoryMenu) {
        match self {
            Self::Survival(screen) => screen.mouse_dragged(mouse.0, mouse.1, menu),
            Self::Creative(screen) => screen.mouse_dragged(mouse.0, mouse.1, menu),
        }
    }

    fn mouse_scrolled(&mut self, y: f64) {
        if let Self::Creative(screen) = self {
            screen.mouse_scrolled(y);
        }
    }
}

use voxel_client::inventory_menu::InventoryMenu;

/// `Minecraft.handleKeybinds` for the inventory key: the survival or the
/// creative inventory screen opens and the mouse is let go, at the window's
/// centre. The creative screen opens on the tab it was left on.
fn open_inventory(
    surface: &mut WindowSurface,
    screen: &mut Option<Screen>,
    creative: bool,
    creative_tab: usize,
) {
    let size = surface.scale();
    let (width, height) = (size.gui_scaled_width(), size.gui_scaled_height());
    *screen = Some(if creative {
        Screen::Creative(Box::new(render::creative_screen::CreativeScreen::new(
            width,
            height,
            creative_tab,
        )))
    } else {
        Screen::Survival(render::inventory_screen::InventoryScreen::new(
            width, height,
        ))
    });
    surface.set_cursor_captured(false);
    let (width, height) = surface.window_size();
    surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
}

/// `AbstractContainerScreen.onClose`: the container closes and the mouse is
/// grabbed again. The creative screen's tab is kept for the next time.
fn close_inventory(
    session: &mut PlaySession,
    surface: &mut WindowSurface,
    screen: &mut Option<Screen>,
    creative_tab: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(Screen::Creative(open)) = screen.as_ref() {
        *creative_tab = open.tab();
    }
    *screen = None;
    session.close_inventory()?;
    surface.set_cursor_captured(true);
    Ok(())
}

/// The screen's clicks, one after another.
fn run_actions(
    session: &mut PlaySession,
    screen: &mut Option<Screen>,
    actions: Vec<Action>,
) -> Result<(), Box<dyn std::error::Error>> {
    for action in actions {
        match (action, screen.as_mut()) {
            (Action::Container(click), _) => {
                session.container_click(click.slot, click.button, click.kind)?;
            }
            (Action::Creative(click), Some(Screen::Creative(open))) => {
                session.creative_click(open.menu_mut(), click)?;
            }
            (Action::Creative(_), _) => {}
        }
    }
    Ok(())
}

/// The default language's texts for the keys asked, in order; a key the
/// language lacks reads as itself.
fn lang_texts(root: &str, keys: &[&str]) -> Vec<String> {
    let object =
        std::fs::read_to_string(std::path::Path::new(root).join("minecraft/lang/en_us.json"))
            .ok()
            .and_then(|text| voxel_client::json::parse(&text).ok())
            .and_then(|value| match value {
                voxel_client::json::JsonValue::Object(object) => Some(object),
                _ => None,
            });
    keys.iter()
        .map(|key| {
            object
                .as_ref()
                .and_then(|object| object.text(key))
                .unwrap_or_else(|| (*key).to_owned())
        })
        .collect()
}

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
    // `ModelBakery.DESTROY_STAGES`, stacked into one image.
    let mut cracks = render::DecodedImage {
        width: 0,
        height: 0,
        rgba: Vec::new(),
    };
    for stage in 0..10 {
        let path = std::path::Path::new(&root).join(format!(
            "minecraft/textures/block/destroy_stage_{stage}.png"
        ));
        let image = render::png::decode_rgba(&std::fs::read(path)?)
            .map_err(|error| format!("destroy_stage_{stage}: {error:?}"))?;
        if stage > 0 && (image.width, image.height) != (cracks.width, cracks.height / stage) {
            return Err(format!("destroy_stage_{stage}: stages differ in size").into());
        }
        cracks.width = image.width;
        cracks.height += image.height;
        cracks.rgba.extend_from_slice(&image.rgba);
    }
    backend.load_crack_texture(&cracks)?;
    // The player's default skin, picked by its profile id as the reference
    // picks one when the profile has no skin of its own.
    let skin = render::player_renderer::default_skin(
        session
            .profile_id()
            .unwrap_or(voxel_client::binary::Uuid { most: 0, least: 0 }),
    );
    backend.load_entity_texture(
        &render::png::decode_rgba(&std::fs::read(
            std::path::Path::new(&root).join(format!("minecraft/textures/{}.png", skin.texture)),
        )?)
        .map_err(|error| format!("skin: {error:?}"))?,
    )?;
    let mut player_model = render::player_model::PlayerModel::new(skin.slim);
    // The HUD: its sprites, its own state, and the wheel's partial steps.
    let font = render::font::Font::load(std::path::Path::new(&root))
        .ok_or("the default font could not be read")?;
    let gui_atlas = render::gui::GuiAtlas::load(std::path::Path::new(&root), &font)
        .ok_or("the HUD sprites could not be read")?;
    backend.load_gui_texture(&gui_atlas.image)?;
    let mut hud = render::gui::Hud::default();
    // Each item's GUI layers, built the first time a slot shows it.
    let mut gui_item_layers =
        std::collections::HashMap::<i32, Vec<render::world_mesh::ItemLayer>>::new();
    let mut scroll_wheel = voxel_client::hotbar::ScrollWheel::default();
    let started_at = std::time::Instant::now();
    // `ItemInHandRenderer`: the stacks the hands show and how high they are.
    let mut item_in_hand = voxel_client::item_in_hand::ItemInHand::default();
    // Each item's first-person layers, by item and hand.
    let mut held_item_layers =
        std::collections::HashMap::<(i32, bool), Vec<render::world_mesh::ItemLayer>>::new();
    // Attack clicks waiting for the next tick, and `Minecraft.missTime`:
    // ticks a missed attack keeps the next one waiting.
    let mut attack_key = voxel_client::attack::AttackKey::default();
    // The use key keeps its clicks and held state the same way.
    let mut use_key = voxel_client::attack::AttackKey::default();
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
    let mut inventory_screen: Option<Screen> = None;
    let mut creative_tab = voxel_client::creative_menu::default_tab();
    let mut gui_mouse = (0.0_f64, 0.0_f64);
    let mut active_button: Option<i32> = None;
    let inventory_title = lang_texts(&root, &["container.crafting"]).remove(0);
    let tab_titles = lang_texts(
        &root,
        &voxel_client::creative_tabs_table::CREATIVE_TABS
            .iter()
            .map(|tab| tab.title)
            .collect::<Vec<_>>(),
    );
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
                    match screen.key_pressed(
                        key,
                        modifiers.contains(glfw::Modifiers::Control),
                        &menu,
                    ) {
                        ScreenKey::Close => {
                            close_inventory(
                                &mut session,
                                &mut surface,
                                &mut inventory_screen,
                                &mut creative_tab,
                            )?;
                            captured = true;
                            last_cursor = None;
                        }
                        ScreenKey::Actions(actions) => {
                            run_actions(&mut session, &mut inventory_screen, actions)?;
                        }
                    }
                }
                glfw::WindowEvent::Key(..) if inventory_screen.is_some() => {}
                glfw::WindowEvent::MouseButton(button, action, modifiers)
                    if inventory_screen.is_some() =>
                {
                    let button = button as i32;
                    let shift = modifiers.contains(glfw::Modifiers::Shift);
                    let menu = session.inventory_menu().clone();
                    let screen = inventory_screen.as_mut().expect("checked above");
                    let actions = if action == glfw::Action::Press {
                        active_button = Some(button);
                        screen.mouse_clicked(gui_mouse, button, shift, &menu)
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
                        session.game_mode() == voxel_client::world::ClientPlayerGameMode::Creative,
                        creative_tab,
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
        if let Some(open) = inventory_screen.as_ref() {
            let creative =
                session.game_mode() == voxel_client::world::ClientPlayerGameMode::Creative;
            let size = surface.scale();
            let (width, height) = (size.gui_scaled_width(), size.gui_scaled_height());
            match open {
                Screen::Survival(_) if creative => {
                    inventory_screen = Some(Screen::Creative(Box::new(
                        render::creative_screen::CreativeScreen::new(width, height, creative_tab),
                    )));
                }
                Screen::Creative(open) if !creative => {
                    creative_tab = open.tab();
                    inventory_screen = Some(Screen::Survival(
                        render::inventory_screen::InventoryScreen::new(width, height),
                    ));
                }
                _ => {}
            }
        }
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
                            session.game_mode()
                                == voxel_client::world::ClientPlayerGameMode::Creative,
                            creative_tab,
                        );
                    }
                }
                if let Some((slot, button, kind)) = held.click {
                    let kind = [
                        voxel_client::inventory_menu::ClickType::Pickup,
                        voxel_client::inventory_menu::ClickType::QuickMove,
                        voxel_client::inventory_menu::ClickType::Swap,
                        voxel_client::inventory_menu::ClickType::Clone,
                        voxel_client::inventory_menu::ClickType::Throw,
                        voxel_client::inventory_menu::ClickType::QuickCraft,
                        voxel_client::inventory_menu::ClickType::PickupAll,
                    ][kind.clamp(0, 6) as usize];
                    match inventory_screen.as_mut() {
                        Some(Screen::Creative(screen)) => {
                            let click = screen.scripted_click(slot, button, kind);
                            run_actions(
                                &mut session,
                                &mut inventory_screen,
                                vec![Action::Creative(click)],
                            )?;
                        }
                        _ => session.container_click(slot, button, kind)?,
                    }
                }
                if let Some(Screen::Creative(screen)) = inventory_screen.as_mut() {
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
                .map_or(voxel_client::game_mode::Aim::Miss, |player| {
                    pick(session.world(), player, eye_position(player, 1.0))
                });
            let grabbed = captured || script.is_some();
            session.keys(
                voxel_client::game_mode::Keys {
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
                let attack_speed = local
                    .and_then(|state| {
                        state
                            .attributes()
                            .iter()
                            .find(|attribute| attribute.attribute_id() == 5)
                    })
                    .map_or(4.0, |attribute| attribute.effective_value(0.0, 1024.0));
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
        let eye_world = voxel_client::math::Vec3::new(
            feet.x,
            feet.y + f64::from(camera.eye_height(partial as f32)),
            feet.z,
        );
        let camera_world = if camera_type == CameraType::FirstPerson {
            eye_world
        } else {
            let forwards =
                voxel_client::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
            let zoom = max_camera_zoom(session.world(), eye_world, forwards, 4.0);
            voxel_client::math::Vec3::new(
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
                voxel_client::game_mode::MAIN_HAND,
                partial as f32,
                &swap,
            );
            let off_height = item_in_hand.inverse_arm_height(
                voxel_client::game_mode::OFF_HAND,
                partial as f32,
                &swap,
            );
            let view = render::player_renderer::HandView {
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
            let main_item = item_in_hand
                .item(voxel_client::game_mode::MAIN_HAND)
                .map(|stack| stack.item_registry_id());
            let off_item = item_in_hand
                .item(voxel_client::game_mode::OFF_HAND)
                .map(|stack| stack.item_registry_id());
            // An empty main hand shows the arm; each stack is drawn in its
            // hand, the main hand on the right.
            let hand = if main_item.is_none() {
                render::player_renderer::hand_vertices(&mut player_model, &view)
            } else {
                Vec::new()
            };
            let mut held = Vec::new();
            for (item, right, attack, height) in [
                (main_item, true, view.attack, main_height),
                (off_item, false, 0.0, off_height),
            ] {
                let Some(item) = item else {
                    continue;
                };
                let layers = held_item_layers.entry((item, right)).or_insert_with(|| {
                    block_renderer.item_layers(
                        item,
                        if right {
                            render::model::DisplayContext::FirstPersonRightHand
                        } else {
                            render::model::DisplayContext::FirstPersonLeftHand
                        },
                        &|temperature, downfall| {
                            biome_tints.grass_at_climate(temperature, downfall)
                        },
                    )
                });
                held.extend(render::player_renderer::held_item_vertices(
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
                render::scene::perspective(f32::to_radians(hud_fov), aspect, 0.05, 100.0);
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
            voxel_client::world::ClientPlayerGameMode::Adventure
                | voxel_client::world::ClientPlayerGameMode::Spectator
        ) && let voxel_client::game_mode::Aim::Block { x, y, z, .. } =
            pick(session.world(), player, eye_position(player, partial))
            && let Some(state) = session.world().block_state_at(x, y, z)
        {
            let shape = voxel_client::world::outline_shape(state as u32, [x, y, z]);
            outline = render::outline::outline_vertices(
                &shape,
                [
                    (x - origin[0]) as f32,
                    (y - origin[1]) as f32,
                    (z - origin[2]) as f32,
                ],
                eye,
                render::outline::OUTLINE_COLOUR,
                render::outline::appropriate_line_width(backend.extent().0),
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
            breaking.extend(render::breaking::breaking_vertices(
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
            player_model.setup_anim(&render::player_model::PlayerRenderState {
                head_y_rot: voxel_client::math::wrap_degrees_f32(
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
            let visually_swimming =
                model_pose.pose == voxel_client::world::ClientEntityPose::Swimming;
            backend.set_entities(&render::player_renderer::player_vertices(
                &player_model,
                &render::player_renderer::Placement {
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
        // `ClientLevel.update`: queued light and the light engine, each frame.
        session.update_light();
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
            println!("play_swings={}", session.swings());
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
        // The HUD, in GUI units.
        {
            let size = surface.scale();
            let status = *session.status();
            let game_mode = session.game_mode();
            // What the HUD reads from the player: its attributes (the
            // registry's ids for max health, armour and attack speed), its
            // effects (regeneration, hunger, poison, wither) and its data.
            let world = session.world();
            let local = world.local_player();
            let attribute = |id: i32, default: f64, min: f64, max: f64| {
                local
                    .and_then(|state| {
                        state
                            .attributes()
                            .iter()
                            .find(|attribute| attribute.attribute_id() == id)
                    })
                    .map_or(default, |attribute| attribute.effective_value(min, max))
            };
            let has_effect = |id: i32| {
                local.is_some_and(|state| {
                    state
                        .effects()
                        .iter()
                        .any(|effect| effect.effect_id() == id)
                })
            };
            let hud_player = session.player();
            let stack = |slot: i16| {
                local
                    .and_then(|state| state.inventory_slot(slot))
                    .filter(|stack| stack.item_count() > 0 && stack.item_registry_id() != 0)
                    .map(|stack| (stack.item_registry_id(), stack.item_count()))
            };
            let mut draw_item =
                |item: i32, x: f32, y: f32, out: &mut Vec<render::gui_items::GuiItemVertex>| {
                    let layers = gui_item_layers.entry(item).or_insert_with(|| {
                        block_renderer.item_layers(
                            item,
                            render::model::DisplayContext::Gui,
                            &|temperature, downfall| {
                                biome_tints.grass_at_climate(temperature, downfall)
                            },
                        )
                    });
                    render::gui_items::item_vertices(layers, x, y, out);
                };
            let frame = hud.build(
                &gui_atlas,
                &font,
                size.gui_scaled_width(),
                size.gui_scaled_height(),
                &render::gui::HudInput {
                    first_person: camera_type == CameraType::FirstPerson,
                    spectator: game_mode == voxel_client::world::ClientPlayerGameMode::Spectator,
                    survival: game_mode.is_survival(),
                    selected_slot: status.selected_slot,
                    health: hud_player.map_or(status.health, |player| player.health()),
                    max_health: attribute(23, 20.0, 1.0, 1024.0) as f32,
                    absorption: local.map_or(0.0, |state| state.absorption()),
                    invulnerable_time: hud_player.map_or(0, |player| player.invulnerable_time()),
                    heart_type: render::gui::HeartType::for_player(
                        has_effect(18),
                        has_effect(19),
                        local.is_some_and(|state| state.is_fully_frozen()),
                    ),
                    regeneration: has_effect(9),
                    hardcore: session.hardcore(),
                    armor: attribute(1, 0.0, 0.0, 30.0).floor() as i32,
                    food: status.food,
                    saturation: status.saturation,
                    hunger: has_effect(16),
                    air_supply: local.map_or(300, |state| state.air_supply()),
                    max_air_supply: 300,
                    eye_in_water: hud_player.is_some_and(|player| player.is_eye_in_water()),
                    experience_progress: status.experience_progress,
                    experience_level: status.experience_level,
                    attack_strength: hud_player.map_or(1.0, |player| {
                        player.attack_strength_scale(0.0, attribute(5, 4.0, 0.0, 1024.0))
                    }),
                    hotbar: std::array::from_fn(|i| stack(36 + i as i16)),
                    offhand: stack(45),
                },
                started_at.elapsed().as_millis(),
                &mut draw_item,
            );
            // The open screen over the HUD.
            let mut frame = frame;
            if let Some(screen) = inventory_screen.as_mut() {
                let menu = session.inventory_menu().clone();
                match screen {
                    Screen::Survival(screen) => screen.build(
                        &mut frame,
                        &gui_atlas,
                        &font,
                        &inventory_title,
                        &menu,
                        gui_mouse,
                        &mut draw_item,
                    ),
                    Screen::Creative(screen) => screen.build(
                        &mut frame,
                        &gui_atlas,
                        &font,
                        &tab_titles,
                        &menu,
                        gui_mouse,
                        &mut draw_item,
                    ),
                }
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
