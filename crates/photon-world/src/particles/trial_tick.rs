//! `TrialSpawner.tickClient` less its spin: the particles of the spawner's state and its ambient
//! sound, both asked of the level's random source.

use photon_core::random::RandomSource;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::local_sound;
use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

/// `ParticleEmission.offsetRandom`: the centre of the block moved by up to half of `offset` on each
/// axis (float arithmetic, as in the reference).
fn offset_centre(random: &mut RandomSource, pos: (i32, i32, i32), offset: f32) -> (f64, f64, f64) {
    let mut shift = || f64::from((random.next_float() - 0.5_f32) * offset);
    let x = f64::from(pos.0) + 0.5 + shift();
    let y = f64::from(pos.1) + 0.5 + shift();
    let z = f64::from(pos.2) + 0.5 + shift();
    (x, y, z)
}

impl ParticleEngine {
    /// `TrialSpawnerState.emitParticles` by the state's name.
    fn trial_state_particles<F: FnMut() -> RandomSource>(
        &mut self,
        (pos, state, ominous): ((i32, i32, i32), &str, bool),
        context: &mut LevelEventContext<'_, F>,
    ) {
        let flame = if ominous { "soul_fire_flame" } else { "flame" };
        match state {
            "waiting_for_players" | "waiting_for_reward_ejection" | "ejecting_reward" => {
                if context.level_random.next_int_bound(2) == Ok(0) {
                    let at = offset_centre(context.level_random, pos, 0.9);
                    let small = if ominous {
                        "soul_fire_flame"
                    } else {
                        "small_flame"
                    };
                    self.level_add(context, small, at, (0.0, 0.0, 0.0));
                }
            }
            "active" => {
                let at = offset_centre(context.level_random, pos, 1.0);
                self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
                self.level_add(context, flame, at, (0.0, 0.0, 0.0));
            }
            "cooldown" => self.trial_cooldown_smoke(pos, context),
            _ => {}
        }
    }

    /// `ParticleEmission.SMOKE_INSIDE_AND_TOP_FACE`.
    fn trial_cooldown_smoke<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        let at = offset_centre(context.level_random, pos, 0.9);
        if context.level_random.next_int_bound(3) == Ok(0) {
            self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
        }
        if context.world.game_time().is_multiple_of(20) {
            let top = (
                f64::from(pos.0) + 0.5,
                f64::from(pos.1) + 1.0,
                f64::from(pos.2) + 0.5,
            );
            let count = context.level_random.next_int_bound(4).unwrap_or(0) + 20;
            for _ in 0..count {
                self.level_add(context, "smoke", top, (0.0, 0.0, 0.0));
            }
        }
    }

    /// The tick of a trial spawner the client runs: the particles of its state, then, in the
    /// states that can spawn, now and then its ambient sound.
    pub(super) fn trial_spawner_tick<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let block = context.world.block_state_or_air(pos.0, pos.1, pos.2);
        let rules = crate::block::rules::property;
        let ominous = rules(block, "ominous") == Some("true");
        let state = rules(block, "trial_spawner_state").unwrap_or("inactive");
        self.trial_state_particles((pos, state, ominous), context);
        let spawns = matches!(state, "waiting_for_players" | "active");
        if spawns && context.level_random.next_float() <= 0.02 {
            let random = &mut *context.level_random;
            let volume = random.next_float() * 0.25 + 0.75;
            let pitch = random.next_float() + 0.5;
            let name = if ominous {
                "minecraft:block.trial_spawner.ambient_ominous"
            } else {
                "minecraft:block.trial_spawner.ambient"
            };
            let centre = (
                f64::from(pos.0) + 0.5,
                f64::from(pos.1) + 0.5,
                f64::from(pos.2) + 0.5,
            );
            let event = (name, LocalSoundSource::Blocks);
            sounds.push(local_sound(event, (volume, pitch), centre, context));
        }
    }
}
