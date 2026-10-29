use super::super::*;

impl ClientEntity {
    /// Whether this is one of the fish `AbstractFish` makes.
    pub fn is_fish(&self) -> bool {
        matches!(
            photon_data::entity_type::name(self.type_id),
            Some(
                "minecraft:cod"
                    | "minecraft:salmon"
                    | "minecraft:tropical_fish"
                    | "minecraft:pufferfish"
            )
        )
    }

    pub(crate) fn is_guardian(&self) -> bool {
        matches!(
            photon_data::entity_type::name(self.type_id),
            Some("minecraft:guardian" | "minecraft:elder_guardian")
        )
    }

    /// `Guardian.getAttackDuration`, shorter for the elder.
    pub fn guardian_attack_duration(&self) -> i32 {
        if photon_data::entity_type::name(self.type_id) == Some("minecraft:elder_guardian") {
            60
        } else {
            80
        }
    }

    /// Whether this is a frog.
    pub(crate) fn is_frog(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:frog")
    }

    pub(crate) fn is_squid(&self) -> bool {
        matches!(
            photon_data::entity_type::name(self.type_id),
            Some("minecraft:squid" | "minecraft:glow_squid")
        )
    }
}

impl EntityViewMut<'_> {
    /// `Squid.aiStep`, which turns the body (and with it the entity) in water.
    pub(crate) fn tick_squid(&mut self) {
        if !self.is_squid() {
            return;
        }
        let (in_water, motion, body) = (
            self.living.in_water,
            self.motion(),
            self.rotation.y_body_rot.get(),
        );
        let turned = self.kind.squid_state().ai_step(in_water, motion, body);
        if let Some(body) = turned {
            self.rotation.y_body_rot = JavaF32::new(body);
            self.rotation.y_rot = JavaF32::new(body);
        }
        // `GlowSquid.aiStep`: the dark ticks a hurt glow squid was given run out.
        if self.living.glow_dark_ticks > 0 {
            self.living.glow_dark_ticks -= 1;
        }
    }

    /// `Guardian.aiStep`'s client half, while the guardian lives: the beam's
    /// target (which the world resolves from its id) is where the head turns.
    pub(crate) fn tick_guardian(&mut self, target: Option<crate::mobs::guardian::GuardianTarget>) {
        if !self.is_guardian() || !self.alive {
            return;
        }
        let eye_height = self
            .dimensions()
            .scale(self.scale(), self.scale())
            .eye_height();
        let place = crate::mobs::guardian::GuardianPlace {
            in_water: self.living.in_water,
            moving: self.living.guardian_moving,
            position: self.position(),
            eye_height,
            target,
        };
        let mut pose = crate::mobs::guardian::GuardianPose {
            y_rot: self.rotation.y_rot.get(),
            x_rot: self.rotation.x_rot.get(),
            y_head_rot: self.rotation.y_head_rot.get(),
            motion: self.motion(),
            on_ground: self.on_ground,
        };
        let has_target = self.living.guardian_target != 0;
        let duration = self.guardian_attack_duration();
        self.kind
            .guardian_state()
            .tick(&place, has_target, duration, &mut pose);
        self.rotation.y_rot = JavaF32::new(pose.y_rot);
        self.rotation.x_rot = JavaF32::new(pose.x_rot);
        self.rotation.y_head_rot = JavaF32::new(pose.y_head_rot);
        self.vel.motion = DVec3::new(pose.motion.0, pose.motion.1, pose.motion.2);
        self.on_ground = pose.on_ground;
    }
}

impl EntityView<'_> {
    /// An axolotl's animators and animation states.
    pub fn axolotl(&self) -> Option<&crate::mobs::axolotl::AxolotlState> {
        self.kind.axolotl()
    }

    /// A guardian's animations.
    pub fn guardian(&self) -> Option<&crate::mobs::guardian::GuardianState> {
        self.kind.guardian()
    }

    /// A frog's animation states.
    pub fn frog(&self) -> Option<&crate::mobs::frog::FrogState> {
        self.kind.frog()
    }

    /// A squid's tentacles and body turns.
    pub fn squid(&self) -> Option<&crate::mobs::squid::SquidState> {
        self.kind.squid()
    }
}

impl EntityViewMut<'_> {
    /// `Frog.tick`'s client half, which runs before the rest of the tick and
    /// so sees where the last one left the frog.
    pub(crate) fn tick_frog(&mut self) {
        if !self.is_frog() {
            return;
        }
        let (in_water, moving, tick_count) =
            (self.living.in_water, self.walk.is_moving(), self.tick_count);
        self.kind.frog_state().tick(in_water, moving, tick_count);
    }

    /// `Axolotl.baseTick` on the client: the animators and animation states
    /// follow where the axolotl is and whether it walks. The rotation test of
    /// the reference never holds here: `LivingEntity.baseTick` has set the old
    /// turn to the new before it.
    pub(crate) fn tick_axolotl(&mut self) {
        if photon_data::entity_type::name(self.type_id) != Some("minecraft:axolotl") {
            return;
        }
        let input = crate::mobs::axolotl::AxolotlInput {
            baby: self.is_baby(),
            playing_dead: self.living.axolotl_playing_dead,
            in_water: self.living.in_water,
            on_ground: self.on_ground,
            moving: self.walk.is_moving(),
            tick_count: self.tick_count,
        };
        self.kind.axolotl_state().tick(input);
    }

    /// `Guardian.onSyncedDataUpdated`: when the data holds the attack target,
    /// the beam starts over.
    pub(crate) fn set_guardian_data(&mut self, moving: Option<bool>, target: Option<i32>) {
        if let Some(moving) = moving {
            self.living.guardian_moving = moving;
        }
        if let Some(target) = target {
            self.living.guardian_target = target;
            self.kind.guardian_state().restart_attack();
        }
    }
}

impl EntityView<'_> {
    /// `Axolotl.getVariant`: the index of the variant (an id no variant has is the first).
    pub fn axolotl_variant(&self) -> usize {
        usize::try_from(self.living.axolotl_variant)
            .ok()
            .filter(|id| *id < 5)
            .unwrap_or(0)
    }

    /// `Turtle.hasEgg`.
    pub fn turtle_has_egg(&self) -> bool {
        self.living.turtle_has_egg
    }

    /// `Turtle.isLayingEgg`.
    pub fn turtle_laying_egg(&self) -> bool {
        self.living.turtle_laying_egg
    }

    /// `Salmon.getVariant`: medium until the data says.
    pub fn salmon_size(&self) -> crate::mobs::fish::SalmonSize {
        crate::mobs::fish::SalmonSize::from_id(self.living.fish_variant.unwrap_or(1))
    }

    /// `TropicalFish.getPackedVariant`: a white kob until the data says.
    pub fn tropical_fish_variant(&self) -> i32 {
        self.living.fish_variant.unwrap_or(0)
    }

    /// `Pufferfish.getPuffState`.
    pub fn puff_state(&self) -> i32 {
        self.living.fish_variant.unwrap_or(0)
    }

    /// `Guardian.getActiveAttackTarget`'s id: what the beam aims at, if anything.
    pub fn guardian_attack_target(&self) -> Option<i32> {
        (self.living.guardian_target != 0).then_some(self.living.guardian_target)
    }

    /// `Guardian.isMoving`.
    pub fn is_guardian_moving(&self) -> bool {
        self.living.guardian_moving
    }
}
