use super::*;

impl LocalPlayer {
    /// `yBodyRotO` and `yBodyRot`: the body's turn at the last tick and now.
    pub fn body_rotations(&self) -> (f32, f32) {
        (self.y_body_rot_old, self.y_body_rot)
    }

    pub fn interpolated_position(&self, partial_tick: f64) -> Vec3 {
        self.old_position.lerp(self.position, partial_tick)
    }

    /// `AbstractBoat.positionRider` for the player: carried to its seat, turned as the boat turns
    /// and held within a quarter turn of where the boat faces (`clampRotation`).
    pub(super) fn position_in_boat(&mut self, boat: &crate::VehicleView) {
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

    /// The rest of `LivingEntity.tick`: the body turns towards where the
    /// player walks, or where it looks, and never more than 50 degrees from
    /// the head.
    pub(super) fn turn_body(&mut self) {
        crate::living::turn_body(
            (&mut self.y_body_rot, &mut self.y_body_rot_old),
            (self.y_rot, self.y_head_rot, &mut self.y_head_rot_old),
            (
                self.position.x - self.old_position.x,
                self.position.z - self.old_position.z,
            ),
        );
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn old_position(&self) -> Vec3 {
        self.old_position
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

    pub(super) fn send_position(&mut self) {
        if self.sprinting != self.was_sprinting {
            let action = if self.sprinting {
                PlayerCommandAction::StartSprinting
            } else {
                PlayerCommandAction::StopSprinting
            };
            self.sent.push(ServerboundMovement::PlayerCommand {
                entity_id: self.entity_id,
                action,
                data: 0,
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

    pub(super) fn set_pos(&mut self, position: Vec3) {
        self.position = position;
        self.bounding_box = self.dimensions.make_bounding_box(position);
    }

    pub(super) fn look_angle(&self) -> Vec3 {
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

    pub(super) fn block_position(&self) -> BlockPos {
        BlockPos::new(
            floor_f64(self.position.x),
            floor_f64(self.position.y),
            floor_f64(self.position.z),
        )
    }
}

/// `Mth.rotLerp`: the shorter way round between two angles.
pub(super) fn rot_lerp(partial_tick: f32, from: f32, to: f32) -> f32 {
    from + partial_tick * photon_core::math::wrap_degrees_f32(to - from)
}
