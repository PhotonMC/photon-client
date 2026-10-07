use super::*;

impl ClientWorld {
    pub fn apply_entity_position_and_rotation(
        &mut self,
        input: EntityPositionInput,
        rotation: EntityRotationInput,
        mode: EntityPositionUpdateMode,
    ) -> bool {
        let Some(resolved) = self.resolve_entity_position(&input) else {
            return false;
        };
        let Some(entity) = self.entities.get(&input.entity_id) else {
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
            self.entities.get(&input.entity_id).map(|entity| {
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
        if applied && let Some(entity) = self.entities.get_mut(&input.entity_id) {
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
        let ticking = new_position.map(|position| {
            let (chunk_x, chunk_z) = entity_chunk_key(position);
            self.contains_chunk(chunk_x, chunk_z)
        });
        let position;
        {
            let entity = self
                .entities
                .get_mut(&input.entity_id)
                .expect("entity checked above");
            if let Some(position) = new_position {
                entity.set_packet_base(position);
            }
            if entity.has_interpolation_handler() {
                let target_position =
                    new_position.unwrap_or_else(|| entity.interpolation_position());
                let (current_y_rot, current_x_rot) = entity.interpolation_rotation();
                let target_y_rot = input.y_rot.unwrap_or(current_y_rot);
                let target_x_rot = input.x_rot.unwrap_or(current_x_rot);
                entity.move_or_interpolate_to(target_position, target_y_rot, target_x_rot);
            } else {
                if input.has_position {
                    let (x, y, z) = new_position.expect("position was requested");
                    entity.position.x = JavaF64::new(x);
                    entity.position.y = JavaF64::new(y);
                    entity.position.z = JavaF64::new(z);
                    entity.ticking =
                        entity.always_ticking || ticking.expect("position ticking state");
                }
                if let Some(y_rot) = input.y_rot {
                    entity.y_rot = JavaF32::new(y_rot % 360.0);
                }
                if let Some(x_rot) = input.x_rot {
                    entity.x_rot = JavaF32::new((x_rot % 360.0).clamp(-90.0, 90.0));
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
        entity_id: i32,
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

    pub(in super::super) fn resolve_entity_position(
        &self,
        input: &EntityPositionInput,
    ) -> Option<ResolvedEntityPosition> {
        let current = self
            .entities
            .get(&input.entity_id)
            .map(|entity| entity.interpolation_position())?;
        let target = (
            if input.relative_flags & ENTITY_RELATIVE_X_FLAG != 0 {
                current.0 + input.x
            } else {
                input.x
            },
            if input.relative_flags & ENTITY_RELATIVE_Y_FLAG != 0 {
                current.1 + input.y
            } else {
                input.y
            },
            if input.relative_flags & ENTITY_RELATIVE_Z_FLAG != 0 {
                current.2 + input.z
            } else {
                input.z
            },
        );
        let (chunk_x, chunk_z) = entity_chunk_key(target);
        Some(ResolvedEntityPosition {
            current,
            target,
            ticking: self.contains_chunk(chunk_x, chunk_z),
        })
    }

    pub(in super::super) fn apply_entity_position_target(
        &mut self,
        entity_id: i32,
        current_position: (f64, f64, f64),
        target_position: (f64, f64, f64),
        ticking: bool,
        target_rotation: Option<(f32, f32)>,
        mode: EntityPositionUpdateMode,
    ) -> bool {
        let Some(previous_position) = self.entities.get(&entity_id).map(ClientEntity::position)
        else {
            return false;
        };
        let position;
        {
            let entity = self
                .entities
                .get_mut(&entity_id)
                .expect("entity position was checked above");
            if mode == EntityPositionUpdateMode::PositionSync {
                entity.set_packet_base(target_position);
            }
            if entity.has_interpolation_handler() {
                let rotation = target_rotation.unwrap_or_else(|| entity.interpolation_rotation());
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
                entity.position.x = JavaF64::new(target_position.0);
                entity.position.y = JavaF64::new(target_position.1);
                entity.position.z = JavaF64::new(target_position.2);
                if let Some((y_rot, x_rot)) = target_rotation {
                    entity.y_rot = JavaF32::new(y_rot % 360.0);
                    entity.x_rot = JavaF32::new((x_rot % 360.0).clamp(-90.0, 90.0));
                }
            }
            entity.ticking = entity.always_ticking || ticking;
            position = entity.position();
        }
        self.update_entity_section(entity_id, previous_position, position);
        true
    }

    pub fn section_counts_at(
        &self,
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
    ) -> Option<(i16, i16)> {
        let chunk = self.chunk_for_query(chunk_x, chunk_z)?;
        let section = chunk.sections.get(&section_y)?;
        Some((section.non_empty_block_count, section.fluid_count))
    }

    /// `getNoiseBiome`: the biome at a quart position, its row clamped into
    /// the dimension.
    pub fn noise_biome(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        self.biome_at(x * 4, self.dimension.clamp_quart_y(y) * 4, z * 4)
    }
}
