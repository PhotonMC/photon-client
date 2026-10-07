use super::*;

impl LocalPlayer {
    /// `ClientAvatarState.getBackwardsInterpolatedWalkDistance` and
    /// `getInterpolatedBob`, which the view bob reads.
    pub fn view_bob(&self, partial_tick: f32) -> (f32, f32) {
        let walked = self.walk_dist - self.walk_dist_old;
        (
            -(self.walk_dist + walked * partial_tick),
            self.bob_old + partial_tick * (self.bob - self.bob_old),
        )
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
        let base = crate::block::rules::item(item)
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
}

/// `Player.getCurrentItemAttackStrengthDelay`: the ticks an attack takes to recharge at an
/// attack speed.
pub fn attack_strength_delay(attack_speed: f64) -> f32 {
    (1.0 / attack_speed * 20.0) as f32
}

/// What the local player's swing and use of items read of the world's player state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandState {
    /// The registry ids of what the main and off hands hold (air when empty).
    pub main_item: i32,
    pub off_item: i32,
    /// The greater of the haste and conduit power amplifiers, if either is on.
    pub dig_speed: Option<i32>,
    pub mining_fatigue: Option<i32>,
    /// The server's flags: an item is in use, and in the off hand.
    pub using_item: bool,
    pub using_off_hand: bool,
}
