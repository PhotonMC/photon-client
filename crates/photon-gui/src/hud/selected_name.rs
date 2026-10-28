//! The name of the held item over the hotbar (`Hud.extractSelectedItemName`), shown for a while
//! after the selection or the item changes.

use crate::{FontRuns, FrameRuns};
use photon_core::ids::ItemId;
use photon_game::text::{Component, Run};
use photon_render::gui::GuiAtlas;
use photon_render::gui::GuiFrame;
use photon_render::gui::font::Font;

/// The stack in the selected slot, as `Hud.tick` compares it.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedItem {
    pub item: ItemId,
    /// `ItemStack.getHoverName`.
    pub hover_name: Component,
    /// `ItemStack.getStyledHoverName`'s runs: the rarity's colour, italic for a custom name.
    pub runs: Vec<Run>,
}

/// `Hud.toolHighlightTimer` and `lastToolHighlight`.
#[derive(Debug, Clone, Default)]
pub struct SelectedName {
    timer: i32,
    last: Option<SelectedItem>,
}

impl SelectedName {
    /// The part of `Hud.tick` that follows the selected stack: the name shows for forty ticks
    /// (scaled by `notificationDisplayTime`) from a change of item or name, and counts down.
    pub fn tick(&mut self, selected: Option<SelectedItem>, display_time: f64) {
        match &selected {
            None => self.timer = 0,
            Some(now) => {
                let same = self
                    .last
                    .as_ref()
                    .is_some_and(|last| last.item == now.item && last.hover_name == now.hover_name);
                if !same {
                    self.timer = (40.0 * display_time) as i32;
                } else if self.timer > 0 {
                    self.timer -= 1;
                }
            }
        }
        self.last = selected;
    }

    /// `extractSelectedItemName`: the name over the hotbar, fading in its last ten ticks, a unit
    /// lower in creative. `backdrop` is the text background's opacity, nought when the option
    /// keeps it to the chat.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        (width, height): (i32, i32),
        (can_hurt_player, backdrop): (bool, f32),
    ) {
        let Some(last) = self.last.as_ref().filter(|_| self.timer > 0) else {
            return;
        };
        let text_width = font.width_of_runs(&last.runs);
        let x = (width - text_width) / 2;
        let y = height - 59 + if can_hurt_player { 0 } else { 14 };
        let alpha = ((self.timer as f32 * 256.0 / 10.0) as i32).min(255);
        if alpha <= 0 {
            return;
        }
        // `GuiGraphics.textWithBackdrop`: black at the option's opacity, times the text's alpha.
        let background = (backdrop * 255.0).floor() as i32;
        if background != 0 {
            frame.fill(
                atlas,
                (x - 2, y - 2, x + text_width + 2, y + 9 + 2),
                ((background * alpha / 255) as u32) << 24,
            );
        }
        frame.styled_text(
            atlas,
            font,
            &last.runs,
            (x, y),
            (alpha as u32) << 24 | 0x00FF_FFFF,
            true,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: i32, name: &str) -> SelectedItem {
        SelectedItem {
            item: ItemId(item),
            hover_name: Component::literal(name),
            runs: Vec::new(),
        }
    }

    /// A new item starts the forty ticks, which run down while it stays and end with an empty hand.
    #[test]
    fn the_timer_starts_on_a_change_and_runs_down() {
        let mut name = SelectedName::default();
        name.tick(Some(stack(1, "a")), 1.0);
        assert_eq!(name.timer, 40);
        name.tick(Some(stack(1, "a")), 1.0);
        assert_eq!(name.timer, 39);
        name.tick(Some(stack(1, "b")), 1.0);
        assert_eq!(name.timer, 40, "a new name starts it again");
        name.tick(Some(stack(2, "b")), 0.5);
        assert_eq!(name.timer, 20, "scaled by the display time");
        name.tick(None, 1.0);
        assert_eq!(name.timer, 0);
        name.tick(Some(stack(2, "b")), 1.0);
        assert_eq!(name.timer, 40, "after an empty hand the item is new");
    }
}
