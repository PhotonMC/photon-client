//! The trial spawner's level events (3011 to 3014, 3017, 3019 to 3021): its sounds and the flames,
//! sparks and items its particles throw about it.

use photon_core::random::RandomSource;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::delayed_sound;
use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;
use super::level_events_last::spread_pitch;

/// The sound an event plays and its volume, when it plays one.
fn trial_sound(event: i32, data: i32) -> Option<(&'static str, f32)> {
    let name = match event {
        3012 => "minecraft:block.trial_spawner.spawn_mob",
        3021 => "minecraft:block.trial_spawner.spawn_item",
        3013 | 3019 => "minecraft:block.trial_spawner.detect_player",
        3014 => "minecraft:block.trial_spawner.eject_item",
        3020 => "minecraft:block.trial_spawner.ominous_activate",
        _ => return None,
    };
    let quiet = event == 3020 && data == 0;
    Some((name, if quiet { 0.3 } else { 1.0 }))
}

impl ParticleEngine {
    /// The trial spawner's events (`TrialSpawner.add*Particles` and its sounds).
    pub(super) fn trial_spawner_event<F: FnMut() -> RandomSource>(
        &mut self,
        (event, pos, data): (i32, (i32, i32, i32), i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let centre = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        if let Some((name, volume)) = trial_sound(event, data) {
            let pitch = spread_pitch(context.level_random);
            let sound = (name, LocalSoundSource::Blocks);
            sounds.push(delayed_sound(sound, (volume, pitch), centre, context));
        }
        match event {
            // `FlameParticle.decode`: a datum out of range is the normal flame.
            3011 | 3012 | 3021 => {
                let flame = if data == 1 {
                    "soul_fire_flame"
                } else {
                    "flame"
                };
                self.trial_spawn_particles(pos, flame, context);
            }
            3013 => self.trial_detect_particles(pos, data, "trial_spawner_detection", context),
            3019 => {
                self.trial_detect_particles(pos, data, "trial_spawner_detection_ominous", context);
            }
            3014 | 3017 => self.trial_eject_particles(pos, context),
            3020 => {
                self.trial_detect_particles(pos, 0, "trial_spawner_detection_ominous", context);
                self.trial_become_ominous_particles(pos, context);
            }
            _ => {}
        }
    }

    /// `TrialSpawner.addSpawnParticles`: smoke and the flame, twenty times about the block.
    fn trial_spawn_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        flame: &str,
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..20 {
            let at = self.trial_about(pos, context.level_random);
            self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
            self.level_add(context, flame, at, (0.0, 0.0, 0.0));
        }
    }

    /// `TrialSpawner.addBecomeOminousParticles`.
    fn trial_become_ominous_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..20 {
            let at = self.trial_about(pos, context.level_random);
            let random = &mut *context.level_random;
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            self.level_add(context, "trial_omen", at, push);
            self.level_add(context, "soul_fire_flame", at, push);
        }
    }

    /// A place within a block's width of its middle, as the trial spawner's puffs are thrown.
    fn trial_about(&self, pos: (i32, i32, i32), random: &mut RandomSource) -> (f64, f64, f64) {
        let mut about = |base: i32| f64::from(base) + 0.5 + (random.next_double() - 0.5) * 2.0;
        let x = about(pos.0);
        let y = about(pos.1);
        let z = about(pos.2);
        (x, y, z)
    }

    /// `TrialSpawner.addDetectPlayerParticles`: more of them the more players are about, up to ten.
    fn trial_detect_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        data: i32,
        name: &str,
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..30 + data.min(10) * 5 {
            let random = &mut *context.level_random;
            let spread_x = f64::from(2.0 * random.next_float() - 1.0) * 0.65;
            let spread_z = f64::from(2.0 * random.next_float() - 1.0) * 0.65;
            let at = (
                f64::from(pos.0) + 0.5 + spread_x,
                f64::from(pos.1) + 0.1 + f64::from(random.next_float()) * 0.8,
                f64::from(pos.2) + 0.5 + spread_z,
            );
            self.level_add(context, name, at, (0.0, 0.0, 0.0));
        }
    }

    /// `TrialSpawner.addEjectItemParticles`: flames and smoke out of the middle of the block.
    fn trial_eject_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for _ in 0..20 {
            let random = &mut *context.level_random;
            let mut within = |base: i32| f64::from(base) + 0.4 + random.next_double() * 0.2;
            let x = within(pos.0);
            let y = within(pos.1);
            let z = within(pos.2);
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            self.level_add(
                context,
                "small_flame",
                (x, y, z),
                (push.0, push.1, push.2 * 0.25),
            );
            self.level_add(context, "smoke", (x, y, z), push);
        }
    }
}
