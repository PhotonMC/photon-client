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
    StartRidingJump = 3,
}

/// A packet the tick asks the connection to send.
#[derive(Debug, Clone, Copy)]
pub enum ServerboundMovement {
    PlayerInput(Input),
    PlayerCommand {
        entity_id: i32,
        action: PlayerCommandAction,
        /// The action's datum: a riding jump's charge, in hundredths.
        data: i32,
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
            Self::PlayerCommand {
                entity_id,
                action,
                data,
            } => {
                var_int(&mut output, entity_id);
                var_int(&mut output, action as i32);
                var_int(&mut output, data);
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
    /// Whether the vehicle changes the box the player's fluids are found in (a boat does).
    modifies_fluid_box: bool,
    can_sprint: bool,
    /// Whether the jump key charges a jump of the vehicle (`PlayerRideableJumping.canJump`).
    can_jump: bool,
}

/// What one tick of the player sitting in a boat gives the session: the packets to send, and the
/// keys to hand the boat (`AbstractBoat.setInput`).
#[derive(Debug, Clone)]
pub struct RideTick {
    pub sent: Vec<ServerboundMovement>,
    pub boat_input: crate::entity::BoatInput,
    /// The jump the rider let go of, as `PlayerRideableJumping.onPlayerJump` is told it
    /// (the charge, in hundredths).
    pub jump: Option<i32>,
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
    pub(crate) jump_factor: f32,
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

    pub(crate) fn is_lava(&self) -> bool {
        matches!(self.fluid, Fluid::Lava | Fluid::FlowingLava)
    }
}

/// The level as the tick reads it.
#[derive(Clone, Copy)]
pub(crate) struct Level<'a> {
    pub(crate) world: &'a ClientWorld,
    /// The height of the feet of an entity that can stand on lava (`EntityCollisionContext` of a
    /// strider): the lava's source blocks below it then hold it up.
    pub(crate) lava_stand: Option<f64>,
}

impl<'a> Level<'a> {
    pub(crate) fn new(world: &'a ClientWorld) -> Self {
        Self {
            world,
            lava_stand: None,
        }
    }

    /// The level as seen by an entity that can stand on lava, with its feet at `bottom`.
    pub(crate) fn standing_on_lava_at(self, bottom: f64) -> Self {
        Self {
            lava_stand: Some(bottom),
            ..self
        }
    }
}

impl Level<'_> {
    pub(crate) fn state_id(&self, pos: BlockPos) -> Option<i32> {
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

    /// `LiquidBlock.getCollisionShape` for an entity that can stand on lava: a source block of lava
    /// has the half-high column of `Strider.getLiquidCollisionShape` where the entity's feet are
    /// above it and no lava lies on top of it.
    fn lava_surface_shape(&self, state: i32, at: [i32; 3], bottom: f64) -> Option<VoxelShape> {
        let info = StateInfo::of(Some(state), self.world.block_state_registry.tags());
        if info.fluid != Fluid::Lava {
            return None;
        }
        let above = self.info(BlockPos::new(at[0], at[1] + 1, at[2]));
        if above.is_lava() || bottom <= f64::from(at[1]) + 0.5 - f64::from(1.0e-5_f32) {
            return None;
        }
        Shapes::box_shape(0.0, 0.0, 0.0, 1.0, 0.5, 1.0).ok()
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
            } else if let Some(bottom) = self.lava_stand
                && let Some(shape) = self.lava_surface_shape(state, at, bottom)
            {
                shape
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

    /// `Entity.collectColliders`: the world border's shape if `source` stands close inside it,
    /// then the blocks'.
    pub(crate) fn colliders(
        &self,
        area: Aabb,
        fall_distance: f64,
        source: Vec3,
    ) -> Result<Vec<VoxelShape>, Unsupported> {
        let mut out: Vec<VoxelShape> = self
            .world
            .border_collider(source, area)
            .into_iter()
            .collect();
        out.extend(
            self.block_collisions(area, fall_distance)?
                .into_iter()
                .map(|(_, shape)| shape),
        );
        Ok(out)
    }

    /// `CollisionGetter.noCollision(source, area)`: no block, and not the world border.
    pub(crate) fn no_collision(
        &self,
        area: Aabb,
        fall_distance: f64,
        source: Vec3,
    ) -> Result<bool, Unsupported> {
        Ok(self.block_collisions(area, fall_distance)?.is_empty()
            && self.world.no_border_collision(
                Some(crate::collision::ClientWorldCollisionSource { position: source }),
                area,
            ))
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
    /// `LocalPlayer.jumpRidingTicks` and `jumpRidingScale`: how long the jump key has charged a
    /// mount's jump, and how hard that is; the jump let go of this tick, for the mount.
    jump_riding_ticks: i32,
    jump_riding_scale: f32,
    ride_jump: Option<i32>,
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
    /// `swingingArm`: the hand that swung last (the main hand until one swings).
    swinging_off_hand: bool,
    attack_anim: f32,
    attack_anim_old: f32,
    /// `Player.attackStrengthTicker` and `itemSwapTicker`: ticks since the
    /// last attack or change of held item.
    attack_strength_ticker: i32,
    /// The main hand item's registry id (air when empty), and the one the
    /// last tick saw.
    main_hand_item: i32,
    last_main_hand_item: i32,
    /// What the hands hold, the effects and the use of an item, as the world last heard them.
    hands: HandState,
    /// How many ticks the item in use has been used (`getTicksUsingItem`).
    using_item_ticks: i32,
    item_swap_ticker: i32,
    /// `LivingEntity`'s health as the client last set it, and the hurt
    /// state `LocalPlayer.hurtTo` keeps: ticks of damage immunity, of the
    /// hurt flash, and whether the next health update may flash at all.
    health: f32,
    flash_on_set_health: bool,
    invulnerable_time: i32,
    hurt_time: i32,
    /// `hurtDuration`: what the flinch started from.
    hurt_duration: i32,
    /// `Player.hurtDir`: where the last blow came from, in degrees.
    hurt_dir: f32,
    /// `deathTime`: the ticks since the health ran out.
    death_time: i32,
    /// How far into the swimming pose the model has turned.
    swim_amount: f32,
    swim_amount_old: f32,
    bob: f32,
    bob_old: f32,
    first_tick: bool,
    tick_count: i32,
    /// `Entity.random`: the pitch of the player's sounds is drawn from it. The reference seeds
    /// it from nowhere in particular; this one from the entity's number.
    random: voxel_core::random::RandomSource,
    /// `BubbleColumnAmbientSoundHandler`: whether the last tick was in a bubble column, and
    /// whether it is the first.
    was_in_bubble_column: bool,
    ambient_first_tick: bool,
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

/// A living body as the blocks it is inside act on it.
pub(crate) struct LivingEffects<'a> {
    pub(crate) delta: &'a mut Vec3,
    pub(crate) fall_distance: &'a mut f64,
    pub(crate) stuck: &'a mut Vec3,
    pub(crate) position: Vec3,
    pub(crate) on_ground: bool,
    pub(crate) width: f32,
    /// The block the body's feet are in.
    pub(crate) block_pos: BlockPos,
}

/// What a block a living entity is inside does to it: honey slides it, a cobweb or a berry bush
/// holds it, powder snow holds it only when its own block is powder snow, and a bubble column
/// carries it. The other blocks have no effect here.
pub(crate) fn living_inside_effect(
    level: &Level<'_>,
    visit: &InsideVisit,
    body: LivingEffects<'_>,
) {
    let InsideVisit {
        pos,
        state,
        ref info,
        precise,
    } = *visit;
    let make_stuck = |body: &mut LivingEffects<'_>, speed_multiplier: Vec3| {
        *body.fall_distance = 0.0;
        *body.stuck = speed_multiplier;
    };
    let mut body = body;
    match info.name {
        "minecraft:honey_block" if is_sliding_down_honey(&body, pos) => {
            let old_delta_y = |delta_y: f64| delta_y / f64::from(0.98_f32) + 0.08;
            let new_delta_y = |delta_y: f64| (delta_y - 0.08) * f64::from(0.98_f32);
            let delta = *body.delta;
            *body.delta = if old_delta_y(delta.y) < -0.13 {
                let factor = -0.05 / old_delta_y(delta.y);
                Vec3::new(delta.x * factor, new_delta_y(-0.05), delta.z * factor)
            } else {
                Vec3::new(delta.x, new_delta_y(-0.05), delta.z)
            };
            *body.fall_distance = 0.0;
        }
        "minecraft:cobweb" => {
            make_stuck(&mut body, Vec3::new(0.25, f64::from(0.05_f32), 0.25));
        }
        "minecraft:sweet_berry_bush" => {
            make_stuck(
                &mut body,
                Vec3::new(f64::from(0.8_f32), 0.75, f64::from(0.8_f32)),
            );
        }
        "minecraft:powder_snow" => {
            if level.info(body.block_pos).name == "minecraft:powder_snow" {
                make_stuck(
                    &mut body,
                    Vec3::new(f64::from(0.9_f32), 1.5, f64::from(0.9_f32)),
                );
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
            let delta = *body.delta;
            let y = if nothing_above {
                if drag_down {
                    (delta.y - 0.03).max(-0.9)
                } else {
                    (delta.y + 0.1).min(1.8)
                }
            } else {
                *body.fall_distance = 0.0;
                if drag_down {
                    (delta.y - 0.03).max(-0.3)
                } else {
                    (delta.y + 0.06).min(0.7)
                }
            };
            *body.delta = Vec3::new(delta.x, y, delta.z);
        }
        _ => {}
    }
}

/// `HoneyBlock.isSlidingDown`.
fn is_sliding_down_honey(body: &LivingEffects<'_>, pos: BlockPos) -> bool {
    if body.on_ground || body.position.y > f64::from(pos.y) + 0.9375 - 1.0e-7 {
        return false;
    }
    if body.delta.y / f64::from(0.98_f32) + 0.08 >= -0.08 {
        return false;
    }
    let dx = (f64::from(pos.x) + 0.5 - body.position.x).abs();
    let dz = (f64::from(pos.z) + 0.5 - body.position.z).abs();
    let overlap = 0.4375 + f64::from(body.width / 2.0);
    dx + 1.0e-7 > overlap || dz + 1.0e-7 > overlap
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

pub(crate) fn compute_modified_friction(friction: f32, modifier: f32) -> f32 {
    (1.0_f32 - (1.0_f32 - friction) * modifier).clamp(0.0, 1.0)
}

/// `SwingAnimation.DEFAULT`'s duration: an empty hand's swing, in ticks.
/// How long a swing lasts with nothing special in the hand (`SwingAnimation.DEFAULT`).
const SWING_DURATION: i32 = 6;

/// What the local player's swing and use of items read of the world's player state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandState {
    /// The registry ids of what the main and off hands hold (air when empty).
    pub main_item: i32,
    pub off_item: i32,
    /// The greater of the haste and conduit power amplifiers, if either is on.
    pub dig_speed: Option<i32>,
    pub mining_fatigue: Option<i32>,
    /// The server's flags: an item is in use, and in the off hand.
    pub using_item: bool,
    pub using_off_hand: bool,
}

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

mod core;
mod effects;
mod movement;
