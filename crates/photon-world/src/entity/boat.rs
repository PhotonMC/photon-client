use super::*;

impl ClientEntity {
    pub fn paddle_state(&self) -> (bool, bool) {
        (self.paddle_left, self.paddle_right)
    }

    /// `Entity.isVehicle`: something rides it.
    pub fn is_vehicle(&self) -> bool {
        !self.passenger_ids.is_empty()
    }

    /// What a boat the local player rides keeps between ticks.
    pub fn boat_ride(&self) -> crate::ride::boat::RideState {
        self.boat_ride
    }

    /// What a saddled horse the local player rides keeps between ticks.
    pub fn mount_ride(&self) -> crate::ride::mount::MountState {
        self.mount_ride
    }

    /// Whether this is a boat, a chest boat or a raft.
    pub fn is_boat(&self) -> bool {
        vanilla_boat_max_passengers(self.type_id).is_some()
    }

    /// Whether a boat's paddle on a side (zero left, one right) rows
    /// (`AbstractBoat.getPaddleState`): its data says so and someone rows it.
    pub fn boat_paddle_rows(&self, side: usize) -> bool {
        let flag = if side == 0 {
            self.paddle_left
        } else {
            self.paddle_right
        };
        flag && !self.passenger_ids.is_empty()
    }
}

pub(crate) fn vanilla_boat_passenger_rotation_clamped_values(
    passenger_y_rot: f32,
    passenger_old_y_rot: f32,
    vehicle_y_rot: f32,
) -> (f32, f32) {
    let delta = wrap_degrees_f32(passenger_y_rot - vehicle_y_rot);
    let target_delta = clamp_f32(
        delta,
        -VANILLA_BOAT_PASSENGER_MAX_ROTATION,
        VANILLA_BOAT_PASSENGER_MAX_ROTATION,
    );
    (
        passenger_old_y_rot + target_delta - delta,
        passenger_y_rot + target_delta - delta,
    )
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
            if VANILLA_CHEST_BOAT_ENTITY_TYPE_IDS.contains(&vehicle_type_id.0) {
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
    pub(crate) fn clamp_vanilla_boat_passenger_rotation(
        &mut self,
        vehicle_y_rot: f32,
        animal_rotation_offset: Option<f32>,
    ) {
        let (old_y_rot, y_rot) = vanilla_boat_passenger_rotation_clamped_values(
            self.rotation.y_rot.get(),
            self.rotation.old_y_rot.get(),
            vehicle_y_rot,
        );
        self.rotation.old_y_rot = JavaF32::new(old_y_rot);
        self.rotation.y_rot = JavaF32::new(y_rot);
        if vanilla_passenger_is_living(self.type_id) {
            self.rotation.y_head_rot = self.rotation.y_rot;
            self.rotation.y_body_rot = JavaF32::new(vehicle_y_rot);
        }
        if let Some(rotation_offset) = animal_rotation_offset {
            self.rotation.y_body_rot =
                JavaF32::new(self.rotation.y_body_rot.get() + rotation_offset);
            self.rotation.y_head_rot =
                JavaF32::new(self.rotation.y_head_rot.get() + rotation_offset);
        }
    }

    pub(crate) fn add_vanilla_boat_passenger_rotation(&mut self, delta_rotation: f32) {
        self.rotation.y_rot = JavaF32::new(self.rotation.y_rot.get() + delta_rotation);
        if vanilla_passenger_is_living(self.type_id) {
            self.rotation.y_head_rot =
                JavaF32::new(self.rotation.y_head_rot.get() + delta_rotation);
        }
    }

    /// `AbstractBoat.tick` on the client, at the start of the tick (`under_water` is
    /// none for an entity that is not a boat).
    pub(crate) fn tick_boat(&mut self, under_water: Option<bool>) {
        let Some(under_water) = under_water else {
            return;
        };
        // `AbstractBoat.tick`: the easing runs after `super.tick`, before the boat moves.
        self.tick_position_interpolation();
        let paddling = [self.boat_paddle_rows(0), self.boat_paddle_rows(1)];
        let tick_count = self.tick_count;
        self.kind
            .boat_state()
            .tick(under_water, paddling, tick_count);
    }

    /// The part of a ridden boat's tick before its own movement.
    pub(crate) fn tick_boat_start(&mut self) {
        self.tick_position_interpolation();
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
            self.paddle_left = left;
        }
        if let Some(right) = data.paddle_right {
            self.paddle_right = right;
        }
    }

    /// The part of a ridden boat's tick after its movement: the paddles row and the bubbles shake.
    pub(crate) fn tick_boat_end(&mut self, paddling: [bool; 2]) {
        let tick_count = self.tick_count;
        self.kind.boat_state().tick_end(paddling, tick_count);
    }
}
