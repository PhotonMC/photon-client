use super::*;

impl MountBody {
    pub(crate) fn new(
        kind: MountKind,
        position: Vec3,
        old_position: Vec3,
        dimensions: ClientEntityDimensions,
        attributes: Attributes,
        state: MountState,
    ) -> Self {
        Self {
            kind,
            position,
            old_position,
            delta: Vec3::ZERO,
            y_rot: 0.0,
            x_rot: 0.0,
            on_ground: false,
            supporting_block: None,
            on_ground_no_blocks: false,
            dimensions,
            attributes,
            state,
            standing: false,
            allow_stand_sliding: false,
            floats_when_ridden: false,
            controller: Controller::default(),
            camel: CamelRide::default(),
            water: FluidTracker::default(),
            lava: FluidTracker::default(),
            movements: Vec::new(),
        }
    }

    pub(super) fn bounding_box(&self) -> Aabb {
        self.dimensions.make_bounding_box(self.position)
    }

    pub(super) fn set_pos(&mut self, position: Vec3) {
        self.position = position;
    }

    pub(super) fn is_in_water(&self) -> bool {
        self.water.height > 0.0
    }

    pub(super) fn is_in_lava(&self) -> bool {
        self.lava.height > 0.0
    }

    /// `Entity.getFluidJumpThreshold`.
    pub(super) fn fluid_jump_threshold(&self) -> f64 {
        if self.dimensions.eye_height() < 0.4 {
            0.0
        } else {
            0.4
        }
    }

    /// `LivingEntity.getEffectiveGravity`: the attribute, none without gravity.
    pub(super) fn gravity(&self) -> f64 {
        self.attributes.gravity
    }

    /// `LivingEntity.tick` for a saddled horse the local player rides: the fluids around it, then
    /// `aiStep`, which hands the mount to its rider's keys (`travelRidden`) and applies what the
    /// blocks it passed through do to it.
    pub(crate) fn tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        self.update_fluid_interaction(level)?;
        self.state.lava_height = self.lava.height;
        self.state.in_liquid = self.is_in_water() || self.is_in_lava();
        // `LivingEntity.aiStep`: a small motion is none, for a mob on each axis.
        self.state.no_jump_delay = (self.state.no_jump_delay - 1).max(0);
        let delta = self.delta;
        self.delta = Vec3::new(
            if delta.x.abs() < 0.003 { 0.0 } else { delta.x },
            if delta.y.abs() < 0.003 { 0.0 } else { delta.y },
            if delta.z.abs() < 0.003 { 0.0 } else { delta.z },
        );
        // Nothing but the rider asks it to jump, so the jump wait stays spent.
        self.state.no_jump_delay = 0;
        self.travel_ridden(level)?;
        self.apply_effects_from_blocks(level);
        if self.kind == MountKind::Strider {
            self.float_strider(level);
        }
        Ok(())
    }

    /// The level as this mount's collisions see it: a strider's feet stand on lava.
    pub(super) fn seen_by<'a>(&self, level: &Level<'a>) -> Level<'a> {
        if self.kind == MountKind::Strider {
            level.standing_on_lava_at(self.position.y)
        } else {
            *level
        }
    }

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

    /// `LivingEntity.travelRidden`.
    pub(super) fn travel_ridden(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let ridden_input = self.ridden_input();
        self.tick_ridden(level, ridden_input);
        if self.kind == MountKind::Camel {
            // `Camel.getRiddenSpeed`, after the dash may have started: faster for a rider that
            // sprints while it can dash.
            let cooldown = self.camel.dash_cooldown.unwrap_or(self.camel.cooldown);
            let bonus = if self.controller.sprinting && cooldown == 0 {
                0.1_f32
            } else {
                0.0
            };
            self.attributes.speed = self.attributes.base_speed as f32 + bonus;
        }
        self.travel(level, ridden_input)
    }

    /// `AbstractHorse.getRiddenInput`: still while it rears, otherwise the rider's keys, sideways
    /// at half and backwards at a quarter.
    pub(super) fn ridden_input(&self) -> Vec3 {
        // `Pig.getRiddenInput`: always straight ahead; the stick's rider only steers.
        if matches!(self.kind, MountKind::Pig | MountKind::Strider) {
            return Vec3::new(0.0, 0.0, 1.0);
        }
        if self.kind == MountKind::Camel && self.camel.refuses_to_move {
            return Vec3::ZERO;
        }
        if self.on_ground
            && self.state.jump_pending_scale == 0.0
            && self.standing
            && !self.allow_stand_sliding
        {
            return Vec3::ZERO;
        }
        let sideways = self.controller.xxa * 0.5;
        let mut forward = self.controller.zza;
        if forward <= 0.0 {
            forward *= 0.25;
        }
        Vec3::new(f64::from(sideways), 0.0, f64::from(forward))
    }

    /// `AbstractHorse.tickRidden`: it faces where the rider does (looking up only half as far),
    /// and, on the ground, carries out the jump the rider asked for.
    pub(super) fn tick_ridden(&mut self, level: &Level<'_>, ridden_input: Vec3) {
        // `Camel.getRiddenRotation`: a camel that sits, or sits down or rises, keeps its look.
        if !(self.kind == MountKind::Camel && self.camel.refuses_to_move) {
            self.x_rot = (self.controller.x_rot * 0.5) % 360.0;
            self.y_rot = self.controller.y_rot % 360.0;
        }
        // A pig has no jump to carry out (`Pig.tickRidden`).
        if matches!(self.kind, MountKind::Horse | MountKind::Camel) && self.on_ground {
            if self.state.jump_pending_scale > 0.0 {
                self.execute_riders_jump(level, self.state.jump_pending_scale, ridden_input);
            }
            self.state.jump_pending_scale = 0.0;
        }
        // `Camel.tickRidden`: pushing forward on a camel that sits makes it rise.
        if self.kind == MountKind::Camel
            && self.controller.zza > 0.0
            && self.camel.sitting
            && !self.camel.in_pose_transition
        {
            self.camel.stand_up = true;
        }
    }

    /// `AbstractHorse.executeRidersJump`.
    pub(super) fn execute_riders_jump(&mut self, level: &Level<'_>, amount: f32, input: Vec3) {
        if self.kind == MountKind::Camel {
            return self.execute_camels_dash(level, amount);
        }
        let impulse = f64::from(self.jump_power(level, amount));
        self.delta = Vec3::new(self.delta.x, impulse, self.delta.z);
        if input.z > 0.0 {
            let sin = mth_sin_f64(f64::from(self.y_rot * DEG_TO_RAD));
            let cos = mth_cos_f64(f64::from(self.y_rot * DEG_TO_RAD));
            self.delta = self.delta.add_vec3(Vec3::new(
                f64::from(-0.4_f32 * sin * amount),
                0.0,
                f64::from(0.4_f32 * cos * amount),
            ));
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

    /// `Entity.getBlockSpeedFactor`.
    pub(super) fn block_speed_factor(&self, level: &Level<'_>) -> f32 {
        let below = self.get_on_pos(level, 0.500_001);
        level.block_speed_factor(self.block_position(), below)
    }

    /// `LivingEntity.getJumpPower(multiplier)`: the jump strength, the block's own jump factor
    /// (no jump boost on a mount here).
    pub(super) fn jump_power(&self, level: &Level<'_>, multiplier: f32) -> f32 {
        self.attributes.jump_strength * multiplier * self.block_jump_factor(level) + 0.0
    }

    pub(super) fn block_position(&self) -> BlockPos {
        BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        )
    }

    pub(super) fn get_on_pos(&self, level: &Level<'_>, offset: f32) -> BlockPos {
        level.on_pos(self.supporting_block, self.position, offset)
    }

    /// `Entity.getBlockJumpFactor`.
    pub(super) fn block_jump_factor(&self, level: &Level<'_>) -> f32 {
        let here = level.info(self.block_position()).jump_factor;
        let below = level.info(self.get_on_pos(level, 0.500_001)).jump_factor;
        if here == 1.0 { below } else { here }
    }

    /// `Entity.updateFluidInteraction`: the water and lava the box is in, and what their
    /// currents do to the mount.
    pub(super) fn update_fluid_interaction(
        &mut self,
        level: &Level<'_>,
    ) -> Result<(), Unsupported> {
        let found = fluid::scan(
            level,
            (
                self.bounding_box(),
                self.position,
                self.dimensions.eye_height(),
            ),
            false,
        )?;
        self.water = found.water;
        self.lava = found.lava;
        if self.water.height > 0.0 {
            self.state.fall_distance = 0.0;
            self.delta =
                fluid::apply_current(self.water, self.delta, WATER_FLOW_SCALE, Pushed::Other);
        }
        if self.lava.height > 0.0 {
            self.delta =
                fluid::apply_current(self.lava, self.delta, LAVA_FLOW_SCALE, Pushed::Other);
        }
        Ok(())
    }

    /// `LivingEntity.travel`: in a fluid, or in the air.
    pub(super) fn travel(&mut self, level: &Level<'_>, input: Vec3) -> Result<(), Unsupported> {
        // `Camel.travel`: a camel that sits, or sits down or rises, stays where it is.
        let input = if self.kind == MountKind::Camel && self.camel.refuses_to_move && self.on_ground
        {
            self.delta = self.delta.multiply_components(0.0, 1.0, 0.0);
            input.multiply_components(0.0, 1.0, 0.0)
        } else {
            input
        };
        // `shouldTravelInFluid`: a strider does not travel in the lava it stands on.
        let stands_on_fluid = self.kind == MountKind::Strider
            && level.info(self.block_position()).fluid == Fluid::Lava;
        if (self.is_in_water() || self.is_in_lava()) && !stands_on_fluid {
            self.travel_in_fluid(level, input)
        } else {
            self.travel_in_air(level, input)
        }
    }

    pub(super) fn travel_in_air(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
    ) -> Result<(), Unsupported> {
        let pos_below = self.get_on_pos(level, 0.500_001_f32);
        let block_friction = if self.on_ground {
            compute_modified_friction(
                level.info(pos_below).friction,
                self.attributes.friction_modifier,
            )
        } else {
            1.0
        };
        let movement =
            self.handle_relative_friction_and_calculate_movement(level, input, block_friction)?;
        // The client level has every chunk (`ClientLevel.hasChunk`).
        let movement_y = movement.y - self.gravity();
        let air_drag = compute_modified_friction(0.91, self.attributes.air_drag_modifier);
        let friction = block_friction * air_drag;
        let vertical_friction = compute_modified_friction(0.98, self.attributes.air_drag_modifier);
        self.delta = Vec3::new(
            movement.x * f64::from(friction),
            movement_y * f64::from(vertical_friction),
            movement.z * f64::from(friction),
        );
        Ok(())
    }

    pub(super) fn travel_in_fluid(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
    ) -> Result<(), Unsupported> {
        let is_falling = self.delta.y <= 0.0;
        let old_y = self.position.y;
        let base_gravity = self.gravity();
        if self.is_in_water() {
            // `travelInWater`.
            let mut slow_down = 0.8_f32;
            let mut speed = 0.02_f32;
            let mut water_walker = self.attributes.water_movement_efficiency;
            if !self.on_ground {
                water_walker *= 0.5;
            }
            if water_walker > 0.0 {
                slow_down += (0.546_000_06 - slow_down) * water_walker;
                speed += (self.attributes.speed - speed) * water_walker;
            }
            self.move_relative(speed, input);
            self.move_self(level, self.delta)?;
            let mut movement = self.delta;
            if self.state.horizontal_collision && self.on_climbable(level) {
                movement = Vec3::new(movement.x, 0.2, movement.z);
            }
            let movement = movement.multiply_components(
                f64::from(slow_down),
                f64::from(0.8_f32),
                f64::from(slow_down),
            );
            self.delta = fluid_falling_adjusted_movement(base_gravity, is_falling, movement);
            self.jump_out_of_fluid(level, old_y)?;
            // `floatInWaterWhileRidden`.
            if self.floats_when_ridden && self.water.height > self.fluid_jump_threshold() {
                self.delta = self.delta.add_components(0.0, f64::from(0.04_f32), 0.0);
            }
            Ok(())
        } else {
            // `travelInLava`.
            self.move_relative(0.02, input);
            self.move_self(level, self.delta)?;
            if self.lava.height <= self.fluid_jump_threshold() {
                self.delta = self.delta.multiply_components(0.5, f64::from(0.8_f32), 0.5);
                self.delta = fluid_falling_adjusted_movement(base_gravity, is_falling, self.delta);
            } else {
                self.delta = self.delta.scale(0.5);
            }
            if base_gravity != 0.0 {
                self.delta = self.delta.add_components(0.0, -base_gravity / 4.0, 0.0);
            }
            self.jump_out_of_fluid(level, old_y)
        }
    }

    /// `LivingEntity.jumpOutOfFluid`.
    pub(super) fn jump_out_of_fluid(
        &mut self,
        level: &Level<'_>,
        old_y: f64,
    ) -> Result<(), Unsupported> {
        let movement = self.delta;
        if self.state.horizontal_collision {
            let lift = movement.y + f64::from(0.6_f32) - self.position.y + old_y;
            let moved = self.bounding_box().move_by(movement.x, lift, movement.z);
            if self
                .seen_by(level)
                .no_collision(moved, self.state.fall_distance, self.position)?
                && !contains_any_liquid(level, moved)
            {
                self.delta = Vec3::new(movement.x, f64::from(0.3_f32), movement.z);
            }
        }
        Ok(())
    }

    /// `LivingEntity.handleRelativeFrictionAndCalculateMovement`.
    pub(super) fn handle_relative_friction_and_calculate_movement(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
        friction: f32,
    ) -> Result<Vec3, Unsupported> {
        let speed = self.friction_influenced_speed(friction);
        self.move_relative(speed, input);
        self.delta = self.handle_on_climbable(level, self.delta);
        self.move_self(level, self.delta)?;
        let mut movement = self.delta;
        // `jumping` is the mount's own, which only its AI sets.
        if self.state.horizontal_collision && self.on_climbable(level) {
            movement = Vec3::new(movement.x, 0.2, movement.z);
        }
        Ok(movement)
    }

    /// `LivingEntity.getFrictionInfluencedSpeed`: in the air a mount a player rides goes at a
    /// tenth of its speed.
    pub(super) fn friction_influenced_speed(&self, block_friction: f32) -> f32 {
        let speed = self.attributes.speed;
        if self.on_ground {
            if f64::from(block_friction) > 0.6 {
                speed * (0.216_000_02_f32 / (block_friction * block_friction * block_friction))
            } else {
                speed
            }
        } else {
            speed * 0.1
        }
    }

    /// `LivingEntity.handleOnClimbable`.
    pub(super) fn handle_on_climbable(&mut self, level: &Level<'_>, delta: Vec3) -> Vec3 {
        if !self.on_climbable(level) {
            return delta;
        }
        self.state.fall_distance = 0.0;
        let max = f64::from(0.15_f32);
        Vec3::new(
            java_math::clamp(delta.x, -max, max),
            java_math::max(delta.y, -max),
            java_math::clamp(delta.z, -max, max),
        )
    }
}
