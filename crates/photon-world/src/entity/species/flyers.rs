use super::super::*;

impl ClientEntity {
    /// Whether this is a parrot.
    pub(crate) fn is_parrot(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:parrot")
    }
}

impl EntityViewMut<'_> {
    /// `Parrot.calculateFlapping` after `super.aiStep`: the wings beat by
    /// whether the parrot is in the air, and one falling there sinks slowly.
    pub(crate) fn tick_parrot(&mut self, jukebox_near: bool) {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:parrot") {
            return;
        }
        let (on_ground, passenger) = (self.on_ground, self.vehicle_id.is_some());
        if !on_ground && self.vel.motion.y.get() < 0.0 {
            self.vel.motion.y = JavaF64::new(self.vel.motion.y.get() * 0.6);
        }
        let parrot = self.kind.parrot_state();
        parrot.end_party_unless(jukebox_near);
        parrot.tick(on_ground, passenger);
    }

    /// `Bat.tick` after `super.tick`: a hanging bat does not move and hangs
    /// from the block above it; one flying slows its fall and climb.
    pub(crate) fn tick_bat(&mut self) {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:bat") {
            return;
        }
        let resting = self.living.bat_resting;
        if resting {
            self.vel.motion.x = JavaF64::new(0.0_f64);
            self.vel.motion.y = JavaF64::new(0.0_f64);
            self.vel.motion.z = JavaF64::new(0.0_f64);
            let height = photon_data::entity_type::pick(self.type_id)
                .map_or(0.0, |pick| pick.size.1 * self.scale());
            let y = self.pos.position.y.get();
            self.pos.position.y = JavaF64::new(y.floor() + 1.0 - f64::from(height));
        } else {
            self.vel.motion.y = JavaF64::new(self.vel.motion.y.get() * 0.6);
        }
        let tick_count = self.tick_count;
        self.kind.bat_state().tick(resting, tick_count);
    }
}

impl EntityView<'_> {
    /// A parrot's wing beats.
    pub fn parrot(&self) -> Option<&crate::mobs::parrot::ParrotState> {
        self.kind.parrot()
    }

    /// A bat's animation states.
    pub fn bat(&self) -> Option<&crate::mobs::bat::BatState> {
        self.kind.bat()
    }

    /// An allay's progress.
    pub fn allay(&self) -> Option<&crate::mobs::allay::AllayState> {
        self.kind.allay()
    }
}

impl EntityViewMut<'_> {
    /// `Parrot.setRecordPlayingNearby`.
    pub(crate) fn set_record_playing(&mut self, jukebox: (i32, i32, i32), playing: bool) {
        self.kind
            .parrot_state()
            .set_record_playing(jukebox, playing);
    }

    /// `Allay.tick`'s client half, after `super.tick`.
    pub(crate) fn tick_allay(&mut self) {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:allay") {
            return;
        }
        let holds = self.equipment(EquipmentSlot::MainHand).is_some();
        let dancing = self.living.allay_dancing;
        self.kind.allay_state().tick(holds, dancing);
    }
}

impl EntityView<'_> {
    /// `Bee.getRollAmount`: how far the bee has rolled over, between two ticks.
    pub fn bee_roll_amount(&self, partial: f32) -> f32 {
        self.living.animal.bee_roll_at(partial)
    }

    /// `Bee.hasStung`.
    pub fn bee_has_stung(&self) -> bool {
        self.living.bee_flags & 4 != 0
    }

    /// `Bee.hasNectar`.
    pub fn bee_has_nectar(&self) -> bool {
        self.living.bee_flags & 8 != 0
    }

    /// `Parrot.getVariant`'s id.
    pub fn parrot_variant(&self) -> i32 {
        self.living.parrot_variant
    }

    /// `Ghast.isCharging`: the ghast has its mouth open for a fireball.
    pub fn is_ghast_charging(&self) -> bool {
        self.living.ghast_charging
    }

    /// `Phantom.getPhantomSize`.
    pub fn phantom_size(&self) -> i32 {
        self.living.phantom_size
    }

    /// `Bat.isResting`.
    pub fn is_bat_resting(&self) -> bool {
        self.living.bat_resting
    }

    /// `Vex.isCharging`.
    pub fn is_vex_charging(&self) -> bool {
        self.living.vex_charging
    }

    /// `Allay.isDancing`.
    pub fn is_allay_dancing(&self) -> bool {
        self.living.allay_dancing
    }
}
