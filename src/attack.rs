//! The attack key, as the reference's game loop reads it each tick.
//!
//! Every press of the key counts one click, and the key also has a held
//! state. Each tick, before the player moves, the loop spends the clicks: a
//! click while the miss wait is running does nothing, otherwise the player
//! attacks what it aims at and swings. Aiming at nothing starts the miss wait
//! (ten ticks, not in creative) and resets the attack strength. After the
//! clicks, a key that is no longer held clears the miss wait; only then does
//! the wait count down. So clicking and letting go attacks every tick, while
//! holding the key down waits out the ten ticks.

/// What the player aims at when it attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aim {
    /// Nothing within reach.
    Miss,
    /// A block's outline within reach.
    Block,
}

/// What one click did, for the caller to carry out in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attack {
    /// A miss: the attack strength resets, then the arm swings.
    Missed,
    /// A block was hit: the arm swings (breaking is not ported).
    HitBlock,
}

/// The attack key and `Minecraft.missTime`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AttackKey {
    clicks: u32,
    down: bool,
    miss_time: i32,
}

impl AttackKey {
    /// `KeyMapping.click` and `setDown(true)`: the key went down.
    pub fn press(&mut self) {
        self.clicks += 1;
        self.down = true;
    }

    /// The key came up.
    pub fn release(&mut self) {
        self.down = false;
    }

    /// Ticks until a missed attack lets the next one through.
    pub fn miss_time(&self) -> i32 {
        self.miss_time
    }

    /// `Minecraft.handleKeybinds`' attack step and the countdown after it.
    /// `aim` is asked once per click that gets through; `grabbed` is whether
    /// the mouse is captured; `spectator` and `creative` are the game mode.
    pub fn tick(
        &mut self,
        grabbed: bool,
        spectator: bool,
        creative: bool,
        mut aim: impl FnMut() -> Aim,
    ) -> Vec<Attack> {
        let mut attacks = Vec::new();
        while self.clicks > 0 {
            self.clicks -= 1;
            // `startAttack`.
            if self.miss_time > 0 || spectator {
                continue;
            }
            match aim() {
                Aim::Block => attacks.push(Attack::HitBlock),
                Aim::Miss => {
                    if !creative {
                        self.miss_time = 10;
                    }
                    attacks.push(Attack::Missed);
                }
            }
        }
        // `continueAttack`: letting go clears the wait.
        if !(self.down && grabbed) {
            self.miss_time = 0;
        }
        if self.miss_time > 0 {
            self.miss_time -= 1;
        }
        attacks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(key: &mut AttackKey) -> Vec<Attack> {
        key.tick(true, false, false, || Aim::Miss)
    }

    #[test]
    fn clicking_and_letting_go_attacks_every_tick() {
        let mut key = AttackKey::default();
        for _ in 0..20 {
            key.press();
            key.release();
            assert_eq!(tick(&mut key), vec![Attack::Missed]);
        }
    }

    #[test]
    fn holding_the_key_waits_out_the_miss() {
        let mut key = AttackKey::default();
        key.press();
        assert_eq!(tick(&mut key), vec![Attack::Missed]);
        // Clicks while it is held and the wait runs are spent for nothing.
        for _ in 0..9 {
            key.press();
            assert!(tick(&mut key).is_empty());
        }
        key.press();
        assert_eq!(tick(&mut key), vec![Attack::Missed]);
    }

    #[test]
    fn two_clicks_in_one_tick_attack_once() {
        let mut key = AttackKey::default();
        key.press();
        key.release();
        key.press();
        key.release();
        assert_eq!(tick(&mut key), vec![Attack::Missed]);
        assert!(tick(&mut key).is_empty());
    }

    #[test]
    fn hitting_a_block_or_creative_starts_no_wait() {
        let mut key = AttackKey::default();
        key.press();
        assert_eq!(
            key.tick(true, false, false, || Aim::Block),
            vec![Attack::HitBlock]
        );
        assert_eq!(key.miss_time(), 0);
        key.press();
        assert_eq!(
            key.tick(true, false, true, || Aim::Miss),
            vec![Attack::Missed]
        );
        key.press();
        assert_eq!(
            key.tick(true, false, true, || Aim::Miss),
            vec![Attack::Missed]
        );
    }

    #[test]
    fn a_spectator_does_not_attack() {
        let mut key = AttackKey::default();
        key.press();
        assert!(key.tick(true, true, false, || Aim::Miss).is_empty());
    }
}
