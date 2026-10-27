use super::*;

impl ClientEntity {
    /// `ThrowableProjectile.tick`'s client-side step, shared by every type that does not
    /// override `getDefaultGravity` (snowball, egg, ender pearl, splash and lingering potion)
    /// and, with its own gravity, `ThrownExperienceBottle` (0.07 in place of the base 0.03).
    pub(crate) fn tick_snowball(&mut self) {
        self.tick_throwable_item(SNOWBALL_GRAVITY);
    }

    pub(crate) fn tick_experience_bottle(&mut self) {
        self.tick_throwable_item(VANILLA_EXPERIENCE_BOTTLE_GRAVITY);
    }

    fn tick_throwable_item(&mut self, default_gravity: f64) {
        let air_drag = f64::from(0.99_f32);
        let gravity = if self.no_gravity {
            0.0
        } else {
            default_gravity
        };
        let current_motion = self.motion();
        let motion = (
            current_motion.0 * air_drag,
            (current_motion.1 - gravity) * air_drag,
            current_motion.2 * air_drag,
        );
        self.motion.x = JavaF64::new(motion.0);
        self.motion.y = JavaF64::new(motion.1);
        self.motion.z = JavaF64::new(motion.2);

        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + motion.0);
        self.position.y = JavaF64::new(current_position.1 + motion.1);
        self.position.z = JavaF64::new(current_position.2 + motion.2);

        let horizontal_distance = (motion.0 * motion.0 + motion.2 * motion.2).sqrt();
        let target_x_rot =
            (mth_atan2(motion.1, horizontal_distance) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot = (mth_atan2(motion.0, motion.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot = JavaF32::new(projectile_lerp_rotation_f32(
            self.old_x_rot.get(),
            target_x_rot,
        ));
        self.y_rot = JavaF32::new(projectile_lerp_rotation_f32(
            self.old_y_rot.get(),
            target_y_rot,
        ));
    }

    pub(crate) fn tick_arrow(&mut self) {
        let current_position = self.position();
        let current_motion = self.motion();
        self.position.x = JavaF64::new(current_position.0 + current_motion.0);
        self.position.y = JavaF64::new(current_position.1 + current_motion.1);
        self.position.z = JavaF64::new(current_position.2 + current_motion.2);

        let horizontal_distance =
            (current_motion.0 * current_motion.0 + current_motion.2 * current_motion.2).sqrt();
        let target_x_rot = (mth_atan2(current_motion.1, horizontal_distance)
            * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot =
            (mth_atan2(current_motion.0, current_motion.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot = JavaF32::new(projectile_lerp_rotation_f32(self.x_rot.get(), target_x_rot));
        self.y_rot = JavaF32::new(projectile_lerp_rotation_f32(self.y_rot.get(), target_y_rot));

        let air_friction = f64::from(VANILLA_ARROW_AIR_FRICTION);
        let motion = (
            current_motion.0 * air_friction,
            current_motion.1 * air_friction,
            current_motion.2 * air_friction,
        );
        self.motion.x = JavaF64::new(motion.0);
        self.motion.y = JavaF64::new(
            motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_ARROW_GRAVITY
                },
        );
        self.motion.z = JavaF64::new(motion.2);
    }

    pub(crate) fn tick_experience_orb(&mut self, path_is_open: bool) {
        self.tick_position_interpolation();
        if self.first_tick {
            self.first_tick = false;
            return;
        }
        if !path_is_open {
            return;
        }

        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_EXPERIENCE_ORB_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + motion.0);
        self.position.y = JavaF64::new(current_position.1 + motion.1);
        self.position.z = JavaF64::new(current_position.2 + motion.2);

        let air_friction = f64::from(VANILLA_EXPERIENCE_ORB_AIR_FRICTION);
        self.motion.x = JavaF64::new(motion.0 * air_friction);
        self.motion.y = JavaF64::new(motion.1 * air_friction);
        self.motion.z = JavaF64::new(motion.2 * air_friction);
    }

    pub(crate) fn tick_primed_tnt(&mut self) {
        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_PRIMED_TNT_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + motion.0);
        self.position.y = JavaF64::new(current_position.1 + motion.1);
        self.position.z = JavaF64::new(current_position.2 + motion.2);

        let air_friction = f64::from(VANILLA_PRIMED_TNT_AIR_FRICTION);
        self.motion.x = JavaF64::new(motion.0 * air_friction);
        self.motion.y = JavaF64::new(motion.1 * air_friction);
        self.motion.z = JavaF64::new(motion.2 * air_friction);
    }

    pub(crate) fn tick_falling_block(&mut self) {
        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_FALLING_BLOCK_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + motion.0);
        self.position.y = JavaF64::new(current_position.1 + motion.1);
        self.position.z = JavaF64::new(current_position.2 + motion.2);

        let air_friction = f64::from(VANILLA_FALLING_BLOCK_AIR_FRICTION);
        self.motion.x = JavaF64::new(motion.0 * air_friction);
        self.motion.y = JavaF64::new(motion.1 * air_friction);
        self.motion.z = JavaF64::new(motion.2 * air_friction);
    }

    pub(crate) fn tick_firework(&mut self) {
        // An attached rocket flies with its target and does not accelerate on its own.
        if self.attached_target.is_some() {
            return;
        }
        let current_motion = self.motion();
        let movement = (
            current_motion.0 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
            current_motion.1 + VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION,
            current_motion.2 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
        );
        let current_position = self.position();
        self.position.x = JavaF64::new(current_position.0 + movement.0);
        self.position.y = JavaF64::new(current_position.1 + movement.1);
        self.position.z = JavaF64::new(current_position.2 + movement.2);

        let horizontal_distance = (movement.0 * movement.0 + movement.2 * movement.2).sqrt();
        let target_x_rot =
            (mth_atan2(movement.1, horizontal_distance) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot =
            (mth_atan2(movement.0, movement.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot = JavaF32::new(projectile_lerp_rotation_f32(
            self.old_x_rot.get(),
            target_x_rot,
        ));
        self.y_rot = JavaF32::new(projectile_lerp_rotation_f32(
            self.old_y_rot.get(),
            target_y_rot,
        ));
        self.motion.x = JavaF64::new(movement.0);
        self.motion.y = JavaF64::new(movement.1);
        self.motion.z = JavaF64::new(movement.2);
    }

    /// `FireworkRocketEntity.isShotAtAngle`.
    pub fn is_shot_at_angle(&self) -> bool {
        self.shot_at_angle
    }

    /// Whether the type draws as an item facing the camera (`ThrownItemRenderer`, with its
    /// scale and whether it is fully lit), and so has its item among its data.
    pub fn thrown_item_look(&self) -> Option<(f32, bool)> {
        match photon_data::entity_type::name(self.type_id)? {
            "minecraft:egg"
            | "minecraft:ender_pearl"
            | "minecraft:experience_bottle"
            | "minecraft:lingering_potion"
            | "minecraft:snowball"
            | "minecraft:splash_potion" => Some((1.0, false)),
            "minecraft:eye_of_ender" => Some((1.0, true)),
            "minecraft:fireball" => Some((3.0, true)),
            "minecraft:small_fireball" => Some((0.75, true)),
            _ => None,
        }
    }

    /// Whether this is an arrow, a spectral arrow or a trident (`AbstractArrow`).
    pub fn is_abstract_arrow(&self) -> bool {
        matches!(
            photon_data::entity_type::name(self.type_id),
            Some("minecraft:arrow" | "minecraft:spectral_arrow" | "minecraft:trident")
        )
    }

    /// Whether this is a fishing bobber (`FishingHook`).
    pub fn is_fishing_bobber(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:fishing_bobber")
    }

    /// Whether this is a lightning bolt (`LightningBolt`).
    pub fn is_lightning_bolt(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:lightning_bolt")
    }

    /// A lightning bolt's shape and flashes, if it has begun to tick.
    pub fn lightning(&self) -> Option<&crate::objects::lightning::LightningState> {
        self.kind.lightning()
    }

    pub(crate) fn take_lightning_pitches(&mut self) -> Option<(f32, f32)> {
        self.kind.lightning_mut()?.take_pitches()
    }

    pub(crate) fn lightning_finished(&self) -> bool {
        self.lightning_done
    }

    /// Whether this is a block of primed tnt (`PrimedTnt`).
    pub fn is_primed_tnt(&self) -> bool {
        self.type_id == VANILLA_PRIMED_TNT_ENTITY_TYPE_ID
    }

    /// A primed tnt's fuse and block, if it has any.
    pub fn primed_tnt(&self) -> Option<&crate::objects::primed_tnt::PrimedTntState> {
        self.kind.primed_tnt()
    }

    pub(crate) fn primed_tnt_mut(&mut self) -> &mut crate::objects::primed_tnt::PrimedTntState {
        self.kind.primed_tnt_or_insert_with(Default::default)
    }

    /// Whether this is a pair of evoker fangs.
    pub fn is_evoker_fangs(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:evoker_fangs")
    }

    /// Evoker fangs' bite, if it has begun to be told of.
    pub fn evoker_fangs(&self) -> Option<&crate::objects::evoker_fangs::FangsState> {
        self.kind.evoker_fangs()
    }

    pub(crate) fn evoker_fangs_mut(&mut self) -> &mut crate::objects::evoker_fangs::FangsState {
        self.kind.evoker_fangs_or_insert_with(Default::default)
    }

    /// An arrow's shake and data, if it has any.
    pub fn arrow(&self) -> Option<&crate::objects::arrow::ArrowState> {
        self.kind.arrow()
    }

    pub(crate) fn arrow_mut(&mut self) -> &mut crate::objects::arrow::ArrowState {
        self.kind.arrow_or_insert_with(Default::default)
    }

    /// An experience orb's value (`ExperienceOrb.getValue`), once told.
    pub fn orb_value(&self) -> Option<i32> {
        self.living.orb_value
    }
}

/// Shared `position + motion - gravity` step; the operand order matches the
/// reference, so the three entities using it stay bit-identical to Java.
pub(super) fn gravity_step(
    position: (f64, f64, f64),
    motion: (f64, f64, f64),
    no_gravity: bool,
    gravity: f64,
) -> (f64, f64, f64) {
    (
        position.0 + motion.0,
        position.1 + motion.1 - if no_gravity { 0.0 } else { gravity },
        position.2 + motion.2,
    )
}

/// Map a pinned entity type id onto its ballistic tick, if it has one.
pub(crate) fn vanilla_projectile_kind(type_id: EntityTypeId) -> Option<VanillaProjectileKind> {
    match type_id {
        VANILLA_SNOWBALL_ENTITY_TYPE_ID
        | VANILLA_EGG_ENTITY_TYPE_ID
        | VANILLA_ENDER_PEARL_ENTITY_TYPE_ID
        | VANILLA_SPLASH_POTION_ENTITY_TYPE_ID
        | VANILLA_LINGERING_POTION_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Snowball),
        VANILLA_EXPERIENCE_BOTTLE_ENTITY_TYPE_ID => Some(VanillaProjectileKind::ExperienceBottle),
        VANILLA_ARROW_ENTITY_TYPE_ID
        | VANILLA_SPECTRAL_ARROW_ENTITY_TYPE_ID
        | VANILLA_TRIDENT_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Arrow),
        VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID => Some(VanillaProjectileKind::ExperienceOrb),
        VANILLA_PRIMED_TNT_ENTITY_TYPE_ID => Some(VanillaProjectileKind::PrimedTnt),
        VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID => Some(VanillaProjectileKind::FallingBlock),
        VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Firework),
        _ => None,
    }
}

/// The reference entities whose client tick is a plain one-step ballistic move
/// guarded by an air check. Each variant reproduces its own Java step exactly;
/// the grouping is only over the shared guard, never over the arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VanillaProjectileKind {
    Snowball,
    /// `ThrownExperienceBottle`: the same step as `Snowball`, its own gravity.
    ExperienceBottle,
    Arrow,
    ExperienceOrb,
    PrimedTnt,
    FallingBlock,
    Firework,
}

impl VanillaProjectileKind {
    /// Position after one tick, in the reference's own operand order.
    pub(crate) fn next_position(
        self,
        position: (f64, f64, f64),
        motion: (f64, f64, f64),
        no_gravity: bool,
    ) -> (f64, f64, f64) {
        match self {
            Self::Snowball | Self::ExperienceBottle => {
                let default_gravity = if self == Self::ExperienceBottle {
                    VANILLA_EXPERIENCE_BOTTLE_GRAVITY
                } else {
                    SNOWBALL_GRAVITY
                };
                let air_drag = f64::from(0.99_f32);
                let gravity = if no_gravity { 0.0 } else { default_gravity };
                (
                    position.0 + motion.0 * air_drag,
                    position.1 + (motion.1 - gravity) * air_drag,
                    position.2 + motion.2 * air_drag,
                )
            }
            Self::Arrow => (
                position.0 + motion.0,
                position.1 + motion.1,
                position.2 + motion.2,
            ),
            Self::ExperienceOrb => {
                gravity_step(position, motion, no_gravity, VANILLA_EXPERIENCE_ORB_GRAVITY)
            }
            Self::PrimedTnt => {
                gravity_step(position, motion, no_gravity, VANILLA_PRIMED_TNT_GRAVITY)
            }
            Self::FallingBlock => {
                gravity_step(position, motion, no_gravity, VANILLA_FALLING_BLOCK_GRAVITY)
            }
            Self::Firework => {
                let movement = (
                    motion.0 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
                    motion.1 + VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION,
                    motion.2 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
                );
                (
                    position.0 + movement.0,
                    position.1 + movement.1,
                    position.2 + movement.2,
                )
            }
        }
    }
}
