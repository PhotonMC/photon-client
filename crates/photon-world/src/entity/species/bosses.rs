use super::super::*;

impl ClientEntity {
    pub(crate) fn is_end_crystal(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:end_crystal")
    }

    pub(crate) fn is_ender_dragon(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:ender_dragon")
    }

    pub(crate) fn is_wither(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:wither")
    }

    pub(crate) fn is_armadillo(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:armadillo")
    }

    pub(crate) fn is_creaking(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:creaking")
    }

    pub(crate) fn is_warden(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:warden")
    }

    pub(crate) fn is_breeze(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:breeze")
    }
}

impl EntityViewMut<'_> {
    /// `EnderDragon.aiStep` on the client, which replaces `LivingEntity.aiStep`: the wing
    /// beat and the flight history, then the position eases to the last packet's. Nothing
    /// fades its motion and its head does not turn after anything; while it dies, none of it.
    pub(crate) fn tick_ender_dragon(&mut self, targets: &EntityTargets) {
        self.ender_dragon_mut().begin_ai_step();
        if self.as_view().is_dead_or_dying() {
            return;
        }
        let tick_count = self.tick_count;
        let heals = self
            .ender_dragon_mut()
            .check_crystals(targets.dragon, tick_count);
        if heals
            && let (Some(health), Some(max)) =
                (self.as_view().health(), self.as_view().max_health())
            && health < max
        {
            self.vitals.health = Some((health + 1.0).to_bits());
        }
        let y_rot = photon_core::math::wrap_degrees_f32(self.rotation.y_rot.get());
        self.rotation.y_rot = JavaF32::new(y_rot);
        let (motion, no_ai, y) = (self.motion(), self.is_no_ai(), self.position().1);
        self.ender_dragon_mut().fly(motion, no_ai, (y, y_rot));
        if !no_ai {
            self.tick_position_interpolation();
            self.rotation.y_body_rot = self.rotation.y_rot;
            let (at, y_rot) = (self.position(), self.rotation.y_rot.get());
            self.ender_dragon_mut().tick_parts(at, y_rot);
        }
    }

    /// `WitherBoss.aiStep` before `super.aiStep`: the motion's height settles by
    /// a factor of 0.6, and a body moving faster than that turns to face where it goes.
    pub(crate) fn tick_wither_motion(&mut self) {
        if !self.is_wither() {
            return;
        }
        let (x, y, z) = self.motion();
        self.vel.motion.y = JavaF64::new(y * 0.6);
        if x * x + z * z > 0.05 {
            let turn = photon_core::math::mth_atan2(z, x) as f32
                * (180.0_f32 / std::f64::consts::PI as f32)
                - 90.0;
            self.rotation.y_rot = JavaF32::new(turn);
        }
    }

    /// `WitherBoss.aiStep` after `super.aiStep`: the side heads turn.
    pub(crate) fn tick_wither_heads(
        &mut self,
        targets: [Option<crate::mobs::guardian::GuardianTarget>; 2],
    ) {
        if !self.is_wither() {
            return;
        }
        let place = crate::mobs::wither::WitherPlace {
            position: self.position(),
            body_rot: self.rotation.y_body_rot.get(),
            scale: self.as_view().scale(),
        };
        self.wither_mut().tick_heads(place, targets);
    }

    /// `Armadillo.tick`'s client half, after the rest of the tick: the animation
    /// states follow the state, a scared armadillo's head is held to its body
    /// (`Mob.clampHeadRotationToBody`), and the time in the state counts on.
    pub(crate) fn tick_armadillo(&mut self) {
        if !self.is_armadillo() {
            return;
        }
        let tick_count = self.tick_count;
        let state = self.armadillo_mut();
        state.animate(tick_count);
        let scared = state.is_scared();
        if scared {
            let (head, body) = (
                self.rotation.y_head_rot.get(),
                self.rotation.y_body_rot.get(),
            );
            let limit = 75.0_f32;
            let delta = photon_core::math::wrap_degrees_f32(body - head);
            let target = photon_core::math::wrap_degrees_f32(body - head).clamp(-limit, limit);
            self.rotation.y_head_rot = JavaF32::new(head + delta - target);
        }
        self.armadillo_mut().finish_tick();
    }
}

impl EntityView<'_> {
    /// An end crystal's age and data.
    pub fn end_crystal(&self) -> Option<&crate::objects::end_crystal::EndCrystalState> {
        self.kind.end_crystal()
    }

    /// An ender dragon's flight history, wing beat and phase.
    pub fn ender_dragon(&self) -> Option<&crate::mobs::ender_dragon::EnderDragonState> {
        self.kind.ender_dragon()
    }

    /// A wither's head turns, targets and invulnerable ticks.
    pub fn wither(&self) -> Option<&crate::mobs::wither::WitherState> {
        self.kind.wither()
    }

    /// The ids of the entities a wither's side heads look at, zero for none.
    pub(crate) fn wither_targets(&self) -> [i32; 2] {
        self.kind.wither().map_or([0; 2], |wither| wither.targets)
    }

    /// An armadillo's state and animation states.
    pub fn armadillo(&self) -> Option<&crate::mobs::armadillo::ArmadilloState> {
        self.kind.armadillo()
    }

    /// A creaking's animation states and flags.
    pub fn creaking(&self) -> Option<&crate::mobs::creaking::CreakingState> {
        self.kind.creaking()
    }

    /// A warden's animation states and pulses.
    pub fn warden(&self) -> Option<&crate::mobs::warden::WardenState> {
        self.kind.warden()
    }

    /// A breeze's animation states.
    pub fn breeze(&self) -> Option<&crate::mobs::breeze::BreezeState> {
        self.kind.breeze()
    }

    /// A sniffer's state and animation states.
    pub fn sniffer(&self) -> Option<&crate::mobs::sniffer::SnifferState> {
        self.kind.sniffer()
    }
}

impl EntityViewMut<'_> {
    pub(crate) fn end_crystal_mut(&mut self) -> &mut crate::objects::end_crystal::EndCrystalState {
        let _seed = i64::from(self.id.0);
        self.kind.end_crystal_state()
    }

    pub(crate) fn ender_dragon_mut(&mut self) -> &mut crate::mobs::ender_dragon::EnderDragonState {
        let _seed = i64::from(self.id.0);
        self.kind.ender_dragon_state()
    }

    pub(crate) fn wither_mut(&mut self) -> &mut crate::mobs::wither::WitherState {
        self.kind.wither_state()
    }

    pub(crate) fn armadillo_mut(&mut self) -> &mut crate::mobs::armadillo::ArmadilloState {
        self.kind.armadillo_state()
    }

    /// The creaking's state, made if it has none yet.
    pub(crate) fn creaking_mut(&mut self) -> &mut crate::mobs::creaking::CreakingState {
        let _seed = i64::from(self.id.0);
        self.kind.creaking_state()
    }

    /// The warden's state, made if it has none yet.
    pub(crate) fn warden_mut(&mut self) -> &mut crate::mobs::warden::WardenState {
        self.kind.warden_state()
    }

    /// `Breeze.tick`'s client half, before the rest of the tick.
    pub(crate) fn tick_breeze(&mut self) {
        if !self.is_breeze() {
            return;
        }
        let (pose, tick_count) = (self.pose, self.tick_count);
        self.kind.breeze_state().tick(pose, tick_count);
    }

    /// `Sniffer.onSyncedDataUpdated(DATA_STATE)`.
    pub(crate) fn set_sniffer_mode(&mut self, mode: crate::mobs::sniffer::SnifferMode) {
        let tick_count = self.tick_count;
        self.kind.sniffer_state().set_mode(mode, tick_count);
    }
}

impl EntityViewMut<'_> {
    /// `Creaking.tick`'s client half, after the rest of the tick.
    pub(crate) fn tick_creaking(&mut self) {
        if self.is_creaking() {
            let (tick_count, death_time) = (self.tick_count, self.living.death_time);
            self.creaking_mut().tick(tick_count, death_time);
        }
    }

    /// `Warden.tick`'s client half, after the rest of the tick.
    pub(crate) fn tick_warden(&mut self) {
        if self.is_warden() {
            let tick_count = self.tick_count;
            self.warden_mut().tick(tick_count);
        }
    }
}
