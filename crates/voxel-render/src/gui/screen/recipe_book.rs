//! The recipe book beside a crafting screen (`RecipeBookComponent`): the
//! toggle button, the panel with its tabs, search box, filter button and
//! pages of recipe buttons, and the ghost recipe over the menu's slots. The
//! recipes and what can be crafted are [`voxel_game::recipe_book`]'s.
//!
//! Not drawn: the overlay of a collection's other recipes (right click),
//! the squeeze animation of new recipes' buttons and tabs, and the toasts.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::Instant;

use super::{ScreenAction, ScreenDraw};
use crate::gui::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::font::Font;
use voxel_game::container_menu::{ContainerMenu, MenuKind};
use voxel_game::recipe_book::{
    BookKind, BookModel, Collection, OverlaySlot, RECIPES_PER_PAGE, button_stack, display_stacks,
    ghost_slots, overlay_origin, overlay_slots, same_display,
};
use voxel_game::text::{Lang, Run, Style};
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
    pub lang: Arc<Lang>,
    pub registries: Arc<voxel_game::registries::Registries>,
    pub search_hint: String,
    /// The filter's tooltip while it shows what can be crafted, by book type.
    pub filtered: [String; 4],
    pub all: String,
    pub more_recipes: String,
    pub next_page: String,
    pub previous_page: String,
}

/// What a collection's search reads: its recipes' results' tooltip lines
/// (lowercased) and ids.
#[derive(Debug, Clone, Default)]
struct SearchText {
    lines: Vec<String>,
    ids: Vec<(String, String)>,
}

/// `OverlayRecipeComponent`: a collection's recipes in a box over its button.
#[derive(Debug, Clone)]
struct Overlay {
    x: i32,
    y: i32,
    buttons: Vec<OverlayButton>,
}

/// One recipe of the overlay.
#[derive(Debug, Clone)]
struct OverlayButton {
    x: i32,
    y: i32,
    recipe: i32,
    craftable: bool,
    slots: Vec<OverlaySlot>,
}

/// The recipe book of one screen.
#[derive(Debug, Clone)]
pub struct RecipeBookPanel {
    overlay: Option<Overlay>,
    kind: BookKind,
    image_width: i32,
    /// The toggle button's corner from the image's left and the GUI's middle.
    button: (i32, i32),
    assets: Arc<RecipeAssets>,
    model: BookModel,
    search: EditBox,
    focused: bool,
    focused_at: Instant,
    /// `RecipeBookComponent.time`: ticks the open book has shown, which the
    /// buttons and the ghost cycle with.
    time: f32,
    last_frame: Option<Instant>,
    size: (i32, i32),
    visible: bool,
    filtering: bool,
    listed: Vec<usize>,
    /// `lastPlacedRecipe`, and the recipe Enter places again.
    last_placed: Option<i32>,
    last_recipe: Option<(i32, i32)>,
    queued: Vec<ScreenAction>,
    /// A press a widget took, so the release is not the menu's.
    swallow_release: bool,
    /// `ignoreTextInput`: the key that focused the search is not typed.
    ignore_text: bool,
    /// The squeeze a tab (by place) and a button (by its collection's first
    /// recipe) is in, in ticks left; 15 when it starts (`animationTime`).
    tab_animation: HashMap<usize, f32>,
    button_animation: HashMap<i32, f32>,
    /// The squeezes as this frame draws them.
    squeezing: (HashMap<usize, f32>, HashMap<i32, f32>),
    /// The book revision the tabs were last checked for new recipes at.
    tabs_checked: Option<u64>,
    /// The overlay was just opened: its slots are filled at the next draw,
    /// when the tags are at hand.
    overlay_pending: bool,
    /// What the search reads of each collection, by its first recipe.
    texts: HashMap<i32, SearchText>,
    texts_for: Option<u64>,
    /// Highlighted recipes already reported seen.
    seen: BTreeSet<i32>,
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
            overlay: None,
            kind,
            image_width,
            button,
            assets,
            model: BookModel::new(kind),
            search: EditBox::new(73, 50).with_colour(0xFFFF_FFFF),
            focused: false,
            focused_at: Instant::now(),
            time: 0.0,
            last_frame: None,
            size,
            visible: false,
            filtering: false,
            listed: Vec::new(),
            last_placed: None,
            last_recipe: None,
            queued: Vec::new(),
            swallow_release: false,
            ignore_text: false,
            tab_animation: HashMap::new(),
            button_animation: HashMap::new(),
            tabs_checked: None,
            squeezing: (HashMap::new(), HashMap::new()),
            overlay_pending: false,
            texts: HashMap::new(),
            texts_for: None,
            seen: BTreeSet::new(),
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

    /// Whether the open book covers the screen, which then draws only its
    /// background and takes no click (`widthTooNarrow`).
    pub fn blocks_screen(&self) -> bool {
        self.visible && self.too_narrow()
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
        if !self.visible {
            // `setInvisible`: a closed book takes its overlay with it.
            self.overlay = None;
        }
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

    /// `SlotSelectTime.currentIndex`: which recipe and which item the buttons
    /// and the ghost show.
    fn select_time(&self) -> usize {
        (self.time / 30.0).floor() as usize
    }

    /// The recipe button cell's corner.
    fn cell(&self, index: usize) -> (i32, i32) {
        let (xo, yo) = self.origin();
        (
            xo + 11 + 25 * (index % 5) as i32,
            yo + 31 + 25 * (index / 5) as i32,
        )
    }

    /// `hasClickedOutside`'s part of the book: the panel's strip beside the
    /// image and the tabs are not outside.
    pub fn claims(&self, mouse: (f64, f64), (left, top): (i32, i32), image_height: i32) -> bool {
        if !self.visible {
            return false;
        }
        let in_strip = f64::from(left - 147) < mouse.0
            && mouse.0 < f64::from(left)
            && f64::from(top) < mouse.1
            && mouse.1 < f64::from(top + image_height);
        let (xo, yo) = self.origin();
        in_strip
            || self.visible_tabs().iter().enumerate().any(|(index, _)| {
                Self::in_rect(mouse, (xo - 30, yo + 3 + 27 * index as i32, 35, 27))
            })
    }

    /// A press, before the screen's own: the toggle button, then the panel's
    /// widgets while it is open. Whether the book took it.
    pub fn mouse_clicked(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        left: i32,
    ) -> bool {
        if self.visible && self.overlay.is_some() {
            return self.overlay_clicked(mouse, button, shift);
        }
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
        // `RecipeBookPage.mouseClicked`: the arrows, then the buttons.
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
        let first = self.model.page * RECIPES_PER_PAGE;
        for cell in 0..RECIPES_PER_PAGE {
            let Some(&collection) = self.listed.get(first + cell) else {
                break;
            };
            let (x, y) = self.cell(cell);
            if Self::in_rect(mouse, (x, y, 25, 25)) {
                if button == 0 {
                    return self.place(collection, shift);
                }
                if button == 1 {
                    self.open_overlay(collection, (x, y));
                }
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
        if Self::in_rect(mouse, (xo + 110, yo + 12, 26, 16)) {
            self.queued
                .push(self.settings_action(self.visible, !self.filtering));
            self.swallow_release = true;
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
        false
    }

    /// `RecipeBookPage.mouseClicked`'s right click: a button with several
    /// recipes opens the overlay over itself.
    fn open_overlay(&mut self, collection: usize, (button_x, button_y): (i32, i32)) {
        let Some(collection) = self.model.collections().get(collection) else {
            return;
        };
        if collection.offered(self.filtering).len() < 2 {
            return;
        }
        let craftable = collection.selected_entries(Some(true));
        let rest = if self.filtering {
            Vec::new()
        } else {
            collection.selected_entries(Some(false))
        };
        let (xo, yo) = self.origin();
        let total = craftable.len() + rest.len();
        let max_row = if total <= 16 { 4 } else { 5 };
        let (x, y) = overlay_origin(
            (button_x, button_y),
            (xo + PANEL.0 / 2, yo + 13 + PANEL.1 / 2),
            25.0,
            total,
        );
        let buttons = craftable
            .iter()
            .map(|id| (*id, true))
            .chain(rest.iter().map(|id| (*id, false)))
            .enumerate()
            .map(|(index, (recipe, craftable))| OverlayButton {
                x: x + 4 + 25 * (index % max_row) as i32,
                y: y + 5 + 25 * (index / max_row) as i32,
                recipe,
                craftable,
                slots: Vec::new(),
            })
            .collect();
        self.overlay = Some(Overlay { x, y, buttons });
        self.overlay_pending = true;
    }

    /// `OverlayRecipeComponent.mouseClicked` through the page: a left click
    /// on one of its recipes places it; any click takes the press, and one
    /// that misses closes the overlay.
    fn overlay_clicked(&mut self, mouse: (f64, f64), button: i32, shift: bool) -> bool {
        let hit = self.overlay.as_ref().and_then(|overlay| {
            overlay
                .buttons
                .iter()
                .find(|b| Self::in_rect(mouse, (b.x, b.y, 24, 24)))
                .map(|b| (b.recipe, b.craftable))
        });
        match (button, hit) {
            (0, Some((recipe, craftable))) => {
                if self.try_place(recipe, craftable, shift) {
                    self.last_recipe = Some((recipe, recipe));
                    if self.too_narrow() {
                        self.queued
                            .push(self.settings_action(false, self.filtering));
                    }
                }
            }
            _ => self.overlay = None,
        }
        true
    }

    /// `tryPlaceRecipe`: the recipe the button shows is asked of the server,
    /// unless it cannot be crafted and was the last one asked for. Whether
    /// the click was taken.
    fn place(&mut self, collection: usize, shift: bool) -> bool {
        let Some(collection) = self.model.collections().get(collection) else {
            return true;
        };
        let entries = collection.offered(self.filtering);
        if entries.is_empty() {
            return true;
        }
        let id = entries[self.select_time() % entries.len()];
        let first = collection.entries[0];
        let taken = self.try_place(id, collection.craftable.contains(&id), shift);
        if taken {
            self.last_recipe = Some((id, first));
            // `!isOffsetNextToMainGUI`: a book with no room beside the
            // screen closes once it has placed.
            if self.too_narrow() {
                self.queued
                    .push(self.settings_action(false, self.filtering));
            }
        }
        taken
    }

    fn try_place(&mut self, id: i32, craftable: bool, shift: bool) -> bool {
        if !craftable && self.last_placed == Some(id) {
            return false;
        }
        self.last_placed = Some(id);
        self.queued.push(ScreenAction::ClearGhost);
        self.queued
            .push(ScreenAction::PlaceRecipe { recipe: id, shift });
        true
    }

    /// A slot of the menu's ingredients or result was clicked: the ghost goes
    /// and the last placed recipe is forgotten (`slotClicked`).
    pub fn slot_clicked(&mut self, slot: i32) {
        let own = match self.kind {
            BookKind::Crafting { size } => size * size + 1,
            BookKind::Furnace(_) => 3,
        };
        if (0..own).contains(&slot) {
            self.last_placed = None;
            self.queued.push(ScreenAction::ClearGhost);
        }
    }

    /// `keyPressed`: Escape closes a book with no room, the search takes
    /// the keys while it has the focus, the chat key focuses it, and Enter or
    /// Space places the last recipe again. Whether the book took the key.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        self.ignore_text = false;
        if !self.visible {
            return false;
        }
        if key == glfw::Key::Escape && self.too_narrow() {
            self.queued
                .push(self.settings_action(false, self.filtering));
            return true;
        }
        if let Some(edit) = edit_key(key)
            && self.focused
        {
            let font = self.assets.font.clone();
            self.search
                .key_pressed(edit, control, shift, &font, clipboard);
            self.search_changed();
            return true;
        }
        if self.focused && key != glfw::Key::Escape {
            return true;
        }
        if key == glfw::Key::T && !self.focused {
            self.ignore_text = true;
            self.focused = true;
            self.focused_at = Instant::now();
            return true;
        }
        if matches!(
            key,
            glfw::Key::Enter | glfw::Key::KpEnter | glfw::Key::Space
        ) && let Some((id, _)) = self.last_recipe
        {
            self.retry(id, shift);
            return true;
        }
        false
    }

    pub fn key_released(&mut self) {
        self.ignore_text = false;
    }

    pub fn char_typed(&mut self, character: char) -> bool {
        if self.ignore_text || !self.visible || !self.focused {
            return false;
        }
        let font = self.assets.font.clone();
        if self.search.char_typed(character, &font) {
            self.search_changed();
        }
        true
    }

    /// `checkSearchStringUpdate`: the page stays where it is.
    fn search_changed(&mut self) {
        let value = self.search.value();
        if value != self.model.search {
            self.model.search = value;
        }
    }

    /// `tryPlaceRecipe` for Enter or Space: the last recipe again, when the
    /// book has it still.
    pub fn retry(&mut self, recipe: i32, shift: bool) {
        let craftable = self
            .model
            .collections()
            .iter()
            .any(|collection| collection.craftable.contains(&recipe));
        self.try_place(recipe, craftable, shift);
    }

    /// What the search reads of each collection: the tooltip lines of its
    /// recipes' results and their ids (`SessionSearchTrees.updateRecipes`).
    fn search_texts(&mut self, draw: &ScreenDraw<'_, '_>) {
        if self.texts_for == Some(draw.book.revision) {
            return;
        }
        self.texts_for = Some(draw.book.revision);
        self.texts.clear();
        for collection in self.model.collections() {
            let mut text = SearchText::default();
            for id in &collection.entries {
                let Some(entry) = draw.book.known.get(id) else {
                    continue;
                };
                for (item, _) in display_stacks(entry.display.result(), draw.tags) {
                    let stack = Stack::new(1, item);
                    text.lines.extend(
                        voxel_game::tooltip::search_lines(
                            &stack,
                            &self.assets.lang,
                            &self.assets.registries,
                        )
                        .into_iter()
                        .map(|line| line.to_lowercase()),
                    );
                    if let Some(item) = usize::try_from(item)
                        .ok()
                        .and_then(|item| voxel_data::item::registry::ITEMS.get(item))
                    {
                        let (namespace, path) = item
                            .name
                            .split_once(':')
                            .unwrap_or(("minecraft", item.name));
                        text.ids
                            .push((namespace.to_lowercase(), path.to_lowercase()));
                    }
                }
            }
            self.texts.insert(collection.entries[0], text);
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
        if self.visible {
            let now = Instant::now();
            if let Some(last) = self.last_frame
                && !draw.control_held
            {
                self.time += (now - last).as_secs_f32() * 20.0;
            }
            self.last_frame = Some(now);
        } else {
            self.last_frame = None;
            self.tabs_checked = None;
        }
        let dt = self
            .last_frame
            .map_or(0.0, |last| last.elapsed().as_secs_f32() * 20.0);
        self.start_tab_animations(draw.book);
        // The squeezes run down (as the frames draw them).
        let squeezing_tabs = self.tab_animation.clone();
        let squeezing_buttons = self.button_animation.clone();
        for time in self
            .tab_animation
            .values_mut()
            .chain(self.button_animation.values_mut())
        {
            *time -= dt;
        }
        self.tab_animation.retain(|_, time| *time > 0.0);
        self.button_animation.retain(|_, time| *time > 0.0);
        self.squeezing = (squeezing_tabs, squeezing_buttons);
        let query = self.model.search.to_lowercase();
        if !query.is_empty() {
            self.search_texts(draw);
        }
        let texts = &self.texts;
        self.listed = self
            .model
            .listed(self.filtering, &|collection: &Collection| {
                texts.get(&collection.entries[0]).is_some_and(|text| {
                    voxel_game::creative_menu::query_matches(&text.lines, &text.ids, &query)
                })
            });
        let pages = self.total_pages();
        if self.model.page >= pages.max(1) {
            self.model.page = 0;
        }
        if !self.model.tab_visible(self.model.tab) {
            self.model.tab = 0;
        }
        self.report_seen(draw.book);
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

    /// `RecipeBookTabButton.startAnimation`: a tab holding a recipe that is
    /// new squeezes, as the tabs are set up (on opening and when the book
    /// changes).
    fn start_tab_animations(&mut self, book: &RecipeBook) {
        if !self.visible || self.tabs_checked == Some(book.revision) {
            return;
        }
        self.tabs_checked = Some(book.revision);
        for tab in 1..=self.kind.tabs().len() {
            let categories = self.kind.tabs()[tab - 1];
            let new = self.model.collections().iter().any(|collection| {
                categories.contains(&collection.category)
                    && collection
                        .offered(self.filtering)
                        .iter()
                        .any(|id| book.highlight.contains(id))
            });
            if new {
                self.tab_animation.insert(tab, 15.0);
            }
        }
    }

    /// `RecipeButton.init`: a recipe that is new and shows on the page is
    /// reported seen.
    fn report_seen(&mut self, book: &RecipeBook) {
        if !self.visible || book.highlight.is_empty() {
            return;
        }
        let first = self.model.page * RECIPES_PER_PAGE;
        for &index in self.listed.iter().skip(first).take(RECIPES_PER_PAGE) {
            for id in self.model.collections()[index].offered(self.filtering) {
                if book.highlight.contains(&id) && self.seen.insert(id) {
                    self.queued.push(ScreenAction::RecipeSeen(id));
                    let first = self.model.collections()[index].entries[0];
                    self.button_animation.insert(first, 15.0);
                }
            }
        }
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
        // The search box: the field sprite, the text or the hint in it.
        let field = if self.focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        draw.frame.nine_slice(
            draw.atlas,
            field,
            (200, 20),
            1,
            false,
            xo + 25,
            yo + 13,
            81,
            14,
        );
        if self.search.is_empty() && !self.focused {
            let hint = [Run {
                text: self.assets.search_hint.clone(),
                style: Style {
                    colour: Some(0xAAAAAA),
                    italic: Some(true),
                    ..Style::default()
                },
            }];
            draw.frame.styled_text(
                draw.atlas,
                draw.font,
                &hint,
                xo + 29,
                yo + 16,
                0xFFFF_FFFF,
                true,
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
        self.draw_overlay(draw, mouse);
        if over {
            let text = if self.filtering {
                &self.assets.filtered[self.kind.book_type()]
            } else {
                &self.assets.all
            };
            self.tooltip(draw, mouse, vec![plain(text)]);
        }
    }

    fn tooltip(&self, draw: &mut ScreenDraw<'_, '_>, mouse: (f64, f64), lines: Vec<Vec<Run>>) {
        draw.frame.tooltip(
            draw.atlas,
            draw.font,
            &lines,
            (mouse.0 as i32, mouse.1 as i32),
            self.size,
        );
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
            let mark = draw.frame.mark();
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
            // `RecipeBookTabButton.extractContents`: a new recipe in the tab
            // squeezes it down and up about its left.
            if let Some(time) = self.squeezing.0.get(&tab) {
                let squeeze = 1.0 + 0.1 * (time / 15.0 * std::f32::consts::PI).sin();
                let base = xo - 30;
                draw.frame
                    .scale_about(mark, (1.0, squeeze), ((base + 8) as f32, (y + 12) as f32));
            }
        }
    }

    /// `RecipeButton.extractWidgetRenderState`: a button with a new recipe
    /// squeezes about a point near its top left.
    fn squeeze_button(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        mark: crate::gui::FrameMark,
        collection: i32,
        (x, y): (i32, i32),
    ) {
        if let Some(time) = self.squeezing.1.get(&collection) {
            let squeeze = 1.0 + 0.1 * (time / 15.0 * std::f32::consts::PI).sin();
            draw.frame
                .scale_about(mark, (squeeze, squeeze), ((x + 8) as f32, (y + 12) as f32));
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
            // `RecipeButton`: the recipes it offers, only the craftable ones
            // while filtering.
            let offered = collection.offered(self.filtering);
            let many = offered.len() > 1;
            let sprite = match (collection.has_craftable(), many) {
                (true, false) => "slot_craftable",
                (true, true) => "slot_many_craftable",
                (false, false) => "slot_uncraftable",
                (false, true) => "slot_many_uncraftable",
            };
            let mark = draw.frame.mark();
            draw.frame
                .sprite(draw.atlas, &format!("recipe_book/{sprite}"), x, y, 25, 25);
            let results: Vec<Vec<(i32, i32)>> = offered
                .iter()
                .filter_map(|id| draw.book.known.get(id))
                .map(|entry| display_stacks(entry.display.result(), draw.tags))
                .collect();
            let Some((item, count)) = button_stack(&results, self.select_time()) else {
                self.squeeze_button(draw, mark, collection.entries[0], (x, y));
                continue;
            };
            let stack = Stack::new(count, item);
            let mut offset = 4;
            if many && same_display(&results) {
                draw.frame
                    .item(draw.item, &stack, x + offset + 1, y + offset + 1);
                offset -= 1;
            }
            draw.frame.item(draw.item, &stack, x + offset, y + offset);
            self.squeeze_button(draw, mark, collection.entries[0], (x, y));
            if Self::in_rect(mouse, (x, y, 25, 25)) {
                hovered = Some((stack, many));
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
                xo - width / 2 + 73,
                yo + 141,
                0xFFFF_FFFF,
            );
        }
        let mut arrow_tooltip = None;
        for (shown, name, x, text) in [
            (
                self.model.page + 1 < pages,
                "page_forward",
                xo + 93,
                &self.assets.next_page,
            ),
            (
                self.model.page > 0,
                "page_backward",
                xo + 38,
                &self.assets.previous_page,
            ),
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
                if over {
                    arrow_tooltip = Some(text.clone());
                }
            }
        }
        if let Some(text) = arrow_tooltip {
            self.tooltip(draw, mouse, vec![plain(&text)]);
        }
        if let Some((stack, many)) = hovered.filter(|_| self.overlay.is_none()) {
            let mut lines = (draw.tooltip)(&stack, false);
            if many {
                lines.push(plain(&self.assets.more_recipes));
            }
            self.tooltip(draw, mouse, lines);
        }
    }

    /// `GhostSlots.extractTooltip`: over a ghost slot, the tooltip of the item
    /// it shows now. Whether it drew one (a tooltip asked for earlier wins).
    pub fn ghost_tooltip(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        hovered: Option<usize>,
        mouse: (f64, f64),
    ) -> bool {
        let (Some(slot), Some((container, display))) = (hovered, &draw.book.ghost) else {
            return false;
        };
        if *container != menu.container_id || !self.visible {
            return false;
        }
        let empty = |slot: usize| menu.slots.get(slot).is_none_or(Option::is_none);
        let Some((_, shown, _)) = ghost_slots(self.kind, display, &empty)
            .into_iter()
            .find(|(place, _, _)| *place == slot)
        else {
            return false;
        };
        let stacks = display_stacks(&shown, draw.tags);
        if stacks.is_empty() {
            return false;
        }
        let (item, count) = stacks[self.select_time() % stacks.len()];
        let lines = (draw.tooltip)(&Stack::new(count, item), false);
        self.tooltip(draw, mouse, lines);
        true
    }

    /// `OverlayRecipeComponent.extractRenderState`: the box, and in it each
    /// recipe's button with its ingredients drawn small.
    fn draw_overlay(&mut self, draw: &mut ScreenDraw<'_, '_>, mouse: (f64, f64)) {
        if self.overlay_pending {
            self.overlay_pending = false;
            if let Some(overlay) = &mut self.overlay {
                for button in &mut overlay.buttons {
                    if let Some(entry) = draw.book.known.get(&button.recipe) {
                        button.slots = overlay_slots(&entry.display, draw.tags);
                    }
                }
            }
        }
        let Some(overlay) = &self.overlay else {
            return;
        };
        let total = overlay.buttons.len();
        let max_row = if total <= 16 { 4 } else { 5 };
        let width = total.min(max_row) as i32;
        let height = total.div_ceil(max_row) as i32;
        draw.frame.nine_slice(
            draw.atlas,
            "recipe_book/overlay_recipe",
            (32, 32),
            4,
            false,
            overlay.x,
            overlay.y,
            width * 25 + 8,
            height * 25 + 8,
        );
        let folder = if matches!(self.kind, BookKind::Furnace(_)) {
            "furnace"
        } else {
            "crafting"
        };
        for button in &overlay.buttons {
            let over = Self::in_rect(mouse, (button.x, button.y, 24, 24));
            let sprite = format!(
                "recipe_book/{folder}_overlay{}{}",
                if button.craftable { "" } else { "_disabled" },
                if over { "_highlighted" } else { "" }
            );
            draw.frame
                .sprite(draw.atlas, &sprite, button.x, button.y, 24, 24);
            for (sx, sy, slot) in &button.slots {
                let stacks = display_stacks(slot, draw.tags);
                if stacks.is_empty() {
                    continue;
                }
                let (item, count) = stacks[self.select_time() % stacks.len()];
                let first = draw.frame.items.len();
                draw.frame.item(draw.item, &Stack::new(count, item), 0, 0);
                // The pose: to the slot's place, 0.375 of the size, about
                // the item's middle.
                let (tx, ty) = ((button.x + 2 + sx) as f32, (button.y + 2 + sy) as f32);
                draw.frame
                    .scale_items_from(first, 0.375, (tx - 0.375 * 8.0, ty - 0.375 * 8.0));
            }
        }
    }

    /// `GhostSlots`: the ghost recipe over the menu's slots, each slot's
    /// item a translucent copy between a red and a white wash.
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
        // `isBiggerResultSlot`: the inventory's result slot is not.
        let big_result = menu.kind != MenuKind::Inventory;
        let empty = |slot: usize| menu.slots.get(slot).is_none_or(Option::is_none);
        for (slot, shown, is_result) in ghost_slots(self.kind, display, &empty) {
            let stacks = display_stacks(&shown, draw.tags);
            if stacks.is_empty() {
                continue;
            }
            let (x, y) = position(slot);
            let (x, y) = (left + x, top + y);
            let rect = if is_result && big_result {
                (x - 4, y - 4, x + 20, y + 20)
            } else {
                (x, y, x + 16, y + 16)
            };
            draw.frame.fill(draw.atlas, rect, 0x30FF_0000);
            let (item, count) = stacks[self.select_time() % stacks.len()];
            let stack = Stack::new(count, item);
            draw.frame.item(draw.item, &stack, x, y);
            draw.frame
                .fill(draw.atlas, (x, y, x + 16, y + 16), 0x30FF_FFFF);
            if is_result && count != 1 {
                draw.frame.text_with_shadow(
                    draw.atlas,
                    draw.font,
                    &count.to_string(),
                    x + 19 - 2 - draw.font.width(&count.to_string()),
                    y + 6 + 3,
                    0xFFFF_FFFF,
                );
            }
        }
    }
}

/// A line of plain text.
fn plain(text: &str) -> Vec<Run> {
    vec![Run {
        text: text.to_owned(),
        style: Style::default(),
    }]
}
