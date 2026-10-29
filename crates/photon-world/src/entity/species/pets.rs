use super::super::*;

impl ClientEntity {
    pub fn is_aggressive(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_AGGRESSIVE)
    }

    /// The game time a neutral mob's anger ends (`getPersistentAngerEndTime`).
    pub fn anger_end(&self) -> i64 {
        self.living.anger_end
    }

    /// `NeutralMob.isAngry`: the anger ends later than the game time now.
    pub fn is_angry(&self, game_time: u64) -> bool {
        let end = self.living.anger_end;
        end > 0 && end - game_time as i64 > 0
    }

    /// `TamableAnimal.isInSittingPose`.
    pub fn is_sitting(&self) -> bool {
        self.living.tamable_flags & 1 != 0
    }

    /// `TamableAnimal.isTame`.
    pub fn is_tame(&self) -> bool {
        self.living.tamable_flags & 4 != 0
    }

    /// `TamableAnimal.getOwnerReference`: the owner's uuid, when the data names one.
    pub fn owner(&self) -> Option<photon_core::binary::Uuid> {
        self.living.owner
    }

    /// The dye of a wolf's collar (`Wolf.getCollarColor`).
    pub fn collar_dye(&self) -> i32 {
        // `DyeColor.RED` until the data says otherwise.
        self.living.collar.unwrap_or(14)
    }

    /// Whether this is a cat whose data says it lies down.
    pub(crate) fn is_lying_cat(&self) -> bool {
        self.living.cat_lying
            && photon_data::entity_type::name(self.type_id) == Some("minecraft:cat")
    }

    /// `Cat.tick` after `super.tick`: the lie-down and relax amounts.
    pub(crate) fn tick_cat(&mut self, sleeper_near: bool) {
        if photon_data::entity_type::name(self.type_id) == Some("minecraft:cat") {
            let (lying, relaxed) = (self.living.cat_lying, self.living.cat_relaxed);
            self.living.cat.tick(lying, relaxed, sleeper_near);
        }
    }

    /// A panda's data and own client state (its counters and how far it sits,
    /// lies on its back and rolls); the starting state until data comes.
    pub fn panda(&self) -> crate::mobs::panda::PandaState {
        self.kind.panda().cloned().unwrap_or_default()
    }

    /// Fixes the source a panda's chance to eat is drawn from (see
    /// [`crate::mobs::panda::PandaState::fix_seed`]).
    pub fn fix_panda_seed(&mut self, seed: i64) {
        self.kind
            .panda_or_insert_with(Default::default)
            .fix_seed(seed);
    }

    /// A ravager's counters (the starting state until an event comes).
    pub fn ravager(&self) -> crate::mobs::ravager::RavagerState {
        self.kind.ravager().cloned().unwrap_or_default()
    }

    /// A fox's flags, variant and own client state (its head roll and crouch).
    pub fn fox(&self) -> &crate::mobs::fox::FoxState {
        &self.living.fox
    }

    /// `LivingEntityRenderer.isShaking` with the overrides of the hoglin
    /// (`Hoglin.isConverting`) and the strider (`Strider.isSuffocating`):
    /// frozen through, converting to a zoglin, or out of lava and cold.
    pub fn is_shaking(&self, piglins_zombify: bool) -> bool {
        self.is_fully_frozen()
            || match photon_data::entity_type::name(self.type_id) {
                Some("minecraft:hoglin") => {
                    !self.living.hoglin_immune && !self.is_no_ai() && piglins_zombify
                }
                Some("minecraft:strider") => self.living.strider_suffocating,
                _ => false,
            }
    }

    /// A polar bear's own client state (how far it has stood up).
    pub fn polar_bear(&self) -> &crate::mobs::polar_bear::PolarBearState {
        &self.living.polar_bear
    }

    /// A cat's own client state (how far it lies and relaxes).
    pub fn cat(&self) -> &crate::mobs::cat::CatState {
        &self.living.cat
    }

    /// A wolf's own client state (its interest and its shake).
    pub fn wolf(&self) -> &crate::mobs::wolf::WolfState {
        &self.living.wolf
    }

    /// `Wolf.getTailAngle`: raised while angry, by health while tame.
    pub fn wolf_tail_angle(&self, game_time: u64) -> f32 {
        let pi = std::f64::consts::PI as f32;
        if self.is_angry(game_time) {
            1.539_380_4
        } else if self.is_tame() {
            let max_health = self.max_health().unwrap_or(0.0);
            let health = self.health().unwrap_or(max_health);
            let damage_ratio = (max_health - health) / max_health;
            (0.55 - damage_ratio * 0.4) * pi
        } else {
            (std::f64::consts::PI / 5.0) as f32
        }
    }
}

impl EntityViewMut<'_> {
    /// `Panda.tick` after `super.tick`; a sitting panda levels its pitch.
    pub(crate) fn tick_panda(&mut self, thundering: bool) {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:panda") {
            return;
        }
        let holds_item = self.equipment[EquipmentSlot::MainHand as usize].is_some();
        let around = crate::mobs::panda::PandaSurroundings {
            thundering,
            holds_item,
        };
        let panda = self.kind.panda_or_insert_with(Default::default);
        if panda.tick(around) {
            self.rotation.x_rot = JavaF32::new(0.0_f32);
        }
    }
}
