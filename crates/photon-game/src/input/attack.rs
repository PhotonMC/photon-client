//! The attack key, as the reference's key mapping records it: every press
//! counts one click, and the key also has a held state. The game mode spends
//! the clicks and reads the held state once a tick.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AttackKey {
    clicks: u32,
    down: bool,
}

impl AttackKey {
    /// `KeyMapping.click` and `setDown(true)`: the key went down.
    pub fn press(&mut self) {
        self.clicks += 1;
        self.down = true;
    }

    /// `KeyMapping.click` and `ToggleKeyMapping.setDown(true)`: the key was pressed while it
    /// toggles, so it is held when it was not, and let go when it was.
    pub fn press_toggling(&mut self) {
        self.clicks += 1;
        self.down = !self.down;
    }

    /// The key came up.
    pub fn release(&mut self) {
        self.down = false;
    }

    /// The clicks since they were last taken.
    pub fn take_clicks(&mut self) -> u32 {
        std::mem::take(&mut self.clicks)
    }

    pub fn is_down(&self) -> bool {
        self.down
    }
}
