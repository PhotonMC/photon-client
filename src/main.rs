use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_probe;
use voxel_client::render::{self, VulkanBackend};
use voxel_client::window::{DisplayData, WindowSurface};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    if env::args().nth(1).as_deref() == Some("window") {
        return run_window(env::args().skip(2));
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
            let properties =
                render::blockstate::parse_properties(&env::var("VOXEL_STATE").unwrap_or_default());
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
            ))
        }
        _ => None,
    };
    let (assets, model_name) = match &chosen {
        Some((root, model, ..)) => (Ok(root.clone()), Ok(model.clone())),
        None => (env::var("VOXEL_ASSETS"), env::var("VOXEL_MODEL")),
    };
    let (mesh, texture_levels) = match (assets, model_name) {
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
            let state =
                voxel_client::render::rotation::model_state_transform(rotate_x, rotate_y, rotate_z)
                    .ok_or("variant rotation must be a multiple of ninety")?;
            let identity = rotate_x % 360 == 0 && rotate_y % 360 == 0 && rotate_z % 360 == 0;
            // Outside a world a block's tint is a fixed value per block, so
            // the probe resolves the first slot from the block's name and
            // leaves the rest untinted.
            let tints =
                render::constant_tint(identifier.path().strip_prefix("block/").unwrap_or(""))
                    .map_or_else(Vec::new, |colour| vec![colour]);
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
                            [step_x, step_y, step_z]
                                .into_iter()
                                .enumerate()
                                .all(|(axis, step)| {
                                    let moved = cell[axis] as i32 + step;
                                    moved >= 0 && moved < grid as i32
                                })
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
    // The scene grows with the grid, so the eye pulls back with it.
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
}
