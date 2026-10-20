use super::*;
use photon_data::item::registry::AIR_ITEM;

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
            .and_then(|item| photon_data::spear::spear_of(item.name))
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
        self.follow_use_flags();
    }

    /// `LocalPlayer.onSyncedDataUpdated`: the server's use flag acts only when it changes. A use
    /// the client ended itself stays ended while the flag still says it is on.
    fn follow_use_flags(&mut self) {
        let (server, off_hand) = (self.hands.using_item, self.hands.using_off_hand);
        let held = if off_hand {
            self.hands.off_item
        } else {
            self.hands.main_item
        };
        let edge = &mut self.use_edge;
        if server != edge.server {
            if server && !edge.started && held != AIR_ITEM {
                edge.started = true;
                edge.off_hand = off_hand;
            } else if !server {
                edge.started = false;
            }
        }
        edge.server = server;
        self.hands.using_item = edge.started;
        self.hands.using_off_hand = edge.off_hand;
    }

    /// `LivingEntity.releaseUsingItem` on the client: the use ends here and now.
    pub fn release_using_item(&mut self) {
        self.use_edge.started = false;
        self.hands.using_item = false;
    }

    /// `LocalPlayer.isUsingItem`: the use the client has started, which the server's flag starts
    /// and stops when it changes.
    pub fn is_using_item(&self) -> bool {
        self.hands.using_item
    }

    /// Whether the off hand is the one using the item.
    pub fn used_item_is_off_hand(&self) -> bool {
        self.hands.using_off_hand
    }

    /// `Player.isScoping`: a spyglass is in use.
    pub fn is_scoping(&self) -> bool {
        let used = if self.hands.using_off_hand {
            self.hands.off_item
        } else {
            self.hands.main_item
        };
        self.hands.using_item
            && crate::block::rules::item(used).is_some_and(|item| {
                item.class == photon_data::item::classes::ItemClass::SpyglassItem
            })
    }

    /// What the use of an item does to the field of view (`getFieldOfViewModifier`): a bow being
    /// drawn narrows it by up to a seventh of the draw's square, and a spyglass in use in the
    /// first person fixes the modifier at a tenth.
    pub fn use_fov(&self, first_person: bool) -> UseFov {
        use photon_data::item::classes::ItemClass;
        if !self.hands.using_item {
            return UseFov::Free;
        }
        let used = if self.hands.using_off_hand {
            self.hands.off_item
        } else {
            self.hands.main_item
        };
        let class = crate::block::rules::item(used).map(|item| item.class);
        if class == Some(ItemClass::BowItem) {
            let scale = (self.use_edge.ticks as f32 / 20.0).min(1.0);
            UseFov::Drawing(1.0 - scale * scale * 0.15)
        } else if first_person && class == Some(ItemClass::SpyglassItem) {
            UseFov::Scoped
        } else {
            UseFov::Free
        }
    }

    /// `LivingEntity.getTicksUsingItem(partialTicks)`.
    pub fn ticks_using_item(&self, partial_tick: f32) -> f32 {
        if self.hands.using_item {
            self.use_edge.ticks as f32 + partial_tick
        } else {
            0.0
        }
    }

    /// How many ticks the stack in a hand has been used, if the player uses that hand's stack
    /// (`getUseItem() == stack`).
    pub fn use_ticks_of(&self, main_hand: bool) -> Option<i32> {
        (self.hands.using_item && self.hands.using_off_hand != main_hand)
            .then_some(self.use_edge.ticks)
    }

    /// [`Self::use_ticks_of`] by arm, the right arm's first (the main arm is the right).
    pub fn arm_uses(&self) -> [Option<i32>; 2] {
        [self.use_ticks_of(true), self.use_ticks_of(false)]
    }

    /// The item now in the main hand, by registry id; air when empty.
    pub fn set_main_hand_item(&mut self, item: ItemId) {
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

impl Default for HandState {
    fn default() -> Self {
        Self {
            main_item: AIR_ITEM,
            off_item: AIR_ITEM,
            dig_speed: None,
            mining_fatigue: None,
            using_item: false,
            using_off_hand: false,
        }
    }
}

/// What the local player's swing and use of items read of the world's player state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandState {
    /// The registry ids of what the main and off hands hold (air when empty).
    pub main_item: ItemId,
    pub off_item: ItemId,
    /// The greater of the haste and conduit power amplifiers, if either is on.
    pub dig_speed: Option<i32>,
    pub mining_fatigue: Option<i32>,
    /// An item is in use, and in the off hand: the server's flags as the world holds them, then
    /// (in the player) the use the client has started.
    pub using_item: bool,
    pub using_off_hand: bool,
}

/// `LocalPlayer.startedUsingItem` and `usingItemHand`, with the server's flag as last seen.
#[derive(Debug, Clone, Copy, Default)]
pub struct UseEdge {
    pub(super) server: bool,
    pub(super) started: bool,
    pub(super) off_hand: bool,
    pub(super) ticks: i32,
}

/// How the use of an item changes the field of view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UseFov {
    Free,
    /// A bow being drawn: the factor it multiplies the modifier by.
    Drawing(f32),
    /// A spyglass held to the eye.
    Scoped,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holding(using: bool) -> HandState {
        HandState {
            main_item: photon_data::item::registry::named("minecraft:bow"),
            off_item: AIR_ITEM,
            dig_speed: None,
            mining_fatigue: None,
            using_item: using,
            using_off_hand: false,
        }
    }

    fn player() -> LocalPlayer {
        LocalPlayer::new(
            photon_core::ids::EntityId(1),
            Vec3::ZERO,
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        )
    }

    /// The server's flag acts when it changes: a use the player let go of stays ended while the
    /// flag is still on, and starts again only when the flag goes off and on.
    #[test]
    fn a_released_use_stays_ended_until_the_flag_changes() {
        let mut player = player();
        player.set_hand_state(holding(true));
        assert!(player.is_using_item());
        player.release_using_item();
        assert!(!player.is_using_item());
        player.set_hand_state(holding(true));
        assert!(!player.is_using_item());
        player.set_hand_state(holding(false));
        player.set_hand_state(holding(true));
        assert!(player.is_using_item());
    }
}
