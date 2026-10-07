use super::*;

impl LocalPlayer {
    /// A player the connection just made, snapped to its spawn pose, with the
    /// game mode's abilities applied.
    pub fn new(
        entity_id: i32,
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        game_mode: ClientPlayerGameMode,
        auto_jump_option: bool,
    ) -> Self {
        let pose = ClientEntityPose::Standing;
        let dimensions = pose.player_dimensions();
        Self {
            entity_id,
            position,
            old_position: position,
            delta: Vec3::ZERO,
            y_rot,
            x_rot,
            bounding_box: dimensions.make_bounding_box(position),
            pose,
            dimensions,
            on_ground: false,
            horizontal_collision: false,
            vertical_collision: false,
            vertical_collision_below: false,
            minor_horizontal_collision: false,
            main_supporting_block: None,
            on_ground_no_blocks: false,
            fall_distance: 0.0,
            stuck_speed_multiplier: Vec3::ZERO,
            walk_dist: 0.0,
            walk_dist_old: 0.0,
            // The reference starts a new player facing at random; this one
            // starts with head and body where it looks.
            y_body_rot: y_rot,
            y_body_rot_old: y_rot,
            y_head_rot: y_rot,
            y_head_rot_old: y_rot,
            walk: crate::living::WalkAnimation::default(),
            riding: None,
            jump_riding_ticks: 0,
            jump_riding_scale: 0.0,
            ride_jump: None,
            x_bob: 0.0,
            y_bob: 0.0,
            x_bob_old: 0.0,
            y_bob_old: 0.0,
            swinging: false,
            swing_time: 0,
            swinging_off_hand: false,
            attack_anim: 0.0,
            attack_anim_old: 0.0,
            attack_strength_ticker: 0,
            elytra: crate::elytra::ElytraState::default(),
            main_hand_item: 0,
            last_main_hand_item: 0,
            hands: HandState::default(),
            using_item_ticks: 0,
            item_swap_ticker: 0,
            health: 20.0,
            flash_on_set_health: false,
            invulnerable_time: 0,
            hurt_time: 0,
            hurt_duration: 0,
            hurt_dir: 0.0,
            death_time: 0,
            swim_amount: 0.0,
            swim_amount_old: 0.0,
            bob: 0.0,
            bob_old: 0.0,
            first_tick: true,
            tick_count: 0,
            random: voxel_core::random::RandomSource::create(i64::from(entity_id)),
            was_in_bubble_column: false,
            ambient_first_tick: true,
            move_dist: 0.0,
            next_step: 1.0,
            pending_sounds: Vec::new(),
            was_touching_water: false,
            was_eye_in_water: false,
            water: FluidTracker::default(),
            lava: FluidTracker::default(),
            sprinting: false,
            swimming: false,
            crouching: false,
            input: KeyboardInput::new(Input::EMPTY),
            xxa: 0.0,
            zza: 0.0,
            jumping: false,
            no_jump_delay: 0,
            jump_trigger_time: 0,
            sprint_trigger_time: 0,
            auto_jump_time: 0,
            abilities: game_mode.update_player_abilities(ClientPlayerAbilities::default()),
            game_mode,
            no_physics: false,
            auto_jump_option,
            auto_jump_enabled: true,
            movements_this_tick: Vec::new(),
            x_last: 0.0,
            y_last: 0.0,
            z_last: 0.0,
            y_rot_last: 0.0,
            x_rot_last: 0.0,
            last_on_ground: false,
            last_horizontal_collision: false,
            was_sprinting: false,
            position_reminder: 0,
            last_sent_input: Input::EMPTY,
            sent: Vec::new(),
        }
    }

    pub fn entity_id(&self) -> i32 {
        self.entity_id
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn old_position(&self) -> Vec3 {
        self.old_position
    }

    pub fn delta_movement(&self) -> Vec3 {
        self.delta
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

    /// `Entity.getBoundingBox`.
    pub fn bounding_box(&self) -> voxel_core::aabb::Aabb {
        self.bounding_box
    }

    pub fn eye_height(&self) -> f32 {
        self.dimensions.eye_height()
    }

    /// `getAttributeValue(MOVEMENT_SPEED)` as the field of view reads it.
    pub fn movement_speed(&self) -> f32 {
        self.speed()
    }

    pub fn abilities(&self) -> ClientPlayerAbilities {
        self.abilities
    }

    /// Whether the eyes are inside water, as the last fluid update found.
    pub fn is_eye_in_water(&self) -> bool {
        self.water.eyes_inside
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn horizontal_collision(&self) -> bool {
        self.horizontal_collision
    }

    pub fn vertical_collision(&self) -> bool {
        self.vertical_collision
    }

    pub fn minor_horizontal_collision(&self) -> bool {
        self.minor_horizontal_collision
    }

    pub fn is_sprinting(&self) -> bool {
        self.sprinting
    }

    /// `LocalPlayer.isShiftKeyDown`: the input, not the synced flag.
    pub fn is_shift_key_down(&self) -> bool {
        self.input.key_presses.shift
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    pub fn fall_distance(&self) -> f64 {
        self.fall_distance
    }

    pub fn is_flying(&self) -> bool {
        self.abilities.flying
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

    /// `Entity.isColliding`: whether a block's collision shape at `pos`
    /// overlaps the player.
    pub fn is_colliding(&self, world: &ClientWorld, pos: BlockPos, state: i32) -> bool {
        let shape = world
            .block_state_registry()
            .collision_shape_at(state, [pos.x, pos.y, pos.z])
            .move_by_block_pos(pos);
        let Ok(player) = Shapes::create_from_aabb(self.bounding_box) else {
            return false;
        };
        Shapes::join_is_not_empty(&shape, &player, BooleanOp::And)
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

    /// `MultiPlayerGameMode.setLocalMode`: the new mode's abilities.
    pub fn set_game_mode(&mut self, game_mode: ClientPlayerGameMode) {
        self.game_mode = game_mode;
        self.abilities = game_mode.update_player_abilities(self.abilities);
    }

    /// `handlePlayerAbilities`: every field but the build permission.
    pub fn set_abilities(&mut self, abilities: ClientPlayerAbilities) {
        let may_build = self.abilities.may_build;
        self.abilities = abilities;
        self.abilities.may_build = may_build;
    }

    /// The feet position between the last two ticks.
    /// `LocalPlayer.hurtTo`: the server set the health. A loss starts the
    /// hurt flash and a second of immunity, a gain half a second; the very
    /// first update only sets it.
    pub fn hurt_to(&mut self, new_health: f32) {
        if self.flash_on_set_health {
            let damage = self.health - new_health;
            if damage <= 0.0 {
                self.health = new_health;
                if damage < 0.0 {
                    self.invulnerable_time = 10;
                }
            } else {
                self.invulnerable_time = 20;
                self.health = new_health;
                self.hurt_time = 10;
                self.hurt_duration = 10;
            }
        } else {
            self.health = new_health;
            self.flash_on_set_health = true;
        }
    }

    /// `Player.animateHurt`: the flinch starts, away from where the blow came from.
    pub fn animate_hurt(&mut self, yaw: f32) {
        self.hurt_duration = 10;
        self.hurt_time = 10;
        self.hurt_dir = yaw;
    }

    /// What the camera reads of the player when the view is turned by a blow (`Camera.extractRenderState`):
    /// the flinch left, the direction, whether it is dying, and how long.
    pub fn hurt_view(&self, partial_tick: f32) -> (f32, i32, f32, bool, f32) {
        (
            self.hurt_time as f32 - partial_tick,
            self.hurt_duration,
            self.hurt_dir,
            self.health <= 0.0,
            self.death_time as f32 + partial_tick,
        )
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    /// Ticks of damage immunity left, which the HUD's hearts blink by.
    pub fn invulnerable_time(&self) -> i32 {
        self.invulnerable_time
    }

    /// `Player.getAttackStrengthScale`: how far the attack has recharged,
    /// over the attack delay the attack speed sets.
    pub fn attack_strength_scale(&self, partial_tick: f32, attack_speed: f64) -> f32 {
        let delay = attack_strength_delay(attack_speed);
        ((self.attack_strength_ticker as f32 + partial_tick) / delay).clamp(0.0, 1.0)
    }

    /// `LivingEntity.swing` for the main hand: start a swing, unless one is
    /// in its first half.
    pub fn swing(&mut self) {
        self.swing_hand(false);
    }

    /// `LivingEntity.swing(hand)`: start a swing of that hand's arm, unless one is under way and
    /// still in its first half.
    pub fn swing_hand(&mut self, off_hand: bool) {
        if !self.swinging
            || self.swing_time >= self.current_swing_duration() / 2
            || self.swing_time < 0
        {
            self.swing_time = -1;
            self.swinging = true;
            self.swinging_off_hand = off_hand;
        }
    }

    /// `LivingEntity.getCurrentSwingDuration`: how long the swinging hand's item swings, less what
    /// haste or the conduit's power takes off and plus what mining fatigue adds.
    pub fn current_swing_duration(&self) -> i32 {
        let item = if self.swinging_off_hand {
            self.hands.off_item
        } else {
            self.hands.main_item
        };
        let base = crate::block_rules::item(item)
            .and_then(|item| voxel_data::spear::spear_of(item.name))
            .map_or(SWING_DURATION, |spear| spear.swing_ticks);
        if let Some(amplifier) = self.hands.dig_speed {
            base - (1 + amplifier)
        } else if let Some(amplifier) = self.hands.mining_fatigue {
            base + (1 + amplifier) * 2
        } else {
            base
        }
    }

    /// Whether the off hand's arm is the one swinging (or last swung): `attackArm`.
    pub fn swinging_off_hand(&self) -> bool {
        self.swinging_off_hand
    }

    pub fn is_swinging(&self) -> bool {
        self.swinging
    }

    /// `LivingEntity.swingTime`.
    pub fn swing_time(&self) -> i32 {
        self.swing_time
    }

    /// What the hands hold, the effects and the use of an item, from the world's player state.
    pub fn set_hand_state(&mut self, hands: HandState) {
        self.main_hand_item = hands.main_item;
        self.hands = hands;
    }

    /// `LivingEntity.isUsingItem`, as the server's flag says.
    pub fn is_using_item(&self) -> bool {
        self.hands.using_item
    }

    /// Whether the off hand is the one using the item.
    pub fn used_item_is_off_hand(&self) -> bool {
        self.hands.using_off_hand
    }

    /// `LivingEntity.getTicksUsingItem(partialTicks)`.
    pub fn ticks_using_item(&self, partial_tick: f32) -> f32 {
        if self.hands.using_item {
            self.using_item_ticks as f32 + partial_tick
        } else {
            0.0
        }
    }

    /// The item now in the main hand, by registry id; air when empty.
    pub fn set_main_hand_item(&mut self, item: i32) {
        self.main_hand_item = item;
        self.hands.main_item = item;
    }

    pub fn attack_strength_ticker(&self) -> i32 {
        self.attack_strength_ticker
    }

    /// `Player.resetAttackStrengthTicker`.
    pub fn reset_attack_strength(&mut self) {
        self.attack_strength_ticker = 0;
        self.item_swap_ticker = 0;
    }

    /// `LivingEntity.getAttackAnim`: how far through the swing, between the
    /// last two ticks, wrapping when a new swing began.
    pub fn attack_anim(&self, partial_tick: f32) -> f32 {
        let mut difference = self.attack_anim - self.attack_anim_old;
        if difference < 0.0 {
            difference += 1.0;
        }
        self.attack_anim_old + difference * partial_tick
    }

    /// `Player.getItemSwapScale`: how far the hand has come back up after
    /// an attack or a change of item, over the attack delay the attack
    /// speed sets (five ticks at the default speed of four).
    pub fn item_swap_scale(&self, partial_tick: f32, attack_speed: f64) -> f32 {
        let delay = (1.0 / attack_speed * 20.0) as f32;
        ((self.item_swap_ticker as f32 + partial_tick) / delay).clamp(0.0, 1.0)
    }

    /// The hand's lagging pitch and yaw between the last two ticks.
    pub fn hand_bob(&self, partial_tick: f32) -> (f32, f32) {
        (
            self.x_bob_old + partial_tick * (self.x_bob - self.x_bob_old),
            self.y_bob_old + partial_tick * (self.y_bob - self.y_bob_old),
        )
    }

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

    pub(super) fn is_passenger(&self) -> bool {
        self.riding.is_some()
    }

    pub(super) fn player_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
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

    pub(super) fn living_tick(&mut self, level: &Level<'_>) -> Result<(), Unsupported> {
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
}
