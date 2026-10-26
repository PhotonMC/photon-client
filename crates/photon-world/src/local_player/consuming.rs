//! The sound of eating and drinking (`Consumable.shouldEmitParticlesAndSounds` and
//! `emitParticlesAndSounds`, driven from `ItemStack.onUseTick`, and once more from `onConsume`
//! when the server's entity event 9 completes the use on the client too).

use super::*;

/// `(float) (Math.PI / 180.0)`.
const RADIANS: f32 = 0.017453292;

#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}

/// `Vec3.xRot` and `Vec3.yRot`: the point turned by the radians, with the table's sine and
/// cosine.
fn turn((x, y, z): (f64, f64, f64), radians: f32, axis: Axis) -> (f64, f64, f64) {
    let cos = f64::from(photon_core::math::mth_cos_f64(f64::from(radians)));
    let sin = f64::from(photon_core::math::mth_sin_f64(f64::from(radians)));
    match axis {
        Axis::X => (x, y * cos + z * sin, z * cos - y * sin),
        Axis::Y => (x * cos + z * sin, y, z * cos - x * sin),
    }
}

/// One crumb of `LivingEntity.spawnItemParticles`: an item particle thrown from 0.6 in front of
/// the eyes, 0.3 to 0.9 below them, along the direction the entity looks. (The push goes through
/// the packet's single floats.)
pub(crate) fn crumb_particle(
    random: &mut photon_core::random::RandomSource,
    item: ItemId,
    eye: Vec3,
    (y_rot, x_rot): (f32, f32),
) -> Option<crate::particles::LevelParticles> {
    let type_id = photon_data::particle_type::id("item")?;
    let (pitch, yaw) = (-x_rot * RADIANS, -y_rot * RADIANS);
    let mut draw = || f64::from(random.next_float());
    let push = ((draw() - 0.5) * 0.1, draw() * 0.1 + 0.1, 0.0);
    let down = -draw() * 0.6 - 0.3;
    let from = ((draw() - 0.5) * 0.3, down, 0.6);
    let push = turn(turn(push, pitch, Axis::X), yaw, Axis::Y);
    let from = turn(turn(from, pitch, Axis::X), yaw, Axis::Y);
    Some(crate::particles::LevelParticles {
        type_id,
        data: crate::particles::ParticleData::Item(item),
        override_limiter: false,
        always_show: false,
        at: (from.0 + eye.x, from.1 + eye.y, from.2 + eye.z),
        spread: (push.0 as f32, (push.1 + 0.05) as f32, push.2 as f32),
        max_speed: 1.0,
        count: 0,
    })
}

/// What the consumable of a stack in use sounds like.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeCue {
    /// `consumeTicks`: the seconds of the component in ticks.
    pub ticks: i32,
    /// The animation is the drinking one rather than the eating one.
    pub drink: bool,
    pub sound: &'static str,
    /// `hasConsumeParticles`.
    pub particles: bool,
}

impl ConsumeCue {
    /// `Consumable.shouldEmitParticlesAndSounds`: every fourth tick left, once the first fifth
    /// (a little more) of the time has passed. `remaining` counts down from the duration, and
    /// keeps counting down until the server ends the use.
    pub fn emits(&self, used_ticks: i32) -> bool {
        let wait = (self.ticks as f32 * 0.21875_f32) as i32;
        used_ticks > wait && (self.ticks - used_ticks) % 4 == 0
    }
}

impl LocalPlayer {
    /// The cue of the stack in the used hand.
    fn consume_cue(&self) -> Option<ConsumeCue> {
        if self.hands.using_off_hand {
            self.hands.off_consume
        } else {
            self.hands.main_consume
        }
    }

    /// `ItemStack.onUseTick` of the item the player is using, as far as it sounds: five crumbs.
    pub(super) fn consume_tick(&mut self) {
        if let Some(cue) = self
            .consume_cue()
            .filter(|cue| cue.emits(self.use_edge.ticks))
        {
            self.emit_consume_effects(cue, 5);
        }
    }

    /// `LivingEntity.completeUsingItem` for the server's entity event 9: when the item in the
    /// used hand is still the one in use, `Consumable.onConsume` sounds and throws sixteen
    /// crumbs, and the use ends. `None` when no use finished; else whether a consumable was
    /// consumed. The stack's shrink and what it leaves are the caller's
    /// (`photon_game::game_mode::finish_use`).
    pub fn complete_using_item(&mut self) -> Option<bool> {
        if !self.hands.using_item {
            return None;
        }
        self.end_use_of_changed_item();
        if !self.hands.using_item {
            return None;
        }
        let cue = self.consume_cue();
        if let Some(cue) = cue {
            self.emit_consume_effects(cue, 16);
        }
        // `SpyglassItem.finishUsingItem`.
        if self.is_scoping() {
            self.play_own_sound("minecraft:item.spyglass.stop_using", (1.0, 1.0));
        }
        self.release_using_item();
        Some(cue.is_some())
    }

    /// `LivingEntity.spawnItemParticles`: crumbs of the item in use, thrown from in front of the
    /// face along its direction.
    fn throw_crumbs(&mut self, count: usize) {
        let item = if self.hands.using_off_hand {
            self.hands.off_item
        } else {
            self.hands.main_item
        };
        let eye = Vec3::new(
            self.position.x,
            self.position.y + f64::from(self.eye_height()),
            self.position.z,
        );
        let rotation = self.rotation();
        for _ in 0..count {
            let crumb = crumb_particle(&mut self.random, item, eye, rotation);
            self.use_edge.particles.extend(crumb);
        }
    }

    /// `LocalPlayer.playSound`: a sound of the player's own, heard at their feet from the
    /// players' volume slider.
    pub fn play_own_sound(&mut self, event: &'static str, (volume, pitch): (f32, f32)) {
        let at = self.position;
        self.pending_sounds.push(LocalSound::at(
            event,
            LocalSoundSource::Players,
            (volume, pitch),
            (at.x, at.y, at.z),
        ));
    }

    /// `LivingEntity.tickEffects`: the swirls of the effects the entity data lists.
    pub(super) fn throw_effect_particles(&mut self, level: &Level<'_>) {
        let Some(state) = level.world.local_player() else {
            return;
        };
        let flags = (state.is_effect_ambient(), state.is_invisible());
        let body = (
            self.position,
            (self.dimensions.width(), self.dimensions.height()),
        );
        let particles = state.effect_particles();
        let thrown = crate::particles::effect_particle(particles, flags, body, &mut self.random);
        self.use_edge.particles.extend(thrown);
    }

    /// `LivingEntity.handleEntityEvent(46)` for the local player: the portal particles of a
    /// teleport. A position packet resets the old position to the new one, so they gather where
    /// the player arrived.
    pub fn throw_teleport_particles(&mut self) {
        let size = (self.dimensions.width(), self.dimensions.height());
        let thrown = crate::particles::teleport_particles(
            &mut self.random,
            (self.old_position, self.position),
            size,
        );
        self.use_edge.particles.extend(thrown);
    }

    /// The particles the player's own tick threw since the last call.
    pub fn take_particles(&mut self) -> Vec<crate::particles::LevelParticles> {
        std::mem::take(&mut self.use_edge.particles)
    }

    /// `Consumable.emitParticlesAndSounds`: the volumes and pitches of both kinds are drawn,
    /// then the crumbs (four draws each), then the sound is played.
    fn emit_consume_effects(&mut self, cue: ConsumeCue, crumbs: usize) {
        let eat_volume = if self.random.next_boolean() { 0.5 } else { 1.0 };
        let eat_pitch = self.random.triangle_f32(1.0, 0.2);
        let drink_pitch = self.random.next_float() * (1.0_f32 - 0.9_f32) + 0.9_f32;
        if cue.particles {
            self.throw_crumbs(crumbs);
        }
        let (volume, pitch) = if cue.drink {
            (0.5, drink_pitch)
        } else {
            (eat_volume, eat_pitch)
        };
        let at = self.position;
        self.pending_sounds.push(LocalSound::at(
            cue.sound,
            LocalSoundSource::Players,
            (volume, pitch),
            (at.x, at.y, at.z),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bread() -> ConsumeCue {
        ConsumeCue {
            ticks: 32,
            drink: false,
            sound: "minecraft:entity.generic.eat",
            particles: true,
        }
    }

    /// 32 ticks: the wait is 7 (`(int)(32 * 0.21875)`), so the first emit is the first tick of
    /// a multiple of four left after more than 7 have passed: 8 used, 24 left. It goes on past
    /// the end (negative ticks left) until the server ends the use.
    #[test]
    fn eating_sounds_every_fourth_tick_after_the_first_seventh() {
        let used: Vec<i32> = (0..40).filter(|used| bread().emits(*used)).collect();
        assert_eq!(used, [8, 12, 16, 20, 24, 28, 32, 36]);
    }

    fn using(cue: ConsumeCue) -> LocalPlayer {
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            Vec3::ZERO,
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        player.hands.using_item = true;
        player.hands.main_consume = Some(cue);
        player
    }

    fn sounded(player: &mut LocalPlayer, ticks: i32) -> Vec<LocalSound> {
        for used in 0..ticks {
            player.use_edge.ticks = used;
            player.consume_tick();
        }
        player.take_sounds()
    }

    /// Eating is heard at the component's sound with half or full volume and a pitch near one;
    /// drinking at half volume and a pitch between 0.9 and one.
    #[test]
    fn the_volume_and_pitch_follow_the_animation() {
        let eaten = sounded(&mut using(bread()), 33);
        assert_eq!(eaten.len(), 7);
        for sound in &eaten {
            assert_eq!(sound.event, "minecraft:entity.generic.eat");
            assert!(sound.volume == 0.5 || sound.volume == 1.0);
            assert!((0.8..=1.2).contains(&sound.pitch));
            assert_eq!(sound.source, LocalSoundSource::Players);
        }
        let drink = ConsumeCue {
            drink: true,
            sound: "minecraft:entity.generic.drink",
            particles: false,
            ..bread()
        };
        for sound in sounded(&mut using(drink), 33) {
            assert_eq!(sound.volume, 0.5);
            assert!((0.9..1.0).contains(&sound.pitch));
        }
    }

    /// Nothing sounds when nothing is being used or the hand holds no consumable.
    #[test]
    fn a_hand_without_a_consumable_is_silent() {
        let mut player = using(bread());
        player.hands.main_consume = None;
        assert!(sounded(&mut player, 40).is_empty());
    }

    /// The server's completion sounds once more at the item's sound and ends the use; with no
    /// use going on it does nothing.
    #[test]
    fn the_completion_event_sounds_once_and_ends_the_use() {
        let mut player = using(bread());
        player.use_edge.item = Some(player.hands.main_item);
        player.complete_using_item();
        assert!(!player.is_using_item());
        let sounds = player.take_sounds();
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0].event, "minecraft:entity.generic.eat");
        player.complete_using_item();
        assert!(player.take_sounds().is_empty());
    }

    /// Five crumbs for each tick that sounds, sixteen at the completion, none when the component
    /// has no particles; each is one particle of the item, thrown in front of the face.
    #[test]
    fn the_crumbs_are_five_a_tick_and_sixteen_at_the_end() {
        let mut player = using(bread());
        player.use_edge.item = Some(player.hands.main_item);
        sounded(&mut player, 33);
        let crumbs = player.take_particles();
        assert_eq!(crumbs.len(), 7 * 5);
        let first = crumbs[0];
        assert_eq!(first.count, 0);
        assert_eq!(
            first.type_id,
            photon_data::particle_type::id("item").expect("the item particle")
        );
        // Looking along +z from the feet at the origin: 0.6 ahead, 0.3 to 0.9 below the eye.
        let eye = f64::from(player.eye_height());
        assert!((first.at.2 - 0.6).abs() < 1e-6);
        assert!(first.at.1 < eye - 0.3 + 1e-9 && first.at.1 > eye - 0.9 - 1e-9);
        assert!(first.at.0.abs() <= 0.15 + 1e-9);
        player.complete_using_item();
        assert_eq!(player.take_particles().len(), 16);
        let mut bare = using(ConsumeCue {
            particles: false,
            ..bread()
        });
        sounded(&mut bare, 33);
        assert!(bare.take_particles().is_empty());
    }

    /// A spyglass whose use the server completes is lowered with its sound.
    #[test]
    fn a_spyglass_completed_by_the_server_sounds_as_it_is_lowered() {
        let mut player = using(bread());
        player.hands.main_consume = None;
        player.hands.main_item = photon_data::item::registry::named("minecraft:spyglass");
        player.use_edge.item = Some(player.hands.main_item);
        player.complete_using_item();
        let sounds = player.take_sounds();
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0].event, "minecraft:item.spyglass.stop_using");
    }

    /// A server teleport moves the old position with the new one, so the portal particles
    /// gather where the player arrived.
    #[test]
    fn a_teleported_player_throws_portal_particles_where_it_arrived() {
        let mut player = using(bread());
        player.position = Vec3::new(40.0, 70.0, -12.0);
        player.old_position = player.position;
        player.throw_teleport_particles();
        let thrown = player.take_particles();
        assert_eq!(thrown.len(), 128);
        assert!(thrown.iter().all(|p| {
            (p.at.0 - 40.0).abs() <= 0.6 && (p.at.2 + 12.0).abs() <= 0.6 && p.at.1 >= 70.0
        }));
    }
}
