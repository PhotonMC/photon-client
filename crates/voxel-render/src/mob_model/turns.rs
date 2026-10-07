use super::*;

/// `Direction.getRotation` of the direction with a data value (down 0, up 1,
/// north 2, south 3, west 4, east 5).
pub fn direction_rotation(data_value: i32) -> Quaternion {
    let half = std::f32::consts::PI / 2.0;
    let pi = std::f32::consts::PI;
    match data_value {
        0 => Quaternion::rotation_x(pi),
        2 => Quaternion::rotation_xyz(half, 0.0, pi),
        3 => Quaternion::rotation_x(half),
        4 => Quaternion::rotation_xyz(half, 0.0, half),
        5 => Quaternion::rotation_xyz(half, 0.0, -half),
        _ => Quaternion::identity(),
    }
}

/// `Direction.getOpposite`, on data values.
pub fn opposite_direction(data_value: i32) -> i32 {
    data_value ^ 1
}

/// `TntRenderer.getSwellAmount`.
pub fn tnt_swell_amount(fuse: f32) -> f32 {
    let mut swell = (1.0 - fuse / 10.0).clamp(0.0, 1.0);
    swell *= swell;
    swell *= swell;
    swell * 0.3
}

/// `TntRenderer.isLit`: the flash of a fuse, every fifth tick.
pub fn tnt_is_lit(fuse: f32) -> bool {
    fuse >= 0.0 && (fuse / 5.0) as i32 % 2 == 0
}

/// `CodRenderer.setupRotations`: the swing about y, and out of water the shift
/// that lays it on its side.
pub fn cod_turns(age_in_ticks: f32, in_water: bool) -> FishTurns {
    FishTurns {
        bob: 0.0,
        swing: Some(4.3 * mth_sin_f64(f64::from(0.6 * age_in_ticks))),
        on_land: (!in_water).then_some([0.1, 0.1, -0.1]),
    }
}

/// `SalmonRenderer.setupRotations`: the swing is larger and quicker out of
/// water, where the fish lies on its side.
pub fn salmon_turns(age_in_ticks: f32, in_water: bool) -> FishTurns {
    let (amplitude, angle) = if in_water {
        (1.0_f32, 1.0_f32)
    } else {
        (1.3, 1.7)
    };
    FishTurns {
        bob: 0.0,
        swing: Some(amplitude * 4.3 * mth_sin_f64(f64::from(angle * 0.6 * age_in_ticks))),
        on_land: (!in_water).then_some([0.2, 0.1, 0.0]),
    }
}

/// `TropicalFishRenderer.setupRotations`.
pub fn tropical_fish_turns(age_in_ticks: f32, in_water: bool) -> FishTurns {
    FishTurns {
        bob: 0.0,
        swing: Some(4.3 * mth_sin_f64(f64::from(0.6 * age_in_ticks))),
        on_land: (!in_water).then_some([0.2, 0.1, 0.0]),
    }
}

/// `PufferfishRenderer.setupRotations`: it only bobs.
pub fn pufferfish_turns(age_in_ticks: f32) -> FishTurns {
    FishTurns {
        bob: mth_cos_f64(f64::from(age_in_ticks * 0.05)) * 0.08,
        swing: None,
        on_land: None,
    }
}

/// `OverlayTexture.pack`: the overlay coordinates of a white flash by
/// progress and of the hurt overlay.
pub fn overlay_coords(red: bool, white_progress: f32) -> i32 {
    (white_progress * 15.0) as i32 | (if red { 3 } else { 10 }) << 16
}

/// What `SulfurCubeInnerLayer` draws with: a fuse that is lit flashes white,
/// else the entity's own overlay (`LivingEntityRenderer.getOverlayCoords`
/// with no white). Whether it is red and the white's progress.
pub fn sulfur_cube_overlay(fuse_remaining: f32, red: bool) -> (bool, f32) {
    if fuse_remaining > 0.0 && tnt_is_lit(fuse_remaining) {
        (false, 1.0)
    } else {
        (red, 0.0)
    }
}

/// `OverlayTexture`'s row for what a vertex is drawn with over its texel,
/// as the entity shader reads it: the hurt overlay is one, a white flash is
/// twice its column (`OverlayTexture.u`).
pub fn overlay_code(red: bool, white_progress: f32) -> f32 {
    if red {
        1.0
    } else {
        // `OverlayTexture.u`: truncated to a whole column.
        (2 * (white_progress * 15.0) as i32) as f32
    }
}

/// `CreeperRenderer.scale`: a creeper swells, wobbling as it nears its blast;
/// the factors along x and z, and along y.
pub fn creeper_scale(swelling: f32) -> [f32; 3] {
    let wobble = 1.0 + mth_sin_f64(f64::from(swelling * 100.0)) * swelling * 0.01;
    let mut swell = swelling.clamp(0.0, 1.0);
    swell *= swell;
    swell *= swell;
    let wide = (1.0 + swell * 0.4) * wobble;
    let tall = (1.0 + swell * 0.1) / wobble;
    [wide, tall, wide]
}

/// `CreeperRenderer.getWhiteOverlayProgress`: a swelling creeper flashes
/// white every other tenth of its swell.
pub fn creeper_white_overlay(swelling: f32) -> f32 {
    if (swelling * 10.0) as i32 % 2 == 0 {
        0.0
    } else {
        swelling.clamp(0.5, 1.0)
    }
}

/// `LivingEntityRenderer.setupRotations`'s fall of a dying entity: the
/// turn about z, in degrees, once its death has begun. It eases out as the
/// square root of the time since the first tick, and stops at the full flip.
pub fn death_roll(death_time: f32, flip_degrees: f32) -> Option<f32> {
    (death_time > 0.0).then(|| {
        let fall = ((death_time - 1.0) / 20.0 * 1.6).sqrt();
        fall.min(1.0) * flip_degrees
    })
}

/// `LivingEntityRenderer.submit` for a mob standing on its feet: scaled,
/// turned to face away from the body's facing, flipped upright and lowered so
/// the feet meet the ground.
pub fn placement_pose(placement: &MobPlacement) -> Pose {
    let mut pose = placement_rotations(placement);
    pose.scale(-1.0, -1.0, 1.0);
    if let Some(cube) = placement.cube_mob {
        cube.apply(&mut pose);
    }
    if let Some([x, y, z]) = placement.hook_scale {
        pose.scale(x, y, z);
    }
    if let Some([x, y, z]) = placement.hook_translate {
        pose.translate(x, y, z);
    }
    pose.translate(0.0, -1.501, 0.0);
    pose
}

/// What `LivingEntityRenderer.setupRotations` adds to the body's facing while
/// it shakes: a cosine of the whole tick, in degrees.
pub fn shake_degrees(age_in_ticks: f32) -> f32 {
    (f64::from(age_in_ticks.floor() * 3.25_f32).cos() * std::f64::consts::PI * f64::from(0.4_f32))
        as f32
}

/// What `LivingEntityRenderer.submit` has done by the end of the renderer's
/// `setupRotations`: the feet, the scale, the turn to face away from the
/// body's facing, the fall of a dead mob and the renderer's own turns.
pub fn placement_rotations(placement: &MobPlacement) -> Pose {
    let mut pose = Pose::default();
    pose.translate(placement.feet[0], placement.feet[1], placement.feet[2]);
    pose.scale(placement.scale, placement.scale, placement.scale);
    if let Some(fish) = placement.fish {
        pose.translate(0.0, fish.bob, 0.0);
    }
    if let Some(squid) = placement.squid {
        // `SquidRenderer.setupRotations`: no fall of a dead one, no shudder.
        pose.translate(0.0, if squid.baby { 0.25 } else { 0.5 }, 0.0);
        let degrees = Quaternion::degrees_to_radians;
        pose.rotate(&Quaternion::rotation_y(degrees(180.0 - placement.body_rot)));
        pose.rotate(&Quaternion::rotation_x(degrees(squid.x_body_rot)));
        pose.rotate(&Quaternion::rotation_y(degrees(squid.z_body_rot)));
        pose.translate(0.0, if squid.baby { -0.6 } else { -1.2 }, 0.0);
        return pose;
    }
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        180.0 - (placement.body_rot + placement.shaking.map_or(0.0, shake_degrees)),
    )));
    if let Some(roll) = death_roll(placement.death_time, placement.flip_degrees) {
        pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
            roll,
        )));
    }
    if let Some(fish) = placement.fish {
        if let Some(swing) = fish.swing {
            pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
                swing,
            )));
        }
        if let Some([x, y, z]) = fish.on_land {
            pose.translate(x, y, z);
            pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
                90.0,
            )));
        }
    }
    if let Some([x, y, z]) = placement.shift {
        pose.translate(x, y, z);
    }
    if let Some(roll) = placement.roll {
        pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
            roll,
        )));
    }
    if let Some([x, y, z]) = placement.roll_shift {
        pose.translate(x, y, z);
    }
    if let Some(pitch) = placement.pitch {
        pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
            pitch,
        )));
    }
    if let Some(panda) = placement.panda {
        panda_rotate(&mut pose, &panda);
    }
    if let Some(face) = placement.attach_face {
        pose.rotate_around(
            &direction_rotation(opposite_direction(face)),
            (0.0, 0.5, 0.0),
        );
    }
    pose
}

/// The posed model's triangles, placed, each vertex carrying its normal in
/// its tint for the entity shader, with texture coordinates in the packed
/// entity textures.
pub fn mob_vertices(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    mob_vertices_coloured(model, placement, texture, None)
}

/// `RenderTypes.entityTranslucent` sorts the quads of what it draws far to near
/// (`VertexSorting.DISTANCE_TO_ORIGIN` over each quad's centre, the midpoint of
/// its first and third corner), so a blended model shows its far faces
/// through its near ones. `vertices` are the six triangle corners of each quad
/// [`mob_vertices`] makes, and `camera` is where the eye is among them.
pub fn sort_translucent_quads(vertices: &mut [Vertex], camera: [f32; 3]) {
    let key = |quad: &[Vertex]| -> f32 {
        let centre =
            |axis: usize| (quad[0].position[axis] + quad[2].position[axis]) / 2.0 - camera[axis];
        let (x, y, z) = (centre(0), centre(1), centre(2));
        x * x + y * y + z * z
    };
    let mut quads: Vec<(f32, [Vertex; 6])> = vertices
        .chunks_exact(6)
        .map(|quad| {
            (
                key(quad),
                [quad[0], quad[1], quad[2], quad[3], quad[4], quad[5]],
            )
        })
        .collect();
    // A stable sort, as the reference's merge sort is.
    quads.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (slot, (_, quad)) in vertices.chunks_exact_mut(6).zip(quads) {
        slot.copy_from_slice(&quad);
    }
}

/// A glowing layer's triangles (`EyesLayer`, `RenderTypes.eyes`): the same
/// posed model, drawn unlit and without the hurt overlay, each vertex's tint
/// the white it is multiplied by.
pub fn mob_vertices_glow(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    mob_vertices_glow_posed(model, &placement_pose(placement), texture)
}

/// [`mob_vertices_glow`] for a model the renderer has already placed.
pub fn mob_vertices_glow_posed(model: &MobModel, pose: &Pose, texture: TextureRect) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for (_, corners) in model.render(pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: 0.0,
                tint: [1.0; 4],
                light: [0.0; 2],
            });
        }
    }
    vertices
}

/// An energy swirl's triangles (`EnergySwirlLayer`, `RenderTypes.energySwirl`):
/// the same posed model, each vertex carrying where its texture is in the
/// packed image (the corner and the size, in its tint), the texture offset
/// (in its light) and the colour (`0xRRGGBB`, in its shade) that the shader
/// moves, wraps and multiplies the texture by.
pub fn mob_vertices_swirl(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    offset: [f32; 2],
    colour: u32,
) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let rect = [
        texture[0],
        texture[1],
        texture[2] - texture[0],
        texture[3] - texture[1],
    ];
    let mut vertices = Vec::new();
    for (_, corners) in model.render(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: (colour & 0x00FF_FFFF) as f32,
                tint: rect,
                light: offset,
            });
        }
    }
    vertices
}

/// A glowing layer drawn lit by faces but not by the world (`EMISSIVE` with
/// per-face lighting, `RenderTypes.entityTranslucentEmissive`): the same posed
/// model, no overlay, its light the full bright the entity shader reads as
/// negative.
pub fn mob_vertices_emissive(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
) -> Vec<Vertex> {
    emissive_vertices(model, placement, texture, (0.0, [-1.0; 2]))
}

/// A layer drawn unlit but shaded by face and blended (`RenderTypes.entityTranslucentEmissive`)
/// by `LivingEntityEmissiveLayer`: the same posed model, its colour white with
/// this alpha (a whole 255th, as `ARGB.white` makes it), and the hurt overlay
/// of the mob it covers.
pub fn mob_vertices_emissive_faded(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    alpha: f32,
) -> Vec<Vertex> {
    let alpha = (alpha * 255.0).floor() / 255.0;
    emissive_vertices(
        model,
        placement,
        texture,
        (overlay_code(placement.red_overlay, 0.0), [-1.0, alpha]),
    )
}

/// An emissive layer's triangles: each vertex carries its face's normal for
/// the shader, and its light the negative full bright; the second value of
/// the light, when not negative, is the layer's alpha.
pub(super) fn emissive_vertices(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    (shade, light): (f32, [f32; 2]),
) -> Vec<Vertex> {
    let pose = placement_pose(placement);
    let mut vertices = Vec::new();
    for (_, corners) in model.render(&pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade,
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], 0.0],
                light,
            });
        }
    }
    vertices
}

/// A layer whose texture scrolls along `u` and repeats (`RenderPipelines.BREEZE_WIND`):
/// lit by the world but not by faces, no overlay. The packed image does not
/// repeat, so each quad the scroll carries across the texture's edge is cut
/// there, and the part beyond is drawn from the start of the texture.
pub fn mob_vertices_scrolled(
    model: &MobModel,
    placement: &MobPlacement,
    texture: TextureRect,
    u_offset: f32,
) -> Vec<Vertex> {
    mob_vertices_scrolled_posed(
        model,
        &placement_pose(placement),
        placement.light,
        (texture, u_offset),
    )
}

/// [`mob_vertices_scrolled`] for a model the renderer has already placed (a wind charge's).
pub fn mob_vertices_scrolled_posed(
    model: &MobModel,
    pose: &Pose,
    packed_light: i32,
    (texture, u_offset): (TextureRect, f32),
) -> Vec<Vertex> {
    let light = crate::block_light::light_uv(packed_light);
    let mut vertices = Vec::new();
    for (_, corners) in model.render(pose) {
        let scrolled: Vec<([f32; 3], [f32; 2])> = corners
            .iter()
            .map(|corner| (corner.position, [corner.uv[0] + u_offset, corner.uv[1]]))
            .collect();
        for (polygon, shift) in [
            (clip_u(&scrolled, true), 0.0),
            (clip_u(&scrolled, false), 1.0),
        ] {
            // A fan of triangles from the first corner.
            for index in 1..polygon.len().saturating_sub(1) {
                for (position, coordinate) in [polygon[0], polygon[index], polygon[index + 1]] {
                    vertices.push(Vertex {
                        position,
                        uv: uv(texture, [coordinate[0] - shift, coordinate[1]]),
                        shade: 0.0,
                        tint: [0.0; 4],
                        light,
                    });
                }
            }
        }
    }
    vertices
}
