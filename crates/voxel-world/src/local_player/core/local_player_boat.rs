use super::*;

impl LocalPlayer {
    /// How many ticks the player has lived, which the model's idle sway
    /// reads.
    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    /// Whether the player only watches.
    pub fn spectating(&self) -> bool {
        self.is_spectator()
    }

    /// Whether the player stands in water.
    pub fn in_water(&self) -> bool {
        self.is_in_water()
    }

    /// What the player model is posed from, between the last two ticks.
    pub fn model_pose(&self, partial_tick: f32) -> ModelPose {
        let lerp = |old: f32, new: f32| old + partial_tick * (new - old);
        ModelPose {
            body_rot: rot_lerp(partial_tick, self.y_body_rot_old, self.y_body_rot),
            head_rot: rot_lerp(partial_tick, self.y_head_rot_old, self.y_head_rot),
            x_rot: self.x_rot,
            walk_position: self.walk.position_at(partial_tick),
            walk_speed: self.walk.speed_at(partial_tick),
            swim_amount: lerp(self.swim_amount_old, self.swim_amount),
            crouching: self.pose == ClientEntityPose::Crouching,
            pose: self.pose,
        }
    }

    /// `ClientAvatarState.getBackwardsInterpolatedWalkDistance` and
    /// `getInterpolatedBob`, which the view bob reads.
    pub fn view_bob(&self, partial_tick: f32) -> (f32, f32) {
        let walked = self.walk_dist - self.walk_dist_old;
        (
            -(self.walk_dist + walked * partial_tick),
            self.bob_old + partial_tick * (self.bob - self.bob_old),
        )
    }

    /// `yBodyRotO` and `yBodyRot`: the body's turn at the last tick and now.
    pub fn body_rotations(&self) -> (f32, f32) {
        (self.y_body_rot_old, self.y_body_rot)
    }

    pub fn interpolated_position(&self, partial_tick: f64) -> Vec3 {
        self.old_position.lerp(self.position, partial_tick)
    }

    /// Run one client tick with the given keys held, and return what it sent.
    pub fn tick(
        &mut self,
        world: &ClientWorld,
        keys: Input,
    ) -> Result<Vec<ServerboundMovement>, Unsupported> {
        let level = Level::new(world);
        self.sent.clear();
        self.input.set_key_states(keys);

        // ClientLevel.tickNonPassenger: the old position first, then the count.
        self.old_position = self.position;
        self.tick_count += 1;

        // AbstractClientPlayer.tick: the avatar state steps first.
        self.walk_dist_old = self.walk_dist;
        self.player_tick(&level)?;

        if self.last_sent_input != self.input.key_presses {
            self.sent
                .push(ServerboundMovement::PlayerInput(self.input.key_presses));
            self.last_sent_input = self.input.key_presses;
        }
        self.send_position();
        self.tick_ambient_sounds(&level);
        Ok(std::mem::take(&mut self.sent))
    }

    /// What the mount the player rides reads of it (`controller.xxa`, `zza` and where it looks).
    pub fn controller(&self) -> crate::Controller {
        crate::Controller {
            xxa: self.xxa,
            zza: self.zza,
            y_rot: self.y_rot,
            x_rot: self.x_rot,
            holds_carrot_on_a_stick: false,
            holds_warped_fungus_on_a_stick: false,
            sprinting: self.sprinting,
        }
    }

    /// One client tick of the player sitting in a vehicle the world has already ticked: what
    /// `LocalPlayer.rideTick` does (the vehicle's own tick came first, as the level ticks the
    /// vehicle before its passengers), with the packets `LocalPlayer.tick` sends of a passenger.
    pub fn tick_riding(
        &mut self,
        world: &ClientWorld,
        keys: Input,
        boat: &crate::VehicleView,
    ) -> Result<RideTick, Unsupported> {
        let level = Level::new(world);
        self.sent.clear();
        self.ride_jump = None;
        self.input.set_key_states(keys);
        self.riding = Some(Riding {
            vehicle_max_y: boat.max_y,
            vehicle_under_water: boat.under_water,
            modifies_fluid_box: boat.kind == crate::VehicleKind::Boat,
            can_sprint: boat.can_sprint,
            can_jump: boat.can_jump,
        });
        // `ClientLevel.tickPassenger`: the old position first, then the count; `Entity.rideTick`
        // stills the player before its own tick.
        self.old_position = self.position;
        self.tick_count += 1;
        self.delta = Vec3::ZERO;
        self.walk_dist_old = self.walk_dist;
        let ticked = self.player_tick(&level);
        if let Err(error) = ticked {
            self.riding = None;
            return Err(error);
        }
        if self.last_sent_input != self.input.key_presses {
            self.sent
                .push(ServerboundMovement::PlayerInput(self.input.key_presses));
            self.last_sent_input = self.input.key_presses;
        }
        // A passenger tells the server which way it looks, and, when it is the vehicle's
        // authority, where the vehicle is.
        self.sent.push(ServerboundMovement::MovePlayerRot {
            y_rot: self.y_rot,
            x_rot: self.x_rot,
            on_ground: self.on_ground,
            horizontal_collision: self.horizontal_collision,
        });
        if boat.controlled {
            self.sent.push(ServerboundMovement::MoveVehicle {
                position: boat.position,
                y_rot: boat.y_rot,
                x_rot: boat.x_rot,
                on_ground: boat.on_ground,
            });
        }
        self.tick_ambient_sounds(&level);
        if self.sprinting != self.was_sprinting {
            self.sent.push(ServerboundMovement::PlayerCommand {
                entity_id: self.entity_id,
                action: if self.sprinting {
                    PlayerCommandAction::StartSprinting
                } else {
                    PlayerCommandAction::StopSprinting
                },
                data: 0,
            });
            self.was_sprinting = self.sprinting;
        }
        // `LivingEntity.rideTick` and then the boat: the fall is forgiven, and the boat is handed the
        // keys, before `positionRider` carries the player to its seat and turns it with the boat.
        self.fall_distance = 0.0;
        let keys = self.input.key_presses;
        let boat_input =
            crate::entity::BoatInput::new(keys.left, keys.right, keys.forward, keys.backward);
        match boat.kind {
            crate::VehicleKind::Boat => self.position_in_boat(boat),
            // `Entity.positionRider`, and `AbstractHorse.positionRider` turning the rider's body
            // with the horse's.
            crate::VehicleKind::Horse => {
                self.set_pos(boat.rider_position);
                self.y_body_rot = boat.y_rot;
            }
            crate::VehicleKind::Pig | crate::VehicleKind::Other => {
                self.set_pos(boat.rider_position);
            }
        }
        self.riding = None;
        Ok(RideTick {
            sent: std::mem::take(&mut self.sent),
            boat_input,
            jump: self.ride_jump.take(),
        })
    }

    /// `AbstractBoat.positionRider` for the player: carried to its seat, turned as the boat turns
    /// and held within a quarter turn of where the boat faces (`clampRotation`).
    pub(in super::super) fn position_in_boat(&mut self, boat: &crate::VehicleView) {
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

    pub(in super::super) fn is_passenger(&self) -> bool {
        self.riding.is_some()
    }

    pub(in super::super) fn player_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        self.no_physics = self.is_spectator();
        if self.is_spectator() || self.is_passenger() {
            self.on_ground = false;
            self.check_supporting_block(level, false, None)?;
        }
        // updateIsUnderwater keeps a copy only the server reads.
        self.living_tick(level)?;
        let clamped_x = self.position.x.clamp(-2.9999999e7, 2.9999999e7);
        let clamped_z = self.position.z.clamp(-2.9999999e7, 2.9999999e7);
        if clamped_x != self.position.x || clamped_z != self.position.z {
            self.set_pos(Vec3::new(clamped_x, self.position.y, clamped_z));
        }
        self.elytra.tick(crate::elytra::ElytraInput {
            fall_flying: false,
            motion: (self.delta.x, self.delta.y, self.delta.z),
            crouching: self.crouching,
        });
        // Player.tick, after the living tick.
        self.attack_strength_ticker += 1;
        self.item_swap_ticker += 1;
        // A different item in the main hand starts the attack strength over.
        if self.main_hand_item != self.last_main_hand_item {
            self.reset_attack_strength();
            self.last_main_hand_item = self.main_hand_item;
        }
        self.update_player_pose(level)
    }

    pub(in super::super) fn living_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        // LivingEntity.baseTick keeps last tick's swing and facings.
        self.attack_anim_old = self.attack_anim;
        self.y_head_rot_old = self.y_head_rot;
        self.y_body_rot_old = self.y_body_rot;
        // Entity.baseTick, as the living base tick runs it.
        self.was_eye_in_water = self.water.eyes_inside;
        self.update_fluid_interaction(level)?;
        self.update_swimming(level);
        if self.is_in_lava() {
            self.fall_distance *= 0.5;
        }
        if self.position.y < f64::from(level.world.dimension().min_y - 64) {
            return unsupported("falling below the world");
        }
        self.first_tick = false;
        // The rest of LivingEntity.baseTick on a client.
        if self.hurt_time > 0 {
            self.hurt_time -= 1;
        }
        // `LivingEntity.tickDeath`: the dying go on counting; a living player starts over.
        self.death_time = if self.health <= 0.0 {
            self.death_time.saturating_add(1)
        } else {
            0
        };
        if self.invulnerable_time > 0 {
            self.invulnerable_time -= 1;
        }
        // LivingEntity.tick: the swimming pose eases in or out.
        self.swim_amount_old = self.swim_amount;
        self.swim_amount = crate::living::ease_swim_amount(
            self.swim_amount,
            self.pose == ClientEntityPose::Swimming,
        );
        self.local_ai_step(level)?;
        self.turn_body();
        Ok(())
    }

    /// The rest of `LivingEntity.tick`: the body turns towards where the
    /// player walks, or where it looks, and never more than 50 degrees from
    /// the head.
    pub(in super::super) fn turn_body(&mut self) {
        crate::living::turn_body(
            (&mut self.y_body_rot, &mut self.y_body_rot_old),
            (self.y_rot, self.y_head_rot, &mut self.y_head_rot_old),
            (
                self.position.x - self.old_position.x,
                self.position.z - self.old_position.z,
            ),
        );
    }
}
