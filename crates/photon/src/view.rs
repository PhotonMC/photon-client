//! Where the player looks from and what they look at: the camera type, its
//! zoom and its projection.

/// The cloud range option's default, in chunks.
pub const CLOUD_RANGE: i32 = 128;

/// `Entity.getEyePosition` between ticks.
pub fn eye_position(
    player: &photon_world::local_player::LocalPlayer,
    partial: f64,
) -> photon_core::math::Vec3 {
    let feet = player.interpolated_position(partial);
    photon_core::math::Vec3::new(feet.x, feet.y + f64::from(player.eye_height()), feet.z)
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
fn max_camera_zoom(
    world: &photon_world::ClientWorld,
    eye: photon_core::math::Vec3,
    forwards: photon_core::math::Vec3,
    mut distance: f32,
) -> f32 {
    use photon_core::math::Vec3;
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
/// The camera's right, up and forward vectors (`Camera.setRotation`).
fn view_basis(y_rot: f32, x_rot: f32) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let look = photon_core::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
    let forward = [look.x as f32, look.y as f32, look.z as f32];
    // The side vector turns with the yaw alone, so looking straight up or
    // down keeps a well-defined view.
    let yaw = y_rot.to_radians();
    let right = [-yaw.cos(), 0.0, -yaw.sin()];
    let up = [
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    ];
    (right, up, forward)
}

/// `Camera.getViewRotationMatrix`: the view's turn with no move, which the
/// culling frustum is built from.
pub fn view_rotation(y_rot: f32, x_rot: f32) -> [[f32; 4]; 4] {
    let (right, up, forward) = view_basis(y_rot, x_rot);
    [
        [right[0], up[0], -forward[0], 0.0],
        [right[1], up[1], -forward[1], 0.0],
        [right[2], up[2], -forward[2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn player_camera(
    aspect: f32,
    fov: f32,
    far: f32,
    eye: [f32; 3],
    y_rot: f32,
    x_rot: f32,
    bob: [[f32; 4]; 4],
) -> [[f32; 4]; 4] {
    let (right, up, forward) = view_basis(y_rot, x_rot);
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let view = [
        [right[0], up[0], -forward[0], 0.0],
        [right[1], up[1], -forward[1], 0.0],
        [right[2], up[2], -forward[2], 0.0],
        [-dot(right, eye), -dot(up, eye), dot(forward, eye), 1.0],
    ];
    let projection = photon_render::scene::multiply(
        photon_render::scene::perspective(fov.to_radians(), aspect, 0.05, far),
        bob,
    );
    photon_render::scene::multiply(projection, view)
}

/// `Camera.alignWithEntity`'s position: the eye, or for a detached camera
/// as far back from it along the view as no block is in the way.
pub fn camera_position(
    world: &photon_world::ClientWorld,
    camera_type: CameraType,
    eye: photon_core::math::Vec3,
    (y_rot, x_rot): (f32, f32),
) -> photon_core::math::Vec3 {
    if camera_type == CameraType::FirstPerson {
        return eye;
    }
    let forwards = photon_core::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
    let zoom = max_camera_zoom(world, eye, forwards, 4.0);
    photon_core::math::Vec3::new(
        eye.x - forwards.x * f64::from(zoom),
        eye.y - forwards.y * f64::from(zoom),
        eye.z - forwards.z * f64::from(zoom),
    )
}

/// The three vectors the camera looks along (`Camera.setRotation`): forwards, left and up.
pub fn camera_vectors(rotation: (f32, f32)) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let mut pose = photon_render::joml::Pose::default();
    pose.rotate(&photon_render::entity::name_tag::camera_orientation(
        rotation.0, rotation.1,
    ));
    (
        pose.pose.transform_direction(0.0, 0.0, -1.0),
        pose.pose.transform_direction(-1.0, 0.0, 0.0),
        pose.pose.transform_direction(0.0, 1.0, 0.0),
    )
}

#[cfg(test)]
mod culling_tests {
    use photon_render::visibility::{Frustum, culling_projection};

    /// The culling frustum the play loop builds, at a camera looking along
    /// a yaw and pitch.
    fn frustum(y_rot: f32, x_rot: f32) -> (Frustum, photon_core::math::Vec3) {
        let camera = [0.5, 64.5, 0.5];
        let frustum = Frustum::new(
            &super::view_rotation(y_rot, x_rot),
            &culling_projection(70.0, 16.0 / 9.0, 512.0),
            camera,
        );
        let look = photon_core::math::Vec3::direction_from_rotation_components(x_rot, y_rot);
        (frustum, look)
    }

    /// The section `distance` blocks from the camera along `look`.
    fn section_along(look: photon_core::math::Vec3, distance: f64) -> (i32, i32, i32) {
        let point = [
            0.5 + look.x * distance,
            64.5 + look.y * distance,
            0.5 + look.z * distance,
        ];
        let [x, y, z] = point.map(|value| (value.floor() as i32).div_euclid(16));
        (x, y, z)
    }

    #[test]
    fn what_the_camera_faces_is_in_view_and_what_is_behind_it_is_not() {
        for y_rot in [0.0, 37.0, 90.0, 180.0, 270.0] {
            for x_rot in [-60.0, 0.0, 45.0] {
                let (frustum, look) = frustum(y_rot, x_rot);
                assert!(
                    frustum.section_visible(section_along(look, 64.0)),
                    "ahead at yaw {y_rot} pitch {x_rot}"
                );
                assert!(
                    !frustum.section_visible(section_along(look, -64.0)),
                    "behind at yaw {y_rot} pitch {x_rot}"
                );
            }
        }
    }

    #[test]
    fn looking_straight_up_or_down_still_backs_off_to_the_camera_cube() {
        for x_rot in [-90.0, 90.0] {
            let (frustum, look) = frustum(0.0, x_rot);
            let offset = frustum.offset_to_fully_include_camera_cube(8);
            assert!(offset.is_visible([0.0, 64.0, 0.0], [8.0, 72.0, 8.0]));
            assert!(offset.section_visible(section_along(look, 48.0)));
        }
    }
}
