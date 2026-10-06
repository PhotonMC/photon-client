//! The local player's whole client tick.
//!
//! This is `LocalPlayer.tick` and everything it runs, in the reference's
//! order: the level's old-position step, `Player.tick`, the living base tick
//! with its fluid update, `LocalPlayer.aiStep` down through `travel` and
//! `Entity.move`, the block effects, the pose update, and the packets the
//! tick sends. The leaves it composes are pinned on their own elsewhere;
//! this module pins the composition.
//!
//! A branch the port does not carry yet is reported as [`Unsupported`]
//! rather than approximated, so a scene that reaches one fails loudly.
//!
//! The reference writes `!(a > b)` where a NaN must fall through the other
//! way than `a <= b` would send it, so those negations are kept as written.
#![expect(
    clippy::neg_cmp_op_on_partial_ord,
    reason = "the reference's negated comparisons, which a NaN also passes"
)]

use crate::input::{Input, KeyboardInput, LocalPlayerInputModifiers, modify_local_player_input};
use crate::java_math;
use voxel_core::aabb::Aabb;
use voxel_core::binary::BlockPos;
use voxel_core::direction::{Axis, Direction};
use voxel_core::math::{
    MTH_DEG_TO_RAD, Vec3, equal_f64, floor_f64, mth_cos_f64, mth_inv_sqrt_f32, mth_sin_f64,
};
use voxel_core::shape::{BooleanOp, Shapes, VoxelShape};

use super::ClientWorld;
use super::collision::{
    collide_entity_movement, is_minor_horizontal_collision, select_step_up_movement,
};
use super::entity::{ClientEntityDimensions, ClientEntityPose};
use super::player::{ClientPlayerAbilities, ClientPlayerGameMode};
use voxel_data::block::classes::BlockClass;
use voxel_data::block::definition::Fluid;
use voxel_data::block::registry;

/// A branch of the reference tick this port does not carry yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsupported(pub String);

impl std::fmt::Display for Unsupported {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "unported local player branch: {}", self.0)
    }
}

impl std::error::Error for Unsupported {}

fn unsupported<T>(what: impl Into<String>) -> Result<T, Unsupported> {
    Err(Unsupported(what.into()))
}

/// `ServerboundPlayerCommandPacket.Action`, by ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerCommandAction {
    StartSprinting = 1,
    StopSprinting = 2,
}

/// A packet the tick asks the connection to send.
#[derive(Debug, Clone, Copy)]
pub enum ServerboundMovement {
    PlayerInput(Input),
    PlayerCommand {
        entity_id: i32,
        action: PlayerCommandAction,
    },
    PlayerAbilities {
        flying: bool,
    },
    MovePlayerPos {
        position: Vec3,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerPosRot {
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerRot {
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerStatusOnly {
        on_ground: bool,
        horizontal_collision: bool,
    },
    /// `ServerboundMoveVehiclePacket`: where the vehicle the player is the authority of is.
    MoveVehicle {
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
    },
    /// `ServerboundPaddleBoatPacket`: which paddles of the boat it rides row.
    PaddleBoat {
        left: bool,
        right: bool,
    },
}

impl ServerboundMovement {
    /// The packet type's resource name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::PlayerInput(_) => "minecraft:player_input",
            Self::PlayerCommand { .. } => "minecraft:player_command",
            Self::PlayerAbilities { .. } => "minecraft:player_abilities",
            Self::MovePlayerPos { .. } => "minecraft:move_player_pos",
            Self::MovePlayerPosRot { .. } => "minecraft:move_player_pos_rot",
            Self::MovePlayerRot { .. } => "minecraft:move_player_rot",
            Self::MovePlayerStatusOnly { .. } => "minecraft:move_player_status_only",
            Self::MoveVehicle { .. } => "minecraft:move_vehicle",
            Self::PaddleBoat { .. } => "minecraft:paddle_boat",
        }
    }

    /// The body the packet's codec writes, without the packet id.
    pub fn encode_body(&self) -> Vec<u8> {
        fn move_flags(on_ground: bool, horizontal_collision: bool) -> u8 {
            u8::from(on_ground) | (u8::from(horizontal_collision) << 1)
        }
        fn var_int(output: &mut Vec<u8>, value: i32) {
            let mut value = value as u32;
            loop {
                if value & !0x7f == 0 {
                    output.push(value as u8);
                    return;
                }
                output.push((value as u8 & 0x7f) | 0x80);
                value >>= 7;
            }
        }
        let mut output = Vec::new();
        match *self {
            Self::PlayerInput(input) => output.push(
                u8::from(input.forward)
                    | u8::from(input.backward) << 1
                    | u8::from(input.left) << 2
                    | u8::from(input.right) << 3
                    | u8::from(input.jump) << 4
                    | u8::from(input.shift) << 5
                    | u8::from(input.sprint) << 6,
            ),
            Self::PlayerCommand { entity_id, action } => {
                var_int(&mut output, entity_id);
                var_int(&mut output, action as i32);
                var_int(&mut output, 0);
            }
            Self::PlayerAbilities { flying } => output.push(if flying { 2 } else { 0 }),
            Self::MovePlayerPos {
                position,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerPosRot {
                position,
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerRot {
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerStatusOnly {
                on_ground,
                horizontal_collision,
            } => {
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MoveVehicle {
                position,
                y_rot,
                x_rot,
                on_ground,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(u8::from(on_ground));
            }
            Self::PaddleBoat { left, right } => {
                output.push(u8::from(left));
                output.push(u8::from(right));
            }
        }
        output
    }
}

// Player attribute defaults, as the reference registers them.
const MOVEMENT_SPEED_BASE: f32 = 0.1;
const SPRINTING_MODIFIER: f32 = 0.3;
const JUMP_STRENGTH: f32 = 0.42;
const STEP_HEIGHT: f64 = 0.6;
const GRAVITY: f64 = 0.08;
const SNEAKING_SPEED: f64 = 0.3;
const SPRINT_WINDOW: i32 = 7;
use crate::fluid::{FluidTracker, LAVA_FLOW_SCALE, WATER_FLOW_SCALE};

/// A stretch of movement recorded for the block-effects pass.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Movement {
    pub(crate) from: Vec3,
    pub(crate) to: Vec3,
    /// The movement asked for, when the move was split per axis.
    pub(crate) original: Option<Vec3>,
}

/// What the vehicle the player rides tells its tick: the top of the vehicle's box and whether the
/// vehicle is under water (`modifyPassengerFluidInteractionBox`), and whether it can be sprinted
/// in (`Entity.canSprint`).
#[derive(Debug, Clone, Copy)]
struct Riding {
    vehicle_max_y: f64,
    vehicle_under_water: bool,
    can_sprint: bool,
}

/// What one tick of the player sitting in a boat gives the session: the packets to send, and the
/// keys to hand the boat (`AbstractBoat.setInput`).
#[derive(Debug, Clone)]
pub struct RideTick {
    pub sent: Vec<ServerboundMovement>,
    pub boat_input: crate::entity::BoatInput,
}

/// The tags and classes movement tests of a block.
mod sounds;
pub use sounds::{LocalSound, LocalSoundSource, SoundQueue};

pub(crate) mod flag {
    pub const CLIMBABLE: u16 = 1 << 0;
    pub const FENCE: u16 = 1 << 1;
    pub const WALL: u16 = 1 << 2;
    pub const CAN_GLIDE_THROUGH: u16 = 1 << 3;
    pub const SUPPRESSES_BOUNCE: u16 = 1 << 4;
    pub const FENCE_GATE: u16 = 1 << 5;
    pub const TRAPDOOR: u16 = 1 << 6;
    pub const LADDER: u16 = 1 << 7;
}

/// What a block state holds for movement, looked up once.
#[derive(Clone, Copy)]
pub(crate) struct StateInfo {
    pub(crate) name: &'static str,
    pub(crate) friction: f32,
    pub(crate) speed_factor: f32,
    jump_factor: f32,
    pub(crate) bounce: f32,
    pub(crate) flags: u16,
    pub(crate) fluid: Fluid,
    own_height: f32,
    falling: bool,
}

impl StateInfo {
    /// A state against the server's tags; a number outside the registry
    /// reads as air.
    fn of(state_id: Option<i32>, tags: &crate::block::tags::BlockTags) -> Self {
        let state = crate::block::state_or_air(state_id.map_or(0, i64::from));
        let block = state.block();
        let index = usize::from(state.block);
        let tagged = [
            ("minecraft:climbable", flag::CLIMBABLE),
            ("minecraft:fences", flag::FENCE),
            ("minecraft:walls", flag::WALL),
            ("minecraft:can_glide_through", flag::CAN_GLIDE_THROUGH),
            ("minecraft:suppresses_bounce", flag::SUPPRESSES_BOUNCE),
        ];
        let classes = [
            (BlockClass::FenceGateBlock, flag::FENCE_GATE),
            (BlockClass::TrapDoorBlock, flag::TRAPDOOR),
            (BlockClass::LadderBlock, flag::LADDER),
        ];
        let flags = tagged
            .iter()
            .filter(|(tag, _)| tags.contains(index, tag))
            .map(|(_, bit)| bit)
            .chain(
                classes
                    .iter()
                    .filter(|(class, _)| block.class.is(*class))
                    .map(|(_, bit)| bit),
            )
            .fold(0, |flags, bit| flags | bit);
        Self {
            name: block.name,
            friction: block.friction,
            speed_factor: block.speed_factor,
            jump_factor: block.jump_factor,
            bounce: block.bounce_restitution,
            flags,
            fluid: state.fluid.fluid,
            own_height: state.fluid.own_height(),
            falling: state.fluid.falling,
        }
    }

    fn is_air(&self) -> bool {
        matches!(
            self.name,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        )
    }

    pub(crate) fn is_water(&self) -> bool {
        matches!(self.fluid, Fluid::Water | Fluid::FlowingWater)
    }

    fn is_lava(&self) -> bool {
        matches!(self.fluid, Fluid::Lava | Fluid::FlowingLava)
    }
}

/// The level as the tick reads it.
pub(crate) struct Level<'a> {
    pub(crate) world: &'a ClientWorld,
}

impl Level<'_> {
    fn state_id(&self, pos: BlockPos) -> Option<i32> {
        self.world.block_state_at(pos.x, pos.y, pos.z)
    }

    pub(crate) fn info(&self, pos: BlockPos) -> StateInfo {
        StateInfo::of(self.state_id(pos), self.world.block_state_registry.tags())
    }

    /// `Entity.checkSupportingBlock`: which block an entity on the ground
    /// stands on, from its box, position and fall distance.
    pub(crate) fn check_supporting_block(
        &self,
        (main_supporting_block, on_ground_no_blocks): (&mut Option<BlockPos>, &mut bool),
        (bounding_box, position, fall_distance): (Aabb, Vec3, f64),
        on_ground: bool,
        movement: Option<Vec3>,
    ) -> Result<(), Unsupported> {
        if on_ground {
            let bb = bounding_box;
            let test_area = Aabb::new(
                bb.min_x,
                bb.min_y - 1.0e-6,
                bb.min_z,
                bb.max_x,
                bb.min_y,
                bb.max_z,
            );
            let mut supporting = self.find_supporting_block(test_area, position, fall_distance)?;
            if supporting.is_some() || *on_ground_no_blocks {
                *main_supporting_block = supporting;
            } else if let Some(movement) = movement {
                supporting = self.find_supporting_block(
                    test_area.move_by(-movement.x, 0.0, -movement.z),
                    position,
                    fall_distance,
                )?;
                *main_supporting_block = supporting;
            }
            *on_ground_no_blocks = supporting.is_none();
        } else {
            *on_ground_no_blocks = false;
            *main_supporting_block = None;
        }
        Ok(())
    }

    fn find_supporting_block(
        &self,
        area: Aabb,
        position: Vec3,
        fall_distance: f64,
    ) -> Result<Option<BlockPos>, Unsupported> {
        let mut main: Option<BlockPos> = None;
        let mut main_distance = f64::MAX;
        for (pos, _) in self.block_collisions(area, fall_distance)? {
            let dx = f64::from(pos.x) + 0.5 - position.x;
            let dy = f64::from(pos.y) + 0.5 - position.y;
            let dz = f64::from(pos.z) + 0.5 - position.z;
            let distance = dx * dx + dy * dy + dz * dz;
            let later =
                |current: BlockPos| (current.y, current.z, current.x) < (pos.y, pos.z, pos.x);
            if distance < main_distance || distance == main_distance && main.is_none_or(later) {
                main = Some(pos);
                main_distance = distance;
            }
        }
        Ok(main)
    }

    /// `Entity.getOnPos(offset)`.
    pub(crate) fn on_pos(
        &self,
        main_supporting_block: Option<BlockPos>,
        position: Vec3,
        offset: f32,
    ) -> BlockPos {
        let y = floor_f64(position.y - f64::from(offset));
        if let Some(pos) = main_supporting_block {
            if !(offset > 1.0e-5) {
                return pos;
            }
            let below = self.info(pos);
            let keeps_support = f64::from(offset) <= 0.5 && below.flags & flag::FENCE != 0
                || below.flags & flag::WALL != 0
                || below.flags & flag::FENCE_GATE != 0;
            if keeps_support {
                pos
            } else {
                BlockPos::new(pos.x, y, pos.z)
            }
        } else {
            BlockPos::new(floor_f64(position.x), y, floor_f64(position.z))
        }
    }

    /// `Entity.getBlockSpeedFactor`, for the block the entity is in and the
    /// one that affects its movement below.
    pub(crate) fn block_speed_factor(&self, here: BlockPos, below: BlockPos) -> f32 {
        let here = self.info(here);
        if matches!(here.name, "minecraft:water" | "minecraft:bubble_column")
            || here.speed_factor != 1.0
        {
            here.speed_factor
        } else {
            self.info(below).speed_factor
        }
    }

    /// `ClientLevel.hasChunk`, which `Level.hasChunkAt` asks: the client level answers yes for
    /// every chunk, loaded or not.
    fn has_chunk_at(&self, _pos: BlockPos) -> bool {
        true
    }

    pub(crate) fn collision_shape(&self, pos: BlockPos) -> VoxelShape {
        match self.state_id(pos) {
            Some(state) => self
                .world
                .block_state_registry
                .collision_shape_at(state, [pos.x, pos.y, pos.z]),
            None => Shapes::empty(),
        }
    }

    /// `getBlockCollisions` for the player, who has fallen `fall_distance`:
    /// powder snow holds it only once it falls far enough. Scaffolding is
    /// refused until its shape is ported.
    pub(crate) fn block_collisions(
        &self,
        area: Aabb,
        fall_distance: f64,
    ) -> Result<Vec<(BlockPos, VoxelShape)>, Unsupported> {
        let registry = &self.world.block_state_registry;
        let collisions = self.world.collect_block_collisions_with(area, |state, at| {
            Some(if is_block(state, "minecraft:powder_snow") {
                powder_snow_collision_shape(fall_distance)
            } else {
                registry.collision_shape_at(state, at)
            })
        });
        let mut out = Vec::with_capacity(collisions.len());
        for collision in collisions {
            let info = self.info(collision.position);
            if info.name == "minecraft:scaffolding" {
                return unsupported(format!("entity-dependent collision shape of {}", info.name));
            }
            out.push((collision.position, collision.shape));
        }
        Ok(out)
    }

    pub(crate) fn colliders(
        &self,
        area: Aabb,
        fall_distance: f64,
    ) -> Result<Vec<VoxelShape>, Unsupported> {
        Ok(self
            .block_collisions(area, fall_distance)?
            .into_iter()
            .map(|(_, shape)| shape)
            .collect())
    }

    pub(crate) fn no_collision(&self, area: Aabb, fall_distance: f64) -> Result<bool, Unsupported> {
        Ok(self.block_collisions(area, fall_distance)?.is_empty())
    }

    fn collides_with_suffocating_block(&self, area: Aabb) -> bool {
        let registry = &self.world.block_state_registry;
        !self
            .world
            .collect_block_collisions_with(area, |state, at| {
                crate::block::state_or_air(i64::from(state))
                    .suffocating
                    .then(|| registry.collision_shape_at(state, at))
            })
            .is_empty()
    }

    /// `FluidState.getHeight`: a full block under the same fluid.
    pub(crate) fn fluid_height(&self, pos: BlockPos, info: &StateInfo) -> f32 {
        let above = self.info(BlockPos::new(pos.x, pos.y + 1, pos.z));
        let same_above =
            (info.is_water() && above.is_water()) || (info.is_lava() && above.is_lava());
        if same_above { 1.0 } else { info.own_height }
    }

    /// `FlowingFluid.isSolidFace` for a horizontal direction: not for the
    /// same fluid, never for ice, otherwise whether the block's face on that
    /// side (the side named by the direction itself) is fully sturdy.
    fn is_solid_face(&self, pos: BlockPos, info: &StateInfo, direction: Direction) -> bool {
        let there = self.info(pos);
        let same_fluid =
            (info.is_water() && there.is_water()) || (info.is_lava() && there.is_lava());
        if same_fluid {
            return false;
        }
        let state = crate::block::state_or_air(self.state_id(pos).map_or(0, i64::from));
        !state.block().class.is(BlockClass::IceBlock) && state.is_face_sturdy(direction)
    }

    /// `FlowingFluid.getFlow`.
    pub(crate) fn fluid_flow(&self, pos: BlockPos, info: &StateInfo) -> Result<Vec3, Unsupported> {
        let same = |other: &StateInfo| {
            other.fluid == Fluid::Empty
                || (info.is_water() && other.is_water())
                || (info.is_lava() && other.is_lava())
        };
        let mut flow_x = 0.0_f64;
        let mut flow_z = 0.0_f64;
        // Direction.Plane.HORIZONTAL: north, east, south, west.
        for (step_x, step_z) in [(0, -1), (1, 0), (0, 1), (-1, 0)] {
            let neighbour_pos = BlockPos::new(pos.x + step_x, pos.y, pos.z + step_z);
            let neighbour = self.info(neighbour_pos);
            if !same(&neighbour) {
                continue;
            }
            let mut neighbour_height = neighbour.own_height;
            let mut distance = 0.0_f32;
            if neighbour_height == 0.0 {
                if !self.blocks_motion(neighbour_pos, &neighbour)? {
                    let below_pos =
                        BlockPos::new(neighbour_pos.x, neighbour_pos.y - 1, neighbour_pos.z);
                    let below = self.info(below_pos);
                    if same(&below) {
                        neighbour_height = below.own_height;
                        if neighbour_height > 0.0 {
                            distance = info.own_height - (neighbour_height - 0.888_888_9_f32);
                        }
                    }
                }
            } else if neighbour_height > 0.0 {
                distance = info.own_height - neighbour_height;
            }
            if distance != 0.0 {
                flow_x += f64::from(step_x) * f64::from(distance);
                flow_z += f64::from(step_z) * f64::from(distance);
            }
        }
        let mut flow = Vec3::new(flow_x, 0.0, flow_z);
        // A falling fluid beside a solid face is pulled down hard.
        if info.falling {
            // North, east, south, west.
            for (step_x, step_z, direction) in [
                (0, -1, Direction::North),
                (1, 0, Direction::East),
                (0, 1, Direction::South),
                (-1, 0, Direction::West),
            ] {
                let side = BlockPos::new(pos.x + step_x, pos.y, pos.z + step_z);
                let above = BlockPos::new(side.x, side.y + 1, side.z);
                if self.is_solid_face(side, info, direction)
                    || self.is_solid_face(above, info, direction)
                {
                    let normal = flow.normalized();
                    flow = Vec3::new(normal.x, normal.y - 6.0, normal.z);
                    break;
                }
            }
        }
        Ok(flow.normalized())
    }

    /// Whether `clip` with `ClipContext.Block.FALLDAMAGE_RESETTING` and
    /// `ClipContext.Fluid.WATER` hits anything between the two points.
    pub(crate) fn clip_hits_fall_reset(&self, from: Vec3, to: Vec3) -> bool {
        if from.x == to.x && from.y == to.y && from.z == to.z {
            return false;
        }
        let lerp = |delta: f64, start: f64, end: f64| start + delta * (end - start);
        let to_x = lerp(-1.0e-7, to.x, from.x);
        let to_y = lerp(-1.0e-7, to.y, from.y);
        let to_z = lerp(-1.0e-7, to.z, from.z);
        let from_x = lerp(-1.0e-7, from.x, to.x);
        let from_y = lerp(-1.0e-7, from.y, to.y);
        let from_z = lerp(-1.0e-7, from.z, to.z);
        let mut block_x = floor_f64(from_x);
        let mut block_y = floor_f64(from_y);
        let mut block_z = floor_f64(from_z);
        if self.fall_reset_hit(BlockPos::new(block_x, block_y, block_z), from, to) {
            return true;
        }
        let dx = to_x - from_x;
        let dy = to_y - from_y;
        let dz = to_z - from_z;
        let sign = |value: f64| {
            if value > 0.0 {
                1
            } else if value < 0.0 {
                -1
            } else {
                0
            }
        };
        let frac = |value: f64| value - voxel_core::math::lfloor_f64(value) as f64;
        let (sign_x, sign_y, sign_z) = (sign(dx), sign(dy), sign(dz));
        let t_delta_x = if sign_x == 0 {
            f64::MAX
        } else {
            f64::from(sign_x) / dx
        };
        let t_delta_y = if sign_y == 0 {
            f64::MAX
        } else {
            f64::from(sign_y) / dy
        };
        let t_delta_z = if sign_z == 0 {
            f64::MAX
        } else {
            f64::from(sign_z) / dz
        };
        let mut t_x = t_delta_x
            * if sign_x > 0 {
                1.0 - frac(from_x)
            } else {
                frac(from_x)
            };
        let mut t_y = t_delta_y
            * if sign_y > 0 {
                1.0 - frac(from_y)
            } else {
                frac(from_y)
            };
        let mut t_z = t_delta_z
            * if sign_z > 0 {
                1.0 - frac(from_z)
            } else {
                frac(from_z)
            };
        while t_x <= 1.0 || t_y <= 1.0 || t_z <= 1.0 {
            if t_x < t_y {
                if t_x < t_z {
                    block_x += sign_x;
                    t_x += t_delta_x;
                } else {
                    block_z += sign_z;
                    t_z += t_delta_z;
                }
            } else if t_y < t_z {
                block_y += sign_y;
                t_y += t_delta_y;
            } else {
                block_z += sign_z;
                t_z += t_delta_z;
            }
            if self.fall_reset_hit(BlockPos::new(block_x, block_y, block_z), from, to) {
                return true;
            }
        }
        false
    }

    fn fall_reset_hit(&self, pos: BlockPos, from: Vec3, to: Vec3) -> bool {
        let info = self.info(pos);
        let resets = info.flags & flag::CLIMBABLE != 0
            || matches!(info.name, "minecraft:sweet_berry_bush" | "minecraft:cobweb");
        if resets && Shapes::block().clip(from, to, pos).is_some() {
            return true;
        }
        if info.is_water() {
            let height = f64::from(self.fluid_height(pos, &info));
            let shape = Shapes::box_shape(0.0, 0.0, 0.0, 1.0, height, 1.0)
                .expect("a fluid's shape is a normalized box");
            return shape.clip(from, to, pos).is_some();
        }
        false
    }

    /// `BlockState.blocksMotion`.
    fn blocks_motion(&self, pos: BlockPos, _info: &StateInfo) -> Result<bool, Unsupported> {
        Ok(self
            .state_id(pos)
            .is_some_and(|state| crate::block::state_or_air(i64::from(state)).blocks_motion))
    }
}

/// The client's own player, ticked as the reference ticks it.
#[derive(Debug, Clone)]
pub struct LocalPlayer {
    entity_id: i32,
    position: Vec3,
    old_position: Vec3,
    delta: Vec3,
    y_rot: f32,
    x_rot: f32,
    bounding_box: Aabb,
    pose: ClientEntityPose,
    dimensions: ClientEntityDimensions,
    on_ground: bool,
    horizontal_collision: bool,
    vertical_collision: bool,
    vertical_collision_below: bool,
    minor_horizontal_collision: bool,
    main_supporting_block: Option<BlockPos>,
    on_ground_no_blocks: bool,
    fall_distance: f64,
    stuck_speed_multiplier: Vec3,
    /// `ClientAvatarState`: the distance walked and the view bob, now and a
    /// tick ago, which only the camera reads.
    walk_dist: f32,
    walk_dist_old: f32,
    /// The body's and the head's facing, now and a tick ago, which the
    /// player model is turned by.
    y_body_rot: f32,
    y_body_rot_old: f32,
    y_head_rot: f32,
    y_head_rot_old: f32,
    /// `WalkAnimationState`: how fast and how far the limbs have swung.
    walk: crate::living::WalkAnimation,
    /// What the vehicle the player sits in makes of it, while it rides.
    riding: Option<Riding>,
    /// The first-person hand's lagging view: it chases the rotation by
    /// half the gap each tick.
    x_bob: f32,
    y_bob: f32,
    x_bob_old: f32,
    y_bob_old: f32,
    /// `LivingEntity`'s arm swing: whether one is under way, how many
    /// ticks in, and how far through, now and a tick ago.
    swinging: bool,
    swing_time: i32,
    attack_anim: f32,
    attack_anim_old: f32,
    /// `Player.attackStrengthTicker` and `itemSwapTicker`: ticks since the
    /// last attack or change of held item.
    attack_strength_ticker: i32,
    /// The main hand item's registry id (air when empty), and the one the
    /// last tick saw.
    main_hand_item: i32,
    last_main_hand_item: i32,
    item_swap_ticker: i32,
    /// `LivingEntity`'s health as the client last set it, and the hurt
    /// state `LocalPlayer.hurtTo` keeps: ticks of damage immunity, of the
    /// hurt flash, and whether the next health update may flash at all.
    health: f32,
    flash_on_set_health: bool,
    invulnerable_time: i32,
    hurt_time: i32,
    /// How far into the swimming pose the model has turned.
    swim_amount: f32,
    swim_amount_old: f32,
    bob: f32,
    bob_old: f32,
    first_tick: bool,
    tick_count: i32,
    /// `LivingEntity.elytraAnimationState`: the local player never glides in this port.
    elytra: crate::elytra::ElytraState,
    /// `Entity.moveDist` and `nextStep`: how far the player has walked, and where the next step sounds.
    move_dist: f32,
    next_step: f32,
    pending_sounds: Vec<LocalSound>,
    was_touching_water: bool,
    was_eye_in_water: bool,
    water: FluidTracker,
    lava: FluidTracker,
    sprinting: bool,
    swimming: bool,
    crouching: bool,
    input: KeyboardInput,
    xxa: f32,
    zza: f32,
    jumping: bool,
    no_jump_delay: i32,
    jump_trigger_time: i32,
    sprint_trigger_time: i32,
    auto_jump_time: i32,
    abilities: ClientPlayerAbilities,
    game_mode: ClientPlayerGameMode,
    no_physics: bool,
    auto_jump_option: bool,
    auto_jump_enabled: bool,
    movements_this_tick: Vec<Movement>,
    x_last: f64,
    y_last: f64,
    z_last: f64,
    y_rot_last: f32,
    x_rot_last: f32,
    last_on_ground: bool,
    last_horizontal_collision: bool,
    was_sprinting: bool,
    position_reminder: i32,
    last_sent_input: Input,
    sent: Vec<ServerboundMovement>,
}

/// `Player.getCurrentItemAttackStrengthDelay`: the ticks an attack takes to recharge at an
/// attack speed.
pub fn attack_strength_delay(attack_speed: f64) -> f32 {
    (1.0 / attack_speed * 20.0) as f32
}

impl LocalPlayer {
    /// A player the connection just made, snapped to its spawn pose, with the
    /// game mode's abilities applied.
    pub fn new(
        entity_id: i32,
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        game_mode: ClientPlayerGameMode,
        auto_jump_option: bool,
    ) -> Self {
        let pose = ClientEntityPose::Standing;
        let dimensions = pose.player_dimensions();
        Self {
            entity_id,
            position,
            old_position: position,
            delta: Vec3::ZERO,
            y_rot,
            x_rot,
            bounding_box: dimensions.make_bounding_box(position),
            pose,
            dimensions,
            on_ground: false,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            minor_horizontal_collision: false,
            main_supporting_block: None,
            on_ground_no_blocks: false,
            fall_distance: 0.0,
            stuck_speed_multiplier: Vec3::ZERO,
            walk_dist: 0.0,
            walk_dist_old: 0.0,
            // The reference starts a new player facing at random; this one
            // starts with head and body where it looks.
            y_body_rot: y_rot,
            y_body_rot_old: y_rot,
            y_head_rot: y_rot,
            y_head_rot_old: y_rot,
            walk: crate::living::WalkAnimation::default(),
            riding: None,
            x_bob: 0.0,
            y_bob: 0.0,
            x_bob_old: 0.0,
            y_bob_old: 0.0,
            swinging: false,
            swing_time: 0,
            attack_anim: 0.0,
            attack_anim_old: 0.0,
            attack_strength_ticker: 0,
            elytra: crate::elytra::ElytraState::default(),
            main_hand_item: 0,
            last_main_hand_item: 0,
            item_swap_ticker: 0,
            health: 20.0,
            flash_on_set_health: false,
            invulnerable_time: 0,
            hurt_time: 0,
            swim_amount: 0.0,
            swim_amount_old: 0.0,
            bob: 0.0,
            bob_old: 0.0,
            first_tick: true,
            tick_count: 0,
            move_dist: 0.0,
            next_step: 1.0,
            pending_sounds: Vec::new(),
            was_touching_water: false,
            was_eye_in_water: false,
            water: FluidTracker::default(),
            lava: FluidTracker::default(),
            sprinting: false,
            swimming: false,
            crouching: false,
            input: KeyboardInput::new(Input::EMPTY),
            xxa: 0.0,
            zza: 0.0,
            jumping: false,
            no_jump_delay: 0,
            jump_trigger_time: 0,
            sprint_trigger_time: 0,
            auto_jump_time: 0,
            abilities: game_mode.update_player_abilities(ClientPlayerAbilities::default()),
            game_mode,
            no_physics: false,
            auto_jump_option,
            auto_jump_enabled: true,
            movements_this_tick: Vec::new(),
            x_last: 0.0,
            y_last: 0.0,
            z_last: 0.0,
            y_rot_last: 0.0,
            x_rot_last: 0.0,
            last_on_ground: false,
            last_horizontal_collision: false,
            was_sprinting: false,
            position_reminder: 0,
            last_sent_input: Input::EMPTY,
            sent: Vec::new(),
        }
    }

    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn old_position(&self) -> Vec3 {
        self.old_position
    }

    pub fn delta_movement(&self) -> Vec3 {
        self.delta
    }

    /// `ElytraAnimationState.getRotX/Y/Z`.
    pub fn elytra_rotations(&self, partial_tick: f32) -> [f32; 3] {
        self.elytra.rotations(partial_tick)
    }

    pub fn rotation(&self) -> (f32, f32) {
        (self.y_rot, self.x_rot)
    }

    /// Turn the view, as the mouse handler does between ticks.
    pub fn set_rotation(&mut self, y_rot: f32, x_rot: f32) {
        self.y_rot = y_rot;
        self.x_rot = x_rot.clamp(-90.0, 90.0);
    }

    /// `Entity.getBoundingBox`.
    pub fn bounding_box(&self) -> voxel_core::aabb::Aabb {
        self.bounding_box
    }

    pub fn eye_height(&self) -> f32 {
        self.dimensions.eye_height()
    }

    /// `getAttributeValue(MOVEMENT_SPEED)` as the field of view reads it.
    pub fn movement_speed(&self) -> f32 {
        self.speed()
    }

    pub fn abilities(&self) -> ClientPlayerAbilities {
        self.abilities
    }

    /// Whether the eyes are inside water, as the last fluid update found.
    pub fn is_eye_in_water(&self) -> bool {
        self.water.eyes_inside
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn horizontal_collision(&self) -> bool {
        self.horizontal_collision
    }

    pub fn vertical_collision(&self) -> bool {
        self.vertical_collision
    }

    pub fn minor_horizontal_collision(&self) -> bool {
        self.minor_horizontal_collision
    }

    pub fn is_sprinting(&self) -> bool {
        self.sprinting
    }

    /// `LocalPlayer.isShiftKeyDown`: the input, not the synced flag.
    pub fn is_shift_key_down(&self) -> bool {
        self.input.key_presses.shift
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    pub fn fall_distance(&self) -> f64 {
        self.fall_distance
    }

    pub fn is_flying(&self) -> bool {
        self.abilities.flying
    }

    /// Move the player where the server put it, as a teleport does.
    pub fn snap_to(&mut self, position: Vec3, y_rot: f32, x_rot: f32) {
        self.set_pos(position);
        self.old_position = position;
        self.set_rotation(y_rot, x_rot);
    }

    /// `Entity.absSnapTo`: the position, kept inside the world's limits,
    /// with no movement from last tick.
    pub fn abs_snap_to(&mut self, position: Vec3) {
        let position = Vec3::new(
            position.x.clamp(-3.0e7, 3.0e7),
            position.y,
            position.z.clamp(-3.0e7, 3.0e7),
        );
        self.old_position = position;
        self.set_pos(position);
    }

    /// `Entity.isColliding`: whether a block's collision shape at `pos`
    /// overlaps the player.
    pub fn is_colliding(&self, world: &ClientWorld, pos: BlockPos, state: i32) -> bool {
        let shape = world
            .block_state_registry()
            .collision_shape_at(state, [pos.x, pos.y, pos.z])
            .move_by_block_pos(pos);
        let Ok(player) = Shapes::create_from_aabb(self.bounding_box) else {
            return false;
        };
        Shapes::join_is_not_empty(&shape, &player, BooleanOp::And)
    }

    /// `setValuesFromPositionPacket` without interpolation: the position,
    /// motion and view the server sent, with nothing to blend from.
    pub fn apply_teleport(&mut self, position: Vec3, delta: Vec3, y_rot: f32, x_rot: f32) {
        self.set_pos(position);
        self.delta = delta;
        self.old_position = position;
        self.set_rotation(y_rot, x_rot);
    }

    /// `Entity.turn`: the mouse's accumulated movement, already scaled by
    /// the sensitivity, turned into degrees.
    pub fn turn(&mut self, y_rot_delta: f64, x_rot_delta: f64) {
        let x_delta = x_rot_delta as f32 * 0.15;
        let y_delta = y_rot_delta as f32 * 0.15;
        self.x_rot = (self.x_rot + x_delta).clamp(-90.0, 90.0);
        self.y_rot += y_delta;
    }

    /// `MultiPlayerGameMode.setLocalMode`: the new mode's abilities.
    pub fn set_game_mode(&mut self, game_mode: ClientPlayerGameMode) {
        self.game_mode = game_mode;
        self.abilities = game_mode.update_player_abilities(self.abilities);
    }

    /// `handlePlayerAbilities`: every field but the build permission.
    pub fn set_abilities(&mut self, abilities: ClientPlayerAbilities) {
        let may_build = self.abilities.may_build;
        self.abilities = abilities;
        self.abilities.may_build = may_build;
    }

    /// The feet position between the last two ticks.
    /// `LocalPlayer.hurtTo`: the server set the health. A loss starts the
    /// hurt flash and a second of immunity, a gain half a second; the very
    /// first update only sets it.
    pub fn hurt_to(&mut self, new_health: f32) {
        if self.flash_on_set_health {
            let damage = self.health - new_health;
            if damage <= 0.0 {
                self.health = new_health;
                if damage < 0.0 {
                    self.invulnerable_time = 10;
                }
            } else {
                self.invulnerable_time = 20;
                self.health = new_health;
                self.hurt_time = 10;
            }
        } else {
            self.health = new_health;
            self.flash_on_set_health = true;
        }
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    /// Ticks of damage immunity left, which the HUD's hearts blink by.
    pub fn invulnerable_time(&self) -> i32 {
        self.invulnerable_time
    }

    /// `Player.getAttackStrengthScale`: how far the attack has recharged,
    /// over the attack delay the attack speed sets.
    pub fn attack_strength_scale(&self, partial_tick: f32, attack_speed: f64) -> f32 {
        let delay = attack_strength_delay(attack_speed);
        ((self.attack_strength_ticker as f32 + partial_tick) / delay).clamp(0.0, 1.0)
    }

    /// `LivingEntity.swing` for the main hand: start a swing, unless one is
    /// in its first half.
    pub fn swing(&mut self) {
        if !self.swinging || self.swing_time >= SWING_DURATION / 2 || self.swing_time < 0 {
            self.swing_time = -1;
            self.swinging = true;
        }
    }

    /// The item now in the main hand, by registry id; air when empty.
    pub fn set_main_hand_item(&mut self, item: i32) {
        self.main_hand_item = item;
    }

    pub fn attack_strength_ticker(&self) -> i32 {
        self.attack_strength_ticker
    }

    /// `Player.resetAttackStrengthTicker`.
    pub fn reset_attack_strength(&mut self) {
        self.attack_strength_ticker = 0;
        self.item_swap_ticker = 0;
    }

    /// `LivingEntity.getAttackAnim`: how far through the swing, between the
    /// last two ticks, wrapping when a new swing began.
    pub fn attack_anim(&self, partial_tick: f32) -> f32 {
        let mut difference = self.attack_anim - self.attack_anim_old;
        if difference < 0.0 {
            difference += 1.0;
        }
        self.attack_anim_old + difference * partial_tick
    }

    /// `Player.getItemSwapScale`: how far the hand has come back up after
    /// an attack or a change of item, over the attack delay the attack
    /// speed sets (five ticks at the default speed of four).
    pub fn item_swap_scale(&self, partial_tick: f32, attack_speed: f64) -> f32 {
        let delay = (1.0 / attack_speed * 20.0) as f32;
        ((self.item_swap_ticker as f32 + partial_tick) / delay).clamp(0.0, 1.0)
    }

    /// The hand's lagging pitch and yaw between the last two ticks.
    pub fn hand_bob(&self, partial_tick: f32) -> (f32, f32) {
        (
            self.x_bob_old + partial_tick * (self.x_bob - self.x_bob_old),
            self.y_bob_old + partial_tick * (self.y_bob - self.y_bob_old),
        )
    }

    /// How many ticks the player has lived, which the model's idle sway
    /// reads.
    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    /// Whether the player only watches.
    pub fn spectating(&self) -> bool {
        self.is_spectator()
    }

    /// Whether the player stands in water.
    pub fn in_water(&self) -> bool {
        self.is_in_water()
    }

    /// What the player model is posed from, between the last two ticks.
    pub fn model_pose(&self, partial_tick: f32) -> ModelPose {
        let lerp = |old: f32, new: f32| old + partial_tick * (new - old);
        ModelPose {
            body_rot: rot_lerp(partial_tick, self.y_body_rot_old, self.y_body_rot),
            head_rot: rot_lerp(partial_tick, self.y_head_rot_old, self.y_head_rot),
            x_rot: self.x_rot,
            walk_position: self.walk.position_at(partial_tick),
            walk_speed: self.walk.speed_at(partial_tick),
            swim_amount: lerp(self.swim_amount_old, self.swim_amount),
            crouching: self.pose == ClientEntityPose::Crouching,
            pose: self.pose,
        }
    }

    /// `ClientAvatarState.getBackwardsInterpolatedWalkDistance` and
    /// `getInterpolatedBob`, which the view bob reads.
    pub fn view_bob(&self, partial_tick: f32) -> (f32, f32) {
        let walked = self.walk_dist - self.walk_dist_old;
        (
            -(self.walk_dist + walked * partial_tick),
            self.bob_old + partial_tick * (self.bob - self.bob_old),
        )
    }

    /// `yBodyRotO` and `yBodyRot`: the body's turn at the last tick and now.
    pub fn body_rotations(&self) -> (f32, f32) {
        (self.y_body_rot_old, self.y_body_rot)
    }

    pub fn interpolated_position(&self, partial_tick: f64) -> Vec3 {
        self.old_position.lerp(self.position, partial_tick)
    }

    /// Run one client tick with the given keys held, and return what it sent.
    pub fn tick(
        &mut self,
        world: &ClientWorld,
        keys: Input,
    ) -> Result<Vec<ServerboundMovement>, Unsupported> {
        let level = Level { world };
        self.sent.clear();
        self.input.set_key_states(keys);

        // ClientLevel.tickNonPassenger: the old position first, then the count.
        self.old_position = self.position;
        self.tick_count += 1;

        // AbstractClientPlayer.tick: the avatar state steps first.
        self.walk_dist_old = self.walk_dist;
        self.player_tick(&level)?;

        if self.last_sent_input != self.input.key_presses {
            self.sent
                .push(ServerboundMovement::PlayerInput(self.input.key_presses));
            self.last_sent_input = self.input.key_presses;
        }
        self.send_position();
        Ok(std::mem::take(&mut self.sent))
    }

    /// One client tick of the player sitting in a boat the world has already ticked: what
    /// `LocalPlayer.rideTick` does (the boat's own tick came first, as the level ticks the vehicle
    /// before its passengers), with the packets `LocalPlayer.tick` sends of a passenger.
    pub fn tick_riding(
        &mut self,
        world: &ClientWorld,
        keys: Input,
        boat: &crate::boat_ride::BoatView,
    ) -> Result<RideTick, Unsupported> {
        let level = Level { world };
        self.sent.clear();
        self.input.set_key_states(keys);
        self.riding = Some(Riding {
            vehicle_max_y: boat.max_y,
            vehicle_under_water: boat.under_water,
            can_sprint: false,
        });
        // `ClientLevel.tickPassenger`: the old position first, then the count; `Entity.rideTick`
        // stills the player before its own tick.
        self.old_position = self.position;
        self.tick_count += 1;
        self.delta = Vec3::ZERO;
        self.walk_dist_old = self.walk_dist;
        let ticked = self.player_tick(&level);
        if let Err(error) = ticked {
            self.riding = None;
            return Err(error);
        }
        if self.last_sent_input != self.input.key_presses {
            self.sent
                .push(ServerboundMovement::PlayerInput(self.input.key_presses));
            self.last_sent_input = self.input.key_presses;
        }
        // A passenger tells the server which way it looks, and, when it is the vehicle's
        // authority, where the vehicle is.
        self.sent.push(ServerboundMovement::MovePlayerRot {
            y_rot: self.y_rot,
            x_rot: self.x_rot,
            on_ground: self.on_ground,
            horizontal_collision: self.horizontal_collision,
        });
        self.sent.push(ServerboundMovement::MoveVehicle {
            position: boat.position,
            y_rot: boat.y_rot,
            x_rot: 0.0,
            on_ground: boat.on_ground,
        });
        if self.sprinting != self.was_sprinting {
            self.sent.push(ServerboundMovement::PlayerCommand {
                entity_id: self.entity_id,
                action: if self.sprinting {
                    PlayerCommandAction::StartSprinting
                } else {
                    PlayerCommandAction::StopSprinting
                },
            });
            self.was_sprinting = self.sprinting;
        }
        // `LivingEntity.rideTick` and then the boat: the fall is forgiven, and the boat is handed the
        // keys, before `positionRider` carries the player to its seat and turns it with the boat.
        self.fall_distance = 0.0;
        let keys = self.input.key_presses;
        let boat_input =
            crate::entity::BoatInput::new(keys.left, keys.right, keys.forward, keys.backward);
        self.position_in_boat(boat);
        self.riding = None;
        Ok(RideTick {
            sent: std::mem::take(&mut self.sent),
            boat_input,
        })
    }

    /// `AbstractBoat.positionRider` for the player: carried to its seat, turned as the boat turns
    /// and held within a quarter turn of where the boat faces (`clampRotation`).
    fn position_in_boat(&mut self, boat: &crate::boat_ride::BoatView) {
        self.set_pos(boat.rider_position);
        self.y_rot += boat.delta_rotation;
        self.y_head_rot += boat.delta_rotation;
        let (_, y_rot) = crate::entity::vanilla_boat_passenger_rotation_clamped_values(
            self.y_rot, self.y_rot, boat.y_rot,
        );
        self.y_rot = y_rot;
        self.y_head_rot = self.y_rot;
        self.y_body_rot = boat.y_rot;
    }

    /// `ClientPacketListener.handleSetEntityPassengersPacket` for a player who has just boarded a
    /// boat: it looks the way the boat faces.
    pub fn face_boat(&mut self, boat_y_rot: f32) {
        self.y_rot = boat_y_rot;
        self.y_head_rot = boat_y_rot;
    }

    /// `AbstractBoat.onPassengerTurned`, after the mouse turned the player: held within a quarter
    /// turn of where the boat faces.
    pub fn clamp_in_boat(&mut self, boat_y_rot: f32) {
        let (_, y_rot) = crate::entity::vanilla_boat_passenger_rotation_clamped_values(
            self.y_rot, self.y_rot, boat_y_rot,
        );
        self.y_rot = y_rot;
        self.y_head_rot = y_rot;
        self.y_body_rot = boat_y_rot;
    }

    fn is_passenger(&self) -> bool {
        self.riding.is_some()
    }

    fn player_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        self.no_physics = self.is_spectator();
        if self.is_spectator() || self.is_passenger() {
            self.on_ground = false;
            self.check_supporting_block(level, false, None)?;
        }
        // updateIsUnderwater keeps a copy only the server reads.
        self.living_tick(level)?;
        let clamped_x = self.position.x.clamp(-2.9999999e7, 2.9999999e7);
        let clamped_z = self.position.z.clamp(-2.9999999e7, 2.9999999e7);
        if clamped_x != self.position.x || clamped_z != self.position.z {
            self.set_pos(Vec3::new(clamped_x, self.position.y, clamped_z));
        }
        self.elytra.tick(crate::elytra::ElytraInput {
            fall_flying: false,
            motion: (self.delta.x, self.delta.y, self.delta.z),
            crouching: self.crouching,
        });
        // Player.tick, after the living tick.
        self.attack_strength_ticker += 1;
        self.item_swap_ticker += 1;
        // A different item in the main hand starts the attack strength over.
        if self.main_hand_item != self.last_main_hand_item {
            self.reset_attack_strength();
            self.last_main_hand_item = self.main_hand_item;
        }
        self.update_player_pose(level)
    }

    fn living_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        // LivingEntity.baseTick keeps last tick's swing and facings.
        self.attack_anim_old = self.attack_anim;
        self.y_head_rot_old = self.y_head_rot;
        self.y_body_rot_old = self.y_body_rot;
        // Entity.baseTick, as the living base tick runs it.
        self.was_eye_in_water = self.water.eyes_inside;
        self.update_fluid_interaction(level)?;
        self.update_swimming(level);
        if self.is_in_lava() {
            self.fall_distance *= 0.5;
        }
        if self.position.y < f64::from(level.world.dimension().min_y - 64) {
            return unsupported("falling below the world");
        }
        self.first_tick = false;
        // The rest of LivingEntity.baseTick on a client.
        if self.hurt_time > 0 {
            self.hurt_time -= 1;
        }
        if self.invulnerable_time > 0 {
            self.invulnerable_time -= 1;
        }
        // LivingEntity.tick: the swimming pose eases in or out.
        self.swim_amount_old = self.swim_amount;
        self.swim_amount = crate::living::ease_swim_amount(
            self.swim_amount,
            self.pose == ClientEntityPose::Swimming,
        );
        self.local_ai_step(level)?;
        self.turn_body();
        Ok(())
    }

    /// The rest of `LivingEntity.tick`: the body turns towards where the
    /// player walks, or where it looks, and never more than 50 degrees from
    /// the head.
    fn turn_body(&mut self) {
        crate::living::turn_body(
            (&mut self.y_body_rot, &mut self.y_body_rot_old),
            (self.y_rot, self.y_head_rot, &mut self.y_head_rot_old),
            (
                self.position.x - self.old_position.x,
                self.position.z - self.old_position.z,
            ),
        );
    }

    #[expect(
        clippy::cognitive_complexity,
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    fn local_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if self.sprint_trigger_time > 0 {
            self.sprint_trigger_time -= 1;
        }

        let was_jumping = self.input.key_presses.jump;
        let was_shift_key_down = self.input.key_presses.shift;
        let had_forward_impulse = self.input.move_vector().y > 1.0e-5;
        self.crouching = !self.abilities.flying
            && !self.is_swimming()
            && !self.is_passenger()
            && self.can_fit(level, ClientEntityPose::Crouching)?
            && (self.is_shift_key_down() || !self.can_fit(level, ClientEntityPose::Standing)?);
        self.input.tick();
        let mut was_auto_jump = false;
        if self.auto_jump_time > 0 {
            self.auto_jump_time -= 1;
            was_auto_jump = true;
            self.input.key_presses.jump = true;
        }

        if !self.no_physics {
            let half = f64::from(self.dimensions.width()) * 0.35;
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x - half, z + half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x - half, z - half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x + half, z - half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x + half, z + half);
        }

        if was_shift_key_down || self.input.key_presses.backward {
            self.sprint_trigger_time = 0;
        }

        if self.can_start_sprinting() {
            if !had_forward_impulse {
                if self.sprint_trigger_time > 0 {
                    self.set_sprinting(true);
                } else {
                    self.sprint_trigger_time = SPRINT_WINDOW;
                }
            }
            if self.input.key_presses.sprint {
                self.set_sprinting(true);
            }
        }

        if self.sprinting {
            if self.is_swimming() {
                if self.should_stop_swim_sprinting() {
                    self.set_sprinting(false);
                }
            } else if self.should_stop_run_sprinting() {
                self.set_sprinting(false);
            }
        }

        let mut just_toggled_creative_flight = false;
        if self.abilities.may_fly {
            if self.is_spectator() {
                if !self.abilities.flying {
                    self.abilities.flying = true;
                    just_toggled_creative_flight = true;
                    self.on_update_abilities();
                }
            } else if !was_jumping && self.input.key_presses.jump && !was_auto_jump {
                if self.jump_trigger_time == 0 {
                    self.jump_trigger_time = 7;
                } else if !self.is_swimming() {
                    self.abilities.flying = !self.abilities.flying;
                    if self.abilities.flying && self.on_ground {
                        self.jump_from_ground(level);
                    }
                    just_toggled_creative_flight = true;
                    self.on_update_abilities();
                    self.jump_trigger_time = 0;
                }
            }
        }
        // Gliding needs an elytra, which this port's player never wears.
        let _ = just_toggled_creative_flight;

        if self.is_in_water() && self.input.key_presses.shift && self.is_affected_by_fluids() {
            self.delta = self.delta.add_components(0.0, f64::from(-0.04_f32), 0.0);
        }

        if self.abilities.flying {
            let mut input_ya = 0;
            if self.input.key_presses.shift {
                input_ya -= 1;
            }
            if self.input.key_presses.jump {
                input_ya += 1;
            }
            if input_ya != 0 {
                let rise = input_ya as f32 * self.abilities.flying_speed() * 3.0_f32;
                self.delta = self.delta.add_components(0.0, f64::from(rise), 0.0);
            }
        }

        // AbstractClientPlayer.aiStep: the bob eases towards the ground speed.
        let target_bob = if self.on_ground && !self.is_swimming() {
            (self.delta.horizontal_distance() as f32).min(0.1)
        } else {
            0.0
        };
        self.bob_old = self.bob;
        self.bob += (target_bob - self.bob) * 0.4;
        self.player_ai_step(level)?;
        if self.on_ground && self.abilities.flying && !self.is_spectator() {
            self.abilities.flying = false;
            self.on_update_abilities();
        }
        Ok(())
    }

    fn player_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if self.jump_trigger_time > 0 {
            self.jump_trigger_time -= 1;
        }
        if self.abilities.flying {
            self.fall_distance = 0.0;
        }
        self.living_ai_step(level)?;
        // `updateSwingTime`: an empty hand swings over six ticks.
        if self.swinging {
            self.swing_time += 1;
            if self.swing_time >= SWING_DURATION {
                self.swing_time = 0;
                self.swinging = false;
            }
        } else {
            self.swing_time = 0;
        }
        self.attack_anim = self.swing_time as f32 / SWING_DURATION as f32;
        self.y_head_rot = self.y_rot;
        Ok(())
    }

    fn living_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if self.no_jump_delay > 0 {
            self.no_jump_delay -= 1;
        }

        let mut dx = self.delta.x;
        let mut dy = self.delta.y;
        let mut dz = self.delta.z;
        if self.delta.horizontal_distance_squared() < 9.0e-6 {
            dx = 0.0;
            dz = 0.0;
        }
        if self.delta.y.abs() < 0.003 {
            dy = 0.0;
        }
        self.delta = Vec3::new(dx, dy, dz);

        // LocalPlayer.applyInput.
        let modified = modify_local_player_input(
            self.input.move_vector(),
            LocalPlayerInputModifiers {
                using_item: false,
                passenger: self.is_passenger(),
                item_use_speed_multiplier: 1.0,
                moving_slowly: self.is_moving_slowly(),
                sneaking_movement_factor: SNEAKING_SPEED as f32,
            },
        );
        self.xxa = modified.x;
        self.zza = modified.y;
        self.jumping = self.input.key_presses.jump;
        self.y_bob_old = self.y_bob;
        self.x_bob_old = self.x_bob;
        self.x_bob += (self.x_rot - self.x_bob) * 0.5;
        self.y_bob += (self.y_rot - self.y_bob) * 0.5;

        if self.jumping && self.is_affected_by_fluids() {
            let fluid_height = if self.is_in_lava() {
                self.lava.height
            } else {
                self.water.height
            };
            let in_water_and_has_fluid_height = self.is_in_water() && fluid_height > 0.0;
            let threshold = self.fluid_jump_threshold();
            if !in_water_and_has_fluid_height || self.on_ground && !(fluid_height > threshold) {
                if !self.is_in_lava() || self.on_ground && self.lava.height <= threshold {
                    if (self.on_ground
                        || in_water_and_has_fluid_height && fluid_height <= threshold)
                        && self.no_jump_delay == 0
                    {
                        self.jump_from_ground(level);
                        self.no_jump_delay = 10;
                    }
                } else {
                    self.delta = self.delta.add_components(0.0, f64::from(0.04_f32), 0.0);
                }
            } else {
                self.delta = self.delta.add_components(0.0, f64::from(0.04_f32), 0.0);
            }
        } else {
            self.no_jump_delay = 0;
        }

        let input = Vec3::new(f64::from(self.xxa), 0.0, f64::from(self.zza));
        self.player_travel(level, input)?;
        self.apply_effects_from_blocks(level)?;
        // `calculateEntityAnimation`: the limbs swing with the distance
        // moved across the ground this tick.
        let x = self.position.x - self.old_position.x;
        let z = self.position.z - self.old_position.z;
        if self.is_passenger() {
            self.walk.stop();
        } else {
            self.walk.follow((x * x + z * z).sqrt(), false);
        }
        Ok(())
    }

    fn player_travel(&mut self, level: &Level<'_>, input: Vec3) -> Result<(), Unsupported> {
        // `Player.travel`: a passenger travels as any living entity does.
        if self.is_passenger() {
            return self.living_travel(level, input);
        }
        if self.is_swimming() {
            let look_y = self.look_angle().y;
            let multiplier = if look_y < -0.2 { 0.085 } else { 0.06 };
            let above = BlockPos::new(
                floor_f64(self.position.x),
                floor_f64(self.position.y + 1.0 - 0.1),
                floor_f64(self.position.z),
            );
            if look_y <= 0.0 || self.jumping || level.info(above).fluid != Fluid::Empty {
                self.delta =
                    self.delta
                        .add_components(0.0, (look_y - self.delta.y) * multiplier, 0.0);
            }
        }

        if self.abilities.flying {
            let original_y = self.delta.y;
            self.living_travel(level, input)?;
            self.delta = self.delta.with(Axis::Y, original_y * 0.6);
            Ok(())
        } else {
            self.living_travel(level, input)
        }
    }

    fn living_travel(&mut self, level: &Level<'_>, input: Vec3) -> Result<(), Unsupported> {
        if (self.is_in_water() || self.is_in_lava()) && self.is_affected_by_fluids() {
            self.travel_in_fluid(level, input)
        } else {
            self.travel_in_air(level, input)
        }
    }

    fn travel_in_air(&mut self, level: &Level<'_>, input: Vec3) -> Result<(), Unsupported> {
        let pos_below = self.get_on_pos(level, 0.500_001_f32);
        let block_friction = if self.on_ground {
            compute_modified_friction(level.info(pos_below).friction, 1.0)
        } else {
            1.0
        };
        let movement =
            self.handle_relative_friction_and_calculate_movement(level, input, block_friction)?;
        let movement_y = if level.has_chunk_at(pos_below) {
            movement.y - GRAVITY
        } else if self.position.y > f64::from(level.world.dimension().min_y) {
            -0.1
        } else {
            0.0
        };
        let air_drag = compute_modified_friction(0.91, 1.0);
        let friction = block_friction * air_drag;
        let vertical_friction = compute_modified_friction(0.98, 1.0);
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
        let base_gravity = GRAVITY;
        if self.is_in_water() {
            let mut slow_down = if self.sprinting { 0.9_f32 } else { 0.8_f32 };
            let speed = 0.02_f32;
            // The player's water movement efficiency is zero: no blend.
            let _ = &mut slow_down;
            self.move_relative(speed, input);
            self.move_self(level, self.delta)?;
            let mut movement = self.delta;
            if self.horizontal_collision && self.on_climbable(level) {
                movement = Vec3::new(movement.x, 0.2, movement.z);
            }
            movement = movement.multiply_components(
                f64::from(slow_down),
                f64::from(0.8_f32),
                f64::from(slow_down),
            );
            self.delta = self.fluid_falling_adjusted_movement(base_gravity, is_falling, movement);
            self.jump_out_of_fluid(level, old_y)
        } else {
            self.move_relative(0.02, input);
            self.move_self(level, self.delta)?;
            if self.lava.height <= self.fluid_jump_threshold() {
                self.delta = self.delta.multiply_components(0.5, f64::from(0.8_f32), 0.5);
                self.delta =
                    self.fluid_falling_adjusted_movement(base_gravity, is_falling, self.delta);
            } else {
                self.delta = self.delta.scale(0.5);
            }
            self.delta = self.delta.add_components(0.0, -base_gravity / 4.0, 0.0);
            self.jump_out_of_fluid(level, old_y)
        }
    }

    fn fluid_falling_adjusted_movement(
        &self,
        base_gravity: f64,
        is_falling: bool,
        movement: Vec3,
    ) -> Vec3 {
        if base_gravity != 0.0 && !self.sprinting {
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

    fn jump_out_of_fluid(&mut self, level: &Level<'_>, old_y: f64) -> Result<(), Unsupported> {
        let movement = self.delta;
        if self.horizontal_collision {
            let lift = movement.y + f64::from(0.6_f32) - self.position.y + old_y;
            let moved = self.bounding_box.move_by(movement.x, lift, movement.z);
            if level.no_collision(moved, self.fall_distance)?
                && !self.contains_any_liquid(level, moved)
            {
                self.delta = Vec3::new(movement.x, f64::from(0.3_f32), movement.z);
            }
        }
        Ok(())
    }

    fn contains_any_liquid(&self, level: &Level<'_>, area: Aabb) -> bool {
        let x0 = floor_f64(area.min_x);
        let x1 = voxel_core::math::ceil_f64(area.max_x);
        let y0 = floor_f64(area.min_y);
        let y1 = voxel_core::math::ceil_f64(area.max_y);
        let z0 = floor_f64(area.min_z);
        let z1 = voxel_core::math::ceil_f64(area.max_z);
        for x in x0..x1 {
            for y in y0..y1 {
                for z in z0..z1 {
                    if level.info(BlockPos::new(x, y, z)).fluid != Fluid::Empty {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn handle_relative_friction_and_calculate_movement(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
        friction: f32,
    ) -> Result<Vec3, Unsupported> {
        let speed = self.friction_influenced_speed(friction);
        self.move_relative(speed, input);
        self.delta = self.handle_on_climbable(level, self.delta)?;
        self.move_self(level, self.delta)?;
        let mut movement = self.delta;
        if (self.horizontal_collision || self.jumping) && self.on_climbable(level) {
            movement = Vec3::new(movement.x, 0.2, movement.z);
        }
        Ok(movement)
    }

    fn friction_influenced_speed(&self, block_friction: f32) -> f32 {
        if self.on_ground {
            // The reference compares the float against a double literal.
            if f64::from(block_friction) > 0.6 {
                self.speed()
                    * (0.216_000_02_f32 / (block_friction * block_friction * block_friction))
            } else {
                self.speed()
            }
        } else if self.abilities.flying {
            if self.sprinting {
                self.abilities.flying_speed() * 2.0
            } else {
                self.abilities.flying_speed()
            }
        } else if self.sprinting {
            0.025_999_999_f32
        } else {
            0.02_f32
        }
    }

    fn handle_on_climbable(&mut self, level: &Level<'_>, delta: Vec3) -> Result<Vec3, Unsupported> {
        if !self.on_climbable(level) {
            return Ok(delta);
        }
        self.fall_distance = 0.0;
        let max = f64::from(0.15_f32);
        let x = java_math::clamp(delta.x, -max, max);
        let z = java_math::clamp(delta.z, -max, max);
        let mut y = java_math::max(delta.y, -max);
        if y < 0.0 && self.is_shift_key_down() && !self.abilities.flying {
            if level.info(self.block_position()).name == "minecraft:scaffolding" {
                return unsupported("climbing scaffolding");
            }
            y = 0.0;
        }
        Ok(Vec3::new(x, y, z))
    }

    fn on_climbable(&self, level: &Level<'_>) -> bool {
        if self.is_spectator() {
            return false;
        }
        let pos = self.block_position();
        let info = level.info(pos);
        if info.flags & flag::CLIMBABLE != 0 {
            return true;
        }
        if info.flags & flag::TRAPDOOR != 0 {
            let state = level.state_id(pos);
            let open = property(state, "open") == Some("true");
            if open {
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

    fn move_relative(&mut self, speed: f32, input: Vec3) {
        let relative = crate::input::local_player_relative_movement(input, speed, self.y_rot);
        self.delta = self.delta.add_vec3(relative);
    }

    /// `LocalPlayer.move`: the entity move, then the auto-jump look-ahead.
    fn move_self(&mut self, level: &Level<'_>, delta: Vec3) -> Result<(), Unsupported> {
        let previous_x = self.position.x;
        let previous_z = self.position.z;
        self.entity_move(level, delta)?;
        let delta_x = (self.position.x - previous_x) as f32;
        let delta_z = (self.position.z - previous_z) as f32;
        self.update_auto_jump(level, delta_x, delta_z)?;
        // `Mth.length(float, float)` squares in double: there is no float
        // `lengthSquared` of two values.
        let (x, z) = (f64::from(delta_x), f64::from(delta_z));
        self.walk_dist += (x * x + z * z).sqrt() as f32 * 0.6;
        Ok(())
    }

    fn entity_move(&mut self, level: &Level<'_>, delta: Vec3) -> Result<(), Unsupported> {
        if self.no_physics {
            self.set_pos(self.position.add_vec3(delta));
            self.horizontal_collision = false;
            self.vertical_collision = false;
            self.vertical_collision_below = false;
            self.minor_horizontal_collision = false;
            return Ok(());
        }

        let mut delta = delta;
        if self.stuck_speed_multiplier.length_squared() > 1.0e-7 {
            let stuck = self.stuck_speed_multiplier;
            delta = delta.multiply_components(stuck.x, stuck.y, stuck.z);
            self.stuck_speed_multiplier = Vec3::ZERO;
            self.delta = Vec3::ZERO;
        }
        let delta = self.maybe_back_off_from_edge(level, delta)?;
        let movement = self.collide(level, delta)?;
        let movement_length = movement.length_squared();
        if movement_length > 1.0e-7 || delta.length_squared() - movement_length < 1.0e-7 {
            if self.fall_distance != 0.0 && movement_length >= 1.0 {
                self.check_fall_reset_clip(level, movement)?;
            }
            let from = self.position;
            let to = from.add_vec3(movement);
            self.movements_this_tick.push(Movement {
                from,
                to,
                original: Some(delta),
            });
            self.set_pos(to);
        }

        let x_collision = !equal_f64(delta.x, movement.x);
        let z_collision = !equal_f64(delta.z, movement.z);
        self.horizontal_collision = x_collision || z_collision;
        let moved_vertically = delta.y.abs() > 0.0;
        // The local player is always its own authority.
        self.vertical_collision = delta.y != movement.y;
        self.vertical_collision_below = self.vertical_collision && delta.y < 0.0;
        self.on_ground = self.vertical_collision_below;
        self.check_supporting_block(level, self.on_ground, Some(movement))?;

        self.minor_horizontal_collision = self.horizontal_collision
            && is_minor_horizontal_collision(self.y_rot, self.xxa, self.zza, movement);

        let effect_pos = self.get_on_pos(level, 0.2);
        let effect_state = level.info(effect_pos);
        self.check_fall_damage(level, movement.y)?;

        if moved_vertically && self.vertical_collision || self.horizontal_collision {
            self.restitute_movement_after_collisions(
                &effect_state,
                x_collision,
                z_collision,
                movement,
            );
        }

        self.apply_movement_emission(level, movement, effect_pos);

        let speed_factor = f64::from(self.block_speed_factor(level));
        self.delta = self
            .delta
            .multiply_components(speed_factor, 1.0, speed_factor);
        Ok(())
    }

    /// `Level.clip` with the fall-damage-resetting block shapes and water:
    /// whether the fall so far crosses anything that forgives it.
    fn check_fall_reset_clip(
        &mut self,
        level: &Level<'_>,
        movement: Vec3,
    ) -> Result<(), Unsupported> {
        let distance = movement.length().min(8.0);
        let from = self.position;
        let to = from.add_vec3(movement.normalized().scale(distance));
        if level.clip_hits_fall_reset(from, to) {
            self.fall_distance = 0.0;
        }
        Ok(())
    }

    fn check_fall_damage(&mut self, level: &Level<'_>, ya: f64) -> Result<(), Unsupported> {
        if !self.is_in_water() {
            self.update_fluid_interaction(level)?;
        }
        if !self.is_in_water() && ya < 0.0 {
            self.fall_distance -= f64::from(ya as f32);
        }
        if self.on_ground {
            // The fall itself only hurts on the server; the client resets.
            self.fall_distance = 0.0;
        }
        Ok(())
    }

    fn restitute_movement_after_collisions(
        &mut self,
        effect_state: &StateInfo,
        x_collision: bool,
        z_collision: bool,
        movement: Vec3,
    ) {
        let suppressing_bounce = self.is_shift_key_down();
        let mut restitution = 0.0_f64;
        let current = self.delta;
        let mut after = current;
        if x_collision {
            after = after.with(Axis::X, -current.x * restitution);
        }
        if z_collision {
            after = after.with(Axis::Z, -current.z * restitution);
        }
        if self.vertical_collision {
            if self.vertical_collision_below {
                restitution = if !(-current.y < GRAVITY)
                    && !suppressing_bounce
                    && effect_state.flags & flag::SUPPRESSES_BOUNCE == 0
                {
                    java_math::max(restitution, f64::from(effect_state.bounce))
                } else {
                    0.0
                };
            }
            let (gravity_compensation, effective_drag) = if restitution > 0.0 {
                let portion = movement.y / current.y;
                let air_drag = f64::from(compute_modified_friction(0.98, 1.0));
                (portion * GRAVITY, 1.0 + portion * (air_drag - 1.0))
            } else {
                (0.0, 1.0)
            };
            after = after.with(
                Axis::Y,
                (gravity_compensation - current.y) * effective_drag * restitution,
            );
        }
        self.delta = after;
    }

    fn collide(&self, level: &Level<'_>, movement: Vec3) -> Result<Vec3, Unsupported> {
        let aabb = self.bounding_box;
        let movement_step = if movement.length_squared() == 0.0 {
            movement
        } else {
            let colliders =
                level.colliders(aabb.expand_towards_vec3(movement), self.fall_distance)?;
            collide_entity_movement(movement, aabb, &colliders)
        };
        let x_collision = movement.x != movement_step.x;
        let y_collision = movement.y != movement_step.y;
        let z_collision = movement.z != movement_step.z;
        let on_ground_after_collision = y_collision && movement.y < 0.0;
        let max_up_step = STEP_HEIGHT as f32;
        if (on_ground_after_collision || self.on_ground) && (x_collision || z_collision) {
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
            let colliders = level.colliders(step_up_area, self.fall_distance)?;
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

    fn maybe_back_off_from_edge(
        &self,
        level: &Level<'_>,
        delta: Vec3,
    ) -> Result<Vec3, Unsupported> {
        let max_down_step = STEP_HEIGHT as f32;
        if self.abilities.flying
            || delta.y > 0.0
            || !self.is_shift_key_down()
            || !self.is_above_ground(level, max_down_step)?
        {
            return Ok(delta);
        }
        let mut delta_x = delta.x;
        let mut delta_z = delta.z;
        let step = 0.05;
        let step_x = java_math::signum(delta_x) * step;
        let step_z = java_math::signum(delta_z) * step;
        let max_down = f64::from(max_down_step);

        while delta_x != 0.0 && self.can_fall_at_least(level, delta_x, 0.0, max_down)? {
            if delta_x.abs() <= step {
                delta_x = 0.0;
                break;
            }
            delta_x -= step_x;
        }
        while delta_z != 0.0 && self.can_fall_at_least(level, 0.0, delta_z, max_down)? {
            if delta_z.abs() <= step {
                delta_z = 0.0;
                break;
            }
            delta_z -= step_z;
        }
        while delta_x != 0.0
            && delta_z != 0.0
            && self.can_fall_at_least(level, delta_x, delta_z, max_down)?
        {
            if delta_x.abs() <= step {
                delta_x = 0.0;
            } else {
                delta_x -= step_x;
            }
            if delta_z.abs() <= step {
                delta_z = 0.0;
            } else {
                delta_z -= step_z;
            }
        }
        Ok(Vec3::new(delta_x, delta.y, delta_z))
    }

    fn is_above_ground(&self, level: &Level<'_>, max_down_step: f32) -> Result<bool, Unsupported> {
        Ok(self.on_ground
            || self.fall_distance < f64::from(max_down_step)
                && !self.can_fall_at_least(
                    level,
                    0.0,
                    0.0,
                    f64::from(max_down_step) - self.fall_distance,
                )?)
    }

    fn can_fall_at_least(
        &self,
        level: &Level<'_>,
        delta_x: f64,
        delta_z: f64,
        min_height: f64,
    ) -> Result<bool, Unsupported> {
        let bb = self.bounding_box;
        level.no_collision(
            Aabb::new(
                bb.min_x + 1.0e-7 + delta_x,
                bb.min_y - min_height - 1.0e-7,
                bb.min_z + 1.0e-7 + delta_z,
                bb.max_x - 1.0e-7 + delta_x,
                bb.min_y,
                bb.max_z - 1.0e-7 + delta_z,
            ),
            self.fall_distance,
        )
    }

    fn check_supporting_block(
        &mut self,
        level: &Level<'_>,
        on_ground: bool,
        movement: Option<Vec3>,
    ) -> Result<(), Unsupported> {
        level.check_supporting_block(
            (
                &mut self.main_supporting_block,
                &mut self.on_ground_no_blocks,
            ),
            (self.bounding_box, self.position, self.fall_distance),
            on_ground,
            movement,
        )
    }

    fn get_on_pos(&self, level: &Level<'_>, offset: f32) -> BlockPos {
        level.on_pos(self.main_supporting_block, self.position, offset)
    }

    fn block_position(&self) -> BlockPos {
        BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        )
    }

    fn block_speed_factor(&self, level: &Level<'_>) -> f32 {
        if self.abilities.flying {
            return 1.0;
        }
        level.block_speed_factor(self.block_position(), self.get_on_pos(level, 0.500_001))
    }

    fn block_jump_factor(&self, level: &Level<'_>) -> f32 {
        let here = level.info(self.block_position()).jump_factor;
        let below = level.info(self.get_on_pos(level, 0.500_001)).jump_factor;
        if here == 1.0 { below } else { here }
    }

    fn jump_from_ground(&mut self, level: &Level<'_>) {
        let jump_power = JUMP_STRENGTH * 1.0_f32 * self.block_jump_factor(level) + 0.0_f32;
        if jump_power <= 1.0e-5 {
            return;
        }
        self.delta = Vec3::new(
            self.delta.x,
            java_math::max(f64::from(jump_power), self.delta.y),
            self.delta.z,
        );
        if self.sprinting {
            let angle = self.y_rot * MTH_DEG_TO_RAD;
            self.delta = self.delta.add_vec3(Vec3::new(
                f64::from(-mth_sin_f64(f64::from(angle))) * 0.2,
                0.0,
                f64::from(mth_cos_f64(f64::from(angle))) * 0.2,
            ));
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    fn update_auto_jump(&mut self, level: &Level<'_>, xa: f32, za: f32) -> Result<(), Unsupported> {
        let can_auto_jump = self.auto_jump_enabled
            && self.auto_jump_time <= 0
            && self.on_ground
            && !self.is_shift_key_down()
            && self.input.move_vector().length_squared() > 0.0
            && self.block_jump_factor(level) >= 1.0;
        if !can_auto_jump {
            return Ok(());
        }
        let move_begin = self.position;
        let move_end = move_begin.add_components(f64::from(xa), 0.0, f64::from(za));
        let mut move_diff = Vec3::new(f64::from(xa), 0.0, f64::from(za));
        let current_speed = self.speed();
        let mut move_dist_sq = move_diff.length_squared() as f32;
        if move_dist_sq <= 0.001 {
            let move_vector = self.input.move_vector();
            let input_xa = current_speed * move_vector.x;
            let input_za = current_speed * move_vector.y;
            let sin = mth_sin_f64(f64::from(self.y_rot * MTH_DEG_TO_RAD));
            let cos = mth_cos_f64(f64::from(self.y_rot * MTH_DEG_TO_RAD));
            move_diff = Vec3::new(
                f64::from(input_xa * cos - input_za * sin),
                move_diff.y,
                f64::from(input_za * cos + input_xa * sin),
            );
            move_dist_sq = move_diff.length_squared() as f32;
            if move_dist_sq <= 0.001 {
                return Ok(());
            }
        }

        let move_dist_inverted = mth_inv_sqrt_f32(move_dist_sq);
        let move_dir = move_diff.scale(f64::from(move_dist_inverted));
        let facing = Vec3::direction_from_rotation_components(self.x_rot, self.y_rot);
        let facing_vs_moving = (facing.x * move_dir.x + facing.z * move_dir.z) as f32;
        if facing_vs_moving < -0.15 {
            return Ok(());
        }
        let mut ceiling_pos = BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.bounding_box.max_y),
            floor_f64(self.position.z),
        );
        if !level.collision_shape(ceiling_pos).is_empty() {
            return Ok(());
        }
        ceiling_pos = BlockPos::new(ceiling_pos.x, ceiling_pos.y + 1, ceiling_pos.z);
        if !level.collision_shape(ceiling_pos).is_empty() {
            return Ok(());
        }

        let jump_height = 1.2_f32;
        let look_ahead_dist = (current_speed * 7.0).max(1.0 / move_dist_inverted);
        let seg_begin = move_begin;
        let seg_end = move_end.add_vec3(move_dir.scale(f64::from(look_ahead_dist)));
        let player_width = self.dimensions.width();
        let player_height = self.dimensions.height();
        let test_box = Aabb::from_vec3(
            seg_begin,
            seg_end.add_components(0.0, f64::from(player_height), 0.0),
        )
        .inflate(f64::from(player_width), 0.0, f64::from(player_width));
        let seg_begin = seg_begin.add_components(0.0, f64::from(0.51_f32), 0.0);
        let seg_end = seg_end.add_components(0.0, f64::from(0.51_f32), 0.0);
        let right_dir = move_dir.cross(Vec3::new(0.0, 1.0, 0.0));
        let right_offset = right_dir.scale(f64::from(player_width * 0.5));
        let left_begin = seg_begin.subtract_vec3(right_offset);
        let left_end = seg_end.subtract_vec3(right_offset);
        let right_begin = seg_begin.add_vec3(right_offset);
        let right_end = seg_end.add_vec3(right_offset);

        let mut obstacle_height = f32::from_bits(1);
        'boxes: for shape in level.colliders(test_box, self.fall_distance)? {
            for bounds in shape.to_aabbs() {
                if bounds.intersects_vec3(left_begin, left_end)
                    || bounds.intersects_vec3(right_begin, right_end)
                {
                    obstacle_height = bounds.max_y as f32;
                    let center = bounds.center();
                    let obstacle_pos = BlockPos::new(
                        floor_f64(center.x),
                        floor_f64(center.y),
                        floor_f64(center.z),
                    );
                    let mut steps = 1;
                    while (steps as f32) < jump_height {
                        let above =
                            BlockPos::new(obstacle_pos.x, obstacle_pos.y + steps, obstacle_pos.z);
                        let shape = level.collision_shape(above);
                        if !shape.is_empty() {
                            obstacle_height = (shape.max(Axis::Y) + f64::from(above.y)) as f32;
                            if f64::from(obstacle_height) - self.position.y > f64::from(jump_height)
                            {
                                return Ok(());
                            }
                        }
                        if steps > 1 {
                            ceiling_pos =
                                BlockPos::new(ceiling_pos.x, ceiling_pos.y + 1, ceiling_pos.z);
                            if !level.collision_shape(ceiling_pos).is_empty() {
                                return Ok(());
                            }
                        }
                        steps += 1;
                    }
                    break 'boxes;
                }
            }
        }

        if obstacle_height != f32::from_bits(1) {
            let y_delta = (f64::from(obstacle_height) - self.position.y) as f32;
            if !(y_delta <= 0.5) && !(y_delta > jump_height) {
                self.auto_jump_time = 1;
            }
        }
        Ok(())
    }

    fn apply_effects_from_blocks(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let mut movements = std::mem::take(&mut self.movements_this_tick);
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
        if self.no_physics {
            return Ok(());
        }
        if self.on_ground {
            let effect_pos = self.get_on_pos(level, 0.2);
            if level.info(effect_pos).name == "minecraft:slime_block" {
                let abs_delta_y = self.delta.y.abs();
                if abs_delta_y < 0.1 && !self.is_shift_key_down() {
                    let scale = 0.4 + abs_delta_y * 0.2;
                    self.delta = self.delta.multiply_components(scale, 1.0, scale);
                }
            }
        }
        self.check_inside_blocks(level, &movements)
    }

    /// `Entity.checkInsideBlocks` for the player: the shared walk, with the blocks that act on a
    /// player acting as they are visited.
    fn check_inside_blocks(
        &mut self,
        level: &Level<'_>,
        movements: &[Movement],
    ) -> Result<(), Unsupported> {
        walk_inside_blocks(self, level, movements)
    }

    /// What a block the player is inside does to it. Honey, a cobweb, a berry bush, powder snow
    /// and a bubble column act; the other blocks have no effect on a player here.
    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        let InsideVisit {
            pos,
            state,
            ref info,
            precise,
        } = *visit;
        match info.name {
            "minecraft:honey_block" if self.is_sliding_down_honey(pos) => {
                self.do_honey_slide();
            }
            "minecraft:cobweb" => {
                self.make_stuck_in_block(Vec3::new(0.25, f64::from(0.05_f32), 0.25));
            }
            "minecraft:sweet_berry_bush" => {
                self.make_stuck_in_block(Vec3::new(f64::from(0.8_f32), 0.75, f64::from(0.8_f32)));
            }
            "minecraft:powder_snow" => {
                if level.info(self.block_position()).name == "minecraft:powder_snow" {
                    self.make_stuck_in_block(Vec3::new(
                        f64::from(0.9_f32),
                        1.5,
                        f64::from(0.9_f32),
                    ));
                }
            }
            "minecraft:bubble_column" if precise => {
                let drag_down = property(state, "drag") == Some("true");
                let above = pos.offset(0, 1, 0);
                let above_state = level.state_id(above);
                let nothing_above = above_state.is_none_or(|above_state| {
                    level
                        .world
                        .block_state_registry
                        .collision_shape(above_state)
                        .is_empty()
                        && crate::block::state_or_air(i64::from(above_state))
                            .fluid
                            .is_empty()
                });
                let delta = self.delta;
                let y = if nothing_above {
                    if drag_down {
                        (delta.y - 0.03).max(-0.9)
                    } else {
                        (delta.y + 0.1).min(1.8)
                    }
                } else {
                    self.fall_distance = 0.0;
                    if drag_down {
                        (delta.y - 0.03).max(-0.3)
                    } else {
                        (delta.y + 0.06).min(0.7)
                    }
                };
                self.delta = Vec3::new(delta.x, y, delta.z);
            }
            _ => {}
        }
    }

    /// `Entity.makeStuckInBlock`.
    fn make_stuck_in_block(&mut self, speed_multiplier: Vec3) {
        self.fall_distance = 0.0;
        self.stuck_speed_multiplier = speed_multiplier;
    }

    fn do_honey_slide(&mut self) {
        let old_delta_y = |delta_y: f64| delta_y / f64::from(0.98_f32) + 0.08;
        let new_delta_y = |delta_y: f64| (delta_y - 0.08) * f64::from(0.98_f32);
        let delta = self.delta;
        self.delta = if old_delta_y(delta.y) < -0.13 {
            let factor = -0.05 / old_delta_y(delta.y);
            Vec3::new(delta.x * factor, new_delta_y(-0.05), delta.z * factor)
        } else {
            Vec3::new(delta.x, new_delta_y(-0.05), delta.z)
        };
        self.fall_distance = 0.0;
    }

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

    fn update_fluid_interaction(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        let found = crate::fluid::scan_in(
            level,
            self.fluid_interaction_box(),
            (
                self.bounding_box.min_y,
                self.position,
                self.dimensions.eye_height(),
            ),
            self.abilities.flying,
        );
        let fluids = found.inspect_err(|_| {
            self.water = FluidTracker::default();
            self.lava = FluidTracker::default();
        })?;
        self.water = fluids.water;
        self.lava = fluids.lava;
        let in_water = self.water.height > 0.0;
        let in_lava = self.lava.height > 0.0;
        if in_water {
            self.fall_distance = 0.0;
        }
        self.was_touching_water = in_water;
        if !self.abilities.flying {
            if in_water {
                self.apply_current(self.water, WATER_FLOW_SCALE);
            }
            if in_lava {
                self.apply_current(self.lava, LAVA_FLOW_SCALE);
            }
        }
        Ok(())
    }

    /// `Entity.getFluidInteractionBox`: the box a thousandth smaller, which the vehicle the player
    /// sits in may change (`AbstractBoat.modifyPassengerFluidInteractionBox`: only what is above
    /// the boat's top, and none of it while the boat's top is above the player's own).
    fn fluid_interaction_box(&self) -> Option<Aabb> {
        let area = self.bounding_box.deflate(0.001);
        let Some(riding) = self.riding else {
            return Some(area);
        };
        if riding.vehicle_under_water {
            return Some(area);
        }
        if riding.vehicle_max_y >= area.max_y {
            return None;
        }
        let min_y = java_math::max(area.min_y, riding.vehicle_max_y);
        Some(Aabb::new(
            area.min_x, min_y, area.min_z, area.max_x, area.max_y, area.max_z,
        ))
    }

    fn apply_current(&mut self, tracker: FluidTracker, scale: f64) {
        self.delta =
            crate::fluid::apply_current(tracker, self.delta, scale, crate::fluid::Pushed::Player);
    }

    fn update_swimming(&mut self, level: &Level<'_>) {
        if self.abilities.flying {
            self.swimming = false;
        } else if self.is_swimming() {
            self.swimming = self.sprinting && self.is_in_water() && !self.is_passenger();
        } else {
            self.swimming = self.sprinting
                && self.is_under_water()
                && !self.is_passenger()
                && level.info(self.block_position()).is_water();
        }
    }

    fn update_player_pose(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if !self.can_fit(level, ClientEntityPose::Swimming)? {
            return Ok(());
        }
        let desired = if self.is_swimming() {
            ClientEntityPose::Swimming
        } else if self.is_shift_key_down() && !self.abilities.flying {
            ClientEntityPose::Crouching
        } else {
            ClientEntityPose::Standing
        };
        let actual =
            if self.is_spectator() || self.is_passenger() || self.can_fit(level, desired)? {
                desired
            } else if self.can_fit(level, ClientEntityPose::Crouching)? {
                ClientEntityPose::Crouching
            } else {
                ClientEntityPose::Swimming
            };
        self.set_pose(actual);
        Ok(())
    }

    fn can_fit(&self, level: &Level<'_>, pose: ClientEntityPose) -> Result<bool, Unsupported> {
        level.no_collision(
            pose.player_dimensions()
                .make_bounding_box(self.position)
                .deflate(1.0e-7),
            self.fall_distance,
        )
    }

    fn set_pose(&mut self, pose: ClientEntityPose) {
        if pose != self.pose {
            self.pose = pose;
            self.dimensions = pose.player_dimensions();
            self.bounding_box = self.dimensions.make_bounding_box(self.position);
        }
    }

    fn move_towards_closest_space(&mut self, level: &Level<'_>, x: f64, z: f64) {
        let pos = BlockPos::new(floor_f64(x), floor_f64(self.position.y), floor_f64(z));
        if !self.suffocates_at(level, pos) {
            return;
        }
        let xd = x - f64::from(pos.x);
        let zd = z - f64::from(pos.z);
        let mut chosen: Option<(i32, i32)> = None;
        let mut closest = f64::MAX;
        // West, east, north, south.
        for (step_x, step_z) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let axis_distance = if step_x != 0 { xd } else { zd };
            let distance_to_edge = if step_x + step_z > 0 {
                1.0 - axis_distance
            } else {
                axis_distance
            };
            let neighbour = BlockPos::new(pos.x + step_x, pos.y, pos.z + step_z);
            if distance_to_edge < closest && !self.suffocates_at(level, neighbour) {
                closest = distance_to_edge;
                chosen = Some((step_x, step_z));
            }
        }
        if let Some((step_x, step_z)) = chosen {
            let old = self.delta;
            self.delta = if step_x != 0 {
                Vec3::new(0.1 * f64::from(step_x), old.y, old.z)
            } else {
                Vec3::new(old.x, old.y, 0.1 * f64::from(step_z))
            };
        }
    }

    fn suffocates_at(&self, level: &Level<'_>, pos: BlockPos) -> bool {
        let bb = self.bounding_box;
        let area = Aabb::new(
            f64::from(pos.x),
            bb.min_y,
            f64::from(pos.z),
            f64::from(pos.x) + 1.0,
            bb.max_y,
            f64::from(pos.z) + 1.0,
        )
        .deflate(1.0e-7);
        level.collides_with_suffocating_block(area)
    }

    fn send_position(&mut self) {
        if self.sprinting != self.was_sprinting {
            let action = if self.sprinting {
                PlayerCommandAction::StartSprinting
            } else {
                PlayerCommandAction::StopSprinting
            };
            self.sent.push(ServerboundMovement::PlayerCommand {
                entity_id: self.entity_id,
                action,
            });
            self.was_sprinting = self.sprinting;
        }

        let delta_x = self.position.x - self.x_last;
        let delta_y = self.position.y - self.y_last;
        let delta_z = self.position.z - self.z_last;
        let delta_y_rot = f64::from(self.y_rot - self.y_rot_last);
        let delta_x_rot = f64::from(self.x_rot - self.x_rot_last);
        self.position_reminder += 1;
        let moved = delta_x * delta_x + delta_y * delta_y + delta_z * delta_z > 2.0e-4 * 2.0e-4
            || self.position_reminder >= 20;
        let rotated = delta_y_rot != 0.0 || delta_x_rot != 0.0;
        let on_ground = self.on_ground;
        let horizontal_collision = self.horizontal_collision;
        if moved && rotated {
            self.sent.push(ServerboundMovement::MovePlayerPosRot {
                position: self.position,
                y_rot: self.y_rot,
                x_rot: self.x_rot,
                on_ground,
                horizontal_collision,
            });
        } else if moved {
            self.sent.push(ServerboundMovement::MovePlayerPos {
                position: self.position,
                on_ground,
                horizontal_collision,
            });
        } else if rotated {
            self.sent.push(ServerboundMovement::MovePlayerRot {
                y_rot: self.y_rot,
                x_rot: self.x_rot,
                on_ground,
                horizontal_collision,
            });
        } else if self.last_on_ground != on_ground
            || self.last_horizontal_collision != horizontal_collision
        {
            self.sent.push(ServerboundMovement::MovePlayerStatusOnly {
                on_ground,
                horizontal_collision,
            });
        }
        if moved {
            self.x_last = self.position.x;
            self.y_last = self.position.y;
            self.z_last = self.position.z;
            self.position_reminder = 0;
        }
        if rotated {
            self.y_rot_last = self.y_rot;
            self.x_rot_last = self.x_rot;
        }
        self.last_on_ground = on_ground;
        self.last_horizontal_collision = horizontal_collision;
        self.auto_jump_enabled = self.auto_jump_option;
    }

    fn on_update_abilities(&mut self) {
        self.sent.push(ServerboundMovement::PlayerAbilities {
            flying: self.abilities.flying,
        });
    }

    fn set_pos(&mut self, position: Vec3) {
        self.position = position;
        self.bounding_box = self.dimensions.make_bounding_box(position);
    }

    fn set_sprinting(&mut self, sprinting: bool) {
        self.sprinting = sprinting;
    }

    /// `getAttributeValue(MOVEMENT_SPEED)`, narrowed as `Player.getSpeed` does.
    fn speed(&self) -> f32 {
        let base = f64::from(MOVEMENT_SPEED_BASE);
        let value = if self.sprinting {
            base * (1.0 + f64::from(SPRINTING_MODIFIER))
        } else {
            base
        };
        value.clamp(0.0, 1024.0) as f32
    }

    fn look_angle(&self) -> Vec3 {
        let real_x_rot = self.x_rot * MTH_DEG_TO_RAD;
        let real_y_rot = -self.y_rot * MTH_DEG_TO_RAD;
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

    fn is_spectator(&self) -> bool {
        self.game_mode.is_spectator()
    }

    fn is_in_water(&self) -> bool {
        self.was_touching_water
    }

    fn is_in_lava(&self) -> bool {
        !self.first_tick && self.lava.height > 0.0
    }

    fn is_under_water(&self) -> bool {
        self.was_eye_in_water && self.is_in_water()
    }

    fn is_affected_by_fluids(&self) -> bool {
        !self.abilities.flying
    }

    /// `Player.isSwimming`: the flag, unless flying or spectating.
    fn is_swimming(&self) -> bool {
        !self.abilities.flying && !self.is_spectator() && self.swimming
    }

    fn is_moving_slowly(&self) -> bool {
        self.crouching || self.pose == ClientEntityPose::Swimming && !self.is_in_water()
    }

    fn fluid_jump_threshold(&self) -> f64 {
        if self.dimensions.eye_height() < 0.4 {
            0.0
        } else {
            0.4
        }
    }

    fn is_sprinting_possible(&self, allowed_in_shallow_water: bool) -> bool {
        // A passenger sprints only in a vehicle that can be (and it is its authority).
        if self.riding.is_some_and(|riding| !riding.can_sprint) {
            return false;
        }
        // No blindness, and a full food bar or the ability to fly.
        allowed_in_shallow_water || !(self.is_in_water() && !self.is_under_water())
    }

    fn has_forward_impulse(&self) -> bool {
        self.input.move_vector().y > 1.0e-5
    }

    fn can_start_sprinting(&self) -> bool {
        !self.sprinting
            && self.has_forward_impulse()
            && self.is_sprinting_possible(self.abilities.flying)
            && (!self.is_moving_slowly() || self.is_under_water())
    }

    fn should_stop_run_sprinting(&self) -> bool {
        !self.is_sprinting_possible(self.abilities.flying)
            || !self.has_forward_impulse()
            || self.horizontal_collision && !self.minor_horizontal_collision
    }

    fn should_stop_swim_sprinting(&self) -> bool {
        !self.is_sprinting_possible(true)
            || !self.is_in_water()
            || !self.has_forward_impulse() && !self.on_ground && !self.input.key_presses.shift
    }
}

/// The fluid a block state holds: whether it is water (rather than lava),
/// its own height, and whether it is falling. Nothing for a dry state.
pub fn state_fluid(state_id: i32) -> Option<(bool, f32, bool)> {
    let fluid = crate::block::state_or_air(i64::from(state_id)).fluid;
    (!fluid.is_empty()).then(|| {
        (
            fluid.is_same(Fluid::Water),
            fluid.own_height(),
            fluid.falling,
        )
    })
}

/// Whether a block state holds any fluid, as a section's fluid count sees it.
pub fn state_has_fluid(state_id: i32) -> bool {
    !crate::block::state_or_air(i64::from(state_id))
        .fluid
        .is_empty()
}

/// A block the walk reached, and what its visit needs to act on.
pub(crate) struct InsideVisit {
    pub(crate) pos: BlockPos,
    pub(crate) state: Option<i32>,
    pub(crate) info: StateInfo,
    /// `isPrecise`: the entity's box really overlaps the block, or it moved a long way.
    pub(crate) precise: bool,
}

/// A body whose movement is walked for the blocks it passes through (`Entity.checkInsideBlocks`).
pub(crate) trait InsideBody {
    /// The body's box with its feet at a position.
    fn box_at(&self, position: Vec3) -> Aabb;
    /// The collision shape powder snow has for this body, which depends on how far it has fallen.
    fn powder_snow_shape(&self) -> VoxelShape;
    /// What the block does to the body.
    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit);
}

impl InsideBody for LocalPlayer {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> VoxelShape {
        powder_snow_collision_shape(self.fall_distance)
    }

    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        LocalPlayer::inside_block_effect(self, level, visit);
    }
}

/// `Entity.checkInsideBlocks`: walk the blocks each movement passed through, in the reference's
/// per-axis split, and let the ones that act on a body inside them act.
pub(crate) fn walk_inside_blocks<B: InsideBody>(
    body: &mut B,
    level: &Level<'_>,
    movements: &[Movement],
) -> Result<(), Unsupported> {
    let mut visited = std::collections::HashSet::new();
    for movement in movements {
        let mut pos = movement.from;
        let delta = movement.to.subtract_vec3(movement.from);
        let mut max_iterations = 16;
        if let Some(original) = movement.original.filter(|_| delta.length_squared() > 0.0) {
            for axis in voxel_core::direction::Direction::axis_step_order(original.x, original.z) {
                let axis_move = delta.get(axis);
                if axis_move != 0.0 {
                    let to = pos.with(axis, pos.get(axis) + axis_move);
                    max_iterations -=
                        walk_inside_between(body, level, (pos, to), &mut visited, max_iterations);
                    pos = to;
                }
            }
        } else {
            max_iterations -=
                walk_inside_between(body, level, (movement.from, movement.to), &mut visited, 16);
        }
        if max_iterations <= 0 {
            walk_inside_between(body, level, (movement.to, movement.to), &mut visited, 1);
        }
    }
    Ok(())
}

fn walk_inside_between<B: InsideBody>(
    body: &mut B,
    level: &Level<'_>,
    (from, to): (Vec3, Vec3),
    visited: &mut std::collections::HashSet<(i32, i32, i32)>,
    max_iterations: i32,
) -> i32 {
    let at_target = body.box_at(to).deflate(f64::from(1.0e-5_f32));
    let moved_far = from.distance_to_sqr(to) > 0.999_990_000_000_252_6_f64.powi(2);
    let mut last_iteration = 0;
    for (pos, iteration) in blocks_intersected_between(from, to, at_target) {
        if iteration >= max_iterations {
            break;
        }
        last_iteration = iteration;
        let state = level.state_id(pos);
        let info = level.info(pos);
        if info.is_air() {
            continue;
        }
        // `getEntityInsideCollisionShape`: the whole block, except powder snow, whose own
        // collision shape counts when it has one.
        let inside_block = match state {
            Some(state) if is_block(state, "minecraft:powder_snow") => {
                let shape = body.powder_snow_shape();
                shape.is_empty() || {
                    let boxes: Vec<Aabb> = shape
                        .to_aabbs()
                        .into_iter()
                        .map(|aabb| aabb.move_by_block_pos(pos))
                        .collect();
                    body.box_at(from)
                        .collided_along_vector(to.subtract_vec3(from), &boxes)
                }
            }
            _ => true,
        };
        // Only a block the body is not fully inside can miss, and the one such block, powder
        // snow, holds no fluid whose effects (fire) would still need the visit.
        if !inside_block || !visited.insert((pos.x, pos.y, pos.z)) {
            continue;
        }
        let precise = moved_far || at_target.intersects_block_pos(pos);
        body.inside_block_effect(
            level,
            &InsideVisit {
                pos,
                state,
                info,
                precise,
            },
        );
    }
    last_iteration + 1
}

/// `BlockGetter.forEachBlockIntersectedBetween`, as the blocks it visits and
/// the iteration each is visited in. The order within one iteration is not
/// kept: only honey acts here, and it acts at most once a tick.
#[expect(
    clippy::too_many_lines,
    reason = "kept whole to read against the reference method line by line"
)]
fn blocks_intersected_between(from: Vec3, to: Vec3, at_target: Aabb) -> Vec<(BlockPos, i32)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    // `BlockPos.betweenClosed` (x fastest, then y, then z) without a
    // direction, `betweenCornersInDirection` with one: from the corner the
    // travel leaves, the axes in step order, the last one fastest.
    let mut add_box = |out: &mut Vec<(BlockPos, i32)>,
                       first: [i32; 3],
                       second: [i32; 3],
                       iteration: i32,
                       direction: Option<Vec3>| {
        let min = [0, 1, 2].map(|axis| first[axis].min(second[axis]));
        let max = [0, 1, 2].map(|axis| first[axis].max(second[axis]));
        let (order, forward) = match direction {
            None => ([2, 1, 0], [true; 3]),
            Some(direction) => {
                let order =
                    voxel_core::direction::Direction::axis_step_order(direction.x, direction.z)
                        .map(|axis| match axis {
                            voxel_core::direction::Axis::X => 0,
                            voxel_core::direction::Axis::Y => 1,
                            voxel_core::direction::Axis::Z => 2,
                        });
                (
                    order,
                    [direction.x >= 0.0, direction.y >= 0.0, direction.z >= 0.0],
                )
            }
        };
        let step = |axis: usize, index: i32| {
            if forward[axis] {
                min[axis] + index
            } else {
                max[axis] - index
            }
        };
        for a in 0..=max[order[0]] - min[order[0]] {
            for b in 0..=max[order[1]] - min[order[1]] {
                for c in 0..=max[order[2]] - min[order[2]] {
                    let mut pos = [0; 3];
                    pos[order[0]] = step(order[0], a);
                    pos[order[1]] = step(order[1], b);
                    pos[order[2]] = step(order[2], c);
                    if seen.insert((pos[0], pos[1], pos[2])) {
                        out.push((BlockPos::new(pos[0], pos[1], pos[2]), iteration));
                    }
                }
            }
        }
    };
    let corners = |aabb: Aabb| {
        (
            [
                floor_f64(aabb.min_x),
                floor_f64(aabb.min_y),
                floor_f64(aabb.min_z),
            ],
            [
                floor_f64(aabb.max_x),
                floor_f64(aabb.max_y),
                floor_f64(aabb.max_z),
            ],
        )
    };
    let travel = to.subtract_vec3(from);
    if travel.length_squared() < f64::from(1.0e-5_f32 * 1.0e-5_f32) {
        let (first, second) = corners(at_target);
        add_box(&mut out, first, second, 0, None);
        return out;
    }
    let (first, second) = corners(at_target.move_by(-travel.x, -travel.y, -travel.z));
    add_box(&mut out, first, second, 0, Some(travel));

    // addCollisionsAlongTravel.
    let size = [at_target.x_size(), at_target.y_size(), at_target.z_size()];
    let corner_dir = furthest_corner(travel);
    let centre = at_target.center();
    let to_corner = Vec3::new(
        centre.x + size[0] * 0.5 * f64::from(corner_dir[0]),
        centre.y + size[1] * 0.5 * f64::from(corner_dir[1]),
        centre.z + size[2] * 0.5 * f64::from(corner_dir[2]),
    );
    let from_corner = to_corner.subtract_vec3(travel);
    let mut block = [
        floor_f64(from_corner.x),
        floor_f64(from_corner.y),
        floor_f64(from_corner.z),
    ];
    let sign = |value: f64| {
        if value > 0.0 {
            1
        } else if value < 0.0 {
            -1
        } else {
            0
        }
    };
    let frac = |value: f64| value - voxel_core::math::lfloor_f64(value) as f64;
    let signs = [sign(travel.x), sign(travel.y), sign(travel.z)];
    let t_delta = [travel.x, travel.y, travel.z]
        .iter()
        .zip(signs)
        .map(|(value, sign)| {
            if sign == 0 {
                f64::MAX
            } else {
                f64::from(sign) / value
            }
        })
        .collect::<Vec<_>>();
    let starts = [from_corner.x, from_corner.y, from_corner.z];
    let mut t = [0.0_f64; 3];
    for axis in 0..3 {
        t[axis] = t_delta[axis]
            * if signs[axis] > 0 {
                1.0 - frac(starts[axis])
            } else {
                frac(starts[axis])
            };
    }
    let mut iterations = 0;
    while t[0] <= 1.0 || t[1] <= 1.0 || t[2] <= 1.0 {
        let axis = if t[0] < t[1] {
            if t[0] < t[2] { 0 } else { 2 }
        } else if t[1] < t[2] {
            1
        } else {
            2
        };
        block[axis] += signs[axis];
        t[axis] += t_delta[axis];
        let hit = Aabb::clip_coords(
            Vec3::new(
                f64::from(block[0]),
                f64::from(block[1]),
                f64::from(block[2]),
            ),
            Vec3::new(
                f64::from(block[0] + 1),
                f64::from(block[1] + 1),
                f64::from(block[2] + 1),
            ),
            from_corner,
            to_corner,
        );
        if let Some(hit) = hit {
            iterations += 1;
            let hit_coords = [hit.x, hit.y, hit.z];
            let mut opposite = [0; 3];
            for axis in 0..3 {
                let low = f64::from(block[axis] as f32 + 1.0e-5_f32);
                let high = f64::from(block[axis]) + 1.0 - f64::from(1.0e-5_f32);
                let clamped = java_math::clamp(hit_coords[axis], low, high);
                opposite[axis] = floor_f64(clamped - size[axis] * f64::from(corner_dir[axis]));
            }
            add_box(&mut out, block, opposite, iterations, Some(travel));
        }
    }
    let (first, second) = corners(at_target);
    add_box(&mut out, first, second, iterations + 1, Some(travel));
    out
}

/// `BlockGetter.getFurthestCorner`.
fn furthest_corner(direction: Vec3) -> [i32; 3] {
    let x_dot = direction.x.abs();
    let y_dot = direction.y.abs();
    let z_dot = direction.z.abs();
    let x_sign = if direction.x >= 0.0 { 1 } else { -1 };
    let y_sign = if direction.y >= 0.0 { 1 } else { -1 };
    let z_sign = if direction.z >= 0.0 { 1 } else { -1 };
    if x_dot <= y_dot && x_dot <= z_dot {
        [-x_sign, -z_sign, y_sign]
    } else if y_dot <= z_dot {
        [z_sign, -y_sign, -x_sign]
    } else {
        [-y_sign, x_sign, -z_sign]
    }
}

fn compute_modified_friction(friction: f32, modifier: f32) -> f32 {
    (1.0_f32 - (1.0_f32 - friction) * modifier).clamp(0.0, 1.0)
}

/// `SwingAnimation.DEFAULT`'s duration: an empty hand's swing, in ticks.
const SWING_DURATION: i32 = 6;

/// What the player model is posed from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPose {
    pub body_rot: f32,
    pub head_rot: f32,
    pub x_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
    pub swim_amount: f32,
    pub crouching: bool,
    pub pose: ClientEntityPose,
}

/// `Mth.rotLerp`: the shorter way round between two angles.
fn rot_lerp(partial_tick: f32, from: f32, to: f32) -> f32 {
    from + partial_tick * voxel_core::math::wrap_degrees_f32(to - from)
}

fn is_block(state_id: i32, name: &str) -> bool {
    registry::block_for_state(state_id as u32).is_some_and(|(block, _)| block.name == name)
}

/// `PowderSnowBlock.getCollisionShape` for a player without leather boots:
/// nothing until it has fallen more than 2.5 blocks, then a floor 0.9 high.
pub(crate) fn powder_snow_collision_shape(fall_distance: f64) -> VoxelShape {
    if fall_distance > 2.5 {
        Shapes::box_shape(0.0, 0.0, 0.0, 1.0, f64::from(0.9_f32), 1.0)
            .expect("the falling powder snow shape is a valid box")
    } else {
        Shapes::empty()
    }
}

pub(crate) fn property(state_id: Option<i32>, name: &str) -> Option<&'static str> {
    let (block, offset) = registry::block_for_state(state_id? as u32)?;
    block
        .properties_at(offset)?
        .into_iter()
        .find(|(property, _)| *property == name)
        .map(|(_, value)| value)
}
