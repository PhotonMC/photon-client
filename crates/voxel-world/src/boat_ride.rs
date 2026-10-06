//! A boat the local player rides: `AbstractBoat.tick` on a client that is its authority. The
//! rider's input steers it (`controlBoat`), the water, the land or the air it is in decides how it
//! floats and slows (`getStatus`, `floatBoat`), and `Entity.move` carries it through the blocks.
//! What the boat does to other entities it meets (pushing them) is not carried.

use voxel_core::aabb::Aabb;
use voxel_core::binary::BlockPos;
use voxel_core::math::{Vec3, ceil_f64, floor_f64, mth_cos_f64, mth_sin_f64};
use voxel_core::shape::{BooleanOp, Shapes, VoxelShape};
use voxel_data::block::definition::Fluid;

use crate::ClientWorld;
use crate::collision::collide_entity_movement;
use crate::entity::{BoatInput, ClientEntityDimensions, Support};
use crate::fluid::{self, LAVA_FLOW_SCALE, Pushed, WATER_FLOW_SCALE};
use crate::java_math;
use crate::local_player::{
    InsideBody, InsideVisit, Level, Movement, Unsupported, flag, walk_inside_blocks,
};

/// `AbstractBoat.getDefaultGravity`.
const GRAVITY: f64 = 0.04;
/// `AbstractBoat.getAirDrag`.
const AIR_DRAG: f64 = 1.0;
/// `(float) (Math.PI / 180.0)`.
const DEG_TO_RAD: f32 = (std::f64::consts::PI / 180.0) as f32;

/// `AbstractBoat.Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    InWater,
    UnderWater,
    UnderFlowingWater,
    OnLand,
    InAir,
}

impl Status {
    /// `AbstractBoat.isUnderWater`.
    pub fn is_under_water(self) -> bool {
        matches!(self, Self::UnderWater | Self::UnderFlowingWater)
    }
}

/// What a boat keeps from one tick to the next beyond its place and motion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RideState {
    /// The status of the last tick and the one before (none before the first tick).
    pub status: Option<Status>,
    pub old_status: Option<Status>,
    pub water_level: f64,
    pub land_friction: f32,
    pub last_yd: f64,
    pub fall_distance: f64,
    pub horizontal_collision: bool,
    pub vertical_collision: bool,
    pub vertical_collision_below: bool,
    /// `Entity.stuckSpeedMultiplier`: set by a cobweb or powder snow, spent by the next move.
    pub stuck_speed_multiplier: (f64, f64, f64),
}

/// The values are finite, so the comparison is a total one.
impl Eq for RideState {}

impl Default for RideState {
    fn default() -> Self {
        Self {
            status: None,
            old_status: None,
            water_level: 0.0,
            land_friction: 0.0,
            last_yd: 0.0,
            fall_distance: 0.0,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            stuck_speed_multiplier: (0.0, 0.0, 0.0),
        }
    }
}

/// The boat as one tick reads and changes it.
#[derive(Debug, Clone)]
pub(crate) struct BoatBody {
    pub(crate) position: Vec3,
    /// Where the boat was at the start of the tick (`Entity.oldPosition`).
    pub(crate) old_position: Vec3,
    pub(crate) delta: Vec3,
    pub(crate) y_rot: f32,
    pub(crate) delta_rotation: f32,
    pub(crate) on_ground: bool,
    pub(crate) supporting_block: Option<BlockPos>,
    pub(crate) on_ground_no_blocks: bool,
    pub(crate) dimensions: ClientEntityDimensions,
    pub(crate) no_gravity: bool,
    pub(crate) input: BoatInput,
    /// Whether the first passenger is a player, and whether there is a passenger at all.
    pub(crate) player_aboard: bool,
    pub(crate) has_passengers: bool,
    pub(crate) paddles: (bool, bool),
    pub(crate) ride: RideState,
    movements: Vec<Movement>,
}

impl BoatBody {
    pub(crate) fn new(
        position: Vec3,
        old_position: Vec3,
        dimensions: ClientEntityDimensions,
        ride: RideState,
    ) -> Self {
        Self {
            position,
            old_position,
            delta: Vec3::ZERO,
            y_rot: 0.0,
            delta_rotation: 0.0,
            on_ground: false,
            supporting_block: None,
            on_ground_no_blocks: false,
            dimensions,
            no_gravity: false,
            input: BoatInput::EMPTY,
            player_aboard: false,
            has_passengers: false,
            paddles: (false, false),
            ride,
            movements: Vec::new(),
        }
    }

    pub(crate) fn bounding_box(&self) -> Aabb {
        self.dimensions.make_bounding_box(self.position)
    }

    fn height(&self) -> f32 {
        self.dimensions.height()
    }

    /// `Entity.setPos`.
    fn set_pos(&mut self, position: Vec3) {
        self.position = position;
    }

    /// `Entity.getGravity`.
    fn gravity(&self) -> f64 {
        if self.no_gravity { 0.0 } else { GRAVITY }
    }

    /// `AbstractBoat.tick` for a boat the local client is the authority of. Returns the paddles
    /// the client tells the server about (`ServerboundPaddleBoatPacket`).
    pub(crate) fn tick(&mut self, level: &Level<'_>) -> Result<(bool, bool), Unsupported> {
        self.ride.old_status = self.ride.status;
        let status = self.status(level);
        self.ride.status = Some(status);
        self.base_tick(level)?;
        if !self.player_aboard {
            self.paddles = (false, false);
        }
        self.float_boat(level)?;
        self.control_boat();
        let paddles = self.paddles;
        self.move_self(level)?;
        // `applyEffectsFromBlocks`, which the boat asks for twice: the second finds no movement
        // left to replay and takes the tick's whole way as one.
        self.apply_effects_from_blocks(level);
        self.apply_effects_from_blocks(level);
        Ok(paddles)
    }

    /// `Entity.baseTick`'s part a boat has: the fluids around it push it and wash off a fall.
    fn base_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let bounding_box = self.bounding_box();
        let found = fluid::scan(
            level,
            (bounding_box, self.position, self.dimensions.eye_height()),
            false,
        )?;
        if found.water.height > 0.0 {
            self.ride.fall_distance = 0.0;
            self.delta =
                fluid::apply_current(found.water, self.delta, WATER_FLOW_SCALE, Pushed::Other);
        }
        if found.lava.height > 0.0 {
            self.delta =
                fluid::apply_current(found.lava, self.delta, LAVA_FLOW_SCALE, Pushed::Other);
            self.ride.fall_distance *= 0.5;
        }
        Ok(())
    }

    /// `AbstractBoat.getStatus`.
    fn status(&mut self, level: &Level<'_>) -> Status {
        if let Some(under) = self.is_underwater(level) {
            self.ride.water_level = self.bounding_box().max_y;
            under
        } else if self.check_in_water(level) {
            Status::InWater
        } else {
            let friction = self.ground_friction(level);
            if friction > 0.0 {
                self.ride.land_friction = friction;
                Status::OnLand
            } else {
                Status::InAir
            }
        }
    }

    /// `AbstractBoat.isUnderwater` (the private status check): whether the water reaches a
    /// thousandth above the box, and whether that water flows.
    fn is_underwater(&self, level: &Level<'_>) -> Option<Status> {
        let bb = self.bounding_box();
        let max_y = bb.max_y + 0.001;
        let mut under_water = false;
        for x in floor_f64(bb.min_x)..ceil_f64(bb.max_x) {
            for y in floor_f64(bb.max_y)..ceil_f64(max_y) {
                for z in floor_f64(bb.min_z)..ceil_f64(bb.max_z) {
                    let pos = BlockPos::new(x, y, z);
                    let info = level.info(pos);
                    if info.is_water()
                        && max_y < f64::from(y as f32 + level.fluid_height(pos, &info))
                    {
                        if info.fluid != Fluid::Water {
                            return Some(Status::UnderFlowingWater);
                        }
                        under_water = true;
                    }
                }
            }
        }
        under_water.then_some(Status::UnderWater)
    }

    /// `AbstractBoat.checkInWater`: whether the water reaches above the bottom of the box, and
    /// how high it stands.
    fn check_in_water(&mut self, level: &Level<'_>) -> bool {
        let bb = self.bounding_box();
        let mut in_water = false;
        self.ride.water_level = -f64::MAX;
        for x in floor_f64(bb.min_x)..ceil_f64(bb.max_x) {
            for y in floor_f64(bb.min_y)..ceil_f64(bb.min_y + 0.001) {
                for z in floor_f64(bb.min_z)..ceil_f64(bb.max_z) {
                    let pos = BlockPos::new(x, y, z);
                    let info = level.info(pos);
                    if info.is_water() {
                        let height = y as f32 + level.fluid_height(pos, &info);
                        self.ride.water_level =
                            java_math::max(f64::from(height), self.ride.water_level);
                        in_water |= bb.min_y < f64::from(height);
                    }
                }
            }
        }
        in_water
    }

    /// `AbstractBoat.getGroundFriction`: the mean friction of the blocks under the boat (not
    /// lily pads), and not a number when it rests on none.
    fn ground_friction(&self, level: &Level<'_>) -> f32 {
        let bb = self.bounding_box();
        let probe = Aabb::new(
            bb.min_x,
            bb.min_y - 0.001,
            bb.min_z,
            bb.max_x,
            bb.min_y,
            bb.max_z,
        );
        let (x0, x1) = (floor_f64(probe.min_x) - 1, ceil_f64(probe.max_x) + 1);
        let (y0, y1) = (floor_f64(probe.min_y) - 1, ceil_f64(probe.max_y) + 1);
        let (z0, z1) = (floor_f64(probe.min_z) - 1, ceil_f64(probe.max_z) + 1);
        let boat_shape = Shapes::create_from_aabb(probe).unwrap_or_else(|_| Shapes::empty());
        let mut friction = 0.0_f32;
        let mut count = 0_i32;
        for x in x0..x1 {
            for z in z0..z1 {
                let edges = i32::from(x == x0 || x == x1 - 1) + i32::from(z == z0 || z == z1 - 1);
                if edges == 2 {
                    continue;
                }
                for y in y0..y1 {
                    if edges > 0 && y != y0 && y != y1 - 1 {
                        continue;
                    }
                    let pos = BlockPos::new(x, y, z);
                    let info = level.info(pos);
                    if info.name != "minecraft:lily_pad"
                        && Shapes::join_is_not_empty(
                            &level.collision_shape(pos).move_by_block_pos(pos),
                            &boat_shape,
                            BooleanOp::And,
                        )
                    {
                        friction += info.friction;
                        count += 1;
                    }
                }
            }
        }
        friction / count as f32
    }

    /// `AbstractBoat.getWaterLevelAbove`: the height of the water's surface above the boat.
    fn water_level_above(&self, level: &Level<'_>) -> f32 {
        let bb = self.bounding_box();
        let (min_x, max_x) = (floor_f64(bb.min_x), ceil_f64(bb.max_x));
        let (min_y, max_y) = (floor_f64(bb.max_y), ceil_f64(bb.max_y - self.ride.last_yd));
        let (min_z, max_z) = (floor_f64(bb.min_z), ceil_f64(bb.max_z));
        for y in min_y..max_y {
            let mut block_height = 0.0_f32;
            let mut full = false;
            'columns: for x in min_x..max_x {
                for z in min_z..max_z {
                    let pos = BlockPos::new(x, y, z);
                    let info = level.info(pos);
                    if info.is_water() {
                        block_height = block_height.max(level.fluid_height(pos, &info));
                    }
                    if block_height >= 1.0 {
                        full = true;
                        break 'columns;
                    }
                }
            }
            if !full {
                return y as f32 + block_height;
            }
        }
        (max_y + 1) as f32
    }

    /// `AbstractBoat.floatBoat`: the pull of gravity and of the water, and how much the medium
    /// slows the boat, by its status.
    fn float_boat(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let status = self.ride.status.unwrap_or(Status::InAir);
        if self.ride.old_status == Some(Status::InAir)
            && status != Status::InAir
            && status != Status::OnLand
        {
            // Coming down onto the water: set on its surface, if there is room.
            self.ride.water_level = self.position.y + f64::from(self.height());
            let target_y = f64::from(self.water_level_above(level) - self.height()) + 0.101;
            let moved = self
                .bounding_box()
                .move_by(0.0, target_y - self.position.y, 0.0);
            if level.no_collision(moved, self.ride.fall_distance)? {
                self.set_pos(Vec3::new(self.position.x, target_y, self.position.z));
                self.delta = self.delta.multiply_components(1.0, 0.0, 1.0);
                self.ride.last_yd = 0.0;
            }
            self.ride.status = Some(Status::InWater);
            return Ok(());
        }
        let gravity = self.gravity();
        let (vspeed, buoyancy, inv_friction) = match status {
            Status::InWater => (
                -gravity,
                (self.ride.water_level - self.position.y) / f64::from(self.height()),
                0.9_f32,
            ),
            Status::UnderFlowingWater => (-7.0e-4, 0.0, 0.9),
            Status::UnderWater => (-gravity, f64::from(0.01_f32), 0.45),
            Status::InAir => (-gravity, 0.0, 0.9),
            Status::OnLand => {
                let friction = self.ride.land_friction;
                if self.player_aboard {
                    self.ride.land_friction /= 2.0;
                }
                (-gravity, 0.0, friction)
            }
        };
        let friction = f64::from(inv_friction);
        self.delta = Vec3::new(
            self.delta.x * friction,
            self.delta.y + vspeed,
            self.delta.z * friction,
        );
        self.delta_rotation *= inv_friction;
        if buoyancy > 0.0 {
            self.delta = Vec3::new(
                self.delta.x,
                (self.delta.y + buoyancy * (GRAVITY / 0.65)) * 0.75,
                self.delta.z,
            );
        }
        Ok(())
    }

    /// `AbstractBoat.controlBoat`: the rider's keys turn the boat and push it along.
    fn control_boat(&mut self) {
        if !self.has_passengers {
            return;
        }
        let input = self.input;
        let mut acceleration = 0.0_f32;
        if input.left {
            self.delta_rotation -= 1.0;
        }
        if input.right {
            self.delta_rotation += 1.0;
        }
        if input.right != input.left && !input.forward && !input.backward {
            acceleration += 0.005;
        }
        self.y_rot += self.delta_rotation;
        if input.forward {
            acceleration += 0.04;
        }
        if input.backward {
            acceleration -= 0.005;
        }
        let x = mth_sin_f64(f64::from(-self.y_rot * DEG_TO_RAD)) * acceleration;
        let z = mth_cos_f64(f64::from(self.y_rot * DEG_TO_RAD)) * acceleration;
        self.delta = self.delta.add_components(f64::from(x), 0.0, f64::from(z));
        self.paddles = (
            input.right && !input.left || input.forward,
            input.left && !input.right || input.forward,
        );
    }

    /// `Entity.move(SELF, ...)` for a boat that is its own authority: it collides with the
    /// blocks (it cannot step up), is stopped or slowed by what it meets, and the block it ends
    /// on sets its speed.
    fn move_self(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let mut delta = self.delta;
        let (sx, sy, sz) = self.ride.stuck_speed_multiplier;
        let stuck = Vec3::new(sx, sy, sz);
        if stuck.length_squared() > 1.0e-7 {
            delta = delta.multiply_components(stuck.x, stuck.y, stuck.z);
            self.ride.stuck_speed_multiplier = (0.0, 0.0, 0.0);
            self.delta = Vec3::ZERO;
        }
        let bb = self.bounding_box();
        let movement = if delta.length_squared() == 0.0 {
            delta
        } else {
            let colliders =
                level.colliders(bb.expand_towards_vec3(delta), self.ride.fall_distance)?;
            collide_entity_movement(delta, bb, &colliders)
        };
        let movement_length = movement.length_squared();
        if movement_length > 1.0e-7 || delta.length_squared() - movement_length < 1.0e-7 {
            if self.ride.fall_distance != 0.0 && movement_length >= 1.0 {
                // A long fall that crosses something that forgives it (water, a cobweb).
                let distance = movement.length().min(8.0);
                let to = self
                    .position
                    .add_vec3(movement.normalized().scale(distance));
                if level.clip_hits_fall_reset(self.position, to) {
                    self.ride.fall_distance = 0.0;
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

        let x_collision = !voxel_core::math::equal_f64(delta.x, movement.x);
        let z_collision = !voxel_core::math::equal_f64(delta.z, movement.z);
        self.ride.horizontal_collision = x_collision || z_collision;
        let moved_vertically = delta.y.abs() > 0.0;
        self.ride.vertical_collision = delta.y != movement.y;
        self.ride.vertical_collision_below = self.ride.vertical_collision && delta.y < 0.0;
        self.on_ground = self.ride.vertical_collision_below;
        let moved_box = self.bounding_box();
        level.check_supporting_block(
            (&mut self.supporting_block, &mut self.on_ground_no_blocks),
            (moved_box, self.position, self.ride.fall_distance),
            self.on_ground,
            Some(movement),
        )?;

        let effect_pos = level.on_pos(self.supporting_block, self.position, 0.2);
        let effect_info = level.info(effect_pos);
        self.check_fall_damage(level, movement.y);

        if moved_vertically && self.ride.vertical_collision || self.ride.horizontal_collision {
            self.restitute_movement_after_collisions(
                effect_info.flags & flag::SUPPRESSES_BOUNCE != 0,
                effect_info.bounce,
                (x_collision, z_collision),
                movement,
            );
        }

        let here = BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        );
        let below = level.on_pos(self.supporting_block, self.position, 0.500_001);
        let speed_factor = f64::from(level.block_speed_factor(here, below));
        self.delta = self
            .delta
            .multiply_components(speed_factor, 1.0, speed_factor);
        Ok(())
    }

    /// `AbstractBoat.checkFallDamage`: the boat remembers how fast it was falling, and, with no
    /// rider's weight on a fall, counts the distance fallen unless it is over water.
    fn check_fall_damage(&mut self, level: &Level<'_>, ya: f64) {
        self.ride.last_yd = self.delta.y;
        // The boat is not itself a passenger.
        if self.on_ground {
            self.ride.fall_distance = 0.0;
        } else {
            let below = BlockPos::new(
                floor_f64(self.position.x),
                floor_f64(self.position.y) - 1,
                floor_f64(self.position.z),
            );
            if !level.info(below).is_water() && ya < 0.0 {
                self.ride.fall_distance -= ya;
            }
        }
    }

    /// `Entity.restituteMovementAfterCollisions` for a boat: it keeps none of a sideways bump and
    /// bounces only off a springy block, at four fifths of its spring.
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
        if self.ride.vertical_collision {
            if self.ride.vertical_collision_below {
                let falls_slowly = -current.y < self.gravity();
                restitution = if !falls_slowly && !suppresses_bounce {
                    java_math::max(restitution, f64::from(block_bounce * 0.8_f32))
                } else {
                    0.0
                };
            }
            let (gravity_compensation, effective_drag) = if restitution > 0.0 {
                let portion = movement.y / current.y;
                (portion * self.gravity(), 1.0 + portion * (AIR_DRAG - 1.0))
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

    /// `Entity.applyEffectsFromBlocks`: a slime block slows what rides over it, then the blocks
    /// the boat moved through act on it.
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
            let effect_pos = level.on_pos(self.supporting_block, self.position, 0.2);
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

    fn make_stuck_in_block(&mut self, speed_multiplier: Vec3) {
        self.ride.fall_distance = 0.0;
        self.ride.stuck_speed_multiplier =
            (speed_multiplier.x, speed_multiplier.y, speed_multiplier.z);
    }
}

impl InsideBody for BoatBody {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> VoxelShape {
        crate::local_player::powder_snow_collision_shape(self.ride.fall_distance)
    }

    /// What a block the boat is inside does to it: honey slides it, a cobweb or powder snow
    /// holds it (a boat is not a living entity), and a bubble column carries it when it is not
    /// at the column's top.
    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        let pos = visit.pos;
        match visit.info.name {
            "minecraft:honey_block" if self.is_sliding_down_honey(pos) => {
                let old_delta_y = |delta_y: f64| delta_y / f64::from(0.98_f32) + 0.08;
                let new_delta_y = |delta_y: f64| (delta_y - 0.08) * f64::from(0.98_f32);
                let delta = self.delta;
                self.delta = if old_delta_y(delta.y) < -0.13 {
                    let factor = -0.05 / old_delta_y(delta.y);
                    Vec3::new(delta.x * factor, new_delta_y(-0.05), delta.z * factor)
                } else {
                    Vec3::new(delta.x, new_delta_y(-0.05), delta.z)
                };
                self.ride.fall_distance = 0.0;
            }
            "minecraft:cobweb" => {
                self.make_stuck_in_block(Vec3::new(0.25, f64::from(0.05_f32), 0.25));
            }
            "minecraft:powder_snow" => {
                self.make_stuck_in_block(Vec3::new(f64::from(0.9_f32), 1.5, f64::from(0.9_f32)));
            }
            "minecraft:bubble_column" if visit.precise => {
                let above = level.info(BlockPos::new(pos.x, pos.y + 1, pos.z));
                let nothing_above = visit.state.is_none() || {
                    let above_state = level.world.block_state_at(pos.x, pos.y + 1, pos.z);
                    above_state.is_none_or(|state| {
                        level
                            .world
                            .block_state_registry
                            .collision_shape(state)
                            .is_empty()
                            && above.fluid == Fluid::Empty
                    })
                };
                // `AbstractBoat.onAboveBubbleColumn` only makes sound and spray on a client.
                if !nothing_above {
                    let drag_down = visit.state.is_some_and(|state| {
                        crate::local_player::property(Some(state), "drag") == Some("true")
                    });
                    let y = if drag_down {
                        (self.delta.y - 0.03).max(-0.3)
                    } else {
                        (self.delta.y + 0.06).min(0.7)
                    };
                    self.delta = Vec3::new(self.delta.x, y, self.delta.z);
                    self.ride.fall_distance = 0.0;
                }
            }
            _ => {}
        }
    }
}

impl BoatBody {
    /// `HoneyBlock.isSlidingDown`.
    fn is_sliding_down_honey(&self, pos: BlockPos) -> bool {
        if self.on_ground || self.position.y > f64::from(pos.y) + 0.9375 - 1.0e-7 {
            return false;
        }
        if self.delta.y / f64::from(0.98_f32) + 0.08 >= -0.08 {
            return false;
        }
        let dx = (f64::from(pos.x) + 0.5 - self.position.x).abs();
        let dz = (f64::from(pos.z) + 0.5 - self.position.z).abs();
        let overlap = 0.4375 + f64::from(self.dimensions.width() / 2.0);
        dx + 1.0e-7 > overlap || dz + 1.0e-7 > overlap
    }
}

/// What the local player's own tick needs to know of the boat it rides, as the boat's tick left it.
#[derive(Debug, Clone, Copy)]
pub struct BoatView {
    pub vehicle_id: i32,
    /// The boat's registry type, and where it and its box's top are.
    pub type_id: i32,
    pub position: Vec3,
    pub y_rot: f32,
    pub delta_rotation: f32,
    pub max_y: f64,
    /// `AbstractBoat.isUnderWater`.
    pub under_water: bool,
    /// Where the rider is carried to (`getPassengerRidingPosition`), and how the boat turns it.
    pub rider_position: Vec3,
    pub on_ground: bool,
    pub horizontal_collision: bool,
}

/// Whether a paddle at this turn makes its stroke sound this tick (`AbstractBoat.tick`'s test of
/// the paddle's position against the start of its sweep).
fn paddle_makes_sound(position: f32) -> bool {
    let turn = (std::f64::consts::PI * 2.0) as f32;
    let quarter = (std::f64::consts::PI / 4.0) as f32;
    let step = (std::f64::consts::PI / 8.0) as f32;
    position % turn <= quarter && (position + step) % turn >= quarter
}

impl ClientWorld {
    /// The boat the local player is the first passenger of, if it rides one.
    pub fn ridden_boat_of_local_player(&self) -> Option<i32> {
        let player_id = self.local_player.as_ref()?.id;
        let vehicle_id = self.entity_vehicle_id(player_id)?;
        let boat = self.entities.get(&vehicle_id)?;
        (boat.is_boat() && boat.passenger_ids.first() == Some(&player_id)).then_some(vehicle_id)
    }

    /// `AbstractBoat.tick` for the boat the local player rides, between this tick's start and its
    /// movement. Its paddles are queued to be told to the server.
    pub(crate) fn tick_ridden_boat(&mut self, entity_id: i32) {
        let Some(entity) = self.entities.get(&entity_id) else {
            return;
        };
        let Some(pick) = voxel_data::entity_type::pick(entity.type_id) else {
            return;
        };
        let (width, height, _) = pick.size;
        let dimensions =
            ClientEntityDimensions::scalable(width, height).scale(entity.scale(), entity.scale());
        let (x, y, z) = entity.position();
        let (ox, oy, oz) = entity.previous_position();
        let mut body = BoatBody::new(
            Vec3::new(x, y, z),
            Vec3::new(ox, oy, oz),
            dimensions,
            entity.boat_ride,
        );
        let (mx, my, mz) = entity.motion();
        body.delta = Vec3::new(mx, my, mz);
        body.y_rot = f32::from_bits(entity.y_rot_bits);
        body.delta_rotation = f32::from_bits(entity.delta_rotation_bits);
        body.on_ground = entity.on_ground;
        body.supporting_block = entity.tracking.support.main_block;
        body.on_ground_no_blocks = entity.tracking.support.no_blocks;
        body.no_gravity = entity.no_gravity;
        body.input = entity.boat_input;
        body.has_passengers = !entity.passenger_ids.is_empty();
        body.player_aboard = entity.passenger_ids.first().is_some_and(|&id| {
            self.local_player
                .as_ref()
                .is_some_and(|player| player.id == id)
                || self
                    .entities
                    .get(&id)
                    .is_some_and(|passenger| passenger.is_player())
        });
        body.paddles = (entity.paddle_left, entity.paddle_right);
        let level = Level { world: self };
        let Ok(paddles) = body.tick(&level) else {
            return;
        };
        // The paddles row, and a stroke at the start of its sweep asks which sound the water or
        // the land makes, which finds the status again (and what finding it leaves behind).
        let silent = entity.silent;
        let paddling = [paddles.0, paddles.1].map(|rows| rows && body.has_passengers);
        let positions = [0, 1].map(|side| {
            entity
                .boat
                .as_deref()
                .map_or(0.0, |boat| boat.paddle_position(side))
        });
        for (rows, position) in paddling.iter().zip(positions) {
            if *rows && !silent && paddle_makes_sound(position) {
                let _ = body.status(&level);
            }
        }
        let entity = self
            .entities
            .get_mut(&entity_id)
            .expect("the boat was read above");
        entity.set_position((body.position.x, body.position.y, body.position.z));
        entity.motion_x_bits = body.delta.x.to_bits();
        entity.motion_y_bits = body.delta.y.to_bits();
        entity.motion_z_bits = body.delta.z.to_bits();
        entity.y_rot_bits = body.y_rot.to_bits();
        entity.delta_rotation_bits = body.delta_rotation.to_bits();
        entity.on_ground = body.on_ground;
        entity.tracking.support = Support {
            main_block: body.supporting_block,
            no_blocks: body.on_ground_no_blocks,
        };
        entity.paddle_left = body.paddles.0;
        entity.paddle_right = body.paddles.1;
        entity.boat_ride = body.ride;
        entity.tick_boat_end(paddling);
        if let Some(state) = entity.boat.as_deref_mut() {
            state.is_under_water = body.ride.status.is_some_and(Status::is_under_water);
        }
        self.paddle_packets.push(paddles);
    }

    /// The paddles the ridden boat's ticks told of since last asked, each to be sent to the server.
    pub fn take_paddle_packets(&mut self) -> Vec<(bool, bool)> {
        std::mem::take(&mut self.paddle_packets)
    }

    /// What the local player's tick reads of the boat it rides.
    pub fn boat_view(&self, vehicle_id: i32) -> Option<BoatView> {
        let player = self.local_player.as_ref()?;
        let boat = self
            .entities
            .get(&vehicle_id)
            .filter(|boat| boat.is_boat())?;
        let pick = voxel_data::entity_type::pick(boat.type_id)?;
        let (width, height, _) = pick.size;
        let (x, y, z) = boat.position();
        let bb = ClientEntityDimensions::scalable(width, height)
            .scale(boat.scale(), boat.scale())
            .make_bounding_box(Vec3::new(x, y, z));
        let rider = self.boat_rider_position(vehicle_id, player.id)?;
        Some(BoatView {
            vehicle_id,
            type_id: boat.type_id,
            position: Vec3::new(x, y, z),
            y_rot: f32::from_bits(boat.y_rot_bits),
            delta_rotation: f32::from_bits(boat.delta_rotation_bits),
            max_y: bb.max_y,
            under_water: boat.boat_ride.status.is_some_and(Status::is_under_water),
            rider_position: rider,
            on_ground: boat.on_ground,
            horizontal_collision: boat.boat_ride.horizontal_collision,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `AbstractBoat.controlBoat` against the numbers the reference gives from the same start
    /// (`docs/client-world-entity-boat-control-conformance.md`): input flags, `deltaRotation`, yaw,
    /// the motion's x and z, and the paddles.
    #[test]
    fn control_applies_java_input_delta_and_paddles() {
        let rows = [
            (
                "left",
                BoatInput::new(true, false, false, false),
                0xbf80_0000_u32,
                0x4110_0000_u32,
                0x3fbf_ccc0_8100_0000_u64,
                0xbfcf_5e2d_1900_0000_u64,
                (false, true),
            ),
            (
                "right",
                BoatInput::new(false, true, false, false),
                0x3f80_0000,
                0x4130_0000,
                0x3fbf_c17d_a440_0000,
                0xbfcf_5f2b_3300_0000,
                (true, false),
            ),
            (
                "forward",
                BoatInput::new(false, false, true, false),
                0x0000_0000,
                0x4120_0000,
                0x3fbe_38e6_d800_0000,
                0xbfca_f52e_e000_0000,
                (true, true),
            ),
            (
                "backward",
                BoatInput::new(false, false, false, true),
                0x0000_0000,
                0x4120_0000,
                0x3fc0_1c71_9280_0000,
                0xbfd0_50ad_1200_0000,
                (false, false),
            ),
            (
                "both_turn",
                BoatInput::new(true, true, false, false),
                0x0000_0000,
                0x4120_0000,
                0x3fc0_0000_0000_0000,
                0xbfd0_0000_0000_0000,
                (false, false),
            ),
            (
                "left_forward",
                BoatInput::new(true, false, true, false),
                0xbf80_0000,
                0x4110_0000,
                0x3fbe_6604_0800_0000,
                0xbfca_f168_c800_0000,
                (true, true),
            ),
        ];
        for (name, input, delta, yaw, motion_x, motion_z, paddles) in rows {
            let mut body = BoatBody::new(
                Vec3::new(2.0, 12.0, 2.0),
                Vec3::new(2.0, 12.0, 2.0),
                ClientEntityDimensions::scalable(1.375, 0.5625),
                RideState::default(),
            );
            body.delta = Vec3::new(0.125, 0.0, -0.25);
            body.y_rot = 10.0;
            body.input = input;
            body.has_passengers = true;
            body.control_boat();
            assert_eq!(body.delta_rotation.to_bits(), delta, "{name}");
            assert_eq!(body.y_rot.to_bits(), yaw, "{name}");
            assert_eq!(body.delta.x.to_bits(), motion_x, "{name}");
            assert_eq!(body.delta.z.to_bits(), motion_z, "{name}");
            assert_eq!(body.paddles, paddles, "{name}");
        }
    }
}
