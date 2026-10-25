use super::*;

/// The loop that plays while the player glides.
pub const GLIDE_SOUND: &str = "minecraft:item.elytra.flying";

impl LocalPlayer {
    /// `Entity.isFallFlying`: shared flag 7, which the client sets itself on a start and the
    /// server's data then keeps or clears.
    pub fn is_fall_flying(&self) -> bool {
        self.elytra.fall_flying
    }

    /// `ClientAvatarState.tick`: the walk distance and the motion of the tick before are kept.
    pub(super) fn begin_avatar_tick(&mut self) {
        self.avatar.walk_dist_old = self.avatar.walk_dist;
        let position = (self.position.x, self.position.y, self.position.z);
        self.avatar.cloak.tick(position);
        self.elytra.previous_motion = [self.delta.x, self.delta.y, self.delta.z];
    }

    /// `getFallFlyingTicks`: how long the player has glided.
    pub fn fall_flying_ticks(&self) -> i32 {
        self.elytra.fall_fly_ticks
    }

    /// `ClientAvatarState.deltaMovementOnPreviousTick`.
    pub fn motion_on_previous_tick(&self) -> (f64, f64, f64) {
        let [x, y, z] = self.elytra.previous_motion;
        (x, y, z)
    }

    /// The server's shared flags arrived with the gliding bit as `set`.
    pub fn set_server_fall_flying(&mut self, set: bool) {
        if set && !self.elytra.fall_flying {
            self.start_glide_sound();
        }
        self.elytra.fall_flying = set;
    }

    /// `ElytraOnPlayerSoundInstance` is started where the glide begins; it is placed by its own
    /// first tick, so it is queued at the origin.
    fn start_glide_sound(&mut self) {
        self.pending_sounds.push(LocalSound::at(
            GLIDE_SOUND,
            LocalSoundSource::Players,
            (0.1, 1.0),
            (0.0, 0.0, 0.0),
        ));
    }

    /// `FireworkRocketEntity.tick` for a rocket attached to this player: a glide is pushed along
    /// the look direction. Whether the player glides.
    pub fn boost_from_rocket(&mut self) -> bool {
        if !self.elytra.fall_flying {
            return false;
        }
        let look = self.look_angle();
        let motion = self.delta;
        self.delta = motion.add_components(
            look.x * 0.1 + (look.x * 1.5 - motion.x) * 0.5,
            look.y * 0.1 + (look.y * 1.5 - motion.y) * 0.5,
            look.z * 0.1 + (look.z * 1.5 - motion.z) * 0.5,
        );
        true
    }

    /// `Entity.getHandHoldingItemAngle`: where the hand that holds the item is, half a block out to
    /// the side the arm is on. `holds_in_off_hand_only` turns the arm to the other one.
    pub fn hand_holding_item_angle(
        &self,
        main_arm_left: bool,
        holds_in_off_hand_only: bool,
    ) -> Vec3 {
        let arm_left = main_arm_left != holds_in_off_hand_only;
        let y_rot = self.y_rot + if arm_left { -80.0 } else { 80.0 };
        let real_y_rot = -y_rot * MTH_DEG_TO_RAD;
        let x_cos = mth_cos_f64(0.0);
        Vec3::new(
            f64::from(mth_sin_f64(f64::from(real_y_rot)) * x_cos) * 0.5,
            f64::from(-mth_sin_f64(0.0)) * 0.5,
            f64::from(mth_cos_f64(f64::from(real_y_rot)) * x_cos) * 0.5,
        )
    }

    /// `Player.canGlide`: not flying, in the air, not riding, not levitating, and with a stack
    /// that lets the player glide.
    pub(super) fn can_glide(&self) -> bool {
        !self.abilities.flying
            && !self.on_ground
            && !self.is_passenger()
            && self.hands.levitation.is_none()
            && self.hands.glider
    }

    /// `Player.tryToStartFallFlying`.
    pub(super) fn try_to_start_fall_flying(&mut self) -> bool {
        if !self.elytra.fall_flying && self.can_glide() && !self.is_in_water() {
            self.elytra.fall_flying = true;
            self.start_glide_sound();
            true
        } else {
            false
        }
    }

    /// `LocalPlayer.aiStep`: a jump pressed in the air with a glider starts the glide, and the
    /// server is told.
    pub(super) fn start_fall_flying_on_jump(
        &mut self,
        level: &Level<'_>,
        (was_jumping, just_toggled_creative_flight): (bool, bool),
    ) {
        if self.input.key_presses.jump
            && !just_toggled_creative_flight
            && !was_jumping
            && !self.on_climbable(level)
            && self.try_to_start_fall_flying()
        {
            self.sent.push(ServerboundMovement::PlayerCommand {
                entity_id: self.entity_id,
                action: PlayerCommandAction::StartFallFlying,
                data: 0,
            });
        }
    }

    /// `LivingEntity.updateFallFlying` on a client: the fall distance stops building up.
    pub(super) fn check_fall_distance_accumulation(&mut self) {
        if self.delta.y > -0.5 && self.fall_distance > 1.0 {
            self.fall_distance = 1.0;
        }
    }

    /// `LivingEntity.travelFallFlying`: on a ladder the glide ends; else the glide's motion and a move.
    pub(super) fn travel_fall_flying(
        &mut self,
        level: &Level<'_>,
        input: Vec3,
    ) -> Result<(), Unsupported> {
        if self.on_climbable(level) {
            self.travel_in_air(level, input)?;
            self.elytra.fall_flying = false;
            return Ok(());
        }
        let look_angle = self.look_angle();
        self.delta = crate::input::local_player_update_fall_flying_movement(
            crate::input::LocalPlayerUpdateFallFlyingMovementInput {
                look_angle,
                delta_movement: self.delta,
                x_rot: self.x_rot,
                gravity: self.effective_gravity(),
            },
        );
        self.move_self(level, self.delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientPlayerGameMode;

    fn airborne(glider: bool) -> LocalPlayer {
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            Vec3::new(0.0, 100.0, 0.0),
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        player.on_ground = false;
        player.set_hand_state(HandState {
            glider,
            ..HandState::default()
        });
        player
    }

    fn jump() -> Input {
        Input::new(false, false, false, false, true, false, false)
    }

    fn started(sent: &[ServerboundMovement]) -> bool {
        sent.iter().any(|packet| {
            matches!(
                packet,
                ServerboundMovement::PlayerCommand {
                    action: PlayerCommandAction::StartFallFlying,
                    ..
                }
            )
        })
    }

    /// A jump in the air with a glider starts the glide and tells the server, once; without a
    /// glider, in water-free air, nothing starts.
    #[test]
    fn a_jump_in_the_air_with_a_glider_starts_the_glide() {
        let world = ClientWorld::default();
        let mut player = airborne(true);
        let sent = player.tick(&world, jump()).expect("tick");
        assert!(player.is_fall_flying() && started(&sent));
        let sent = player.tick(&world, jump()).expect("tick");
        assert!(!started(&sent));

        let mut bare = airborne(false);
        let sent = bare.tick(&world, jump()).expect("tick");
        assert!(!bare.is_fall_flying() && !started(&sent));
    }

    /// The start sound plays once per glide: the server's echo of the flag adds none, and a flag
    /// that was cleared and comes back starts it again.
    #[test]
    fn the_start_sound_plays_once_per_glide() {
        let world = ClientWorld::default();
        let mut player = airborne(true);
        let count =
            |sounds: Vec<LocalSound>| sounds.iter().filter(|s| s.event == GLIDE_SOUND).count();
        player.tick(&world, jump()).expect("tick");
        assert_eq!(count(player.take_sounds()), 1);
        player.set_server_fall_flying(true);
        assert_eq!(count(player.take_sounds()), 0);
        player.set_server_fall_flying(false);
        player.set_server_fall_flying(true);
        assert_eq!(count(player.take_sounds()), 1);
    }

    /// A rocket pushes a glide toward the look direction and leaves a player on foot alone.
    #[test]
    fn a_rocket_pushes_only_a_glide() {
        let world = ClientWorld::default();
        let mut player = airborne(true);
        let before = player.delta_movement();
        assert!(!player.boost_from_rocket() && player.delta_movement().z == before.z);
        player.tick(&world, jump()).expect("tick");
        let before = player.delta_movement();
        assert!(player.boost_from_rocket());
        let look = player.look_angle();
        // The push is a tenth of the look plus half the way to one and a half times it.
        let after = player.delta_movement();
        let expected = before.z + look.z * 0.1 + (look.z * 1.5 - before.z) * 0.5;
        assert_eq!(after.z, expected);
    }

    /// The server's shared flags end a glide, and the pose follows the flag.
    #[test]
    fn the_servers_flags_end_the_glide() {
        let world = ClientWorld::default();
        let mut player = airborne(true);
        player.tick(&world, jump()).expect("tick");
        player
            .tick(
                &world,
                Input::new(false, false, false, false, false, false, false),
            )
            .expect("tick");
        assert_eq!(player.pose(), ClientEntityPose::FallFlying);
        player.set_server_fall_flying(false);
        player
            .tick(
                &world,
                Input::new(false, false, false, false, false, false, false),
            )
            .expect("tick");
        assert!(!player.is_fall_flying());
        assert_eq!(player.pose(), ClientEntityPose::Standing);
    }

    /// The tick the server ends a glide, the pose is still the glide's: the player counts as
    /// crawling for that tick (`isVisuallySwimming`), so the input is slowed.
    #[test]
    fn the_tick_a_glide_ends_counts_as_crawling() {
        let world = ClientWorld::default();
        let mut player = airborne(true);
        player.tick(&world, jump()).expect("tick");
        player
            .tick(
                &world,
                Input::new(false, false, false, false, false, false, false),
            )
            .expect("tick");
        assert_eq!(player.pose(), ClientEntityPose::FallFlying);
        assert!(!player.is_moving_slowly());
        player.set_server_fall_flying(false);
        assert!(player.is_moving_slowly());
        player
            .tick(
                &world,
                Input::new(false, false, false, false, false, false, false),
            )
            .expect("tick");
        assert!(!player.is_moving_slowly());
    }

    /// Gliding level drops slower than falling does.
    #[test]
    fn a_glide_falls_slower_than_a_fall() {
        let world = ClientWorld::default();
        let mut gliding = airborne(true);
        gliding.tick(&world, jump()).expect("tick");
        let mut falling = airborne(false);
        falling
            .tick(
                &world,
                Input::new(false, false, false, false, false, false, false),
            )
            .expect("tick");
        for _ in 0..10 {
            gliding
                .tick(
                    &world,
                    Input::new(false, false, false, false, false, false, false),
                )
                .expect("tick");
            falling
                .tick(
                    &world,
                    Input::new(false, false, false, false, false, false, false),
                )
                .expect("tick");
        }
        assert!(gliding.position.y > falling.position.y);
        assert!(gliding.is_fall_flying());
    }
}
