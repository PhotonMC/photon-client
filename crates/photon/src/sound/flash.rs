//! The End's flash sound (`DirectionalSoundInstance`): heard from the way the flash hangs.

use super::*;
use photon_game::sounds::engine::Attenuation;

/// How many ticks the sound waits after the flash starts (`EndFlashState.SOUND_DELAY_IN_TICKS`).
const DELAY_TICKS: i32 = 30;

impl SoundSystem {
    /// `weather.end_flash`, ten blocks from the camera towards the angles, without falloff, moved
    /// with the camera each tick, after a delay.
    pub fn play_end_flash(&mut self, (x_angle, y_angle): (f32, f32)) {
        self.local_sounds += 1;
        let seed = i64::from(self.local_sounds).wrapping_mul(0x5DEE_CE66_D1CE_4E5B);
        let mut instance = SoundInstance::new(
            "minecraft:weather.end_flash",
            SoundSource::Weather,
            RandomSource::create(seed),
        );
        instance.attenuation = Attenuation::None;
        let direction = Vec3::direction_from_rotation_components(x_angle, y_angle);
        let place = move |camera: Vec3| {
            (
                camera.x + direction.x * 10.0,
                camera.y + direction.y * 10.0,
                camera.z + direction.z * 10.0,
            )
        };
        let camera = self.camera.lock().map_or(Vec3::ZERO, |camera| *camera);
        instance.position = place(camera);
        let followed = Arc::clone(&self.camera);
        instance.ticker = Some(Box::new(move |state| {
            if let Ok(camera) = followed.lock() {
                *state.position = place(*camera);
            }
        }));
        let id = self.engine.add(instance);
        self.engine.play_delayed(id, DELAY_TICKS);
    }
}
