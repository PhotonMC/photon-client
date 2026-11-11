use super::*;

impl ClientEntity {
    /// Whether this is a boat, a chest boat or a raft.
    pub fn is_boat(&self) -> bool {
        vanilla_boat_max_passengers(self.type_id).is_some()
    }
}

/// `AbstractBoat.clampRotation` for a passenger that is not a `Mob`: its yaw held within a quarter
/// turn of where its body is made to face, and how far that moved it.
pub(crate) fn vanilla_boat_passenger_clamped_y_rot(
    passenger_y_rot: f32,
    body_y_rot: f32,
) -> (f32, f32) {
    let delta = wrap_degrees_f32(passenger_y_rot - body_y_rot);
    let target_delta = clamp_f32(
        delta,
        -VANILLA_BOAT_PASSENGER_MAX_ROTATION,
        VANILLA_BOAT_PASSENGER_MAX_ROTATION,
    );
    (passenger_y_rot + target_delta - delta, target_delta - delta)
}

/// `Mob.clampHeadRotationToBody`: the head held within `limit` degrees of the body.
pub(crate) fn vanilla_clamped_head_y_rot(head_y_rot: f32, body_y_rot: f32, limit: f32) -> f32 {
    let delta = wrap_degrees_f32(body_y_rot - head_y_rot);
    let target_delta = clamp_f32(wrap_degrees_f32(body_y_rot - head_y_rot), -limit, limit);
    head_y_rot + delta - target_delta
}

/// `Mob.getMaxHeadYRot`: how far a mob's head may turn from its body.
pub(crate) fn vanilla_max_head_y_rot(type_id: EntityTypeId) -> f32 {
    use photon_data::entity_type::ids;
    let limits = [
        (ids::AXOLOTL, 1.0),
        (ids::DOLPHIN, 1.0),
        (ids::FROG, 5.0),
        (ids::GOAT, 15.0),
        (ids::CAMEL, 30.0),
        (ids::CAMEL_HUSK, 30.0),
        (ids::BREEZE, 30.0),
        (ids::ARMADILLO, 32.0),
        (ids::RAVAGER, 45.0),
        (ids::SNIFFER, 50.0),
        (ids::SHULKER, 180.0),
    ];
    limits
        .iter()
        .find(|&&(known, _)| known == type_id)
        .map_or(75.0, |&(_, limit)| limit)
}

/// Whether the type is a `Mob` (a living entity that is not a player, an armor stand or a
/// mannequin).
pub(crate) fn vanilla_passenger_is_mob(passenger_type_id: EntityTypeId) -> bool {
    use photon_data::entity_type::ids;
    vanilla_passenger_is_living(passenger_type_id)
        && ![ids::PLAYER, ids::ARMOR_STAND, ids::MANNEQUIN].contains(&passenger_type_id)
}

pub(crate) fn vanilla_passenger_can_turn_in_boat(passenger_type_id: EntityTypeId) -> bool {
    passenger_type_id == VANILLA_BREEZE_ENTITY_TYPE_ID
}

pub(crate) fn vanilla_passenger_is_animal(passenger_type_id: EntityTypeId) -> bool {
    VANILLA_ANIMAL_ENTITY_TYPE_IDS.contains(&passenger_type_id.0)
}

pub(crate) fn vanilla_passenger_is_living(passenger_type_id: EntityTypeId) -> bool {
    VANILLA_LIVING_ENTITY_TYPE_IDS.contains(&passenger_type_id.0)
}

pub(crate) fn vanilla_passenger_vehicle_attachment_y(
    passenger_type_id: EntityTypeId,
    is_baby: bool,
) -> f64 {
    match passenger_type_id {
        VANILLA_PLAYER_ENTITY_TYPE_ID => VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y,
        VANILLA_SPIDER_ENTITY_TYPE_ID => VANILLA_SPIDER_VEHICLE_ATTACHMENT_Y,
        VANILLA_ZOMBIE_ENTITY_TYPE_ID if is_baby => 0.1875,
        _ => 0.0,
    }
}

pub(crate) fn vanilla_boat_max_passengers(vehicle_type_id: EntityTypeId) -> Option<usize> {
    VANILLA_BOAT_PASSENGER_GEOMETRY
        .iter()
        .any(|&(type_id, _, _)| type_id == vehicle_type_id.0)
        .then(|| {
            if VANILLA_CHEST_BOAT_ENTITY_TYPE_IDS.contains(&vehicle_type_id.0)
                // The poplar chest boat only the newest version has.
                || vehicle_type_id == photon_data::entity_type::ids::POPLAR_CHEST_BOAT
            {
                1
            } else {
                2
            }
        })
}

pub(crate) fn is_vanilla_boat_entity_type(entity_type_id: EntityTypeId) -> bool {
    VANILLA_BOAT_PASSENGER_GEOMETRY
        .iter()
        .any(|&(type_id, _, _)| type_id == entity_type_id.0)
}

/// The four movement flags copied by Java's `AbstractBoat.setInput`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoatInput {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub backward: bool,
}

pub(crate) const VANILLA_BOAT_PASSENGER_MAX_ROTATION: f32 = 105.0_f32;

impl BoatInput {
    pub const EMPTY: Self = Self::new(false, false, false, false);

    pub const fn new(left: bool, right: bool, forward: bool, backward: bool) -> Self {
        Self {
            left,
            right,
            forward,
            backward,
        }
    }
}

impl EntityViewMut<'_> {
    /// `AbstractBoat.clampRotation`: the body turned to face `body_y_rot`, a mob's yaw made its
    /// body's and its head held to it, anyone else's yaw held within a quarter turn of it. How far
    /// the yaw moved.
    pub(crate) fn clamp_vanilla_boat_passenger_rotation(&mut self, body_y_rot: f32) -> f32 {
        let uniform = photon_core::versions::rules().world.boat_clamp_is_uniform;
        if vanilla_passenger_is_living(self.type_id) {
            self.rotation.y_body_rot = JavaF32::new(body_y_rot);
        }
        if vanilla_passenger_is_mob(self.type_id) && !uniform {
            self.rotation.y_rot = JavaF32::new(body_y_rot);
            let head = vanilla_clamped_head_y_rot(
                self.rotation.y_head_rot.get(),
                body_y_rot,
                vanilla_max_head_y_rot(self.type_id),
            );
            self.rotation.y_head_rot = JavaF32::new(head);
            return wrap_degrees_f32(body_y_rot - self.rotation.y_rot.get());
        }
        let (y_rot, delta) =
            vanilla_boat_passenger_clamped_y_rot(self.rotation.y_rot.get(), body_y_rot);
        self.rotation.y_rot = JavaF32::new(y_rot);
        if uniform && vanilla_passenger_is_living(self.type_id) {
            self.rotation.y_head_rot = JavaF32::new(y_rot);
        }
        delta
    }

    /// The clamp of a passenger carried by its boat. 26.2 also moves the previous yaw along and
    /// turns a crowded animal afterwards (`crowded` holds whether its id is even).
    pub(crate) fn follow_boat_position(&mut self, body_y_rot: f32, crowded: Option<bool>) {
        let delta = self.clamp_vanilla_boat_passenger_rotation(body_y_rot);
        if !photon_core::versions::rules().world.boat_clamp_is_uniform {
            return;
        }
        let old = self.rotation.old_y_rot.get();
        self.rotation.old_y_rot = JavaF32::new(old + delta);
        if let Some(even_id) = crowded {
            self.add_crowded_boat_offset(even_id);
        }
    }

    /// 26.2's `positionRider` for an animal in a full boat: its body and head turned a further
    /// quarter turn one way or the other after the clamp.
    pub(crate) fn add_crowded_boat_offset(&mut self, even_id: bool) {
        let offset = if even_id { 90.0_f32 } else { 270.0_f32 };
        let body = self.rotation.y_body_rot.get() + offset;
        let head = self.rotation.y_head_rot.get() + offset;
        self.rotation.y_body_rot = JavaF32::new(body);
        self.rotation.y_head_rot = JavaF32::new(head);
    }

    /// `AbstractBoat.tick` on the client, at the start of the tick (`under_water` is
    /// none for an entity that is not a boat).
    pub(crate) fn tick_boat(&mut self, under_water: Option<bool>) {
        let Some(under_water) = under_water else {
            return;
        };
        let paddling = [
            self.as_view().boat_paddle_rows(0),
            self.as_view().boat_paddle_rows(1),
        ];
        let tick_count = self.tick_count;
        self.kind
            .boat_state()
            .tick(under_water, paddling, tick_count);
    }

    /// The part of a ridden boat's tick before its own movement.
    pub(crate) fn tick_boat_start(&mut self) {
        self.kind.boat_state().tick_start(false);
    }
}

impl EntityView<'_> {
    /// A boat's hit, bubbles and paddles.
    pub fn boat(&self) -> Option<&crate::objects::boat::BoatState> {
        self.kind.boat()
    }
}

impl EntityViewMut<'_> {
    /// A boat's data: its hit, paddles and bubbles.
    pub(crate) fn set_boat_data(&mut self, data: crate::metadata::BoatData) {
        let boat = self.kind.boat_state();
        if let Some(hurt) = data.hurt_time {
            boat.hurt_time = hurt;
        }
        if let Some(dir) = data.hurt_dir {
            boat.hurt_dir = dir;
        }
        if let Some(damage) = data.damage {
            boat.damage = damage;
        }
        if let Some(bubbles) = data.bubble_time {
            boat.bubble_time = bubbles;
        }
        if let Some(left) = data.paddle_left {
            self.ride.paddle_left = left;
        }
        if let Some(right) = data.paddle_right {
            self.ride.paddle_right = right;
        }
    }

    /// The part of a ridden boat's tick after its movement: the paddles row and the bubbles shake.
    pub(crate) fn tick_boat_end(&mut self, paddling: [bool; 2]) {
        let tick_count = self.tick_count;
        self.kind.boat_state().tick_end(paddling, tick_count);
    }
}

/// What the local player's ride keeps on the vehicle: its steering input, what a boat or a
/// saddled mount it rides keeps between ticks, and the paddles of a boat.
#[derive(Debug, Clone, PartialEq, Eq, bevy_ecs::component::Component)]
pub struct Riding {
    pub(crate) boat_input: BoatInput,
    /// What a boat the local player rides keeps between ticks (status, water level, fall).
    pub(crate) boat_ride: crate::ride::boat::RideState,
    /// The same of a saddled horse the local player rides.
    pub(crate) mount_ride: crate::ride::mount::MountState,
    pub(crate) paddle_left: bool,
    pub(crate) paddle_right: bool,
}

impl Default for Riding {
    fn default() -> Self {
        Self {
            boat_input: BoatInput::EMPTY,
            boat_ride: crate::ride::boat::RideState::default(),
            mount_ride: crate::ride::mount::MountState::default(),
            paddle_left: false,
            paddle_right: false,
        }
    }
}

impl EntityView<'_> {
    pub fn paddle_state(&self) -> (bool, bool) {
        (self.ride.paddle_left, self.ride.paddle_right)
    }

    /// What a boat the local player rides keeps between ticks.
    pub fn boat_ride(&self) -> crate::ride::boat::RideState {
        self.ride.boat_ride
    }

    /// What a saddled horse the local player rides keeps between ticks.
    pub fn mount_ride(&self) -> crate::ride::mount::MountState {
        self.ride.mount_ride
    }

    /// Whether a boat's paddle on a side (zero left, one right) rows
    /// (`AbstractBoat.getPaddleState`): its data says so and someone rows it.
    pub fn boat_paddle_rows(&self, side: usize) -> bool {
        let flag = if side == 0 {
            self.ride.paddle_left
        } else {
            self.ride.paddle_right
        };
        flag && !self.links.passenger_ids.is_empty()
    }
}

impl EntityView<'_> {
    /// `Entity.isVehicle`: something rides it.
    pub fn is_vehicle(&self) -> bool {
        !self.links.passenger_ids.is_empty()
    }
}
