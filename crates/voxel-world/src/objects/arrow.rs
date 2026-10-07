//! What an arrow does of its own on the client: how long it has shaken since it
//! stuck (`AbstractArrow.shakeTime`, started when the server says it is in the ground
//! and counted down each tick) and what its data says (the ground, the effect colour).

/// `AbstractArrow`'s data ids (`Entity` has eight): the flags, the pierce level, whether it is in the
/// ground; then `Arrow.ID_EFFECT_COLOR` (an arrow's own).
pub const IN_GROUND_DATA_ID: u8 = 10;
pub const EFFECT_COLOR_DATA_ID: u8 = 11;

/// `AbstractArrow.shakeTime` after it sticks.
const SHAKE_TICKS: i32 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowState {
    shake_time: i32,
    in_ground: bool,
    /// `Arrow.getColor`: the potion's colour, minus one for none.
    color: i32,
}

impl Default for ArrowState {
    fn default() -> Self {
        Self {
            shake_time: 0,
            in_ground: false,
            color: -1,
        }
    }
}

impl ArrowState {
    /// `AbstractArrow.shakeTime`.
    pub fn shake_time(&self) -> i32 {
        self.shake_time
    }

    pub fn is_in_ground(&self) -> bool {
        self.in_ground
    }

    /// `Arrow.getColor() > 0`: it carries a potion's effect (`TippableArrowRenderer`).
    pub fn is_tipped(&self) -> bool {
        self.color > 0
    }

    /// `onSyncedDataUpdated(IN_GROUND)`: sticking, after the first tick and while it is not shaking
    /// already, starts the shake.
    pub(crate) fn set_in_ground(&mut self, in_ground: bool, ticked: bool) {
        self.in_ground = in_ground;
        if ticked && self.shake_time <= 0 && in_ground {
            self.shake_time = SHAKE_TICKS;
        }
    }

    pub(crate) fn set_color(&mut self, color: i32) {
        self.color = color;
    }

    /// `AbstractArrow.tick`: the shake runs down.
    pub(crate) fn tick(&mut self) {
        if self.shake_time > 0 {
            self.shake_time -= 1;
        }
    }
}
