use super::super::*;

impl ClientEntity {
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

    /// A parrot's wing beats, if it has ticked.
    pub fn parrot(&self) -> Option<&crate::mobs::parrot::ParrotState> {
        self.kind.parrot()
    }

    /// `Parrot.calculateFlapping` after `super.aiStep`: the wings beat by
    /// whether the parrot is in the air, and one falling there sinks slowly.
    pub(crate) fn tick_parrot(&mut self) {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:parrot") {
            return;
        }
        let (on_ground, passenger) = (self.on_ground, self.vehicle_id.is_some());
        if !on_ground && self.motion.y.get() < 0.0 {
            self.motion.y = JavaF64::new(self.motion.y.get() * 0.6);
        }
        self.kind
            .parrot_or_insert_with(Default::default)
            .tick(on_ground, passenger);
    }

    /// A bat's animation states, if it has ticked.
    pub fn bat(&self) -> Option<&crate::mobs::bat::BatState> {
        self.kind.bat()
    }

    /// An allay's progress, if it has ticked.
    pub fn allay(&self) -> Option<&crate::mobs::allay::AllayState> {
        self.kind.allay()
    }

    /// `Bat.tick` after `super.tick`: a hanging bat does not move and hangs
    /// from the block above it; one flying slows its fall and climb.
    pub(crate) fn tick_bat(&mut self) {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:bat") {
            return;
        }
        let resting = self.living.bat_resting;
        if resting {
            self.motion.x = JavaF64::new(0.0_f64);
            self.motion.y = JavaF64::new(0.0_f64);
            self.motion.z = JavaF64::new(0.0_f64);
            let height = photon_data::entity_type::pick(self.type_id)
                .map_or(0.0, |pick| pick.size.1 * self.scale());
            let y = self.position.y.get();
            self.position.y = JavaF64::new(y.floor() + 1.0 - f64::from(height));
        } else {
            self.motion.y = JavaF64::new(self.motion.y.get() * 0.6);
        }
        let tick_count = self.tick_count;
        self.kind
            .bat_or_insert_with(Default::default)
            .tick(resting, tick_count);
    }

    /// `Allay.tick`'s client half, after `super.tick`.
    pub(crate) fn tick_allay(&mut self) {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:allay") {
            return;
        }
        let holds = self.equipment[EquipmentSlot::MainHand as usize].is_some();
        let dancing = self.living.allay_dancing;
        self.kind
            .allay_or_insert_with(Default::default)
            .tick(holds, dancing);
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
