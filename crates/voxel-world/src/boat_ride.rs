//! A boat the local player rides: `AbstractBoat.tick` on a client that is its authority. The
//! rider's input steers it (`controlBoat`), the water, the land or the air it is in decides how it
//! floats and slows (`getStatus`, `floatBoat`), and `Entity.move` carries it through the blocks.
//! What the boat does to other entities it meets (pushing them) is not carried.

use voxel_core::aabb::Aabb;
use voxel_core::binary::BlockPos;
use voxel_core::java_float::{JavaF32, JavaF64};
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

/// Whether a paddle at this turn makes its stroke sound this tick (`AbstractBoat.tick`'s test of
/// the paddle's position against the start of its sweep).
fn paddle_makes_sound(position: f32) -> bool {
    let turn = (std::f64::consts::PI * 2.0) as f32;
    let quarter = (std::f64::consts::PI / 4.0) as f32;
    let step = (std::f64::consts::PI / 8.0) as f32;
    position % turn <= quarter && (position + step) % turn >= quarter
}

impl ClientWorld {
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
        body.y_rot = entity.y_rot.get();
        body.delta_rotation = entity.delta_rotation.get();
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
        let level = Level::new(self);
        let Ok(paddles) = body.tick(&level) else {
            return;
        };
        // The paddles row, and a stroke at the start of its sweep asks which sound the water or
        // the land makes, which finds the status again (and what finding it leaves behind).
        let silent = entity.silent;
        let paddling = [paddles.0, paddles.1].map(|rows| rows && body.has_passengers);
        let positions = [0, 1].map(|side| {
            entity
                .kind
                .boat()
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
        entity.motion.x = JavaF64::new(body.delta.x);
        entity.motion.y = JavaF64::new(body.delta.y);
        entity.motion.z = JavaF64::new(body.delta.z);
        entity.y_rot = JavaF32::new(body.y_rot);
        entity.delta_rotation = JavaF32::new(body.delta_rotation);
        entity.on_ground = body.on_ground;
        entity.tracking.support = Support {
            main_block: body.supporting_block,
            no_blocks: body.on_ground_no_blocks,
        };
        entity.paddle_left = body.paddles.0;
        entity.paddle_right = body.paddles.1;
        entity.boat_ride = body.ride;
        entity.tick_boat_end(paddling);
        if let Some(state) = entity.kind.boat_mut() {
            state.is_under_water = body.ride.status.is_some_and(Status::is_under_water);
        }
        self.paddle_packets.push(paddles);
    }

    /// The paddles the ridden boat's ticks told of since last asked, each to be sent to the server.
    pub fn take_paddle_packets(&mut self) -> Vec<(bool, bool)> {
        std::mem::take(&mut self.paddle_packets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `AbstractBoat.controlBoat` against the numbers the reference gives from the same start
    /// (`docs/records/client-world-entity-boat-control-conformance.md`): input flags, `deltaRotation`, yaw,
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

mod boat_body_stuck;
mod boat_body_water;
