//! `ClientWorld` entity layer split out of the former single-file module.

use super::*;

use photon_core::aabb::Aabb;
use photon_core::java_float::{DVec3, JavaF32, JavaF64};
use photon_core::math::{
    Vec3, clamp_f32, mth_atan2, mth_cos_f64, mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32,
    mth_rot_lerp_f64, mth_sin_f64, projectile_lerp_rotation_f32, wrap_degrees_f32,
};

#[derive(Debug, Clone, PartialEq, Eq, bevy_ecs::component::Component)]
pub struct ClientEntity {
    pub(crate) id: EntityId,
    pub uuid: photon_core::binary::Uuid,
    pub type_id: EntityTypeId,
    pub(crate) tick_count: i32,
    pub(crate) first_tick: bool,
    /// `FallingBlockEntity.getStartPos`: the block the entity was added in.
    pub(crate) start_block: (i32, i32, i32),
    pub(crate) delta_rotation: JavaF32,
    /// A flying projectile's owner and whether it has left it.
    pub(crate) flight: crate::objects::flight::FlightState,
    pub(crate) on_ground: bool,
    pub(crate) tracking: Tracking,
    pub(crate) always_ticking: bool,
    pub(crate) ticking: bool,
    /// The limbs' walk animation and the swimming pose's ease (a player's).
    pub(crate) walk: crate::living::WalkAnimation,
    pub(crate) swim_amount: JavaF32,
    pub(crate) old_swim_amount: JavaF32,
    /// A mob's `BodyRotationControl` state: how long its head has been still,
    /// and where it was when it last moved far.
    pub(crate) head_stable_time: i32,
    pub(crate) last_stable_y_head_rot: JavaF32,
    /// What a dropped item holds (`ItemEntity.DATA_ITEM`), and where in its
    /// spin it started (`bobOffs`).
    pub data: i32,
    pub(crate) metadata: Option<Vec<u8>>,
    pub(crate) shared_flags: u8,
    pub(crate) living_flags: u8,
    pub(crate) mob_flags: u8,
    pub(crate) custom_name_present: bool,
    pub(crate) custom_name_visible: bool,
    pub(crate) no_gravity: bool,
    pub(crate) silent: bool,
    pub(crate) pose: ClientEntityPose,
    pub(crate) baby: bool,
    pub(crate) alive: bool,
    pub(crate) spectator: bool,
    pub(crate) on_climbable: bool,
    pub(crate) flying: bool,
    pub(crate) happy_ghast_still_timeout: bool,
    pub(crate) happy_ghast_is_vehicle: bool,
    /// A primed tnt's fuse and block, made when its data or ticks first reach it.
    /// `ItemFrame.getRotation`: the eighths of a turn the item in it is turned.
    pub(crate) item_frame_rotation: i32,
    /// Whether the bolt ran out of flashes at its last tick (`LightningBolt.discard`).
    pub(crate) lightning_done: bool,
    /// `FireworkRocketEntity.isShotAtAngle`: fired from a crossbow, which turns how it is drawn.
    pub(crate) shot_at_angle: bool,
    /// `FireworkRocketEntity.isAttachedToEntity`: its data names an entity it flies with.
    pub(crate) attached_target: Option<EntityId>,
}

mod animation;
mod attributes;
pub use attributes::*;
pub use photon_data::attribute_state::{ClientAttributeModifierState, ClientAttributeState};
mod boat;
mod cloak;
pub use boat::*;
pub(crate) mod equipment;
pub use equipment::*;
mod flags;
pub(crate) use flags::*;
mod identity;
pub(crate) use identity::*;
mod inputs;
pub use inputs::*;
mod interpolation;
pub(crate) use interpolation::*;
mod culling;
mod item_frame;
mod minecart;
pub(crate) use minecart::*;
mod motion;
pub(crate) use motion::*;
mod mounts;
mod pose;
pub use pose::*;
mod projectile;
pub(crate) use projectile::*;
mod rotation;
pub use position::{Interpolation, Motion, Position};
pub use rotation::Rotation;
mod position;
mod view;
pub(crate) use rotation::*;
pub use view::{EntityParts, EntityView, EntityViewMut};
mod tracking;
pub(crate) use tracking::*;
mod types;
pub(crate) use types::*;
mod species;
