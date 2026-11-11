use super::*;

impl ClientEntity {
    /// `Entity.getPositionCodec().getBase()`.
    pub(crate) fn packet_base(&self) -> (f64, f64, f64) {
        let [x, y, z] = self.tracking.packet_base_bits.map(f64::from_bits);
        (x, y, z)
    }

    pub(crate) fn set_packet_base(&mut self, base: (f64, f64, f64)) {
        self.tracking.packet_base_bits = [base.0.to_bits(), base.1.to_bits(), base.2.to_bits()];
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }
}

pub(crate) fn decode_entity_delta(base: f64, delta: i16) -> u64 {
    if delta == 0 {
        base.to_bits()
    } else {
        let encoded_base = (base * 4096.0 + 0.5).floor() as i64;
        ((encoded_base + i64::from(delta)) as f64 / 4096.0).to_bits()
    }
}

/// `VecDelta.Stepped.decode`: each step is an offset from the one before, the first from the
/// packet base. Returns the places with their ticks; the end is the last of them.
pub(crate) fn decode_entity_steps(
    base: (f64, f64, f64),
    steps: &[EntityDeltaStep],
) -> Vec<EntityPathStep> {
    let mut at = base;
    steps
        .iter()
        .map(|step| {
            at = (
                f64::from_bits(decode_entity_delta(at.0, step.delta_x)),
                f64::from_bits(decode_entity_delta(at.1, step.delta_y)),
                f64::from_bits(decode_entity_delta(at.2, step.delta_z)),
            );
            (at, step.ticks)
        })
        .collect()
}

/// Where a move packet puts the entity: the end of its steps, or the plain offset from the base.
pub(crate) fn move_target(
    base: (f64, f64, f64),
    input: &EntityMoveInput,
    path: &[EntityPathStep],
) -> Option<(f64, f64, f64)> {
    input.has_position.then(|| match path.last() {
        Some((end, _)) => *end,
        None => (
            f64::from_bits(decode_entity_delta(base.0, input.delta_x)),
            f64::from_bits(decode_entity_delta(base.1, input.delta_y)),
            f64::from_bits(decode_entity_delta(base.2, input.delta_z)),
        ),
    })
}

pub(crate) fn resolve_entity_teleport_motion(
    current_motion: (f64, f64, f64),
    current_rotation: (f32, f32),
    target_rotation: (f32, f32),
    delta_movement: (f64, f64, f64),
    relative_flags: i32,
) -> (f64, f64, f64) {
    let current_motion = if relative_flags & ENTITY_RELATIVE_ROTATE_DELTA_FLAG != 0 {
        rotate_entity_motion(current_motion, current_rotation, target_rotation)
    } else {
        current_motion
    };
    (
        if relative_flags & ENTITY_RELATIVE_DELTA_X_FLAG != 0 {
            current_motion.0 + delta_movement.0
        } else {
            delta_movement.0
        },
        if relative_flags & ENTITY_RELATIVE_DELTA_Y_FLAG != 0 {
            current_motion.1 + delta_movement.1
        } else {
            delta_movement.1
        },
        if relative_flags & ENTITY_RELATIVE_DELTA_Z_FLAG != 0 {
            current_motion.2 + delta_movement.2
        } else {
            delta_movement.2
        },
    )
}

pub(crate) fn rotate_entity_motion(
    current_motion: (f64, f64, f64),
    current_rotation: (f32, f32),
    target_rotation: (f32, f32),
) -> (f64, f64, f64) {
    let diff_y_rot = current_rotation.0 - target_rotation.0;
    let diff_x_rot = current_rotation.1 - target_rotation.1;
    let x_radians = (f64::from(diff_x_rot) * (std::f64::consts::PI / 180.0)) as f32;
    let y_radians = (f64::from(diff_y_rot) * (std::f64::consts::PI / 180.0)) as f32;
    let x_cos = f64::from(mth_cos_f64(f64::from(x_radians)));
    let x_sin = f64::from(mth_sin_f64(f64::from(x_radians)));
    let x_rotated = (
        current_motion.0,
        current_motion.1 * x_cos + current_motion.2 * x_sin,
        current_motion.2 * x_cos - current_motion.1 * x_sin,
    );
    let y_cos = f64::from(mth_cos_f64(f64::from(y_radians)));
    let y_sin = f64::from(mth_sin_f64(f64::from(y_radians)));
    (
        x_rotated.0 * y_cos + x_rotated.2 * y_sin,
        x_rotated.1,
        x_rotated.2 * y_cos - x_rotated.0 * y_sin,
    )
}

impl EntityViewMut<'_> {
    pub(crate) fn capture_old_transform(&mut self) {
        self.pos.old_position.x = self.pos.position.x;
        self.pos.old_position.y = self.pos.position.y;
        self.pos.old_position.z = self.pos.position.z;
        self.rotation.old_x_rot = self.rotation.x_rot;
        self.rotation.old_y_rot = self.rotation.y_rot;
        self.rotation.old_y_head_rot = self.rotation.y_head_rot;
        self.rotation.old_y_body_rot = self.rotation.y_body_rot;
    }

    /// A position and turn a packet gave to an entity that does not interpolate.
    pub(crate) fn set_plain_position(
        &mut self,
        position: Option<(f64, f64, f64)>,
        (y_rot, x_rot): (Option<f32>, Option<f32>),
    ) {
        if let Some((x, y, z)) = position {
            self.pos.position.x = JavaF64::new(x);
            self.pos.position.y = JavaF64::new(y);
            self.pos.position.z = JavaF64::new(z);
        }
        if let Some(y_rot) = y_rot {
            self.rotation.y_rot = JavaF32::new(y_rot % 360.0);
        }
        if let Some(x_rot) = x_rot {
            self.rotation.x_rot = JavaF32::new((x_rot % 360.0).clamp(-90.0, 90.0));
        }
    }

    /// `Entity.baseTick`'s fluid pass, then `LivingEntity.aiStep`: a living entity
    /// eases to the position and turn its last packet gave and its head to the head
    /// turn, and a mob's motion fades; then what a species does of its own there.
    pub(crate) fn tick_ai_step(
        &mut self,
        living: Option<LivingKind>,
        in_water: Option<bool>,
        targets: EntityTargets,
        easing: AiStepEasing,
    ) {
        if self.is_ender_dragon() {
            self.tick_ender_dragon(&targets, easing);
            return;
        }
        if self.is_minecart() {
            self.tick_minecart();
            return;
        }
        if let Some(display) = self.kind.display_mut() {
            // `Display.tick`'s client half: the notes the data left, then the position's easing.
            display.tick(self.entity.tick_count);
            return;
        }
        self.tick_frog();
        self.tick_breeze();
        self.tick_wither_motion();
        if let Some(in_water) = in_water {
            self.living.in_water = in_water;
        }
        self.tick_axolotl();
        self.tick_guardian(&targets);
        let was_interpolating = mob_motion_sees_easing(
            easing.interpolating_before_tick,
            self.as_view().is_interpolating(),
        );
        self.tick_head_rotation();
        if matches!(living, Some(LivingKind::Mob | LivingKind::Other)) {
            self.tick_mob_motion(was_interpolating);
        }
        if living == Some(LivingKind::Mob) {
            self.tick_squid();
            self.tick_cube_mob();
            self.tick_shulker();
        }
        self.tick_wither_heads(targets.wither);
    }
}

/// What an entity's tick needs to know about the easing of its position to the packets': whether
/// it was running when the tick began, and the inputs of one step.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AiStepEasing {
    pub(crate) interpolating_before_tick: bool,
    pub(crate) tick_speed: f32,
    pub(crate) follow_position: bool,
}

/// Whether the ender dragon steps its position easing inside its own tick, after its flight
/// history took the position (26.2), instead of in the common tick before it (26.3).
pub(crate) fn dragon_eases_in_own_tick() -> bool {
    !photon_core::versions::rules()
        .world
        .easing_steps_in_common_tick
}

/// Whether a mob's motion fade sees the position easing as running: with 26.3 the easing has
/// already stepped when the fade looks (so its last step counts as over), with 26.2 the fade
/// looks first.
fn mob_motion_sees_easing(before_tick: bool, after_easing: bool) -> bool {
    if photon_core::versions::rules()
        .world
        .easing_steps_in_common_tick
    {
        after_easing
    } else {
        before_tick
    }
}

impl EntityViewMut<'_> {
    pub(crate) fn set_position(&mut self, position: (f64, f64, f64)) {
        if self.is_shulker() {
            let block = block_of(self.position());
            self.shulker_set_pos(position, block);
            return;
        }
        self.pos.position.x = JavaF64::new(position.0);
        self.pos.position.y = JavaF64::new(position.1);
        self.pos.position.z = JavaF64::new(position.2);
    }

    pub(crate) fn set_motion_if_finite(&mut self, motion: (f64, f64, f64)) {
        if motion.0.is_finite() && motion.1.is_finite() && motion.2.is_finite() {
            self.vel.motion.x = JavaF64::new(motion.0);
            self.vel.motion.y = JavaF64::new(motion.1);
            self.vel.motion.z = JavaF64::new(motion.2);
        }
    }

    /// What `LivingEntity.aiStep` does to the motion of a mob the client does not move:
    /// it decays by a fiftieth less while no position interpolation runs, and motion
    /// under three thousandths a tick is dropped.
    pub(crate) fn tick_mob_motion(&mut self, was_interpolating: bool) {
        let (mut x, mut y, mut z) = self.motion();
        if !was_interpolating {
            x *= 0.98;
            y *= 0.98;
            z *= 0.98;
        }
        if x.abs() < 0.003 {
            x = 0.0;
        }
        if z.abs() < 0.003 {
            z = 0.0;
        }
        if y.abs() < 0.003 {
            y = 0.0;
        }
        self.vel.motion.x = JavaF64::new(x);
        self.vel.motion.y = JavaF64::new(y);
        self.vel.motion.z = JavaF64::new(z);
    }
}

impl EntityView<'_> {
    /// What `DolphinRenderer.extractRenderState` calls moving: the motion the
    /// server gave it covers more than a ten-millionth of a block squared in a tick.
    pub fn is_moving_horizontally(&self) -> bool {
        let (x, _, z) = self.motion();
        x * x + z * z > 1.0E-7
    }
}

impl EntityView<'_> {
    /// `ClientAvatarState.deltaMovementOnPreviousTick`.
    pub fn previous_motion(&self) -> (f64, f64, f64) {
        let [x, y, z] = self.living.previous_motion.map(f64::from_bits);
        (x, y, z)
    }
}
