//! The recipe book beside a crafting screen (`RecipeBookComponent`): the
//! toggle button, the panel with its tabs, search box, filter button and
//! pages of recipe buttons, and the ghost recipe over the menu's slots. The
//! recipes and what can be crafted are [`voxel_game::recipe_book`]'s.
//!
//! Not drawn: the overlay of a collection's other recipes (right click),
//! the highlight of new recipes and the toasts; a right click places the
//! button's recipe like a left one.

use std::sync::Arc;
use std::time::Instant;

use super::{ScreenAction, ScreenDraw};
use crate::gui::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::font::Font;
use voxel_game::container_menu::{ContainerMenu, MenuKind};
use voxel_game::recipe_book::{BookKind, BookModel, RECIPES_PER_PAGE, display_items, ghost_slots};
use voxel_world::{ClientItemStackState as Stack, RecipeBook};

/// The panel's width and height, and where the button, tabs and pages sit
/// in it.
const PANEL: (i32, i32) = (147, 166);
/// A GUI narrower than this has no room beside the screen.
const TOO_NARROW: i32 = 379;

/// What the panel writes, in the client's language, and the font its search
/// box measures with.
#[derive(Debug, Clone, Default)]
pub struct RecipeAssets {
    pub font: Arc<Font>,
    pub search_hint: String,
    /// The filter's tooltip while it shows what can be crafted, by book type.
    pub filtered: [String; 4],
    pub all: String,
}

/// The recipe book of one screen.
#[derive(Debug, Clone)]
pub struct RecipeBookPanel {
    kind: BookKind,
    image_width: i32,
    /// The toggle button's corner from the image's left and the GUI's middle.
    button: (i32, i32),
    assets: Arc<RecipeAssets>,
    model: BookModel,
    search: EditBox,
    focused: bool,
    focused_at: Instant,
    started: Instant,
    size: (i32, i32),
    visible: bool,
    filtering: bool,
    listed: Vec<usize>,
    last_placed: Option<i32>,
    queued: Vec<ScreenAction>,
    /// A press a widget took, so the release is not the menu's.
    swallow_release: bool,
}

impl RecipeBookPanel {
    pub fn new(
        kind: BookKind,
        image_width: i32,
        button: (i32, i32),
        assets: Arc<RecipeAssets>,
        size: (i32, i32),
    ) -> Self {
        Self {
            kind,
            image_width,
            button,
            assets,
            model: BookModel::new(kind),
            search: EditBox::new(73, 50).with_colour(0xFFFF_FFFF),
            focused: false,
            focused_at: Instant::now(),
            started: Instant::now(),
            size,
            visible: false,
            filtering: false,
            listed: Vec::new(),
            last_placed: None,
            queued: Vec::new(),
            swallow_release: false,
        }
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
    }

    fn too_narrow(&self) -> bool {
        self.size.0 < TOO_NARROW
    }

    /// `AbstractRecipeBookScreen.initialLeftPos`: the screen's `leftPos`,
    /// moved right by the open panel when there is room.
    pub fn left_pos(&self) -> i32 {
        if self.visible && !self.too_narrow() {
            177 + (self.size.0 - self.image_width - 200) / 2
        } else {
            (self.size.0 - self.image_width) / 2
        }
    }

    /// The panel's corner (`xo`, `yo`).
    fn origin(&self) -> (i32, i32) {
        let x_offset = if self.too_narrow() { 0 } else { 86 };
        (
            (self.size.0 - PANEL.0) / 2 - x_offset,
            (self.size.1 - PANEL.1) / 2,
        )
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    /// `tick`: open and filtering as the book's settings have them.
    pub fn sync(&mut self, book: &RecipeBook) {
        let settings = book.settings[self.kind.book_type()];
        self.visible = settings.open;
        self.filtering = settings.filtering;
    }

    pub fn take_swallowed_release(&mut self) -> bool {
        std::mem::take(&mut self.swallow_release)
    }

    pub fn drain_actions(&mut self) -> Vec<ScreenAction> {
        std::mem::take(&mut self.queued)
    }

    fn settings_action(&self, open: bool, filtering: bool) -> ScreenAction {
        ScreenAction::RecipeBookSettings {
            book_type: self.kind.book_type() as i32,
            open,
            filtering,
        }
    }

    fn button_corner(&self, left: i32) -> (i32, i32) {
        (left + self.button.0, self.size.1 / 2 + self.button.1)
    }

    fn in_rect(mouse: (f64, f64), (x, y, w, h): (i32, i32, i32, i32)) -> bool {
        mouse.0 >= f64::from(x)
            && mouse.1 >= f64::from(y)
            && mouse.0 < f64::from(x + w)
            && mouse.1 < f64::from(y + h)
    }

    /// The tabs that show, as indices into the book's tabs (0 the search).
    fn visible_tabs(&self) -> Vec<usize> {
        (0..=self.kind.tabs().len())
            .filter(|tab| self.model.tab_visible(*tab))
            .collect()
    }

    fn total_pages(&self) -> usize {
        self.listed.len().div_ceil(RECIPES_PER_PAGE)
    }

    /// The recipe button cell's corner.
    fn cell(&self, index: usize) -> (i32, i32) {
        let (xo, yo) = self.origin();
        (
            xo + 11 + 25 * (index % 5) as i32,
            yo + 31 + 25 * (index / 5) as i32,
        )
    }

    /// A press, before the screen's own: the toggle button, then the panel's
    /// widgets while it is open. Whether the book took it.
    pub fn mouse_clicked(&mut self, mouse: (f64, f64), shift: bool, left: i32) -> bool {
        let (bx, by) = self.button_corner(left);
        if Self::in_rect(mouse, (bx, by, 20, 18)) {
            self.queued
                .push(self.settings_action(!self.visible, self.filtering));
            self.swallow_release = true;
            return true;
        }
        if !self.visible {
            return false;
        }
        let (xo, yo) = self.origin();
        // The filter button, the page arrows and the tabs.
        if Self::in_rect(mouse, (xo + 110, yo + 12, 26, 16)) {
            self.queued
                .push(self.settings_action(self.visible, !self.filtering));
            self.model.page = 0;
            self.swallow_release = true;
            return true;
        }
        let pages = self.total_pages();
        if pages > 1
            && self.model.page + 1 < pages
            && Self::in_rect(mouse, (xo + 93, yo + 137, 12, 17))
        {
            self.model.page += 1;
            return true;
        }
        if self.model.page > 0 && Self::in_rect(mouse, (xo + 38, yo + 137, 12, 17)) {
            self.model.page -= 1;
            return true;
        }
        for (index, tab) in self.visible_tabs().into_iter().enumerate() {
            if Self::in_rect(mouse, (xo - 30, yo + 3 + 27 * index as i32, 35, 27)) {
                if self.model.tab != tab {
                    self.model.tab = tab;
                    self.model.page = 0;
                }
                return true;
            }
        }
        // The recipes of the page.
        let first = self.model.page * RECIPES_PER_PAGE;
        for cell in 0..RECIPES_PER_PAGE {
            let Some(&collection) = self.listed.get(first + cell) else {
                break;
            };
            let (x, y) = self.cell(cell);
            if Self::in_rect(mouse, (x, y, 25, 25)) {
                self.place(collection, shift);
                return true;
            }
        }
        // The search box, and the magnifier left of it.
        if Self::in_rect(mouse, (xo + 8, yo + 13, 98, 14)) {
            self.focused = true;
            self.focused_at = Instant::now();
            return true;
        }
        self.focused = false;
        false
    }

    /// `tryPlaceRecipe`: the recipe the button shows is asked of the server,
    /// unless it cannot be crafted and was the last one asked for.
    fn place(&mut self, collection: usize, shift: bool) {
        let Some(collection) = self.model.collections().get(collection) else {
            return;
        };
        let entries = collection.selected_entries(None);
        if entries.is_empty() {
            return;
        }
        let id = entries[self.shown_index(entries.len())];
        if !collection.craftable.contains(&id) && self.last_placed == Some(id) {
            return;
        }
        self.last_placed = Some(id);
        self.queued.push(ScreenAction::ClearGhost);
        self.queued
            .push(ScreenAction::PlaceRecipe { recipe: id, shift });
    }

    /// Which of a collection's recipes the button shows now: one for each 30
    /// ticks.
    fn shown_index(&self, count: usize) -> usize {
        let ticks = self.started.elapsed().as_secs_f32() * 20.0;
        (ticks / 30.0) as usize % count.max(1)
    }

    /// A slot of the menu's ingredients or result was clicked: the ghost goes.
    pub fn slot_clicked(&mut self, slot: i32) {
        let own = match self.kind {
            BookKind::Crafting { size } => size * size + 1,
            BookKind::Furnace(_) => 3,
        };
        if (0..own).contains(&slot) {
            self.queued.push(ScreenAction::ClearGhost);
        }
    }

    /// Keys go to the search box while it has the focus. Whether it took it.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        if !self.visible || !self.focused {
            return false;
        }
        if key == glfw::Key::Escape {
            return false;
        }
        if let Some(edit) = edit_key(key) {
            let font = self.assets.font.clone();
            self.search
                .key_pressed(edit, control, shift, &font, clipboard);
            self.search_changed();
        }
        true
    }

    pub fn char_typed(&mut self, character: char) -> bool {
        if !self.visible || !self.focused {
            return false;
        }
        let font = self.assets.font.clone();
        if self.search.char_typed(character, &font) {
            self.search_changed();
        }
        true
    }

    fn search_changed(&mut self) {
        let value = self.search.value();
        if value != self.model.search {
            self.model.search = value;
            self.model.page = 0;
        }
    }

    /// The panel and the toggle button, under the screen's slots.
    pub fn draw(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        left: i32,
    ) {
        self.sync(draw.book);
        self.model.refresh(draw.book, menu, draw.tags);
        let name = |item: i32| -> String {
            usize::try_from(item)
                .ok()
                .and_then(|id| voxel_data::item::registry::ITEMS.get(id))
                .map_or_else(String::new, |item| {
                    let path = item.name.trim_start_matches("minecraft:");
                    format!("{path} {}", path.replace('_', " "))
                })
        };
        self.listed = self.model.listed(self.filtering, &name, draw.book);
        let pages = self.total_pages();
        if self.model.page >= pages.max(1) {
            self.model.page = 0;
        }
        if !self.model.tab_visible(self.model.tab) {
            self.model.tab = 0;
        }
        let (bx, by) = self.button_corner(left);
        let button = if Self::in_rect(mouse, (bx, by, 20, 18)) {
            "recipe_book/button_highlighted"
        } else {
            "recipe_book/button"
        };
        draw.frame.sprite(draw.atlas, button, bx, by, 20, 18);
        if !self.visible {
            return;
        }
        self.draw_panel(draw, mouse, pages);
    }

    fn draw_panel(&mut self, draw: &mut ScreenDraw<'_, '_>, mouse: (f64, f64), pages: usize) {
        let (xo, yo) = self.origin();
        draw.frame.blit_texture(
            draw.atlas,
            "recipe_book",
            (256, 256),
            (1, 1),
            xo,
            yo,
            PANEL.0,
            PANEL.1,
        );
        // The search box: a frame round black, the text or the hint in it.
        let frame = if self.focused {
            0xFFFF_FFFF
        } else {
            0xFFA0_A0A0
        };
        draw.frame
            .fill(draw.atlas, (xo + 25, yo + 13, xo + 106, yo + 27), frame);
        draw.frame.fill(
            draw.atlas,
            (xo + 26, yo + 14, xo + 105, yo + 26),
            0xFF00_0000,
        );
        if self.search.is_empty() && !self.focused {
            draw.frame.text_with_shadow(
                draw.atlas,
                draw.font,
                &self.assets.search_hint,
                xo + 29,
                yo + 16,
                0xFFAA_AAAA,
            );
        } else {
            self.search.draw(
                draw.frame,
                draw.atlas,
                draw.font,
                (xo + 29, yo + 16),
                self.focused_at,
            );
        }
        // The filter button.
        let prefix = if matches!(self.kind, BookKind::Furnace(_)) {
            "furnace_"
        } else {
            ""
        };
        let state = if self.filtering {
            "enabled"
        } else {
            "disabled"
        };
        let over = Self::in_rect(mouse, (xo + 110, yo + 12, 26, 16));
        let highlight = if over { "_highlighted" } else { "" };
        draw.frame.sprite(
            draw.atlas,
            &format!("recipe_book/{prefix}filter_{state}{highlight}"),
            xo + 110,
            yo + 12,
            26,
            16,
        );
        self.draw_tabs(draw, (xo, yo));
        self.draw_recipes(draw, mouse, pages, (xo, yo));
        if over {
            let text = if self.filtering {
                &self.assets.filtered[self.kind.book_type()]
            } else {
                &self.assets.all
            };
            let lines = vec![vec![voxel_game::text::Run {
                text: text.clone(),
                style: voxel_game::text::Style::default(),
            }]];
            draw.frame.tooltip(
                draw.atlas,
                draw.font,
                &lines,
                (mouse.0 as i32, mouse.1 as i32),
                self.size,
            );
        }
    }

    fn draw_tabs(&self, draw: &mut ScreenDraw<'_, '_>, (xo, yo): (i32, i32)) {
        let icons: &[&[&str]] = match self.kind {
            BookKind::Crafting { .. } => &[
                &["iron_axe", "golden_sword"],
                &["bricks"],
                &["lava_bucket", "apple"],
                &["redstone"],
            ],
            BookKind::Furnace(voxel_game::container_menu::Smelter::Furnace) => {
                &[&["porkchop"], &["stone"], &["lava_bucket", "emerald"]]
            }
            BookKind::Furnace(voxel_game::container_menu::Smelter::BlastFurnace) => {
                &[&["redstone_ore"], &["iron_shovel", "golden_leggings"]]
            }
            BookKind::Furnace(voxel_game::container_menu::Smelter::Smoker) => &[&["porkchop"]],
        };
        for (index, tab) in self.visible_tabs().into_iter().enumerate() {
            let selected = tab == self.model.tab;
            let y = yo + 3 + 27 * index as i32;
            let x = xo - 30 - if selected { 2 } else { 0 };
            let sprite = if selected {
                "recipe_book/tab_selected"
            } else {
                "recipe_book/tab"
            };
            draw.frame.sprite(draw.atlas, sprite, x, y, 35, 27);
            let names: &[&str] = if tab == 0 {
                &["compass"]
            } else {
                icons[tab - 1]
            };
            let x_offsets: &[i32] = if names.len() == 2 { &[3, 14] } else { &[9] };
            for (name, dx) in names.iter().zip(x_offsets) {
                if let Some(id) = voxel_data::item::registry::index(&format!("minecraft:{name}")) {
                    let stack = Stack::new(1, id as i32);
                    draw.frame.item(draw.item, &stack, x + dx, y + 5);
                }
            }
        }
    }

    fn draw_recipes(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        mouse: (f64, f64),
        pages: usize,
        (xo, yo): (i32, i32),
    ) {
        let first = self.model.page * RECIPES_PER_PAGE;
        let mut hovered = None;
        for cell in 0..RECIPES_PER_PAGE {
            let Some(&index) = self.listed.get(first + cell) else {
                break;
            };
            let collection = &self.model.collections()[index];
            let (x, y) = self.cell(cell);
            let many = collection.selected.len() > 1;
            let sprite = match (collection.has_craftable(), many) {
                (true, false) => "slot_craftable",
                (true, true) => "slot_many_craftable",
                (false, false) => "slot_uncraftable",
                (false, true) => "slot_many_uncraftable",
            };
            draw.frame
                .sprite(draw.atlas, &format!("recipe_book/{sprite}"), x, y, 25, 25);
            let entries = collection.selected_entries(None);
            let Some(entry) = entries
                .get(self.shown_index(entries.len()))
                .and_then(|id| draw.book.known.get(id))
            else {
                continue;
            };
            let items = display_items(entry.display.result(), draw.tags);
            if let Some(&item) = items.get(self.shown_index(items.len())) {
                let stack = Stack::new(1, item);
                draw.frame.item(draw.item, &stack, x + 4, y + 4);
                if Self::in_rect(mouse, (x, y, 25, 25)) {
                    hovered = Some(stack);
                }
            }
        }
        // The page arrows and the page number.
        if pages > 1 {
            let text = format!("{}/{}", self.model.page + 1, pages);
            let width = draw.font.width(&text);
            draw.frame.text(
                draw.atlas,
                draw.font,
                &text,
                xo + 73 - width / 2,
                yo + 141,
                0xFFFF_FFFF,
            );
        }
        for (shown, name, x) in [
            (self.model.page + 1 < pages, "page_forward", xo + 93),
            (self.model.page > 0, "page_backward", xo + 38),
        ] {
            if shown {
                let over = Self::in_rect(mouse, (x, yo + 137, 12, 17));
                let highlight = if over { "_highlighted" } else { "" };
                draw.frame.sprite(
                    draw.atlas,
                    &format!("recipe_book/{name}{highlight}"),
                    x,
                    yo + 137,
                    12,
                    17,
                );
            }
        }
        if let Some(stack) = hovered {
            let lines = (draw.tooltip)(&stack, false);
            draw.frame.tooltip(
                draw.atlas,
                draw.font,
                &lines,
                (mouse.0 as i32, mouse.1 as i32),
                self.size,
            );
        }
    }

    /// `GhostSlots`: the ghost recipe over the menu's slots, each slot's
    /// item a translucent copy in a tinted square.
    pub fn draw_ghost(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
        position: &dyn Fn(usize) -> (i32, i32),
    ) {
        let Some((container, display)) = &draw.book.ghost else {
            return;
        };
        if *container != menu.container_id || matches!(menu.kind, MenuKind::Chest(_)) {
            return;
        }
        let empty = |slot: usize| menu.slots.get(slot).is_none_or(Option::is_none);
        for (slot, shown, is_result) in ghost_slots(self.kind, display, &empty) {
            let (x, y) = position(slot);
            let (x, y) = (left + x, top + y);
            let (from, to) = if is_result {
                ((x - 4, y - 4), (x + 20, y + 20))
            } else {
                ((x, y), (x + 16, y + 16))
            };
            draw.frame
                .fill(draw.atlas, (from.0, from.1, to.0, to.1), 0x3100_0000);
            let items = display_items(&shown, draw.tags);
            if let Some(&item) = items.get(self.shown_index(items.len())) {
                draw.frame.item(draw.item, &Stack::new(1, item), x, y);
            }
            draw.frame
                .fill(draw.atlas, (x, y, x + 16, y + 16), 0x3100_FFFF);
        }
    }
}
