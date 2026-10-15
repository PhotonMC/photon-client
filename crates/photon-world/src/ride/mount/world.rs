use super::*;

impl ClientWorld {
    /// Which mount the local player drives this entity as, if it does: a saddled horse it sits
    /// in the first seat of, or a saddled pig it steers with a carrot on a stick.
    pub(crate) fn mount_kind_of(&self, entity: &crate::entity::ClientEntity) -> Option<MountKind> {
        let player = self.local_player.as_ref()?;
        if entity.passenger_ids.first() != Some(&player.id) {
            return None;
        }
        if entity.is_controllable_horse() {
            Some(if entity.is_camel() {
                MountKind::Camel
            } else {
                MountKind::Horse
            })
        } else if entity.is_saddled_pig() && self.rider_controls.holds_carrot_on_a_stick {
            Some(MountKind::Pig)
        } else if entity.is_saddled_strider() && self.rider_controls.holds_warped_fungus_on_a_stick
        {
            Some(MountKind::Strider)
        } else {
            None
        }
    }

    /// What the rider's keys make of the mount's input, as the next tick of the mount reads them.
    pub fn set_rider_controls(&mut self, controls: Controller) {
        self.rider_controls = controls;
    }

    /// `AbstractHorse.onPlayerJump`: the rider let go of the jump key after charging a jump.
    pub fn on_player_jump(&mut self, vehicle_id: i32, jump_amount: i32) {
        let Some(horse) = self
            .entities
            .at_mut(vehicle_id)
            .filter(|entity| entity.is_controllable_horse())
        else {
            return;
        };
        let camel = horse.is_camel();
        let on_ground = horse.on_ground;
        let state = horse
            .kind
            .horse_or_insert_with(|| Box::new(crate::mobs::horse::HorseState::new()));
        // `Camel.onPlayerJump`: only when it has rested from its last dash and stands on the ground.
        if camel && !(state.camel.dash_cooldown() <= 0 && on_ground) {
            return;
        }
        let amount = if jump_amount < 0 {
            0
        } else {
            state.set_allow_stand_sliding(true);
            // A camel does not rear (`Camel.canPerformRearing`).
            if !camel {
                state.stand_if_possible();
            }
            jump_amount
        };
        // `PlayerRideableJumping.getPlayerJumpPendingScale`.
        horse.mount_ride.jump_pending_scale = if amount >= 90 {
            1.0
        } else {
            0.4 + 0.4 * amount as f32 / 90.0
        };
    }

    /// `LivingEntity.tick` for the saddled horse the local player rides.
    pub(crate) fn tick_ridden_mount(&mut self, entity_id: i32) {
        let Some(entity) = self.entities.at(entity_id) else {
            return;
        };
        let Some(pick) = photon_data::entity_type::pick(entity.type_id) else {
            return;
        };
        let (width, height, eye_height) = pick.size;
        // A camel that sits is a good deal lower (`Camel.getDefaultDimensions`).
        let (height, eye_height) =
            if entity.is_camel() && entity.pose() == crate::entity::ClientEntityPose::Sitting {
                (height - 1.43, 0.845)
            } else {
                (height, eye_height)
            };
        let dimensions = ClientEntityDimensions::scalable(width, height)
            .with_eye_height(eye_height)
            .scale(entity.scale(), entity.scale());
        let Some(kind) = self.mount_kind_of(entity) else {
            return;
        };
        if kind == MountKind::Strider {
            // `Strider.tick`: it takes the cold or the warmth before it moves.
            let suffocating = strider_is_suffocating(self, entity);
            if let Some(strider) = self.entities.at_mut(entity_id) {
                strider.set_strider_suffocating(suffocating);
            }
        }
        let Some(entity) = self.entities.at(entity_id) else {
            return;
        };
        let attributes = mount_attributes(entity, kind);
        let (x, y, z) = entity.position();
        let (ox, oy, oz) = entity.previous_position();
        let mut body = MountBody::new(
            kind,
            Vec3::new(x, y, z),
            Vec3::new(ox, oy, oz),
            dimensions,
            attributes,
            entity.mount_ride,
        );
        let (mx, my, mz) = entity.motion();
        body.delta = Vec3::new(mx, my, mz);
        body.y_rot = entity.y_rot.get();
        body.x_rot = entity.x_rot.get();
        body.on_ground = entity.on_ground;
        body.supporting_block = entity.tracking.support.main_block;
        body.on_ground_no_blocks = entity.tracking.support.no_blocks;
        body.standing = entity.horse().is_some_and(|horse| horse.is_standing());
        body.allow_stand_sliding = entity
            .horse()
            .is_some_and(|horse| horse.allow_stand_sliding());
        body.floats_when_ridden = matches!(kind, MountKind::Horse | MountKind::Camel);
        body.controller = self.rider_controls;
        if kind == MountKind::Camel {
            let game_time = self.level.game_time as i64;
            if let Some(horse) = entity.horse() {
                body.camel = CamelRide {
                    sitting: horse.camel.sitting(),
                    refuses_to_move: horse.camel.refuses_to_move(game_time),
                    in_pose_transition: horse.camel.in_pose_transition(game_time),
                    cooldown: horse.camel.dash_cooldown(),
                    ..CamelRide::default()
                };
            }
        }
        if body.tick(&Level::new(self)).is_err() {
            return;
        }
        let game_time = self.level.game_time as i64;
        let in_water = body.is_in_water();
        let entity = self
            .entities
            .at_mut(entity_id)
            .expect("the mount was read above");
        entity.set_position((body.position.x, body.position.y, body.position.z));
        entity.motion.x = JavaF64::new(body.delta.x);
        entity.motion.y = JavaF64::new(body.delta.y);
        entity.motion.z = JavaF64::new(body.delta.z);
        // `tickRidden`: it faces where the rider does, body and head too.
        entity.old_y_rot = JavaF32::new(body.y_rot);
        entity.y_rot = JavaF32::new(body.y_rot);
        entity.x_rot = JavaF32::new(body.x_rot);
        entity.y_body_rot = JavaF32::new(body.y_rot);
        entity.y_head_rot = JavaF32::new(body.y_rot);
        entity.on_ground = body.on_ground;
        entity.tracking.support = Support {
            main_block: body.supporting_block,
            no_blocks: body.on_ground_no_blocks,
        };
        entity.mount_ride = body.state;
        if kind == MountKind::Camel {
            write_back_camel(entity, &body.camel, in_water, game_time);
        }
    }
}
