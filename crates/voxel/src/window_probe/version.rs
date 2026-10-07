use super::*;

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
pub(super) fn open_backend(surface: &WindowSurface) -> Result<VulkanBackend> {
    let scale = surface.scale();
    let (display_handle, window_handle) = surface
        .raw_handles()
        .map_err(|error| format!("window handles unavailable: {error}"))?;
    // SAFETY: the handles are the window's, and the caller's surface is
    // declared before the backend, so it is dropped after it.
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
pub(super) fn show(
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
pub(super) fn scene() -> Result<(voxel_render::Mesh, Vec<voxel_render::DecodedImage>)> {
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

/// A block name takes the longer road: its blockstate file is read, the
/// variant its properties select is taken, and the model and rotation come
/// from there rather than from the environment.
pub(super) fn choose_block(block: &str, root: String) -> Result<ChosenBlock> {
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
pub(super) fn block_properties(
    block: &str,
    name: &str,
) -> Result<std::collections::BTreeMap<String, String>> {
    let definition = voxel_data::block::registry::block(name)
        .ok_or_else(|| format!("{block}: not in the block registry"))?;
    let mut properties =
        voxel_render::blockstate::parse_properties(&env::var("VOXEL_STATE").unwrap_or_default());
    for (key, value) in &properties {
        let property = definition
            .properties
            .iter()
            .find(|property| property.name == key)
            .ok_or_else(|| format!("{block}: no property named {key}"))?;
        if !property.values.contains(&value.as_str()) {
            return Err(format!("{block}: {key} cannot be {value}").into());
        }
    }
    for (property, default) in definition.properties.iter().zip(definition.default) {
        properties
            .entry(property.name.to_owned())
            .or_insert_with(|| (*default).to_owned());
    }
    println!("block_state_id={:?}", definition.state_id(&properties));
    Ok(properties)
}

/// A model by name, turned, tinted and repeated in a grid as the
/// environment or the chosen block says, with its atlas.
pub(super) fn model_scene(
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

/// A model repeated `grid` times along each axis around the origin, turned,
/// the faces between two copies dropped.
pub(super) fn grid_mesh(
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
pub(super) fn builtin_scene() -> (voxel_render::Mesh, Vec<voxel_render::DecodedImage>) {
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

/// Read the clear colour from `VOXEL_CLEAR_COLOUR` as `r,g,b` in 0..1.
///
/// Only scaffolding: it lets a run be identified on screen while the real
/// pipeline does not exist yet.
pub(super) fn clear_colour() -> [f32; 4] {
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

pub(super) fn vk_version_major(version: u32) -> u32 {
    version >> 22
}

pub(super) fn vk_version_minor(version: u32) -> u32 {
    (version >> 12) & 0x3ff
}

pub(super) fn vk_version_patch(version: u32) -> u32 {
    version & 0xfff
}
