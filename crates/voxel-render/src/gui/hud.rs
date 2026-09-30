//! The in-game HUD, laid out the way the reference's HUD lays it out.
//!
//! Sprites come from the GUI atlas, the reference's `gui/sprites` folder,
//! drawn at the size the HUD asks for.

use super::atlas::GuiAtlas;
use super::frame::{GuiBlend, GuiFrame, ItemDrawer};
use voxel_core::random::RandomSource;

/// Which hearts the player's health is drawn with: `Hud.HeartType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartType {
    Container,
    Normal,
    Poisoned,
    Withered,
    Absorbing,
    Frozen,
}

impl HeartType {
    /// `HeartType.forPlayer`: poison, then wither, then being frozen through.
    pub fn for_player(poisoned: bool, withered: bool, fully_frozen: bool) -> Self {
        if poisoned {
            Self::Poisoned
        } else if withered {
            Self::Withered
        } else if fully_frozen {
            Self::Frozen
        } else {
            Self::Normal
        }
    }

    /// `HeartType.getSprite`.
    fn sprite(self, hardcore: bool, half: bool, blinking: bool) -> String {
        let (prefix, container) = match self {
            Self::Container => ("container", true),
            Self::Normal => ("", false),
            Self::Poisoned => ("poisoned_", false),
            Self::Withered => ("withered_", false),
            Self::Absorbing => ("absorbing_", false),
            Self::Frozen => ("frozen_", false),
        };
        let blink = if blinking { "_blinking" } else { "" };
        if container {
            let hardcore = if hardcore { "_hardcore" } else { "" };
            return format!("hud/heart/container{hardcore}{blink}");
        }
        let hardcore = if hardcore { "hardcore_" } else { "" };
        let size = if half { "half" } else { "full" };
        format!("hud/heart/{prefix}{hardcore}{size}{blink}")
    }
}

/// What the HUD shows, as the client knows it this frame.
#[derive(Debug, Clone)]
pub struct HudInput {
    /// The camera is in the player's eyes: the crosshair shows.
    pub first_person: bool,
    pub spectator: bool,
    /// Survival or adventure: `canHurtPlayer` and `hasExperience`.
    pub survival: bool,
    pub selected_slot: i32,
    pub health: f32,
    pub max_health: f32,
    pub absorption: f32,
    /// Ticks of damage immunity left.
    pub invulnerable_time: i32,
    pub heart_type: HeartType,
    pub regeneration: bool,
    pub hardcore: bool,
    pub armor: i32,
    pub food: i32,
    pub saturation: f32,
    pub hunger: bool,
    pub air_supply: i32,
    pub max_air_supply: i32,
    pub eye_in_water: bool,
    pub experience_progress: f32,
    pub experience_level: i32,
    /// `getAttackStrengthScale(0)`.
    pub attack_strength: f32,
    /// The hotbar's stacks and the off hand's.
    pub hotbar: [Option<voxel_world::ClientItemStackState>; 9],
    pub offhand: Option<voxel_world::ClientItemStackState>,
}

/// The HUD's own state across frames: its tick count, the health it last
/// showed, and when the hearts blink.
#[derive(Debug)]
pub struct Hud {
    tick_count: i32,
    last_health: i32,
    display_health: i32,
    last_health_time: u128,
    health_blink_time: i64,
    random: RandomSource,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            tick_count: 0,
            last_health: 0,
            display_health: 0,
            last_health_time: 0,
            health_blink_time: 0,
            random: RandomSource::create(0),
        }
    }
}

impl Hud {
    /// `Hud.tick`: the HUD's tick count moves on.
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// `Hud.extractRenderState` for what this client has: the crosshair,
    /// then the hotbar and its decorations.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        width: i32,
        height: i32,
        input: &HudInput,
        now_millis: u128,
        draw_item: &mut ItemDrawer<'_>,
    ) -> GuiFrame {
        let mut frame = GuiFrame::default();
        self.crosshair(&mut frame, atlas, width, height, input);
        let t = GuiBlend::Translucent;
        let centre = width / 2;
        // `extractItemHotbar`; a spectator's hotbar is its own menu.
        if !input.spectator {
            frame.blit(atlas, t, "hud/hotbar", centre - 91, height - 22, 182, 22);
            frame.blit(
                atlas,
                t,
                "hud/hotbar_selection",
                centre - 91 - 1 + input.selected_slot * 20,
                height - 22 - 1,
                24,
                23,
            );
            // The off hand is on the left of a right-handed player, which
            // is the default main arm.
            if input.offhand.is_some() {
                frame.blit(
                    atlas,
                    t,
                    "hud/hotbar_offhand_left",
                    centre - 91 - 29,
                    height - 23,
                    29,
                    24,
                );
            }
            // `extractSlot` for each hotbar slot, then the off hand: the
            // item, then its count when it is not one.
            let y = height - 16 - 3;
            let slots = (0..9)
                .map(|i| (&input.hotbar[i], centre - 90 + i as i32 * 20 + 2))
                .chain(std::iter::once((&input.offhand, centre - 91 - 26)));
            for (stack, x) in slots {
                let Some(stack) = stack else {
                    continue;
                };
                frame.item(draw_item, stack, x, y);
                if stack.item_count() != 1 {
                    let text = stack.item_count().to_string();
                    frame.text_with_shadow(
                        atlas,
                        font,
                        &text,
                        x + 19 - 2 - font.width(&text),
                        y + 6 + 3,
                        0xFFFF_FFFF,
                    );
                }
            }
        }
        if input.survival {
            self.player_health(&mut frame, atlas, width, height, input, now_millis);
            // `ExperienceBar.extractBackground`: every level needs some
            // experience, so the bar always shows.
            let left = (width - 182) / 2;
            let top = height - 24 - 5;
            let progress = (input.experience_progress * 183.0) as i32;
            frame.blit(atlas, t, "hud/experience_bar_background", left, top, 182, 5);
            if progress > 0 {
                frame.blit_part(
                    atlas,
                    t,
                    "hud/experience_bar_progress",
                    (182, 5),
                    (0, 0),
                    left,
                    top,
                    progress,
                    5,
                );
            }
            // `ContextualBar.extractExperienceLevel`: the level in green,
            // outlined by four black copies a unit off.
            if input.experience_level > 0 {
                let text = input.experience_level.to_string();
                let x = (width - font.width(&text)) / 2;
                let y = height - 24 - 9 - 2;
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    frame.text(atlas, font, &text, x + dx, y + dy, 0xFF00_0000);
                }
                frame.text(atlas, font, &text, x, y, 0xFF80_FF20);
            }
        }
        frame
    }

    /// `Hud.extractCrosshair` with the attack indicator under it, as the
    /// default option places it. Nothing is picked as an entity yet, so the
    /// full indicator never shows.
    fn crosshair(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        width: i32,
        height: i32,
        input: &HudInput,
    ) {
        if !input.first_person || input.spectator {
            return;
        }
        let invert = GuiBlend::Invert;
        frame.blit(
            atlas,
            invert,
            "hud/crosshair",
            (width - 15) / 2,
            (height - 15) / 2,
            15,
            15,
        );
        if input.attack_strength < 1.0 {
            let y = height / 2 - 7 + 16;
            let x = width / 2 - 8;
            let progress = (input.attack_strength * 17.0) as i32;
            frame.blit(
                atlas,
                invert,
                "hud/crosshair_attack_indicator_background",
                x,
                y,
                16,
                4,
            );
            frame.blit_part(
                atlas,
                invert,
                "hud/crosshair_attack_indicator_progress",
                (16, 4),
                (0, 0),
                x,
                y,
                progress,
                4,
            );
        }
    }

    /// `Hud.extractPlayerHealth`: armour, hearts, food and air.
    fn player_health(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        width: i32,
        height: i32,
        input: &HudInput,
        now_millis: u128,
    ) {
        let t = GuiBlend::Translucent;
        let current = input.health.ceil() as i32;
        let tick = i64::from(self.tick_count);
        let blink = self.health_blink_time > tick && (self.health_blink_time - tick) / 3 % 2 == 1;
        if current < self.last_health && input.invulnerable_time > 0 {
            self.last_health_time = now_millis;
            self.health_blink_time = tick + 20;
        } else if current > self.last_health && input.invulnerable_time > 0 {
            self.last_health_time = now_millis;
            self.health_blink_time = tick + 10;
        }
        if now_millis.saturating_sub(self.last_health_time) > 1000 {
            self.display_health = current;
            self.last_health_time = now_millis;
        }
        self.last_health = current;
        let old_health = self.display_health;
        // The tick count times the constant is an int product.
        self.random
            .set_seed(i64::from(self.tick_count.wrapping_mul(312_871)));
        let x_left = width / 2 - 91;
        let x_right = width / 2 + 91;
        let y_base = height - 39;
        let max_health = input.max_health.max(old_health.max(current) as f32);
        let absorption = input.absorption.ceil() as i32;
        let rows = ((max_health + absorption as f32) / 2.0 / 10.0).ceil() as i32;
        let row_height = (10 - (rows - 2)).max(3);
        let mut y_air = y_base - 10;
        let heart_offset_index = if input.regeneration {
            self.tick_count % (max_health + 5.0).ceil() as i32
        } else {
            -1
        };

        // `extractArmor`.
        if input.armor > 0 {
            let y = y_base - (rows - 1) * row_height - 10;
            for i in 0..10 {
                let x = x_left + i * 8;
                if i * 2 + 1 < input.armor {
                    frame.blit(atlas, t, "hud/armor_full", x, y, 9, 9);
                }
                if i * 2 + 1 == input.armor {
                    frame.blit(atlas, t, "hud/armor_half", x, y, 9, 9);
                }
                if i * 2 + 1 > input.armor {
                    frame.blit(atlas, t, "hud/armor_empty", x, y, 9, 9);
                }
            }
        }

        // `extractHearts`.
        let health_containers = (f64::from(max_health) / 2.0).ceil() as i32;
        let absorption_containers = (f64::from(absorption) / 2.0).ceil() as i32;
        let max_health_halves = health_containers * 2;
        let hardcore = input.hardcore;
        let heart =
            |frame: &mut GuiFrame, kind: HeartType, x: i32, y: i32, blinking: bool, half: bool| {
                frame.blit(atlas, t, &kind.sprite(hardcore, half, blinking), x, y, 9, 9);
            };
        for index in (0..health_containers + absorption_containers).rev() {
            let row = index / 10;
            let column = index % 10;
            let x = x_left + column * 8;
            let mut y = y_base - row * row_height;
            if current + absorption <= 4 {
                y += self.random.next_int_bound(2).unwrap_or(0);
            }
            if index < health_containers && index == heart_offset_index {
                y -= 2;
            }
            heart(frame, HeartType::Container, x, y, blink, false);
            let halves = index * 2;
            if index >= health_containers {
                let absorption_halves = halves - max_health_halves;
                if absorption_halves < absorption {
                    let half = absorption_halves + 1 == absorption;
                    let kind = if input.heart_type == HeartType::Withered {
                        HeartType::Withered
                    } else {
                        HeartType::Absorbing
                    };
                    heart(frame, kind, x, y, false, half);
                }
            }
            if blink && halves < old_health {
                heart(
                    frame,
                    input.heart_type,
                    x,
                    y,
                    true,
                    halves + 1 == old_health,
                );
            }
            if halves < current {
                heart(frame, input.heart_type, x, y, false, halves + 1 == current);
            }
        }

        // `extractFood`, with no vehicle's hearts in its place.
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
            frame.blit(atlas, t, empty, x, y, 9, 9);
            if i * 2 + 1 < input.food {
                frame.blit(atlas, t, full, x, y, 9, 9);
            }
            if i * 2 + 1 == input.food {
                frame.blit(atlas, t, half, x, y, 9, 9);
            }
        }
        y_air -= 10;

        // `extractAirBubbles`: with no vehicle, the row sits one line up
        // from where the vehicle's rows would start.
        let max_air = input.max_air_supply;
        let air = input.air_supply.clamp(0, max_air);
        let under_water = input.eye_in_water;
        if under_water || air < max_air {
            y_air += 10;
            let bubble =
                |offset: i32| (((air + offset) * 10) as f32 / max_air as f32).ceil() as i32;
            let full_bubbles = bubble(-2);
            let popping = bubble(0);
            let delay = if air != 0 && under_water { 1 } else { 0 };
            let empty_bubbles = 10 - bubble(delay);
            let is_popping = full_bubbles != popping;
            for index in 1..=10 {
                let x = x_right - (index - 1) * 8 - 9;
                if index <= full_bubbles {
                    frame.blit(atlas, t, "hud/air", x, y_air, 9, 9);
                } else if is_popping && index == popping && under_water {
                    frame.blit(atlas, t, "hud/air_bursting", x, y_air, 9, 9);
                } else if index > 10 - empty_bubbles {
                    let wobble = if empty_bubbles == 10 && self.tick_count % 2 == 0 {
                        self.random.next_int_bound(2).unwrap_or(0)
                    } else {
                        0
                    };
                    frame.blit(atlas, t, "hud/air_empty", x, y_air + wobble, 9, 9);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heart_sprites_follow_the_reference_names() {
        assert_eq!(
            HeartType::Container.sprite(false, false, true),
            "hud/heart/container_blinking"
        );
        assert_eq!(
            HeartType::Container.sprite(true, false, false),
            "hud/heart/container_hardcore"
        );
        assert_eq!(
            HeartType::Normal.sprite(false, true, false),
            "hud/heart/half"
        );
        assert_eq!(
            HeartType::Poisoned.sprite(true, false, true),
            "hud/heart/poisoned_hardcore_full_blinking"
        );
    }
}
