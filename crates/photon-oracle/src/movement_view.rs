//! How the movement cases describe what the player sees and hears.

/// The near plane of the culling frustum the game sees through: 70 degrees on a 16:9 window,
/// looking straight ahead.
pub(crate) fn viewing_near_plane() -> ([f64; 3], [f64; 3]) {
    photon_render::visibility::Frustum::new(
        &[
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        &photon_render::visibility::culling_projection(70.0, 16.0 / 9.0, 256.0),
        [0.0; 3],
    )
    .near_plane_bounds()
}

/// A sound as the cases compare it: event, source, volume, pitch and place.
pub(crate) fn sound_line(sound: &photon_world::local_player::LocalSound) -> String {
    // The swim, splash, hurt and death sounds' pitch is random in Java and is compared by volume only.
    let pitch = if crate::view_state::has_random_pitch(sound.event) {
        0.0_f32
    } else {
        sound.pitch
    };
    format!(
        "{},{},{:08x},{:08x},{:016x},{:016x},{:016x}",
        sound.event,
        sound.source.name(),
        sound.volume.to_bits(),
        pitch.to_bits(),
        sound.position.0.to_bits(),
        sound.position.1.to_bits(),
        sound.position.2.to_bits()
    )
}
