use super::*;

impl LocalPlayer {
    /// `Entity.isFallFlying`: shared flag 7, which the client sets itself on a start and the
    /// server's data then keeps or clears.
    pub fn is_fall_flying(&self) -> bool {
        self.elytra.fall_flying
    }

    /// `ClientAvatarState.tick`: the walk distance and the motion of the tick before are kept.
    pub(super) fn begin_avatar_tick(&mut self) {
        self.walk_dist_old = self.walk_dist;
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
        self.elytra.fall_flying = set;
    }

    /// `Player.canGlide`: not flying, in the air, not riding, not levitating, and with a stack
    /// that lets the player glide.
    pub(super) fn can_glide(&self) -> bool {
        !self.abilities.flying
            && !self.on_ground
            && !self.is_passenger()
            && !self.hands.levitating
            && self.hands.glider
    }

    /// `Player.tryToStartFallFlying`.
    pub(super) fn try_to_start_fall_flying(&mut self) -> bool {
        if !self.elytra.fall_flying && self.can_glide() && !self.is_in_water() {
            self.elytra.fall_flying = true;
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

    /// `LivingEntity.getEffectiveGravity`: slow falling caps a fall's gravity.
    fn effective_gravity(&self) -> f64 {
        if self.delta.y <= 0.0 && self.hands.slow_falling {
            GRAVITY.min(0.01)
        } else {
            GRAVITY
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
