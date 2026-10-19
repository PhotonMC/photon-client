//! The recipe toast (`RecipeToast`): the recipes the player unlocked, shown one after the other
//! in a single toast.

use super::{Toast, ToastEnv, Visibility};
use crate::FrameRuns;
use photon_core::ids::ItemId;
use photon_game::text::{Component, Run, Style};
use photon_render::gui::{GuiAtlas, GuiFrame, ItemDrawer};
use photon_world::ClientItemStackState;

/// How long it stays after the last recipe was added, in milliseconds (`DISPLAY_TIME`).
const DISPLAY_TIME: f64 = 5000.0;

/// What the recipe toast shows of one recipe (`RecipeToast.Entry`): its crafting station and
/// the item it unlocks.
struct Entry {
    station: Option<ClientItemStackState>,
    result: Option<ClientItemStackState>,
}

pub struct RecipeToast {
    entries: Vec<Entry>,
    title: Vec<Run>,
    description: Vec<Run>,
    last_changed: i64,
    changed: bool,
    wanted: Visibility,
    displayed: usize,
}

impl RecipeToast {
    pub fn new(env: &ToastEnv<'_>) -> Self {
        let words = |key: &str| {
            env.lang
                .runs(&Component::translatable(key), &Style::default())
        };
        Self {
            entries: Vec::new(),
            title: words("recipe.toast.title"),
            description: words("recipe.toast.description"),
            last_changed: 0,
            changed: false,
            wanted: Visibility::Hide,
            displayed: 0,
        }
    }

    /// `addItem`: another recipe, which keeps the toast longer.
    pub fn add_item(&mut self, station: Option<ItemId>, result: Option<ItemId>) {
        let stack = |item: ItemId| ClientItemStackState::new(1, item);
        self.entries.push(Entry {
            station: station.map(stack),
            result: result.map(stack),
        });
        self.changed = true;
    }

    /// The index of the recipe shown (`displayedRecipeIndex`), which is kept for the oracle.
    pub fn displayed_index(&self) -> usize {
        self.displayed
    }
}

impl Toast for RecipeToast {
    fn wanted_visibility(&self) -> Visibility {
        self.wanted
    }

    fn update(&mut self, env: &ToastEnv<'_>, fully_visible_for_ms: i64) {
        if self.changed {
            self.last_changed = fully_visible_for_ms;
            self.changed = false;
        }
        let count = self.entries.len();
        let hold = DISPLAY_TIME * env.display_time;
        self.wanted = if count == 0 || (fully_visible_for_ms - self.last_changed) as f64 >= hold {
            Visibility::Hide
        } else {
            Visibility::Show
        };
        if count > 0 {
            let each = (hold / count as f64).max(1.0);
            self.displayed = (fully_visible_for_ms as f64 / each % count as f64) as usize;
        }
    }

    fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>) {
        frame.sprite(atlas, "toast/recipe", 0, 0, self.width(env), self.height());
        frame.styled_text(atlas, env.font, &self.title, 30, 7, 0xFF50_0050, false);
        frame.styled_text(
            atlas,
            env.font,
            &self.description,
            30,
            18,
            0xFF00_0000,
            false,
        );
    }

    fn draw_items(&self, frame: &mut GuiFrame, item: &mut ItemDrawer<'_>, origin: (f32, f32)) {
        let Some(entry) = self.entries.get(self.displayed) else {
            return;
        };
        if let Some(station) = &entry.station {
            frame.item_scaled(item, station, (3.0, 3.0), (0.6, origin));
        }
        if let Some(result) = &entry.result {
            frame.item_at(item, result, origin.0 + 8.0, origin.1 + 8.0);
        }
    }

    fn as_recipe_mut(&mut self) -> Option<&mut RecipeToast> {
        Some(self)
    }
}
