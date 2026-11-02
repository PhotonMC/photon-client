//! The sounds an entity makes in its own tick with `Level.playLocalSound`, played where it
//! stands after the tick: what its states asked for, and the ones that need only the tick count.

use crate::entity::TickSound;
use crate::local_player::{LocalSound, LocalSoundSource};
use crate::mobs::burst::EntityBurst;
use crate::*;
use photon_data::entity_type::ids::{BLAZE, ELDER_GUARDIAN, ILLUSIONER, PHANTOM, SNIFFER};
use photon_data::sound_event::vanilla;

/// `Phantom.tick`: the wing beat's angle moves by this many degrees a tick.
const PHANTOM_FLAP_DEGREES: f32 = 7.448_451;

/// What the sounds of an entity's tick depend on.
struct Facts {
    id: EntityId,
    type_id: EntityTypeId,
    tick_count: i32,
    at: (f64, f64, f64),
    silent: bool,
    /// An invisible illusioner that was hit this tick moves its mirrors.
    mirrors: bool,
    searching: bool,
}

impl ClientWorld {
    /// The sounds the entity's tick made, queued for the sound engine.
    pub(crate) fn play_tick_sounds(&mut self, id: EntityId) {
        let Some(mut entity) = self.entities.at_mut(id) else {
            return;
        };
        let requests = std::mem::take(&mut entity.sound_requests);
        let (type_id, tick_count) = (entity.type_id, entity.tick_count);
        let view = entity.as_view();
        let at = view.position();
        let eye = (at.0, at.1 + f64::from(view.eye_height()), at.2);
        let view_invisible = view.is_invisible();
        let facts = Facts {
            id,
            type_id,
            tick_count,
            at,
            silent: view.is_silent(),
            mirrors: view_invisible && (view.living.hurt_time == 1 || tick_count % 1200 == 0),
            searching: view.sniffer().is_some_and(|sniffer| sniffer.is_searching()),
        };
        if type_id == ILLUSIONER && view_invisible {
            let hurt_time = entity.living.hurt_time;
            let random = self.entity_sound_random.source();
            // `hurtDuration` of a mob is ten.
            let settle = hurt_time == 9 && !facts.mirrors;
            entity.living.illusion.tick(facts.mirrors, settle, random);
            if facts.mirrors {
                let burst = EntityBurst {
                    at,
                    size: (entity.dimensions().width(), entity.as_view().bb_height()),
                    particle: "cloud",
                    mirror: true,
                };
                self.level_events.0.push(LevelEvent::EntityBurst(burst));
            }
        }
        let mut sounds = self.counted_sounds(&facts);
        for request in requests {
            sounds.push(self.requested_sound(request, (at, eye), type_id == ELDER_GUARDIAN));
        }
        self.sounds.0.extend(sounds);
    }

    /// The sounds that need only the tick count and the state.
    fn counted_sounds(&mut self, facts: &Facts) -> Vec<LocalSound> {
        let (type_id, at) = (facts.type_id, facts.at);
        let mut sounds = Vec::new();
        if type_id == BLAZE {
            sounds.extend(self.blaze_burn(at, facts.silent));
        }
        if type_id == PHANTOM && phantom_flaps(facts.id, facts.tick_count) {
            sounds.push(self.phantom_flap(at));
        }
        if type_id == SNIFFER && facts.searching && facts.tick_count % 20 == 0 {
            let event = vanilla("entity.sniffer.searching");
            sounds.push(LocalSound::at(
                event,
                LocalSoundSource::Neutral,
                (1.0, 1.0),
                at,
            ));
        }
        // `Illusioner.aiStep`: an invisible illusioner that is hit, or on the minute, moves its mirrors.
        if type_id == ILLUSIONER && facts.mirrors {
            let event = vanilla("entity.illusioner.mirror_move");
            sounds.push(LocalSound::at(
                event,
                LocalSoundSource::Hostile,
                (1.0, 1.0),
                at,
            ));
        }
        sounds
    }

    /// `Blaze.aiStep`: one tick in twenty-four it crackles.
    fn blaze_burn(&mut self, (x, y, z): (f64, f64, f64), silent: bool) -> Option<LocalSound> {
        let random = self.entity_sound_random.source();
        if random.next_int_bound(24) != Ok(0) || silent {
            return None;
        }
        let volume = 1.0 + random.next_float();
        let pitch = random.next_float() * 0.7 + 0.3;
        Some(LocalSound::at(
            vanilla("entity.blaze.burn"),
            LocalSoundSource::Hostile,
            (volume, pitch),
            (x + 0.5, y + 0.5, z + 0.5),
        ))
    }

    /// `Phantom.tick`: the sound of a wing beat.
    fn phantom_flap(&mut self, at: (f64, f64, f64)) -> LocalSound {
        let random = self.entity_sound_random.source();
        let volume = 0.95 + random.next_float() * 0.05;
        let pitch = 0.95 + random.next_float() * 0.05;
        LocalSound::at(
            vanilla("entity.phantom.flap"),
            LocalSoundSource::Hostile,
            (volume, pitch),
            at,
        )
    }

    /// The sound a state asked for.
    fn requested_sound(
        &mut self,
        request: TickSound,
        (at, eye): ((f64, f64, f64), (f64, f64, f64)),
        elder: bool,
    ) -> LocalSound {
        let random = self.entity_sound_random.source();
        let (event, volume, pitch) = match request {
            TickSound::WardenHeartbeat => {
                let pitch = (random.next_float() - random.next_float()) * 0.2 + 1.0;
                (vanilla("entity.warden.heartbeat"), 5.0, pitch)
            }
            TickSound::BreezeWhirl => {
                let pitch = 0.7 + 0.4 * random.next_float();
                let volume = 0.8 + 0.2 * random.next_float();
                (vanilla("entity.breeze.whirl"), volume, pitch)
            }
            other => fixed_sound(other, elder),
        };
        let source = match request {
            TickSound::CopperSpin(_) => LocalSoundSource::Neutral,
            _ => LocalSoundSource::Hostile,
        };
        // A stare is heard from the eyes.
        let place = if request == TickSound::EndermanStare {
            eye
        } else {
            at
        };
        LocalSound::at(event, source, (volume, pitch), place)
    }
}

/// The sounds that draw nothing: their event, volume and pitch.
fn fixed_sound(request: TickSound, elder: bool) -> (&'static str, f32, f32) {
    match request {
        TickSound::GuardianFlop if elder => (vanilla("entity.elder_guardian.flop"), 1.0, 1.0),
        TickSound::GuardianFlop => (vanilla("entity.guardian.flop"), 1.0, 1.0),
        TickSound::DragonFlap(pitch) => (vanilla("entity.ender_dragon.flap"), 5.0, pitch),
        TickSound::DragonGrowl(pitch) => (vanilla("entity.ender_dragon.growl"), 2.5, pitch),
        TickSound::EndermanStare => (vanilla("entity.enderman.stare"), 2.5, 1.0),
        TickSound::CopperSpin(weathering) => (copper_spin(weathering), 1.0, 1.0),
        TickSound::WardenHeartbeat | TickSound::BreezeWhirl => {
            unreachable!("these two draw their volume or pitch")
        }
    }
}

/// `Phantom.tick`: whether the wing beat's cosine falls through zero this tick.
fn phantom_flaps(id: EntityId, tick_count: i32) -> bool {
    let offset = id.0.wrapping_mul(3);
    let angle = |ticks: i32| {
        let degrees = ticks as f32 * PHANTOM_FLAP_DEGREES * (std::f64::consts::PI / 180.0) as f32;
        photon_core::math::mth_cos_f64(f64::from(degrees + std::f64::consts::PI as f32))
    };
    angle(offset.wrapping_add(tick_count)) > 0.0
        && angle(offset.wrapping_add(tick_count).wrapping_add(1)) <= 0.0
}

/// `CopperGolemOxidationLevels`: the spin of a golem by how far it has weathered.
fn copper_spin(weathering: u8) -> &'static str {
    match weathering {
        2 => vanilla("entity.copper_golem_weathered.spin"),
        3 => vanilla("entity.copper_golem_oxidized.spin"),
        _ => vanilla("entity.copper_golem.spin"),
    }
}
