//! Where the player looks from and what they look at: the camera type, its
//! zoom, its projection, and the block picked.

/// The cloud range option's default, in chunks.
pub const CLOUD_RANGE: i32 = 128;

/// `GameRenderer.pick` for blocks: the outline seen from `eye` along the
/// view within reach (4.5 blocks, 5 in creative), or a miss. Entities are
/// not picked.
pub fn pick(
    world: &voxel_world::ClientWorld,
    player: &voxel_world::local_player::LocalPlayer,
    eye: voxel_core::math::Vec3,
) -> voxel_game::game_mode::Aim {
    let reach = if player.abilities().instabuild() {
        5.0
    } else {
        4.5
    };
    let (y_rot, x_rot) = player.rotation();
    let look = voxel_core::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
    let to = voxel_core::math::Vec3::new(
        eye.x + look.x * reach,
        eye.y + look.y * reach,
        eye.z + look.z * reach,
    );
    match world.clip_outlines(eye, to) {
        // `filterHitResult`: a hit past the reach is a miss.
        Some(hit) if hit.location.distance_to_sqr(eye) < reach * reach => {
            voxel_game::game_mode::Aim::Block {
                x: hit.block_pos.x,
                y: hit.block_pos.y,
                z: hit.block_pos.z,
                face: hit.direction.data3d() as u8,
                location: [hit.location.x, hit.location.y, hit.location.z],
                inside: hit.inside,
            }
        }
        _ => voxel_game::game_mode::Aim::Miss,
    }
}

/// `Entity.getEyePosition` between ticks.
pub fn eye_position(
    player: &voxel_world::local_player::LocalPlayer,
    partial: f64,
) -> voxel_core::math::Vec3 {
    let feet = player.interpolated_position(partial);
    voxel_core::math::Vec3::new(feet.x, feet.y + f64::from(player.eye_height()), feet.z)
}

/// Where the camera looks from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraType {
    FirstPerson,
    ThirdPersonBack,
    ThirdPersonFront,
}

impl CameraType {
    /// `CameraType.cycle`.
    pub fn next(self) -> Self {
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
pub fn max_camera_zoom(
    world: &voxel_world::ClientWorld,
    eye: voxel_core::math::Vec3,
    forwards: voxel_core::math::Vec3,
    mut distance: f32,
) -> f32 {
    use voxel_core::math::Vec3;
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

/// View-projection from the player's eye, measured from the mesh origin.
pub fn player_camera(
    aspect: f32,
    fov: f32,
    far: f32,
    eye: [f32; 3],
    y_rot: f32,
    x_rot: f32,
    bob: [[f32; 4]; 4],
) -> [[f32; 4]; 4] {
    let look = voxel_core::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
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
    let projection = voxel_render::scene::multiply(
        voxel_render::scene::perspective(fov.to_radians(), aspect, 0.05, far),
        bob,
    );
    voxel_render::scene::multiply(projection, view)
}
