use crate::*;

impl ClientWorld {
    /// Apply Java's `Entity.turn` deltas and notify a boat passenger's vehicle.
    pub fn turn_entity(&mut self, entity_id: EntityId, mouse_x: f64, mouse_y: f64) -> bool {
        let vehicle_id = self.entity_vehicle_id(entity_id);
        if let Some(mut entity) = self.entities.at_mut(entity_id) {
            entity.turn(mouse_x, mouse_y);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.turn(mouse_x, mouse_y);
        } else {
            return false;
        }
        if let Some(vehicle_id) = vehicle_id {
            self.clamp_entity_passenger_rotation_after_turn(vehicle_id, entity_id);
        }
        true
    }

    pub fn apply_entity_position_and_rotation(
        &mut self,
        input: EntityPositionInput,
        rotation: EntityRotationInput,
        mode: EntityPositionUpdateMode,
    ) -> bool {
        let Some(resolved) = self.resolve_entity_position(&input) else {
            return false;
        };
        let Some(entity) = self.entities.at(input.entity_id) else {
            return false;
        };
        let (current_y_rot, current_x_rot) = entity.interpolation_rotation();
        let target_y_rot = rotation.y_rot.map_or(current_y_rot, |y_rot| {
            if rotation.relative_y_rot {
                current_y_rot + y_rot
            } else {
                y_rot
            }
        });
        let target_x_rot = rotation.x_rot.map_or(current_x_rot, |x_rot| {
            if rotation.relative_x_rot {
                current_x_rot + x_rot
            } else {
                x_rot
            }
        });
        let target_x_rot = if mode == EntityPositionUpdateMode::Teleport {
            target_x_rot.clamp(-90.0, 90.0)
        } else {
            target_x_rot
        };
        self.apply_entity_position_target(
            input.entity_id,
            resolved.current,
            resolved.target,
            resolved.ticking,
            Some((target_y_rot, target_x_rot)),
            mode,
        )
    }

    pub fn apply_entity_teleport(&mut self, input: EntityTeleportInput) -> bool {
        let Some((current_motion, current_y_rot, current_x_rot)) =
            self.entities.at(input.entity_id).map(|entity| {
                (
                    entity.motion(),
                    entity.interpolation_rotation().0,
                    entity.interpolation_rotation().1,
                )
            })
        else {
            return false;
        };
        let target_y_rot = if input.relative_flags & ENTITY_RELATIVE_Y_ROT_FLAG != 0 {
            current_y_rot + input.y_rot
        } else {
            input.y_rot
        };
        let target_x_rot = if input.relative_flags & ENTITY_RELATIVE_X_ROT_FLAG != 0 {
            current_x_rot + input.x_rot
        } else {
            input.x_rot
        }
        .clamp(-90.0, 90.0);
        let target_motion = resolve_entity_teleport_motion(
            current_motion,
            (current_y_rot, current_x_rot),
            (target_y_rot, target_x_rot),
            (input.delta_x, input.delta_y, input.delta_z),
            input.relative_flags,
        );
        let applied = self.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id: input.entity_id,
                x: input.x,
                y: input.y,
                z: input.z,
                relative_flags: input.relative_flags,
            },
            EntityRotationInput {
                entity_id: input.entity_id,
                y_rot: Some(input.y_rot),
                x_rot: Some(input.x_rot),
                y_head_rot: None,
                relative_y_rot: input.relative_flags & ENTITY_RELATIVE_Y_ROT_FLAG != 0,
                relative_x_rot: input.relative_flags & ENTITY_RELATIVE_X_ROT_FLAG != 0,
            },
            EntityPositionUpdateMode::Teleport,
        );
        if applied && let Some(mut entity) = self.entities.at_mut(input.entity_id) {
            entity.set_motion_if_finite(target_motion);
        }
        applied
    }

    pub fn apply_entity_move(&mut self, input: EntityMoveInput) -> bool {
        let Some((previous_position, current)) = self
            .entities
            .get(&input.entity_id)
            .map(|entity| (entity.position(), entity.packet_base()))
        else {
            return false;
        };
        let new_position = input.has_position.then(|| {
            (
                f64::from_bits(decode_entity_delta(current.0, input.delta_x)),
                f64::from_bits(decode_entity_delta(current.1, input.delta_y)),
                f64::from_bits(decode_entity_delta(current.2, input.delta_z)),
            )
        });
        if self.is_locally_authoritative(input.entity_id) {
            self.keep_packet_base_only(input.entity_id, new_position);
            return false;
        }
        let ticking = new_position.map(|position| {
            let (chunk_x, chunk_z) = entity_chunk_key(position);
            self.contains_chunk(chunk_x, chunk_z)
        });
        let position;
        {
            let mut entity = self
                .entities
                .at_mut(input.entity_id)
                .expect("entity checked above");
            if let Some(position) = new_position {
                entity.set_packet_base(position);
            }
            if entity.has_interpolation_handler() {
                let target_position =
                    new_position.unwrap_or_else(|| entity.interpolation_position());
                let (current_y_rot, current_x_rot) = entity.as_view().interpolation_rotation();
                let target_y_rot = input.y_rot.unwrap_or(current_y_rot);
                let target_x_rot = input.x_rot.unwrap_or(current_x_rot);
                entity.move_or_interpolate_to(target_position, target_y_rot, target_x_rot);
            } else {
                entity.set_plain_position(new_position, (input.y_rot, input.x_rot));
                if input.has_position {
                    entity.ticking =
                        entity.always_ticking || ticking.expect("position ticking state");
                }
            }
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                entity.ticking = true;
            }
            position = entity.position();
        }
        self.update_entity_section(input.entity_id, previous_position, position);
        true
    }

    /// Apply Java's absolute entity position-sync path for a remote entity.
    pub fn apply_entity_position_sync(
        &mut self,
        entity_id: EntityId,
        position: (f64, f64, f64),
        rotation: (f32, f32),
    ) -> bool {
        self.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id,
                x: position.0,
                y: position.1,
                z: position.2,
                relative_flags: 0,
            },
            EntityRotationInput {
                entity_id,
                y_rot: Some(rotation.0),
                x_rot: Some(rotation.1),
                y_head_rot: None,
                relative_y_rot: false,
                relative_x_rot: false,
            },
            EntityPositionUpdateMode::PositionSync,
        )
    }

    pub(crate) fn apply_entity_position_target(
        &mut self,
        entity_id: EntityId,
        current_position: (f64, f64, f64),
        target_position: (f64, f64, f64),
        ticking: bool,
        target_rotation: Option<(f32, f32)>,
        mode: EntityPositionUpdateMode,
    ) -> bool {
        let Some(previous_position) = self.entities.at(entity_id).map(|entity| entity.position())
        else {
            return false;
        };
        let authoritative = self.is_locally_authoritative(entity_id);
        let position;
        {
            let mut entity = self
                .entities
                .at_mut(entity_id)
                .expect("entity position was checked above");
            if mode == EntityPositionUpdateMode::PositionSync {
                entity.set_packet_base(target_position);
                if authoritative {
                    return false;
                }
            }
            if entity.has_interpolation_handler() {
                let rotation =
                    target_rotation.unwrap_or_else(|| entity.as_view().interpolation_rotation());
                let distance_position = match mode {
                    EntityPositionUpdateMode::PositionSync => entity.position(),
                    EntityPositionUpdateMode::Teleport => current_position,
                };
                let delta_x = target_position.0 - distance_position.0;
                let delta_y = target_position.1 - distance_position.1;
                let delta_z = target_position.2 - distance_position.2;
                let can_interpolate =
                    match mode {
                        EntityPositionUpdateMode::PositionSync => entity.ticking,
                        EntityPositionUpdateMode::Teleport => true,
                    } && delta_x * delta_x + delta_y * delta_y + delta_z * delta_z
                        <= ENTITY_INTERPOLATION_DISTANCE_SQR;
                if can_interpolate {
                    entity.move_or_interpolate_to(target_position, rotation.0, rotation.1);
                } else {
                    entity.snap_to(target_position, rotation.0, rotation.1);
                }
            } else {
                entity.set_plain_position(
                    Some(target_position),
                    (target_rotation.map(|r| r.0), target_rotation.map(|r| r.1)),
                );
            }
            entity.ticking = entity.always_ticking || ticking;
            position = entity.position();
        }
        self.update_entity_section(entity_id, previous_position, position);
        true
    }

    /// `handleMoveEntity` for a vehicle this client drives: only the packet base follows.
    fn keep_packet_base_only(&mut self, entity_id: EntityId, position: Option<(f64, f64, f64)>) {
        if let Some(position) = position
            && let Some(mut entity) = self.entities.at_mut(entity_id)
        {
            entity.set_packet_base(position);
        }
    }

    /// `Entity.isLocalInstanceAuthoritative` of a vehicle: the local player steers it.
    fn is_locally_authoritative(&self, entity_id: EntityId) -> bool {
        self.entities.at(entity_id).is_some_and(|entity| {
            self.boat_is_local_instance_authoritative(entity.entity)
                && vanilla_boat_max_passengers(entity.type_id).is_some()
                || self.mount_kind_of(entity.entity).is_some()
        })
    }

    pub fn apply_entity_ground_state(&mut self, entity_id: EntityId, on_ground: bool) -> bool {
        let Some(mut entity) = self.entities.at_mut(entity_id) else {
            return false;
        };
        entity.on_ground = on_ground;
        self.check_item_supporting_block(entity_id);
        true
    }

    pub fn apply_entity_rotation(&mut self, input: EntityRotationInput) -> bool {
        {
            let Some(mut entity) = self.entities.at_mut(input.entity_id) else {
                return false;
            };
            if (entity.has_interpolation_handler())
                && (input.y_rot.is_some() || input.x_rot.is_some())
            {
                let (current_y_rot, current_x_rot) = entity.as_view().interpolation_rotation();
                let y_rot = input.y_rot.map_or(current_y_rot, |y_rot| {
                    if input.relative_y_rot {
                        current_y_rot + y_rot
                    } else {
                        y_rot
                    }
                });
                let x_rot = input.x_rot.map_or(current_x_rot, |x_rot| {
                    if input.relative_x_rot {
                        current_x_rot + x_rot
                    } else {
                        x_rot
                    }
                });
                entity.move_or_interpolate_to(entity.interpolation_position(), y_rot, x_rot);
            } else {
                if let Some(y_rot) = input.y_rot {
                    let current = entity.rotation.y_rot.get();
                    let value = if input.relative_y_rot {
                        current + y_rot
                    } else {
                        y_rot
                    };
                    entity.rotation.y_rot = JavaF32::new(value % 360.0);
                }
                if let Some(x_rot) = input.x_rot {
                    let current = entity.rotation.x_rot.get();
                    let value = if input.relative_x_rot {
                        current + x_rot
                    } else {
                        x_rot
                    };
                    entity.rotation.x_rot = JavaF32::new((value % 360.0).clamp(-90.0, 90.0));
                }
            }
        }
        if let Some(y_head_rot) = input.y_head_rot
            && !self.apply_entity_head_rotation(input.entity_id, y_head_rot)
        {
            return false;
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn apply_entity_position(&mut self, input: EntityPositionInput) -> bool {
        let Some(resolved) = self.resolve_entity_position(&input) else {
            return false;
        };
        self.apply_entity_position_target(
            input.entity_id,
            resolved.current,
            resolved.target,
            resolved.ticking,
            None,
            EntityPositionUpdateMode::PositionSync,
        )
    }
}

impl Entities {
    pub fn apply_motion(&mut self, input: EntityMotionInput) -> bool {
        let Some(mut entity) = self.at_mut(input.entity_id) else {
            return false;
        };
        if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            entity.motion_lerp_target.x = JavaF64::new(input.x);
            entity.motion_lerp_target.y = JavaF64::new(input.y);
            entity.motion_lerp_target.z = JavaF64::new(input.z);
            entity.motion_lerp_steps = REMOTE_PLAYER_MOTION_LERP_STEPS;
        } else {
            entity.motion.x = JavaF64::new(input.x);
            entity.motion.y = JavaF64::new(input.y);
            entity.motion.z = JavaF64::new(input.z);
        }
        true
    }
}

impl ClientWorld {
    /// Apply the three-tick head rotation interpolation target from a
    /// clientbound RotateHead packet to an entity in the client world.
    pub fn apply_entity_head_rotation(&mut self, entity_id: EntityId, y_head_rot: f32) -> bool {
        let Some(mut entity) = self.entities.at_mut(entity_id) else {
            return false;
        };
        if entity.living_kind().is_some() {
            entity.head_rotation_target = JavaF32::new(y_head_rot);
            entity.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        } else {
            entity.rotation.y_head_rot = JavaF32::new(y_head_rot);
        }
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.head_rotation_target = JavaF32::new(y_head_rot);
            player.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        }
        true
    }
}
