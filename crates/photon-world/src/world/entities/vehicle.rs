use crate::*;

impl ClientWorld {
    /// `handleMoveVehicle`: the server puts the vehicle the local player drives where it says,
    /// if that is not where the vehicle is going already, and the vehicle's own pose is what the
    /// player answers with. A vehicle the player does not drive, or none, is left alone.
    pub fn move_driven_vehicle(
        &mut self,
        target: (f64, f64, f64),
        y_rot: f32,
        x_rot: f32,
    ) -> Option<DrivenVehicle> {
        let player = self.local_player.as_ref()?.id;
        let vehicle_id = self.root_vehicle_id(player);
        if vehicle_id == player || !self.is_locally_authoritative(vehicle_id) {
            return None;
        }
        let view = self.entities.at(vehicle_id)?;
        let current = view
            .interpolation_target_position()
            .unwrap_or_else(|| view.position());
        let (dx, dy, dz) = (
            target.0 - current.0,
            target.1 - current.1,
            target.2 - current.2,
        );
        let previous = view.position();
        if (dx * dx + dy * dy + dz * dz).sqrt() > f64::from(1.0e-5_f32) {
            let mut vehicle = self.entities.at_mut(vehicle_id)?;
            vehicle.cancel_interpolation();
            vehicle.abs_snap_to(target, y_rot, x_rot);
            let position = vehicle.position();
            self.update_entity_section(vehicle_id, previous, position);
        }
        let view = self.entities.at(vehicle_id)?;
        let (position, (y_rot, x_rot)) = if let (Some(position), Some(rotation)) = (
            view.interpolation_target_position(),
            view.interpolation_target_rotation(),
        ) {
            (position, rotation)
        } else {
            let (y_rot, x_rot, _) = view.rotation();
            (view.position(), (y_rot, x_rot))
        };
        Some(DrivenVehicle {
            position,
            y_rot,
            x_rot,
            on_ground: view.on_ground(),
        })
    }
}

/// A vehicle the local player drives, as `ServerboundMoveVehiclePacket.fromEntity` reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrivenVehicle {
    pub position: (f64, f64, f64),
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
}
