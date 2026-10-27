use super::*;
use crate::input::{LocalPlayerInputState, apply_local_player_input};
use photon_core::math::Vec2;

impl LocalPlayer {
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
        self.begin_avatar_tick();
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
        self.begin_avatar_tick();
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

    pub(super) fn player_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        self.no_physics = self.is_spectator();
        if self.is_spectator() || self.is_passenger() {
            self.on_ground = false;
            self.check_supporting_block(level, false, None)?;
        }
        self.update_is_underwater();
        self.living_tick(level)?;
        let clamped_x = self.position.x.clamp(-2.9999999e7, 2.9999999e7);
        let clamped_z = self.position.z.clamp(-2.9999999e7, 2.9999999e7);
        if clamped_x != self.position.x || clamped_z != self.position.z {
            self.set_pos(Vec3::new(clamped_x, self.position.y, clamped_z));
        }
        self.elytra.fall_fly_ticks = if self.elytra.fall_flying {
            self.elytra.fall_fly_ticks.saturating_add(1)
        } else {
            0
        };
        self.elytra.tick(crate::elytra::ElytraInput {
            fall_flying: self.elytra.fall_flying,
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
        if self.is_passenger() {
            self.reset_bob();
        }
        self.update_player_pose(level)
    }

    pub(super) fn living_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        // LivingEntity.baseTick keeps last tick's swing and facings.
        self.attack_anim_old = self.attack_anim;
        self.view.y_rot_old = self.y_rot;
        self.y_head_rot_old = self.y_head_rot;
        self.y_body_rot_old = self.y_body_rot;
        // Entity.baseTick, as the living base tick runs it: the sprint flake reads last tick's
        // fluids, so it comes before the fluid update.
        self.throw_sprint_particle(level);
        self.eyes.was_in_water = self.water.eyes_inside;
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
        self.throw_effect_particles(level);
        // LivingEntity.tick: the swimming pose eases in or out.
        let swim_amount = self.swim_amount.0;
        self.swim_amount = (
            crate::living::ease_swim_amount(swim_amount, self.is_visually_swimming()),
            swim_amount,
        );
        self.local_ai_step(level)?;
        self.turn_body();
        Ok(())
    }

    pub fn delta_movement(&self) -> Vec3 {
        self.delta
    }

    /// `Entity.setDeltaMovement`: the motion a server sets (`Entity.lerpMotion` of the local
    /// player) replaces the one it has at once.
    pub fn set_delta_movement(&mut self, motion: Vec3) {
        self.delta = motion;
    }

    /// `Entity.addDeltaMovement`: a push on top of the motion it has (an explosion's).
    pub fn add_delta_movement(&mut self, push: Vec3) {
        self.delta = self.delta.add_vec3(push);
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn fall_distance(&self) -> f64 {
        self.fall_distance
    }

    pub(super) fn fluid_jump_threshold(&self) -> f64 {
        if self.dimensions.eye_height() < 0.4 {
            0.0
        } else {
            0.4
        }
    }

    pub(super) fn handle_on_climbable(
        &mut self,
        level: &Level<'_>,
        delta: Vec3,
    ) -> Result<Vec3, Unsupported> {
        if !self.on_climbable(level) {
            return Ok(delta);
        }
        self.fall_distance = 0.0;
        let max = f64::from(0.15_f32);
        let x = java_math::clamp(delta.x, -max, max);
        let z = java_math::clamp(delta.z, -max, max);
        let mut y = java_math::max(delta.y, -max);
        if y < 0.0 && self.is_shift_key_down() && !self.abilities.flying {
            if level.info(self.block_position()).name == "minecraft:scaffolding" {
                return unsupported("climbing scaffolding");
            }
            y = 0.0;
        }
        Ok(Vec3::new(x, y, z))
    }

    pub(super) fn on_climbable(&self, level: &Level<'_>) -> bool {
        if self.is_spectator() {
            return false;
        }
        let pos = self.block_position();
        let info = level.info(pos);
        if self.elytra.fall_flying && info.flags & flag::CAN_GLIDE_THROUGH != 0 {
            return false;
        }
        if info.flags & flag::CLIMBABLE != 0 {
            return true;
        }
        if info.flags & flag::TRAPDOOR != 0 {
            let state = level.state_id(pos);
            let open = property(state, "open") == Some("true");
            if open {
                let below_pos = BlockPos::new(pos.x, pos.y - 1, pos.z);
                let below = level.info(below_pos);
                if below.flags & flag::LADDER != 0 {
                    return property(level.state_id(below_pos), "facing")
                        == property(state, "facing");
                }
            }
        }
        false
    }

    pub(super) fn block_jump_factor(&self, level: &Level<'_>) -> f32 {
        let here = level.info(self.block_position()).jump_factor;
        let below = level.info(self.get_on_pos(level, 0.500_001)).jump_factor;
        if here == 1.0 { below } else { here }
    }

    pub(super) fn jump_from_ground(&mut self, level: &Level<'_>) {
        let jump_power = self.jump_power(level);
        if jump_power <= 1.0e-5 {
            return;
        }
        self.delta = Vec3::new(
            self.delta.x,
            java_math::max(f64::from(jump_power), self.delta.y),
            self.delta.z,
        );
        if self.sprinting {
            let angle = self.y_rot * MTH_DEG_TO_RAD;
            self.delta = self.delta.add_vec3(Vec3::new(
                f64::from(-mth_sin_f64(f64::from(angle))) * 0.2,
                0.0,
                f64::from(mth_cos_f64(f64::from(angle))) * 0.2,
            ));
        }
    }

    /// `LocalPlayer.applyInput`, which only the camera entity takes input for (oracle-checked as
    /// `apply_local_player_input`).
    fn apply_input(&mut self) {
        let applied = apply_local_player_input(
            LocalPlayerInputState {
                move_vector: self.input.move_vector(),
                jump_pressed: self.input.key_presses.jump,
                current_impulses: Vec2::new(self.xxa, self.zza),
                current_jumping: self.jumping,
                current_bob: Vec2::new(self.x_bob, self.y_bob),
                previous_bob: Vec2::new(self.x_bob_old, self.y_bob_old),
                rotation: Vec2::new(self.x_rot, self.y_rot),
                controlled_camera: self.view.controlled_camera,
            },
            LocalPlayerInputModifiers {
                using_item: self.is_using_item(),
                passenger: self.is_passenger(),
                item_use_speed_multiplier: self.item_use_speed_multiplier(),
                moving_slowly: self.is_moving_slowly(),
                sneaking_movement_factor: self.attribute("minecraft:sneaking_speed") as f32,
            },
        );
        self.xxa = applied.movement.x;
        self.zza = applied.movement.y;
        self.jumping = applied.jumping;
        self.x_bob = applied.x_bob;
        self.y_bob = applied.y_bob;
        self.x_bob_old = applied.x_bob_old;
        self.y_bob_old = applied.y_bob_old;
    }

    /// The vertical input of a flying player, which only the camera entity gives.
    fn fly_vertically(&mut self) {
        if !self.abilities.flying || !self.view.controlled_camera {
            return;
        }
        let input_ya =
            i32::from(self.input.key_presses.jump) - i32::from(self.input.key_presses.shift);
        if input_ya != 0 {
            let rise = input_ya as f32 * self.abilities.flying_speed() * 3.0_f32;
            self.delta = self.delta.add_components(0.0, f64::from(rise), 0.0);
        }
    }

    pub(super) fn local_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        self.tick_start_timers();

        let was_jumping = self.input.key_presses.jump;
        let was_shift_key_down = self.input.key_presses.shift;
        let had_forward_impulse = self.input.move_vector().y > 1.0e-5;
        self.crouching = !self.abilities.flying
            && !self.is_swimming()
            && !self.is_passenger()
            && self.can_fit(level, ClientEntityPose::Crouching)?
            && (self.is_shift_key_down() || !self.can_fit(level, ClientEntityPose::Standing)?);
        self.input.tick();
        let mut was_auto_jump = false;
        if self.auto_jump_time > 0 {
            self.auto_jump_time -= 1;
            was_auto_jump = true;
            self.input.key_presses.jump = true;
        }

        if !self.no_physics {
            let half = f64::from(self.dimensions.width()) * 0.35;
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x - half, z + half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x - half, z - half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x + half, z - half);
            let (x, z) = (self.position.x, self.position.z);
            self.move_towards_closest_space(level, x + half, z + half);
        }

        self.open_sprint_window(was_shift_key_down, had_forward_impulse);

        if self.sprinting {
            if self.is_swimming() {
                if self.should_stop_swim_sprinting() {
                    self.set_sprinting(false);
                }
            } else if self.should_stop_run_sprinting() {
                self.set_sprinting(false);
            }
        }

        let mut just_toggled_creative_flight = false;
        if self.abilities.may_fly {
            if self.is_spectator() {
                if !self.abilities.flying {
                    self.abilities.flying = true;
                    just_toggled_creative_flight = true;
                    self.on_update_abilities();
                }
            } else if !was_jumping && self.input.key_presses.jump && !was_auto_jump {
                if self.jump_trigger_time == 0 {
                    self.jump_trigger_time = 7;
                } else if !self.is_swimming() {
                    self.abilities.flying = !self.abilities.flying;
                    if self.abilities.flying && self.on_ground {
                        self.jump_from_ground(level);
                    }
                    just_toggled_creative_flight = true;
                    self.on_update_abilities();
                    self.jump_trigger_time = 0;
                }
            }
        }
        self.start_fall_flying_on_jump(level, (was_jumping, just_toggled_creative_flight));

        if self.is_in_water() && self.input.key_presses.shift && self.is_affected_by_fluids() {
            self.delta = self.delta.add_components(0.0, f64::from(-0.04_f32), 0.0);
        }

        self.fly_vertically();

        // `jumpableVehicle`: holding the jump key charges a jump of the mount (a saddled horse),
        // which letting go of it makes, and tells the server of.
        if self.riding.is_some_and(|riding| riding.can_jump) {
            if self.jump_riding_ticks < 0 {
                self.jump_riding_ticks += 1;
                if self.jump_riding_ticks == 0 {
                    self.jump_riding_scale = 0.0;
                }
            }
            if was_jumping && !self.input.key_presses.jump {
                self.jump_riding_ticks = -10;
                let charge = floor_f64(f64::from(self.jump_riding_scale * 100.0));
                self.ride_jump = Some(charge);
                self.sent.push(ServerboundMovement::PlayerCommand {
                    entity_id: self.entity_id,
                    action: PlayerCommandAction::StartRidingJump,
                    data: charge,
                });
            } else if !was_jumping && self.input.key_presses.jump {
                self.jump_riding_ticks = 0;
                self.jump_riding_scale = 0.0;
            } else if was_jumping {
                self.jump_riding_ticks += 1;
                if self.jump_riding_ticks < 10 {
                    self.jump_riding_scale = self.jump_riding_ticks as f32 * 0.1;
                } else {
                    self.jump_riding_scale = 0.8 + 2.0 / (self.jump_riding_ticks - 9) as f32 * 0.1;
                }
            }
        } else {
            self.jump_riding_scale = 0.0;
        }

        // AbstractClientPlayer.aiStep: the bob eases towards the ground speed.
        let target_bob = if self.on_ground && !self.is_swimming() {
            (self.delta.horizontal_distance() as f32).min(0.1)
        } else {
            0.0
        };
        self.bob_old = self.bob;
        self.bob += (target_bob - self.bob) * 0.4;
        self.player_ai_step(level)?;
        if self.on_ground && self.abilities.flying && !self.is_spectator() {
            self.abilities.flying = false;
            self.on_update_abilities();
        }
        Ok(())
    }

    pub(super) fn player_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if self.jump_trigger_time > 0 {
            self.jump_trigger_time -= 1;
        }
        if self.abilities.flying {
            self.fall_distance = 0.0;
        }
        self.end_use_of_changed_item();
        self.living_ai_step(level)?;
        // `updateSwingTime`: the swing lasts as long as the swinging hand's item says.
        let swing_duration = self.current_swing_duration();
        if self.swinging {
            self.swing_time += 1;
            if self.swing_time >= swing_duration {
                self.swing_time = 0;
                self.swinging = false;
            }
        } else {
            self.swing_time = 0;
        }
        self.attack_anim = self.swing_time as f32 / swing_duration as f32;
        if self.hands.using_item {
            self.consume_tick();
        }
        // `getTicksUsingItem`: counted while the item is in use.
        self.use_edge.ticks = if self.hands.using_item {
            self.use_edge.ticks.saturating_add(1)
        } else {
            0
        };
        self.y_head_rot = self.y_rot;
        Ok(())
    }

    pub(super) fn living_ai_step(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
        if self.no_jump_delay > 0 {
            self.no_jump_delay -= 1;
        }

        let mut dx = self.delta.x;
        let mut dy = self.delta.y;
        let mut dz = self.delta.z;
        if self.delta.horizontal_distance_squared() < 9.0e-6 {
            dx = 0.0;
            dz = 0.0;
        }
        if self.delta.y.abs() < 0.003 {
            dy = 0.0;
        }
        self.delta = Vec3::new(dx, dy, dz);

        self.apply_input();
        // `LivingEntity.aiStep`: a sleeper is immobile.
        if self.is_sleeping() {
            (self.jumping, self.xxa, self.zza) = (false, 0.0, 0.0);
        }

        self.jump_or_rise_in_fluid(level);

        let input = Vec3::new(f64::from(self.xxa), 0.0, f64::from(self.zza));
        // `aiStep`: slow falling and levitation keep a fall from building up.
        if self.hands.slow_falling || self.hands.levitation.is_some() {
            self.fall_distance = 0.0;
        }
        self.travel_and_spin(level, input)?;
        // `calculateEntityAnimation`: the limbs swing with the distance
        // moved across the ground this tick.
        let x = self.position.x - self.old_position.x;
        let z = self.position.z - self.old_position.z;
        if self.is_passenger() {
            self.walk.stop();
        } else {
            self.walk.follow((x * x + z * z).sqrt(), false);
        }
        Ok(())
    }

    pub(super) fn jump_out_of_fluid(
        &mut self,
        level: &Level<'_>,
        old_y: f64,
    ) -> Result<(), Unsupported> {
        let movement = self.delta;
        if self.horizontal_collision {
            let lift = movement.y + f64::from(0.6_f32) - self.position.y + old_y;
            let moved = self.bounding_box.move_by(movement.x, lift, movement.z);
            if level.no_collision(moved, self.fall_distance, self.position)?
                && !self.contains_any_liquid(level, moved)
            {
                self.delta = Vec3::new(movement.x, f64::from(0.3_f32), movement.z);
            }
        }
        Ok(())
    }
}
