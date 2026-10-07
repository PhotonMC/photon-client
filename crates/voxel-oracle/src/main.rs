//! JSONL test glue. Behavior is delegated to the production library.

#![expect(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    reason = "the adapter answers every behaviour family in one dispatch"
)]
// `cargo build --all-targets` also builds the binary with the test harness,
// which has no `main`, so everything `main` reaches looks unused there.
#![cfg_attr(test, allow(dead_code))]

mod gui;

use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use std::rc::Rc;
use voxel_core::aabb::{Aabb, AabbBuilder, AabbBuilderError, BlockHitResult, BoundingBox};
use voxel_core::axis_cycle::AxisCycle;
use voxel_core::binary::{
    self, BitSet, BlockPos, BlockRotation, ChunkPos, MutableBlockPos, SectionPos, Uuid,
};
use voxel_core::direction::{
    Axis, AxisDirection, Direction, DirectionYRotError, DirectionYRotationError, Plane,
};
use voxel_core::identifier::Identifier;
use voxel_core::math::{self, Quaternionf, Vec2, Vec3, Vec3f};
use voxel_core::nbt::{self, Field, FieldValue};
use voxel_core::random::RandomSource;
use voxel_core::shape::{BooleanOp, ShapeRotation, Shapes, VoxelShape};
use voxel_core::smooth::SmoothDouble;
use voxel_core::tags::{ClientboundUpdateTagsPacket, RegistryTags, TagEntry};
use voxel_core::timer::Timer;
use voxel_core::utf8;
use voxel_core::wire::{self, ReadError};
use voxel_game::input::event::{
    CharacterEvent, InputWithModifiers, KeyEvent as InputKeyEvent, MouseButtonEvent,
    MouseButtonInfo, PreeditEvent,
};
use voxel_game::input::key_mapping::{InputKey, KeyMapping, KeyMappingRegistry};
use voxel_game::input::scroll::ScrollWheelHandler;
use voxel_game::options::values::{
    AltEnum, ClampingLazyMaxIntRange, CycleableEnum, IntRange, LazyEnum, SliderableEnum, UnitDouble,
};
use voxel_net::common::{
    ClientboundKeepAlivePacket, ClientboundPingPacket, ServerboundKeepAlivePacket,
    ServerboundPongPacket,
};
use voxel_net::configuration::{
    ChatVisibility, ClientInformation, ClientboundCodeOfConductPacket,
    ClientboundFinishConfigurationPacket, ClientboundResetChatPacket,
    ClientboundUpdateEnabledFeaturesPacket, MainHand, ParticleStatus,
    ServerboundAcceptCodeOfConductPacket, ServerboundClientInformationPacket,
    ServerboundFinishConfigurationPacket,
};
use voxel_net::connection::{
    decode_chunk_section_data, parse_game_event_packet, parse_player_abilities_packet,
};
use voxel_net::frame::{self, FrameError, Varint21FrameDecoder};
use voxel_net::handshake::{self, ClientIntent, ClientIntentionPacket};
use voxel_net::login::{
    ClientboundHelloPacket, ClientboundLoginCompressionPacket, ClientboundLoginFinishedPacket,
    GameProfile, ProfileProperty, ServerboundHelloPacket, ServerboundLoginAcknowledgedPacket,
};
use voxel_net::protocol::{ConnectionProtocol, PacketFlow};
use voxel_net::registry_data::{ClientboundRegistryDataPacket, PackedRegistryEntry};
use voxel_net::resources::{
    self, AssetIndex, AssetResolution, KnownPack, KnownPackCodecError, KnownPackListCodecError,
    PackCompatibility, PackFormat, ReloadableResourceManager, ResourcePack, ResourcePackStack,
    VanillaPackResources,
};
use voxel_net::status::{
    ClientboundPongResponsePacket, ServerboundPingRequestPacket, ServerboundStatusRequestPacket,
};
use voxel_render::window;
use voxel_world::input::{
    Input, KeyboardInput, LocalPlayerClimbableMovementInput, LocalPlayerFluidFallingInput,
    LocalPlayerFrictionInfluencedSpeedInput, LocalPlayerHandleRelativeFrictionInput,
    LocalPlayerInputModifiers, LocalPlayerInputState, LocalPlayerModifiedFrictionInput,
    LocalPlayerSprintGateInput, LocalPlayerSprintTriggerConditions, LocalPlayerTravelDispatchInput,
    LocalPlayerTravelDispatchParameters, LocalPlayerTravelFlyingInput, LocalPlayerTravelInAirInput,
    LocalPlayerTravelInLavaInput, LocalPlayerTravelInWaterInput,
    LocalPlayerTravelInWaterParameters, LocalPlayerUpdateFallFlyingMovementInput,
    adjust_local_player_fluid_falling_movement, apply_local_player_input,
    can_start_local_player_sprinting, clamp_local_player_climbable_movement,
    local_player_friction_influenced_speed, local_player_handle_relative_friction,
    local_player_modified_friction, local_player_travel_dispatch, local_player_travel_flying,
    local_player_travel_in_air, local_player_travel_in_lava, local_player_travel_in_water,
    local_player_update_fall_flying_movement, modify_local_player_input,
};
use voxel_world::{
    BoatInput, ClientAttributeModifierState, ClientAttributeState, ClientChunkCacheSnapshot,
    ClientEntity, ClientEntityCollisionCandidate, ClientEntityDimensions, ClientEntityPose,
    ClientPlayerAbilities, ClientPlayerGameMode, ClientPlayerPoseInput, ClientWorld,
    ClientWorldBorder, ClientWorldBorderInitialization, ClientWorldBorderStatus,
    ClientWorldCollisionSource, DimensionBounds, EntityInput, EntityMotionInput, EntityMoveInput,
    EntityTeleportInput, LightLayerInput, LightUpdateInput, SectionInput,
    apply_entity_movement_state, collect_candidate_step_up_heights,
    collect_entity_collision_shapes, collide_entity_movement, collide_entity_movement_state,
    is_minor_horizontal_collision, living_entity_dimensions, refresh_entity_dimensions,
    select_step_up_movement, world_border_collision_shape,
};

mod hex;
use hex::*;
mod information;
use information::*;
mod error;
use error::*;
mod enum_ops;
use enum_ops::*;
mod resource;
use resource::*;
mod pack;
use pack::*;
mod blockpos;
use blockpos::*;
mod chunkpos;
use chunkpos::*;
mod snapshot;
use snapshot::*;
mod shape;
use shape::*;
mod travel;
use travel::*;
mod light;
use light::*;
mod transform;
use transform::*;
mod animal;
use animal::*;
mod client;
use client::*;
mod chunk;
use chunk::*;
mod math_cases;
use math_cases::*;
mod timer;
use timer::*;
mod nbt_cases;
use nbt_cases::*;
mod tooltip;
use tooltip::*;
mod block;
use block::*;
mod vanilla;
use vanilla::*;
mod shadows;
use shadows::*;
mod sky;
use sky::*;
mod movement;
use movement::*;
mod quads;
use quads::*;
mod frog;
use frog::*;
mod boat;
use boat::*;
mod sound;
use sound::*;
mod pose;
use pose::*;
mod display;
use display::*;
mod guardian;
use guardian::*;
mod animation;
use animation::*;
mod strider;
use strider::*;
mod sheep;
use sheep::*;
mod walk;
use walk::*;
mod entity;
use entity::*;
mod command;
use command::*;
mod container;
use container::*;
mod animal_models;
use animal_models::*;
mod blockstate;
use blockstate::*;
mod boat_ticks;
use boat_ticks::*;
mod declarations;
use declarations::*;
mod entity_motion;
use entity_motion::*;
mod entity_state;
use entity_state::*;
mod humanoid_models;
use humanoid_models::*;
mod image;
use image::*;
mod living_ticks;
use living_ticks::*;
mod model;
use model::*;
mod model_layers;
use model_layers::*;
mod passenger_positions;
use passenger_positions::*;
mod passenger_rotations;
use passenger_rotations::*;
mod player_attributes;
use player_attributes::*;
mod player_input;
use player_input::*;
mod projectiles;
use projectiles::*;
mod ticks;
use ticks::*;
