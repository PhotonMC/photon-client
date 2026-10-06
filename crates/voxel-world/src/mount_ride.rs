//! A mount the local player rides: the saddled horse family, as a client that is its authority
//! ticks it (`LivingEntity.aiStep` into `travelRidden`: the rider's keys steer it, and its own
//! attributes decide how fast and how high it goes). The movement is `LivingEntity`'s; the
//! player's own port of it is `LocalPlayer`'s.

use voxel_core::aabb::Aabb;
use voxel_core::binary::BlockPos;
use voxel_core::math::{Vec3, ceil_f64, equal_f64, floor_f64, mth_cos_f64, mth_sin_f64};
use voxel_data::block::definition::Fluid;

use crate::ClientWorld;
use crate::collision::{collide_entity_movement, select_step_up_movement};
use crate::entity::{ClientEntityDimensions, Support};
use crate::fluid::{self, FluidTracker, LAVA_FLOW_SCALE, Pushed, WATER_FLOW_SCALE};
use crate::java_math;
use crate::local_player::{
    InsideBody, InsideVisit, Level, LivingEffects, Movement, Unsupported,
    compute_modified_friction, flag, living_inside_effect, property, walk_inside_blocks,
};

/// `(float) (Math.PI / 180.0)`.
const DEG_TO_RAD: f32 = (std::f64::consts::PI / 180.0) as f32;

/// Which mount it is, for the parts of its tick that differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountKind {
    /// A saddled horse, donkey, mule or undead horse.
    Horse,
    /// A saddled pig steered with a carrot on a stick.
    Pig,
    /// A saddled strider steered with a warped fungus on a stick, which walks on lava.
    Strider,
}

/// What a ridden mount keeps from one tick to the next beyond its place and motion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MountState {
    pub fall_distance: f64,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    /// `Entity.stuckSpeedMultiplier`, spent by the next move.
    pub stuck_speed_multiplier: (f64, f64, f64),
    pub no_jump_delay: i32,
    /// `AbstractHorse.playerJumpPendingScale`: how hard the rider asked to jump, until the horse
    /// is on the ground to do it.
    pub jump_pending_scale: f32,
    /// How deep the lava it stands in reached at the last look (`getFluidHeight(LAVA)`), which a
    /// strider's warmth reads the next tick.
    pub lava_height: f64,
}

/// The values are finite, so the comparison is a total one.
impl Eq for MountState {}

impl Default for MountState {
    fn default() -> Self {
        Self {
            fall_distance: 0.0,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            stuck_speed_multiplier: (0.0, 0.0, 0.0),
            no_jump_delay: 0,
            jump_pending_scale: 0.0,
            lava_height: 0.0,
        }
    }
}

/// The player who steers the mount, as the mount's tick reads it: the keys' effect on the player's
/// own movement input (`xxa`, `zza`) and where the player looks.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Controller {
    pub xxa: f32,
    pub zza: f32,
    pub y_rot: f32,
    pub x_rot: f32,
    /// Whether a hand of the player holds a carrot on a stick, which steers a saddled pig.
    pub holds_carrot_on_a_stick: bool,
    /// Likewise for a warped fungus on a stick, which steers a saddled strider.
    pub holds_warped_fungus_on_a_stick: bool,
}

/// The values are finite, so the comparison is a total one.
impl Eq for Controller {}

/// The attributes the mount's movement reads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Attributes {
    pub(crate) speed: f32,
    pub(crate) gravity: f64,
    pub(crate) step_height: f32,
    pub(crate) jump_strength: f32,
    pub(crate) friction_modifier: f32,
    pub(crate) air_drag_modifier: f32,
    pub(crate) water_movement_efficiency: f32,
}

/// The mount as one tick reads and changes it.
#[derive(Debug, Clone)]
pub(crate) struct MountBody {
    pub(crate) kind: MountKind,
    pub(crate) position: Vec3,
    pub(crate) old_position: Vec3,
    pub(crate) delta: Vec3,
    pub(crate) y_rot: f32,
    pub(crate) x_rot: f32,
    pub(crate) on_ground: bool,
    pub(crate) supporting_block: Option<BlockPos>,
    pub(crate) on_ground_no_blocks: bool,
    pub(crate) dimensions: ClientEntityDimensions,
    pub(crate) attributes: Attributes,
    pub(crate) state: MountState,
    /// Standing (reared) and eating, which hold a saddled horse where it is.
    pub(crate) standing: bool,
    /// `AbstractHorse.allowStandSliding`.
    pub(crate) allow_stand_sliding: bool,
    /// Whether the type floats while ridden (`EntityTypeTags.CAN_FLOAT_WHILE_RIDDEN`).
    pub(crate) floats_when_ridden: bool,
    pub(crate) controller: Controller,
    water: FluidTracker,
    lava: FluidTracker,
    movements: Vec<Movement>,
}

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
            water: FluidTracker::default(),
            lava: FluidTracker::default(),
            movements: Vec::new(),
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.dimensions.make_bounding_box(self.position)
    }

    fn set_pos(&mut self, position: Vec3) {
        self.position = position;
    }

    fn is_in_water(&self) -> bool {
        self.water.height > 0.0
    }

    fn is_in_lava(&self) -> bool {
        self.lava.height > 0.0
    }

    /// `Entity.getFluidJumpThreshold`.
    fn fluid_jump_threshold(&self) -> f64 {
        if self.dimensions.eye_height() < 0.4 {
            0.0
        } else {
            0.4
        }
    }

    /// `LivingEntity.getEffectiveGravity`: the attribute, none without gravity.
    fn gravity(&self) -> f64 {
        self.attributes.gravity
    }

    /// `LivingEntity.tick` for a saddled horse the local player rides: the fluids around it, then
    /// `aiStep`, which hands the mount to its rider's keys (`travelRidden`) and applies what the
    /// blocks it passed through do to it.
    pub(crate) fn tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        self.update_fluid_interaction(level)?;
        self.state.lava_height = self.lava.height;
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
    fn seen_by<'a>(&self, level: &Level<'a>) -> Level<'a> {
        if self.kind == MountKind::Strider {
            level.standing_on_lava_at(self.position.y)
        } else {
            *level
        }
    }

    /// `Strider.floatStrider`: in lava, it is held up by the lava's surface (on the ground) when
    /// its feet are above it, and otherwise rises, slowed.
    fn float_strider(&mut self, level: &Level<'_>) {
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
    fn travel_ridden(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let ridden_input = self.ridden_input();
        self.tick_ridden(level, ridden_input);
        self.travel(level, ridden_input)
    }

    /// `AbstractHorse.getRiddenInput`: still while it rears, otherwise the rider's keys, sideways
    /// at half and backwards at a quarter.
    fn ridden_input(&self) -> Vec3 {
        // `Pig.getRiddenInput`: always straight ahead; the stick's rider only steers.
        if matches!(self.kind, MountKind::Pig | MountKind::Strider) {
            return Vec3::new(0.0, 0.0, 1.0);
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
    fn tick_ridden(&mut self, level: &Level<'_>, ridden_input: Vec3) {
        self.x_rot = (self.controller.x_rot * 0.5) % 360.0;
        self.y_rot = self.controller.y_rot % 360.0;
        // A pig has no jump to carry out (`Pig.tickRidden`).
        if self.kind == MountKind::Horse && self.on_ground {
            if self.state.jump_pending_scale > 0.0 {
                self.execute_riders_jump(level, self.state.jump_pending_scale, ridden_input);
            }
            self.state.jump_pending_scale = 0.0;
        }
    }

    /// `AbstractHorse.executeRidersJump`.
    fn execute_riders_jump(&mut self, level: &Level<'_>, amount: f32, input: Vec3) {
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

    /// `LivingEntity.getJumpPower(multiplier)`: the jump strength, the block's own jump factor
    /// (no jump boost on a mount here).
    fn jump_power(&self, level: &Level<'_>, multiplier: f32) -> f32 {
        self.attributes.jump_strength * multiplier * self.block_jump_factor(level) + 0.0
    }

    fn block_position(&self) -> BlockPos {
        BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        )
    }

    fn get_on_pos(&self, level: &Level<'_>, offset: f32) -> BlockPos {
        level.on_pos(self.supporting_block, self.position, offset)
    }

    /// `Entity.getBlockJumpFactor`.
    fn block_jump_factor(&self, level: &Level<'_>) -> f32 {
        let here = level.info(self.block_position()).jump_factor;
        let below = level.info(self.get_on_pos(level, 0.500_001)).jump_factor;
        if here == 1.0 { below } else { here }
    }

    /// `Entity.updateFluidInteraction`: the water and lava the box is in, and what their
    /// currents do to the mount.
    fn update_fluid_interaction(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
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
    fn travel(&mut self, level: &Level<'_>, input: Vec3) -> Result<(), Unsupported> {
        // `shouldTravelInFluid`: a strider does not travel in the lava it stands on.
        let stands_on_fluid = self.kind == MountKind::Strider
            && level.info(self.block_position()).fluid == Fluid::Lava;
        if (self.is_in_water() || self.is_in_lava()) && !stands_on_fluid {
            self.travel_in_fluid(level, input)
        } else {
            self.travel_in_air(level, input)
        }
    }

    fn travel_in_air(&mut self, level: &Level<'_>, input: Vec3) -> Result<(), Unsupported> {
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

    fn travel_in_fluid(&mut self, level: &Level<'_>, input: Vec3) -> Result<(), Unsupported> {
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
    fn jump_out_of_fluid(&mut self, level: &Level<'_>, old_y: f64) -> Result<(), Unsupported> {
        let movement = self.delta;
        if self.state.horizontal_collision {
            let lift = movement.y + f64::from(0.6_f32) - self.position.y + old_y;
            let moved = self.bounding_box().move_by(movement.x, lift, movement.z);
            if self
                .seen_by(level)
                .no_collision(moved, self.state.fall_distance)?
                && !contains_any_liquid(level, moved)
            {
                self.delta = Vec3::new(movement.x, f64::from(0.3_f32), movement.z);
            }
        }
        Ok(())
    }

    /// `LivingEntity.handleRelativeFrictionAndCalculateMovement`.
    fn handle_relative_friction_and_calculate_movement(
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
    fn friction_influenced_speed(&self, block_friction: f32) -> f32 {
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
    fn handle_on_climbable(&mut self, level: &Level<'_>, delta: Vec3) -> Vec3 {
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

    /// `LivingEntity.onClimbable`; `AbstractHorse` overrides it to never.
    fn on_climbable(&self, level: &Level<'_>) -> bool {
        if self.kind == MountKind::Horse {
            return false;
        }
        let pos = self.block_position();
        let info = level.info(pos);
        if info.flags & flag::CLIMBABLE != 0 {
            return true;
        }
        if info.flags & flag::TRAPDOOR != 0 {
            let state = level.state_id(pos);
            if property(state, "open") == Some("true") {
                let below_pos = BlockPos::new(pos.x, pos.y - 1, pos.z);
                let below = level.info(below_pos);
                if below.flags & flag::LADDER != 0 {
                    return property(level.state_id(below_pos), "facing")
                        == property(state, "facing");
                }
            }
        }
        false
    }

    /// `Entity.moveRelative`.
    fn move_relative(&mut self, speed: f32, input: Vec3) {
        let relative = crate::input::local_player_relative_movement(input, speed, self.y_rot);
        self.delta = self.delta.add_vec3(relative);
    }

    /// `Entity.move(SELF, ...)`: collision (it steps up what its step height allows), the flags,
    /// the fall, what the blocks do and the block's speed factor.
    fn move_self(&mut self, level: &Level<'_>, delta: Vec3) -> Result<(), Unsupported> {
        let mut delta = delta;
        let (sx, sy, sz) = self.state.stuck_speed_multiplier;
        let stuck = Vec3::new(sx, sy, sz);
        if stuck.length_squared() > 1.0e-7 {
            delta = delta.multiply_components(stuck.x, stuck.y, stuck.z);
            self.state.stuck_speed_multiplier = (0.0, 0.0, 0.0);
            self.delta = Vec3::ZERO;
        }
        let movement = self.collide(level, delta)?;
        let movement_length = movement.length_squared();
        if movement_length > 1.0e-7 || delta.length_squared() - movement_length < 1.0e-7 {
            if self.state.fall_distance != 0.0 && movement_length >= 1.0 {
                let distance = movement.length().min(8.0);
                let to = self
                    .position
                    .add_vec3(movement.normalized().scale(distance));
                if level.clip_hits_fall_reset(self.position, to) {
                    self.state.fall_distance = 0.0;
                }
            }
            let from = self.position;
            let to = from.add_vec3(movement);
            self.movements.push(Movement {
                from,
                to,
                original: Some(delta),
            });
            self.set_pos(to);
        }

        let x_collision = !equal_f64(delta.x, movement.x);
        let z_collision = !equal_f64(delta.z, movement.z);
        self.state.horizontal_collision = x_collision || z_collision;
        let moved_vertically = delta.y.abs() > 0.0;
        self.state.vertical_collision = delta.y != movement.y;
        self.state.vertical_collision_below = self.state.vertical_collision && delta.y < 0.0;
        self.on_ground = self.state.vertical_collision_below;
        let moved_box = self.bounding_box();
        self.seen_by(level).check_supporting_block(
            (&mut self.supporting_block, &mut self.on_ground_no_blocks),
            (moved_box, self.position, self.state.fall_distance),
            self.on_ground,
            Some(movement),
        )?;

        let effect_pos = self.get_on_pos(level, 0.2);
        let effect_info = level.info(effect_pos);
        self.check_fall_damage(level, movement.y)?;

        if moved_vertically && self.state.vertical_collision || self.state.horizontal_collision {
            self.restitute_movement_after_collisions(
                effect_info.flags & flag::SUPPRESSES_BOUNCE != 0,
                effect_info.bounce,
                (x_collision, z_collision),
                movement,
            );
        }

        let below = self.get_on_pos(level, 0.500_001);
        let speed_factor = f64::from(level.block_speed_factor(self.block_position(), below));
        self.delta = self
            .delta
            .multiply_components(speed_factor, 1.0, speed_factor);
        Ok(())
    }

    /// `Entity.collide`: the blocks stop the move, and a mount on the ground steps up what its
    /// step height lets it.
    fn collide(&self, level: &Level<'_>, movement: Vec3) -> Result<Vec3, Unsupported> {
        let level = &self.seen_by(level);
        let aabb = self.bounding_box();
        let movement_step = if movement.length_squared() == 0.0 {
            movement
        } else {
            let colliders =
                level.colliders(aabb.expand_towards_vec3(movement), self.state.fall_distance)?;
            collide_entity_movement(movement, aabb, &colliders)
        };
        let x_collision = movement.x != movement_step.x;
        let y_collision = movement.y != movement_step.y;
        let z_collision = movement.z != movement_step.z;
        let on_ground_after_collision = y_collision && movement.y < 0.0;
        let max_up_step = self.attributes.step_height;
        if max_up_step > 0.0
            && (on_ground_after_collision || self.on_ground)
            && (x_collision || z_collision)
        {
            let grounded = if on_ground_after_collision {
                aabb.move_by(0.0, movement_step.y, 0.0)
            } else {
                aabb
            };
            let mut step_up_area =
                grounded.expand_towards(movement.x, f64::from(max_up_step), movement.z);
            if !on_ground_after_collision {
                step_up_area = step_up_area.expand_towards(0.0, f64::from(-1.0e-5_f32), 0.0);
            }
            let colliders = level.colliders(step_up_area, self.state.fall_distance)?;
            return Ok(select_step_up_movement(
                movement,
                movement_step,
                aabb,
                grounded,
                &colliders,
                max_up_step,
                movement_step.y as f32,
            ));
        }
        Ok(movement_step)
    }

    /// `LivingEntity.checkFallDamage` on a client: the fluids are looked at again, and a fall adds
    /// up until the ground ends it.
    fn check_fall_damage(&mut self, level: &Level<'_>, ya: f64) -> Result<(), Unsupported> {
        // `Strider.checkFallDamage`: in lava a fall is forgiven.
        if self.kind == MountKind::Strider && self.is_in_lava() {
            self.state.fall_distance = 0.0;
            return Ok(());
        }
        if !self.is_in_water() {
            self.update_fluid_interaction(level)?;
        }
        if !self.is_in_water() && ya < 0.0 {
            self.state.fall_distance -= f64::from(ya as f32);
        }
        if self.on_ground {
            self.state.fall_distance = 0.0;
        }
        Ok(())
    }

    /// `Entity.restituteMovementAfterCollisions` for a mob: a bump keeps none of its speed, a
    /// springy block throws it back at four fifths of its spring (a living entity at all of it).
    fn restitute_movement_after_collisions(
        &mut self,
        suppresses_bounce: bool,
        block_bounce: f32,
        (x_collision, z_collision): (bool, bool),
        movement: Vec3,
    ) {
        let mut restitution = 0.0_f64;
        let current = self.delta;
        let mut after = current;
        if x_collision {
            after = Vec3::new(-current.x * restitution, after.y, after.z);
        }
        if z_collision {
            after = Vec3::new(after.x, after.y, -current.z * restitution);
        }
        if self.state.vertical_collision {
            if self.state.vertical_collision_below {
                let falls_slowly = -current.y < self.gravity();
                restitution = if !falls_slowly && !suppresses_bounce {
                    java_math::max(restitution, f64::from(block_bounce))
                } else {
                    0.0
                };
            }
            let (gravity_compensation, effective_drag) = if restitution > 0.0 {
                let portion = movement.y / current.y;
                let air_drag = f64::from(compute_modified_friction(
                    0.98,
                    self.attributes.air_drag_modifier,
                ));
                (portion * self.gravity(), 1.0 + portion * (air_drag - 1.0))
            } else {
                (0.0, 1.0)
            };
            after = Vec3::new(
                after.x,
                (gravity_compensation - current.y) * effective_drag * restitution,
                after.z,
            );
        }
        self.delta = after;
    }

    /// `Entity.applyEffectsFromBlocks`.
    fn apply_effects_from_blocks(&mut self, level: &Level<'_>) {
        let mut movements = std::mem::take(&mut self.movements);
        match movements.last() {
            None => movements.push(Movement {
                from: self.old_position,
                to: self.position,
                original: None,
            }),
            Some(last)
                if last.to.distance_to_sqr(self.position) > f64::from(9.999_999_4e-11_f32) =>
            {
                let from = last.to;
                movements.push(Movement {
                    from,
                    to: self.position,
                    original: None,
                });
            }
            Some(_) => {}
        }
        if self.on_ground {
            let effect_pos = self.get_on_pos(level, 0.2);
            if level.info(effect_pos).name == "minecraft:slime_block" {
                let abs_delta_y = self.delta.y.abs();
                if abs_delta_y < 0.1 {
                    let scale = 0.4 + abs_delta_y * 0.2;
                    self.delta = self.delta.multiply_components(scale, 1.0, scale);
                }
            }
        }
        let _ = walk_inside_blocks(self, level, &movements);
    }
}

impl InsideBody for MountBody {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> voxel_core::shape::VoxelShape {
        crate::local_player::powder_snow_collision_shape(self.state.fall_distance)
    }

    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        let (sx, sy, sz) = self.state.stuck_speed_multiplier;
        let mut stuck = Vec3::new(sx, sy, sz);
        let block_pos = self.block_position();
        living_inside_effect(
            level,
            visit,
            LivingEffects {
                delta: &mut self.delta,
                fall_distance: &mut self.state.fall_distance,
                stuck: &mut stuck,
                position: self.position,
                on_ground: self.on_ground,
                width: self.dimensions.width(),
                block_pos,
            },
        );
        self.state.stuck_speed_multiplier = (stuck.x, stuck.y, stuck.z);
    }
}

/// `LivingEntity.getFluidFallingAdjustedMovement`: gravity's share in the fluid.
fn fluid_falling_adjusted_movement(base_gravity: f64, is_falling: bool, movement: Vec3) -> Vec3 {
    if base_gravity != 0.0 {
        let y = if is_falling
            && (movement.y - 0.005).abs() >= 0.003
            && (movement.y - base_gravity / 16.0).abs() < 0.003
        {
            -0.003
        } else {
            movement.y - base_gravity / 16.0
        };
        Vec3::new(movement.x, y, movement.z)
    } else {
        movement
    }
}

/// `Level.containsAnyLiquid`.
fn contains_any_liquid(level: &Level<'_>, area: Aabb) -> bool {
    for x in floor_f64(area.min_x)..ceil_f64(area.max_x) {
        for y in floor_f64(area.min_y)..ceil_f64(area.max_y) {
            for z in floor_f64(area.min_z)..ceil_f64(area.max_z) {
                if level.info(BlockPos::new(x, y, z)).fluid != Fluid::Empty {
                    return true;
                }
            }
        }
    }
    false
}

/// Whether an entity type is one of the horses a saddle makes rideable (`AbstractHorse` that is
/// not a llama).
pub(crate) fn is_horse_type(type_id: i32) -> bool {
    [
        "minecraft:horse",
        "minecraft:donkey",
        "minecraft:mule",
        "minecraft:skeleton_horse",
        "minecraft:zombie_horse",
    ]
    .iter()
    .any(|name| voxel_data::entity_type::id(name) == Some(type_id))
}

impl ClientWorld {
    /// Which mount the local player drives this entity as, if it does: a saddled horse it sits
    /// in the first seat of, or a saddled pig it steers with a carrot on a stick.
    pub(crate) fn mount_kind_of(&self, entity: &crate::entity::ClientEntity) -> Option<MountKind> {
        let player = self.local_player.as_ref()?;
        if entity.passenger_ids.first() != Some(&player.id) {
            return None;
        }
        if entity.is_controllable_horse() {
            Some(MountKind::Horse)
        } else if entity.is_saddled_pig() && self.rider_controls.holds_carrot_on_a_stick {
            Some(MountKind::Pig)
        } else if entity.is_saddled_strider() && self.rider_controls.holds_warped_fungus_on_a_stick
        {
            Some(MountKind::Strider)
        } else {
            None
        }
    }

    /// What the rider's keys make of the mount's input, as the next tick of the mount reads them.
    pub fn set_rider_controls(&mut self, controls: Controller) {
        self.rider_controls = controls;
    }

    /// `AbstractHorse.onPlayerJump`: the rider let go of the jump key after charging a jump.
    pub fn on_player_jump(&mut self, vehicle_id: i32, jump_amount: i32) {
        let Some(horse) = self
            .entities
            .get_mut(&vehicle_id)
            .filter(|entity| entity.is_controllable_horse())
        else {
            return;
        };
        let amount = if jump_amount < 0 {
            0
        } else {
            let state = horse
                .horse
                .get_or_insert_with(|| Box::new(crate::horse::HorseState::new()));
            state.set_allow_stand_sliding(true);
            state.stand_if_possible();
            jump_amount
        };
        // `PlayerRideableJumping.getPlayerJumpPendingScale`.
        horse.mount_ride.jump_pending_scale = if amount >= 90 {
            1.0
        } else {
            0.4 + 0.4 * amount as f32 / 90.0
        };
    }

    /// `LivingEntity.tick` for the saddled horse the local player rides.
    pub(crate) fn tick_ridden_mount(&mut self, entity_id: i32) {
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let Some(pick) = voxel_data::entity_type::pick(entity.type_id) else {
            return;
        };
        let (width, height, eye_height) = pick.size;
        let dimensions = ClientEntityDimensions::scalable(width, height)
            .with_eye_height(eye_height)
            .scale(entity.scale(), entity.scale());
        let Some(kind) = self.mount_kind_of(entity) else {
            return;
        };
        if kind == MountKind::Strider {
            // `Strider.tick`: it takes the cold or the warmth before it moves.
            let suffocating = strider_is_suffocating(self, entity);
            if let Some(strider) = self.entities.get_mut(&entity_id) {
                strider.set_strider_suffocating(suffocating);
            }
        }
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let attribute = |name: &str, default: f64| entity.attribute_value(name).unwrap_or(default);
        let attributes = Attributes {
            // `getRiddenSpeed`: a pig's is its speed times 0.225 (and its boost, which is one here).
            speed: match kind {
                MountKind::Horse => attribute("minecraft:movement_speed", 0.7) as f32,
                MountKind::Pig => {
                    (attribute("minecraft:movement_speed", 0.25) * 0.225 * 1.0) as f32
                }
                // `Strider.getRiddenSpeed`: slower while it is cold (suffocating).
                MountKind::Strider => {
                    let suffocating = entity.is_suffocating();
                    (attribute("minecraft:movement_speed", 0.175)
                        * f64::from(if suffocating { 0.35_f32 } else { 0.55_f32 })
                        * 1.0) as f32
                }
            },
            gravity: attribute("minecraft:gravity", 0.08),
            step_height: attribute("minecraft:step_height", 0.6) as f32,
            jump_strength: attribute("minecraft:jump_strength", 0.42) as f32,
            friction_modifier: attribute("minecraft:friction_modifier", 1.0) as f32,
            air_drag_modifier: attribute("minecraft:air_drag_modifier", 1.0) as f32,
            water_movement_efficiency: attribute("minecraft:water_movement_efficiency", 0.0) as f32,
        };
        let (x, y, z) = entity.position();
        let (ox, oy, oz) = entity.previous_position();
        let mut body = MountBody::new(
            kind,
            Vec3::new(x, y, z),
            Vec3::new(ox, oy, oz),
            dimensions,
            attributes,
            entity.mount_ride,
        );
        let (mx, my, mz) = entity.motion();
        body.delta = Vec3::new(mx, my, mz);
        body.y_rot = f32::from_bits(entity.y_rot_bits);
        body.x_rot = f32::from_bits(entity.x_rot_bits);
        body.on_ground = entity.on_ground;
        body.supporting_block = entity.tracking.support.main_block;
        body.on_ground_no_blocks = entity.tracking.support.no_blocks;
        body.standing = entity.horse().is_some_and(|horse| horse.is_standing());
        body.allow_stand_sliding = entity
            .horse()
            .is_some_and(|horse| horse.allow_stand_sliding());
        body.floats_when_ridden = kind == MountKind::Horse;
        body.controller = self.rider_controls;
        if body.tick(&Level::new(self)).is_err() {
            return;
        }
        let entity = self
            .entities
            .get_mut(&entity_id)
            .expect("the mount was read above");
        entity.set_position((body.position.x, body.position.y, body.position.z));
        entity.motion_x_bits = body.delta.x.to_bits();
        entity.motion_y_bits = body.delta.y.to_bits();
        entity.motion_z_bits = body.delta.z.to_bits();
        // `tickRidden`: it faces where the rider does, body and head too.
        entity.old_y_rot_bits = body.y_rot.to_bits();
        entity.y_rot_bits = body.y_rot.to_bits();
        entity.x_rot_bits = body.x_rot.to_bits();
        entity.y_body_rot_bits = body.y_rot.to_bits();
        entity.y_head_rot_bits = body.y_rot.to_bits();
        entity.on_ground = body.on_ground;
        entity.tracking.support = Support {
            main_block: body.supporting_block,
            no_blocks: body.on_ground_no_blocks,
        };
        entity.mount_ride = body.state;
    }
}

/// `Strider.tick`'s setting of the suffocating flag: cold unless it is in or on a warm block or in
/// lava (the depth it last looked at; a strider that rides a warm one is not a case here).
fn strider_is_suffocating(world: &ClientWorld, strider: &crate::entity::ClientEntity) -> bool {
    let level = Level::new(world);
    let (x, y, z) = strider.position();
    let inside = BlockPos::new(floor_f64(x), floor_f64(y), floor_f64(z));
    let on = level.on_pos(strider.tracking.support.main_block, Vec3::new(x, y, z), 0.2);
    let warm = |pos: BlockPos| {
        let state = crate::block::state_or_air(level.state_id(pos).map_or(0, i64::from));
        world
            .block_state_registry
            .tags()
            .contains(usize::from(state.block), "minecraft:strider_warm_blocks")
    };
    !(warm(inside) || warm(on) || strider.mount_ride.lava_height > 0.0)
}

#[cfg(test)]
mod tests {

    use crate::{ClientWorld, DimensionBounds, EntityInput};
    use voxel_core::binary::Uuid;

    #[test]
    fn a_horse_has_the_attributes_its_riding_reads() {
        let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
        let id = voxel_data::entity_type::id("minecraft:horse").expect("horse");
        assert!(world.apply_entity(EntityInput {
            entity_id: 5,
            uuid: Uuid::new(0, 5),
            type_id: id,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let horse = world.entities().find(|entity| entity.id() == 5).unwrap();
        // The values a saddled horse's riding reads, as the type starts with them.
        for (name, value) in [
            ("minecraft:gravity", 0.08),
            ("minecraft:step_height", 1.0),
            ("minecraft:jump_strength", 0.7),
            ("minecraft:friction_modifier", 1.0),
            ("minecraft:air_drag_modifier", 1.0),
            ("minecraft:water_movement_efficiency", 0.0),
        ] {
            assert_eq!(horse.attribute_value(name), Some(value), "{name}");
        }
        assert_eq!(
            horse
                .attribute_value("minecraft:movement_speed")
                .map(|speed| speed as f32),
            Some(0.225)
        );
    }
}
