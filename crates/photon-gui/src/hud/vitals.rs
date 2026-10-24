//! The rows under the hearts: the food, the vehicle's hearts and the air bubbles
//! (`Hud.extractFood`, `extractVehicleHealth`, `extractAirBubbles`).

use super::*;

/// `Hud.getVisibleVehicleHeartRows`.
fn rows_of(hearts: i32) -> i32 {
    (f64::from(hearts) / 10.0).ceil() as i32
}

/// The vehicle the player rides when it has health to show (`Hud.getPlayerVehicleWithHealth`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleInput {
    pub health: f32,
    pub max_health: f32,
}

/// The jumpable mount the player rides: how far its jump is charged (`getJumpRidingScale`) and
/// whether the mount waits to jump again (`getJumpCooldown() > 0`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JumpInput {
    pub scale: f32,
    pub cooldown: bool,
}

/// `Hud.ContextualInfo`: which bar sits over the hotbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Bar {
    Empty,
    Experience,
    Locator,
    Jumpable,
}

/// `Hud.nextContextualInfoState`.
pub(super) fn contextual_info(input: &HudInput) -> Bar {
    let jump = input.jump;
    let experience = input.survival;
    if input.locator.is_some() {
        let prioritised = jump.is_some_and(|jump| jump.scale > 0.0 || jump.cooldown);
        if jump.is_some() && prioritised {
            Bar::Jumpable
        } else if experience && input.experience_prioritised {
            Bar::Experience
        } else {
            Bar::Locator
        }
    } else if jump.is_some() {
        Bar::Jumpable
    } else if experience {
        Bar::Experience
    } else {
        Bar::Empty
    }
}

/// `JumpableVehicleBar.extractBackground`.
pub(super) fn jump_bar(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    (left, top): (i32, i32),
    jump: JumpInput,
) {
    let t = GuiBlend::Translucent;
    frame.sprite_blend(atlas, t, "hud/jump_bar_background", (left, top, 182, 5));
    if jump.cooldown {
        frame.sprite_blend(atlas, t, "hud/jump_bar_cooldown", (left, top, 182, 5));
        return;
    }
    // `Mth.lerpDiscrete(scale, 0, 182)`.
    let progress = (jump.scale * 181.0).floor() as i32 + i32::from(jump.scale > 0.0);
    if progress > 0 {
        frame.sprite_region_blend(
            atlas,
            t,
            "hud/jump_bar_progress",
            (182, 5),
            (0, 0),
            (left, top, progress, 5),
        );
    }
}

impl VehicleInput {
    /// `Hud.getVehicleMaxHearts`: the hearts its health fills, at most thirty.
    pub fn hearts(&self) -> i32 {
        (((self.max_health + 0.5) as i32) / 2).min(30)
    }
}

impl Hud {
    /// `Hud.extractFood`.
    pub(super) fn food_row(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        input: &HudInput,
        (x_right, y_base): (i32, i32),
    ) {
        let t = GuiBlend::Translucent;
        let (empty, half, full) = if input.hunger {
            (
                "hud/food_empty_hunger",
                "hud/food_half_hunger",
                "hud/food_full_hunger",
            )
        } else {
            ("hud/food_empty", "hud/food_half", "hud/food_full")
        };
        for i in 0..10 {
            let mut y = y_base;
            if input.saturation <= 0.0 && self.tick_count % (input.food * 3 + 1) == 0 {
                y += self.random.next_int_bound(3).unwrap_or(1) - 1;
            }
            let x = x_right - i * 8 - 9;
            frame.sprite_blend(atlas, t, empty, (x, y, 9, 9));
            if i * 2 + 1 < input.food {
                frame.sprite_blend(atlas, t, full, (x, y, 9, 9));
            }
            if i * 2 + 1 == input.food {
                frame.sprite_blend(atlas, t, half, (x, y, 9, 9));
            }
        }
    }

    /// `Hud.extractVehicleHealth`: the hearts of the mount, from the right edge, rows going up.
    pub fn vehicle_health(
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        (width, height): (i32, i32),
        vehicle: &VehicleInput,
    ) {
        let t = GuiBlend::Translucent;
        let mut hearts = vehicle.hearts();
        if hearts == 0 {
            return;
        }
        let current = f64::from(vehicle.health).ceil() as i32;
        let x_right = width / 2 + 91;
        let mut y = height - 39;
        let mut base = 0;
        while hearts > 0 {
            let row = hearts.min(10);
            hearts -= row;
            for i in 0..row {
                let x = x_right - i * 8 - 9;
                frame.sprite_blend(atlas, t, "hud/heart/vehicle_container", (x, y, 9, 9));
                if i * 2 + 1 + base < current {
                    frame.sprite_blend(atlas, t, "hud/heart/vehicle_full", (x, y, 9, 9));
                }
                if i * 2 + 1 + base == current {
                    frame.sprite_blend(atlas, t, "hud/heart/vehicle_half", (x, y, 9, 9));
                }
            }
            y -= 10;
            base += 20;
        }
    }

    /// The empty bubbles all shake on every other tick.
    fn wobble(&mut self, empty_bubbles: i32) -> i32 {
        if empty_bubbles == 10 && self.tick_count % 2 == 0 {
            self.random.next_int_bound(2).unwrap_or(0)
        } else {
            0
        }
    }

    /// `Hud.extractAirBubbles`: with no vehicle, the row sits one line up from where the
    /// vehicle's rows would start.
    pub(super) fn air_bubbles(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        input: &HudInput,
        (x_right, y_air, vehicle_hearts): (i32, i32, i32),
    ) {
        let t = GuiBlend::Translucent;
        let max_air = input.max_air_supply;
        let air = input.air_supply.clamp(0, max_air);
        let under_water = input.eye_in_water;
        if under_water || air < max_air {
            let y_air = y_air - (rows_of(vehicle_hearts) - 1) * 10;
            let counts = BubbleCounts::of(air, max_air, under_water);
            let (full_bubbles, popping, empty_bubbles) =
                (counts.full, counts.popping, counts.empty);
            let is_popping = counts.is_popping();
            for index in 1..=10 {
                let x = x_right - (index - 1) * 8 - 9;
                if index <= full_bubbles {
                    frame.sprite_blend(atlas, t, "hud/air", (x, y_air, 9, 9));
                } else if is_popping && index == popping && under_water {
                    frame.sprite_blend(atlas, t, "hud/air_bursting", (x, y_air, 9, 9));
                } else if index > 10 - empty_bubbles {
                    let wobble = self.wobble(empty_bubbles);
                    frame.sprite_blend(atlas, t, "hud/air_empty", (x, y_air + wobble, 9, 9));
                }
            }
        }
    }
}

/// How many bubbles are full, which one is popping and how many are empty.
struct BubbleCounts {
    full: i32,
    popping: i32,
    empty: i32,
}

impl BubbleCounts {
    fn of(air: i32, max_air: i32, under_water: bool) -> Self {
        let bubble = |offset: i32| (((air + offset) * 10) as f32 / max_air as f32).ceil() as i32;
        let delay = i32::from(air != 0 && under_water);
        Self {
            full: bubble(-2),
            popping: bubble(0),
            empty: 10 - bubble(delay),
        }
    }

    fn is_popping(&self) -> bool {
        self.full != self.popping
    }
}
