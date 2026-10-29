use super::*;

impl MountBody {
    /// `Strider.floatStrider`: in lava, it is held up by the lava's surface (on the ground) when
    /// its feet are above it, and otherwise rises, slowed.
    pub(super) fn float_strider(&mut self, level: &Level<'_>) {
        if !self.is_in_lava() {
            return;
        }
        let block = self.block_position();
        let above = level.info(BlockPos::new(block.x, block.y + 1, block.z));
        if self.position.y > f64::from(block.y) + 0.5 - f64::from(1.0e-5_f32) && !above.is_lava() {
            self.on_ground = true;
        } else {
            self.delta = self.delta.scale(0.5).add_components(0.0, 0.05, 0.0);
        }
    }

    /// `Camel.executeRidersJump`: a dash along where it looks, level, and a hop.
    pub(super) fn execute_camels_dash(&mut self, level: &Level<'_>, amount: f32) {
        let jump_momentum = f64::from(self.jump_power(level, 1.0));
        let look = view_vector(self.x_rot, self.y_rot);
        let along = look
            .multiply_components(1.0, 0.0, 1.0)
            .normalized()
            .scale(
                f64::from(22.2222_f32 * amount)
                    * self.attributes.base_speed
                    * f64::from(self.block_speed_factor(level)),
            )
            .add_components(0.0, f64::from(1.4285_f32 * amount) * jump_momentum, 0.0);
        self.delta = self.delta.add_vec3(along);
        self.camel.dash_cooldown = Some(55);
    }
}

/// What a ridden camel's tick leaves in its state: standing up, a dash begun, the water.
pub(super) fn write_back_camel(
    entity: &mut crate::entity::EntityViewMut<'_>,
    ride: &CamelRide,
    in_water: bool,
    game_time: i64,
) {
    let horse = entity.kind.horse_state();
    if ride.stand_up {
        horse.camel.stand_up(game_time);
        entity.status.pose = crate::entity::ClientEntityPose::Standing;
    }
    if let Some(cooldown) = ride.dash_cooldown {
        let horse = entity.kind.horse_mut().expect("made above");
        horse.camel.set_dash_cooldown(cooldown);
        horse.camel.set_dashing(true);
    }
    // `Camel.tick`: a sitting camel in the water stands up at once.
    let horse = entity.kind.horse_mut().expect("made above");
    if horse.camel.sitting() && in_water {
        horse.camel.stand_up_instantly(game_time);
        entity.status.pose = crate::entity::ClientEntityPose::Standing;
    }
}

/// `Entity.calculateViewVector`: where an entity looks, from its pitch and its yaw.
pub(super) fn view_vector(x_rot: f32, y_rot: f32) -> Vec3 {
    let real_x_rot = x_rot * DEG_TO_RAD;
    let real_y_rot = -y_rot * DEG_TO_RAD;
    let y_cos = mth_cos_f64(f64::from(real_y_rot));
    let y_sin = mth_sin_f64(f64::from(real_y_rot));
    let x_cos = mth_cos_f64(f64::from(real_x_rot));
    let x_sin = mth_sin_f64(f64::from(real_x_rot));
    Vec3::new(
        f64::from(y_sin * x_cos),
        f64::from(-x_sin),
        f64::from(y_cos * x_cos),
    )
}

/// What a ridden camel's pose and dash are, going into its tick, and what the tick makes of them.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CamelRide {
    pub(crate) sitting: bool,
    pub(crate) refuses_to_move: bool,
    pub(crate) in_pose_transition: bool,
    /// `Camel.dashCooldown` going into the tick.
    pub(crate) cooldown: i32,
    /// The tick asks it to stand up, and the dash's new cooldown.
    pub(crate) stand_up: bool,
    pub(crate) dash_cooldown: Option<i32>,
}

/// `Strider.tick`'s setting of the suffocating flag: cold unless it is in or on a warm block or in
/// lava (the depth it last looked at; a strider that rides a warm one is not a case here).
pub(super) fn strider_is_suffocating(
    world: &ClientWorld,
    strider: crate::entity::EntityView<'_>,
) -> bool {
    let level = Level::new(world);
    let (x, y, z) = strider.position();
    let inside = BlockPos::new(floor_f64(x), floor_f64(y), floor_f64(z));
    let on = level.on_pos(strider.tracking.support.main_block, Vec3::new(x, y, z), 0.2);
    let warm = |pos: BlockPos| {
        let state = crate::block::state_or_air(
            level
                .state_id(pos)
                .unwrap_or(crate::constants::VANILLA_AIR_BLOCK_STATE_ID),
        );
        world
            .block_state_registry
            .tags()
            .contains(usize::from(state.block), "minecraft:strider_warm_blocks")
    };
    !(warm(inside) || warm(on) || strider.ride.mount_ride.lava_height > 0.0)
}
