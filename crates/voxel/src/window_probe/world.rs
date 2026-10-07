use super::*;

/// View-projection for the probe's camera, orbiting the block.
///
/// None of this matches the reference: the client's own camera arrives with
/// P14, and until then these numbers exist only to make the cube visible.
pub(super) fn camera(aspect: f32, angle: f32) -> [[f32; 4]; 4] {
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

/// A fallback texture so the probe runs with no assets present.
pub(super) fn checkerboard() -> (Vec<u8>, u32) {
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
pub(super) fn load_world(
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
            voxel_render::block::biome_tint::TintKind::Grass => 0xFF91_BD59,
            voxel_render::block::biome_tint::TintKind::Foliage => 0xFF77_AB2F,
            voxel_render::block::biome_tint::TintKind::DryFoliage => 0xFFA0_7545,
            voxel_render::block::biome_tint::TintKind::Water => 0xFF3F_76E4,
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
