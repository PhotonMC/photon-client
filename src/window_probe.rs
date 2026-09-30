//! `voxel-client window`: the window and Vulkan probe, which draws one block,
//! a grid of them, or a box of a server's world, and can capture the frame.

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_probe;
use voxel_render::VulkanBackend;
use voxel_render::window::{DisplayData, WindowSurface};

use crate::assets::FolderAssets;
use crate::ppm;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Open the client window and report the sizing the reference rules produce.
///
/// This is the surface only: nothing is drawn yet, so the window stays blank
/// until the Vulkan backend lands. `frames` bounds the loop so the probe can be
/// run non-interactively; `0` keeps it open until the window is closed.
pub fn run_window(mut args: impl Iterator<Item = String>) -> Result<()> {
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
    let mut backend = open_backend(&surface)?;
    let (mesh, texture_levels) = scene()?;
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
    if let Ok(path) = env::var("VOXEL_CAPTURE_PPM") {
        ppm::write(&path, captured.width, captured.height, &captured.rgba)?;
        println!("captured_frame={path}");
        println!("captured_size={}x{}", captured.width, captured.height);
    }
    let rendered = show(&mut surface, &mut backend, (colour, aspect), frames)?;
    println!("rendered_frames={rendered}");
    println!("window_closed=true");
    Ok(())
}

/// Bring the Vulkan backend up against the live surface, and report the
/// device and the swapchain. Until a frame is presented a Wayland
/// compositor never maps the window, so this is also what makes it appear
/// at all.
fn open_backend(surface: &WindowSurface) -> Result<VulkanBackend> {
    let scale = surface.scale();
    let (display_handle, window_handle) = surface
        .raw_handles()
        .map_err(|error| format!("window handles unavailable: {error}"))?;
    let backend = unsafe {
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
    Ok(backend)
}

/// Draw frames, the camera orbiting, until the window closes or `frames`
/// have been drawn; how many were drawn, the captured one included.
fn show(
    surface: &mut WindowSurface,
    backend: &mut VulkanBackend,
    (colour, aspect): ([f32; 4], f32),
    frames: u64,
) -> Result<u64> {
    let mut rendered = 1_u64;
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
    Ok(rendered)
}

/// What the probe draws and its texture's mip levels.
///
/// With a server address the probe joins it, waits for the chunks around
/// the player, and draws a box of the real world. Otherwise assets come from
/// a folder the user names; nothing is bundled. With a folder and a model or
/// block name the probe draws that model; with neither it falls back to its
/// own cube and checkerboard, and says which it used.
fn scene() -> Result<(voxel_render::Mesh, Vec<voxel_render::DecodedImage>)> {
    let from_server = match (env::var("VOXEL_SERVER"), env::var("VOXEL_ASSETS")) {
        (Ok(server), Ok(root)) => Some(load_world(&server, &root)?),
        _ => None,
    };
    let chosen = match (env::var("VOXEL_BLOCK"), env::var("VOXEL_ASSETS")) {
        (Ok(block), Ok(root)) => Some(choose_block(&block, root)?),
        _ => None,
    };
    let (assets, model_name) = match &chosen {
        Some(chosen) => (Ok(chosen.root.clone()), Ok(chosen.model.clone())),
        None => (env::var("VOXEL_ASSETS"), env::var("VOXEL_MODEL")),
    };
    if let Some(loaded) = from_server {
        return Ok(loaded);
    }
    match (assets, model_name) {
        (Ok(root), Ok(name)) => model_scene(&root, &name, chosen.as_ref()),
        _ => Ok(builtin_scene()),
    }
}

/// The variant a `VOXEL_BLOCK` name and its `VOXEL_STATE` properties select.
struct ChosenBlock {
    root: String,
    model: String,
    /// Quarter turns about x, y and z, in degrees.
    rotation: [i32; 3],
    uvlock: bool,
    /// The block's name, which its fixed tints are looked up by.
    name: String,
}

/// A block name takes the longer road: its blockstate file is read, the
/// variant its properties select is taken, and the model and rotation come
/// from there rather than from the environment.
fn choose_block(block: &str, root: String) -> Result<ChosenBlock> {
    let source = FolderAssets {
        root: std::path::PathBuf::from(&root),
    };
    let name = voxel_core::identifier::Identifier::parse(block)
        .map_err(|error| format!("{block}: {error:?}"))?;
    let text = voxel_render::AssetSource::block_state(&source, &name)
        .ok_or_else(|| format!("{block}: no blockstate file"))?;
    let file = voxel_render::parse_block_state(&text)
        .ok_or_else(|| format!("{block}: blockstate file was refused"))?;
    let properties = block_properties(block, &name.as_string())?;
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
    Ok(ChosenBlock {
        root,
        model: variant.model.as_string(),
        rotation: [
            variant.x.degrees(),
            variant.y.degrees(),
            variant.z.degrees(),
        ],
        uvlock: variant.uvlock,
        name: name.as_string(),
    })
}

/// The block's properties from `VOXEL_STATE`, checked against the registry
/// so a typo is caught rather than silently picking the wrong variant, and
/// completed with the default state's values, which is what a blockstate
/// selector expects to find.
fn block_properties(block: &str, name: &str) -> Result<std::collections::BTreeMap<String, String>> {
    use voxel_data::block::registry::PROPERTIES;
    let definition = voxel_data::block::registry::block(name)
        .ok_or_else(|| format!("{block}: not in the block registry"))?;
    let mut properties =
        voxel_render::blockstate::parse_properties(&env::var("VOXEL_STATE").unwrap_or_default());
    for (key, value) in &properties {
        let property = definition
            .properties
            .iter()
            .map(|index| &PROPERTIES[*index as usize])
            .find(|property| property.name == key)
            .ok_or_else(|| format!("{block}: no property named {key}"))?;
        if !property.values.contains(&value.as_str()) {
            return Err(format!("{block}: {key} cannot be {value}").into());
        }
    }
    for (slot, index) in definition.properties.iter().enumerate() {
        let property = &PROPERTIES[*index as usize];
        properties
            .entry(property.name.to_owned())
            .or_insert_with(|| property.values[usize::from(definition.default[slot])].to_owned());
    }
    println!("block_state_id={:?}", definition.state_id(&properties));
    Ok(properties)
}

/// A model by name, turned, tinted and repeated in a grid as the
/// environment or the chosen block says, with its atlas.
fn model_scene(
    root: &str,
    name: &str,
    chosen: Option<&ChosenBlock>,
) -> Result<(voxel_render::Mesh, Vec<voxel_render::DecodedImage>)> {
    let source = FolderAssets {
        root: std::path::PathBuf::from(root),
    };
    let identifier = voxel_core::identifier::Identifier::parse(name)
        .map_err(|error| format!("{name}: {error:?}"))?;
    let model = voxel_render::resolve_model(&source, &identifier)
        .map_err(|error| format!("{name}: {error:?}"))?;
    // The reference's own default is four levels, lowered when a sprite in
    // the set is too small or not a clean multiple.
    let requested: i32 = env::var("VOXEL_MIP_LEVELS")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(4);
    let atlas = voxel_render::build_atlas(&source, &model, 2048, requested)
        .map_err(|error| format!("{name}: {error:?}"))?;
    // A blockstate variant turns its model in quarter turns. When a block
    // was named the turns come from the variant it selected; otherwise the
    // probe takes them directly.
    let angle = |variable: &str, axis: usize| -> std::result::Result<i32, String> {
        match chosen {
            Some(chosen) => Ok(chosen.rotation[axis]),
            None => env::var(variable)
                .ok()
                .map(|value| value.parse::<i32>())
                .transpose()
                .map_err(|error| format!("{variable}: {error}"))
                .map(|parsed| parsed.unwrap_or(0)),
        }
    };
    let rotation = [
        angle("VOXEL_ROTATE_X", 0)?,
        angle("VOXEL_ROTATE_Y", 1)?,
        angle("VOXEL_ROTATE_Z", 2)?,
    ];
    let [rotate_x, rotate_y, rotate_z] = rotation;
    let state = voxel_render::rotation::model_state_transform(rotate_x, rotate_y, rotate_z)
        .ok_or("variant rotation must be a multiple of ninety")?;
    let identity = rotate_x % 360 == 0 && rotate_y % 360 == 0 && rotate_z % 360 == 0;
    // Outside a world a block's tints are fixed per block, so they come from
    // the registry rather than from the world. A model drawn by name rather
    // than by block has none.
    let tints: Vec<u32> = chosen
        .map(|chosen| voxel_render::constant_tints(&chosen.name).to_vec())
        .unwrap_or_default();
    // A variant may lock its UVs, which turns each face's texture coordinate
    // back by as much as the model turned.
    let uv_lock = chosen.is_some_and(|chosen| chosen.uvlock);
    // A grid repeats the block, so the faces between two of them can be
    // dropped. That is the smallest scene where culling does any work at
    // all.
    let grid: u32 = env::var("VOXEL_GRID")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(1);
    let turn = Turn {
        state: (!identity).then_some(&state),
        rotation,
        uv_lock,
    };
    let mesh = grid_mesh(name, (&model, &atlas), &turn, &tints, grid)?;
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
    // Each level can be written out, which is how the padding fill and the
    // halving get looked at rather than assumed.
    if let Ok(prefix) = env::var("VOXEL_ATLAS_DUMP") {
        for (level, image) in atlas.levels.iter().enumerate() {
            let path = format!("{prefix}{level}.ppm");
            ppm::write(&path, image.width, image.height, &image.rgba)?;
            println!("atlas_level_{level}={}x{}", image.width, image.height);
        }
    }
    Ok((mesh, atlas.levels))
}

/// How a model is turned: its transform unless the turn is a whole one, the
/// quarter turns it came from, and whether its UVs stay put.
struct Turn<'a> {
    state: Option<&'a voxel_render::Matrix4>,
    rotation: [i32; 3],
    uv_lock: bool,
}

/// A model repeated `grid` times along each axis around the origin, turned,
/// the faces between two copies dropped.
fn grid_mesh(
    name: &str,
    (model, atlas): (&voxel_render::ResolvedModel, &voxel_render::BuiltAtlas),
    turn: &Turn<'_>,
    tints: &[u32],
    grid: u32,
) -> Result<voxel_render::Mesh> {
    let [rotate_x, rotate_y, rotate_z] = turn.rotation;
    let uv_lock = turn.uv_lock;
    let uv_transform = |facing| {
        voxel_render::rotation::inverse_face_transform(
            rotate_x, rotate_y, rotate_z, facing, uv_lock,
        )
        .unwrap_or(voxel_render::rotation::FLAT_IDENTITY)
    };
    let mut mesh = voxel_render::Mesh::default();
    for cell_x in 0..grid {
        for cell_y in 0..grid {
            for cell_z in 0..grid {
                let cell = [cell_x, cell_y, cell_z];
                let occluded = |facing: voxel_core::direction::Direction| {
                    let (step_x, step_y, step_z) = facing.step();
                    [step_x, step_y, step_z]
                        .into_iter()
                        .enumerate()
                        .all(|(axis, step)| {
                            let moved = cell[axis] as i32 + step;
                            moved >= 0 && moved < grid as i32
                        })
                };
                let part = voxel_render::build_mesh(
                    model,
                    atlas,
                    turn.state,
                    tints,
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
    Ok(mesh)
}

/// The probe's own cube and checkerboard, when no model is named.
fn builtin_scene() -> (voxel_render::Mesh, Vec<voxel_render::DecodedImage>) {
    println!("model_source=built-in cube and checkerboard");
    let (rgba, size) = checkerboard();
    println!("model_rotation=0,0,0");
    let mesh = voxel_render::full_cube(voxel_render::SpriteRect {
        min_u: 0.0,
        min_v: 0.0,
        max_u: 1.0,
        max_v: 1.0,
    });
    (
        mesh,
        vec![voxel_render::DecodedImage {
            width: size,
            height: size,
            rgba,
        }],
    )
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

/// View-projection for the probe's camera, orbiting the block.
///
/// None of this matches the reference: the client's own camera arrives with
/// P14, and until then these numbers exist only to make the cube visible.
fn camera(aspect: f32, angle: f32) -> [[f32; 4]; 4] {
    // Inside a world the eye is the player's and it turns on the spot; with a
    // single block or a grid there is no player, so the eye orbits it instead.
    if let Some(eye) = CAMERA_EYE.get().copied() {
        let look = [eye[0] + angle.cos(), eye[1] - 0.25, eye[2] + angle.sin()];
        let view = voxel_render::scene::look_at(eye, look, [0.0, 1.0, 0.0]);
        let projection =
            voxel_render::scene::perspective(70.0_f32.to_radians(), aspect, 0.05, 512.0);
        return voxel_render::scene::multiply(projection, view);
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
    let view = voxel_render::scene::look_at(eye, [0.5, 0.5, 0.5], [0.0, 1.0, 0.0]);
    let projection = voxel_render::scene::perspective(70.0_f32.to_radians(), aspect, 0.05, 100.0);
    voxel_render::scene::multiply(projection, view)
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

/// Join a server, wait for the chunks around the player, and build one mesh
/// for the box of world in front of them.
///
/// The box is small on purpose: this is the first time the network side and
/// the renderer meet, so it draws what fits in one look rather than a render
/// distance.
fn load_world(
    server: &str,
    root: &str,
) -> Result<(voxel_render::Mesh, Vec<voxel_render::DecodedImage>)> {
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
    let built = voxel_render::build_world_mesh(
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
            voxel_render::biome_tint::TintKind::Grass => 0xFF91_BD59,
            voxel_render::biome_tint::TintKind::Foliage => 0xFF77_AB2F,
            voxel_render::biome_tint::TintKind::DryFoliage => 0xFFA0_7545,
            voxel_render::biome_tint::TintKind::Water => 0xFF3F_76E4,
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
