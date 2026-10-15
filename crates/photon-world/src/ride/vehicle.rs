//! The vehicle the local player rides, as the player's own tick reads it: where it is, where it
//! carries its rider and what it lets the rider do.

use photon_core::math::{Vec3, mth_cos_f64, mth_sin_f64};

use crate::ClientWorld;
use crate::constants::VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y;
use crate::entity::ClientEntityDimensions;
use crate::ride::boat::Status;

/// What kind of vehicle it is, for the parts of a ride that differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleKind {
    Boat,
    /// A saddled horse, donkey, mule or undead horse.
    Horse,
    /// A pig, saddled or not.
    Pig,
    /// Any other vehicle, which the server moves.
    Other,
}

/// What the local player's tick reads of the vehicle it rides.
#[derive(Debug, Clone, Copy)]
pub struct VehicleView {
    pub vehicle_id: i32,
    pub kind: VehicleKind,
    /// Whether the local player is the vehicle's authority (it drives it).
    pub controlled: bool,
    pub type_id: i32,
    pub position: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    /// Where the rider is carried to (`getPassengerRidingPosition`).
    pub rider_position: Vec3,
    /// A boat's turn, which turns its rider, the top of its box, and whether it is under water.
    pub delta_rotation: f32,
    pub max_y: f64,
    pub under_water: bool,
    /// `Entity.canSprint`: whether the rider may sprint in it.
    pub can_sprint: bool,
    /// `PlayerRideableJumping.canJump`: whether the rider's jump key charges a jump.
    pub can_jump: bool,
}

/// `Vec3.yRot`: a point turned about the vertical axis.
pub(crate) fn rotate_y(point: Vec3, radians: f32) -> Vec3 {
    let cos = f64::from(mth_cos_f64(f64::from(radians)));
    let sin = f64::from(mth_sin_f64(f64::from(radians)));
    Vec3::new(
        point.x * cos + point.z * sin,
        point.y,
        point.z * cos - point.x * sin,
    )
}

impl ClientWorld {
    /// The vehicle the local player sits in, whether it drives it or not.
    pub fn ridden_vehicle_of_local_player(&self) -> Option<i32> {
        let player_id = self.local_player.as_ref()?.id;
        let vehicle_id = self.entity_vehicle_id(player_id)?;
        let vehicle = self.entities.at(vehicle_id)?;
        vehicle
            .passenger_ids
            .contains(&player_id)
            .then_some(vehicle_id)
    }

    /// `Entity.hasIndirectPassenger(player)`: whether the local player sits in this entity, or in
    /// something that sits in it.
    pub fn entity_carries_local_player(&self, entity_id: i32) -> bool {
        let Some(player_id) = self.local_player.as_ref().map(|player| player.id) else {
            return false;
        };
        let mut current = self.entity_vehicle_id(player_id);
        // A vehicle chain is never this deep; the bound only keeps a bad packet from looping.
        for _ in 0..64 {
            match current {
                Some(id) if id == entity_id => return true,
                Some(id) => current = self.entity_vehicle_id(id),
                None => return false,
            }
        }
        false
    }

    /// `Entity.isPassenger` for the local player.
    pub fn local_player_is_passenger(&self) -> bool {
        self.local_player
            .as_ref()
            .is_some_and(|player| player.vehicle_id.is_some())
    }

    /// Whether the level has an entity of this id.
    pub fn has_entity(&self, entity_id: i32) -> bool {
        self.entities.contains_key(&entity_id)
    }

    /// What the local player's tick reads of the vehicle it rides.
    pub fn vehicle_view(&self, vehicle_id: i32) -> Option<VehicleView> {
        let player = self.local_player.as_ref()?;
        let vehicle = self.entities.at(vehicle_id)?;
        let (x, y, z) = vehicle.position();
        let position = Vec3::new(x, y, z);
        if vehicle.is_boat() {
            return self.boat_view(vehicle_id, vehicle, position);
        }
        let driven = self.mount_kind_of(vehicle);
        let kind = if vehicle.is_horse() {
            VehicleKind::Horse
        } else if driven == Some(crate::MountKind::Pig) || vehicle.is_pig() {
            VehicleKind::Pig
        } else {
            VehicleKind::Other
        };
        let y_rot = vehicle.y_rot.get();
        // `getDefaultPassengerAttachmentPoint`: the type's seat for this passenger's place, turned
        // with the vehicle; `AbstractHorse` adds, rearing, up and back.
        let seats = photon_data::entity_type::pick(vehicle.type_id)?.passengers;
        let index = vehicle
            .passenger_ids
            .iter()
            .position(|&id| id == player.id)
            .unwrap_or(0);
        let seat = seats
            .get(index.min(seats.len().saturating_sub(1)))
            .map_or(Vec3::ZERO, |&(x, y, z)| Vec3::new(x, y, z));
        let turn = -y_rot * (std::f64::consts::PI / 180.0) as f32;
        let mut attachment = rotate_y(seat, turn);
        let camel = vehicle.is_camel();
        if camel {
            attachment = self.camel_seat(vehicle, index, turn);
        }
        if kind == VehicleKind::Horse && !camel {
            let stand_old = vehicle
                .horse()
                .map_or(0.0, |horse| horse.stand_animation_old());
            // Doubles times the float standing animation, times the scale of one.
            attachment = attachment.add_vec3(rotate_y(
                Vec3::new(
                    0.0,
                    0.15 * f64::from(stand_old) * 1.0,
                    -0.7 * f64::from(stand_old) * 1.0,
                ),
                turn,
            ));
        }
        if photon_data::entity_type::name(vehicle.type_id) == Some("minecraft:strider") {
            // `Strider.getPassengerAttachmentPoint` on a client: the seat bobs with its stride.
            let walk = vehicle.walk_animation();
            let anim_speed = 0.25_f32.min(walk.speed());
            let anim_pos = walk.position();
            let offset =
                0.12_f32 * (mth_cos_f64(f64::from(anim_pos * 1.5)) as f32) * 2.0 * anim_speed;
            attachment = attachment.add_components(0.0, f64::from(offset * 1.0), 0.0);
        }
        // `Entity.positionRider`: the seat, less where the rider's own attachment point is.
        let rider = Vec3::new(
            position.x + attachment.x,
            position.y + attachment.y - VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y,
            position.z + attachment.z,
        );
        Some(VehicleView {
            vehicle_id,
            kind,
            controlled: driven.is_some(),
            type_id: vehicle.type_id,
            position,
            y_rot,
            x_rot: vehicle.x_rot.get(),
            on_ground: vehicle.on_ground,
            horizontal_collision: vehicle.mount_ride.horizontal_collision,
            rider_position: rider,
            delta_rotation: 0.0,
            max_y: 0.0,
            under_water: true,
            // Only a camel and a player can sprint (`Entity.canSprint`).
            can_sprint: camel,
            can_jump: match driven {
                Some(crate::MountKind::Horse) => true,
                Some(crate::MountKind::Camel) => vehicle.horse().is_none_or(|horse| {
                    !horse.camel.refuses_to_move(self.level.game_time as i64)
                        && horse.camel.dash_cooldown() == 0
                }),
                _ => false,
            },
        })
    }

    /// `Camel.getPassengerAttachmentPoint`: where the body is as it sits or rises, and the seat
    /// forward or back by the passenger's place.
    fn camel_seat(&self, vehicle: &crate::entity::ClientEntity, index: usize, turn: f32) -> Vec3 {
        let (sitting, pose_time) = vehicle.horse().map_or((false, i64::MAX / 2), |horse| {
            horse.camel.pose_for_anchor(self.level.game_time as i64)
        });
        let height =
            photon_data::entity_type::pick(vehicle.type_id).map_or(0.0, |pick| pick.size.1);
        let height = if vehicle.pose() == crate::entity::ClientEntityPose::Sitting {
            height - 1.43
        } else {
            height
        };
        let anchor = photon_data::leash::camel_anchor_offset(
            height,
            vehicle.is_baby(),
            photon_data::leash::CamelPose {
                sitting,
                pose_time,
                partial: 0.0,
            },
            index == 0,
        );
        let forward = if vehicle.passenger_ids.len() > 1 && index != 0 {
            -0.7_f32
        } else {
            0.5_f32
        };
        rotate_y(Vec3::new(0.0, anchor, f64::from(forward)), turn)
    }

    /// What the local player's tick reads of a boat it rides.
    fn boat_view(
        &self,
        vehicle_id: i32,
        vehicle: &crate::entity::ClientEntity,
        position: Vec3,
    ) -> Option<VehicleView> {
        let player = self.local_player.as_ref()?;
        let pick = photon_data::entity_type::pick(vehicle.type_id)?;
        let (width, height, _) = pick.size;
        let bb = ClientEntityDimensions::scalable(width, height)
            .scale(vehicle.scale(), vehicle.scale())
            .make_bounding_box(position);
        Some(VehicleView {
            vehicle_id,
            kind: VehicleKind::Boat,
            controlled: vehicle.passenger_ids.first() == Some(&player.id),
            type_id: vehicle.type_id,
            position,
            y_rot: vehicle.y_rot.get(),
            x_rot: 0.0,
            on_ground: vehicle.on_ground,
            horizontal_collision: vehicle.boat_ride.horizontal_collision,
            rider_position: self.boat_rider_position(vehicle_id, player.id)?,
            delta_rotation: vehicle.delta_rotation.get(),
            max_y: bb.max_y,
            under_water: vehicle.boat_ride.status.is_some_and(Status::is_under_water),
            can_sprint: false,
            can_jump: false,
        })
    }
}
