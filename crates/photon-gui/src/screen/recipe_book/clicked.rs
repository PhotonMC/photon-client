use super::*;

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

    pub(super) fn too_narrow(&self) -> bool {
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
    pub(super) fn origin(&self) -> (i32, i32) {
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

    pub(super) fn settings_action(&self, open: bool, filtering: bool) -> ScreenAction {
        ScreenAction::RecipeBookSettings {
            book_type: self.kind.book_type() as i32,
            open,
            filtering,
        }
    }

    pub(super) fn button_corner(&self, left: i32) -> (i32, i32) {
        (left + self.button.0, self.size.1 / 2 + self.button.1)
    }

    pub(super) fn in_rect(mouse: (f64, f64), (x, y, w, h): (i32, i32, i32, i32)) -> bool {
        mouse.0 >= f64::from(x)
            && mouse.1 >= f64::from(y)
            && mouse.0 < f64::from(x + w)
            && mouse.1 < f64::from(y + h)
    }

    /// The tabs that show, as indices into the book's tabs (0 the search).
    pub(super) fn visible_tabs(&self) -> Vec<usize> {
        (0..=self.kind.tabs().len())
            .filter(|tab| self.model.tab_visible(*tab))
            .collect()
    }

    pub(super) fn total_pages(&self) -> usize {
        self.listed.len().div_ceil(RECIPES_PER_PAGE)
    }

    /// `SlotSelectTime.currentIndex`: which recipe and which item the buttons
    /// and the ghost show.
    pub(super) fn select_time(&self) -> usize {
        (self.time / 30.0).floor() as usize
    }

    /// The recipe button cell's corner.
    pub(super) fn cell(&self, index: usize) -> (i32, i32) {
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
            click_sound(button, false);
            self.queued
                .push(self.settings_action(!self.visible, self.filtering));
            self.swallow_release = true;
            return true;
        }
        if !self.visible {
            return false;
        }
        let (xo, yo) = self.origin();
        self.page_arrow_clicked(mouse, button, (xo, yo))
            || self.cells_clicked(mouse, button, shift)
            || self.search_and_tabs_clicked(mouse, button, (xo, yo))
    }

    /// `RecipeBookPage.mouseClicked`: the arrows.
    fn page_arrow_clicked(&mut self, mouse: (f64, f64), button: i32, (xo, yo): (i32, i32)) -> bool {
        let pages = self.total_pages();
        if pages > 1
            && self.model.page + 1 < pages
            && Self::in_rect(mouse, (xo + 93, yo + 137, 12, 17))
        {
            click_sound(button, false);
            self.model.page += 1;
            return true;
        }
        if self.model.page > 0 && Self::in_rect(mouse, (xo + 38, yo + 137, 12, 17)) {
            click_sound(button, false);
            self.model.page -= 1;
            return true;
        }
        false
    }

    /// `RecipeBookPage.mouseClicked`: the recipe buttons.
    fn cells_clicked(&mut self, mouse: (f64, f64), button: i32, shift: bool) -> bool {
        let first = self.model.page * RECIPES_PER_PAGE;
        for cell in 0..RECIPES_PER_PAGE {
            let Some(&collection) = self.listed.get(first + cell) else {
                break;
            };
            let (x, y) = self.cell(cell);
            if Self::in_rect(mouse, (x, y, 25, 25)) {
                click_sound(button, true);
                if button == 0 {
                    return self.place(collection, shift);
                }
                if button == 1 {
                    self.open_overlay(collection, (x, y));
                }
                return true;
            }
        }
        false
    }

    /// The search box, the magnifier left of it, and the tabs.
    fn search_and_tabs_clicked(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        (xo, yo): (i32, i32),
    ) -> bool {
        if Self::in_rect(mouse, (xo + 8, yo + 13, 98, 14)) {
            self.focused = true;
            self.focused_at = Instant::now();
            return true;
        }
        self.focused = false;
        if Self::in_rect(mouse, (xo + 110, yo + 12, 26, 16)) {
            click_sound(button, false);
            self.queued
                .push(self.settings_action(self.visible, !self.filtering));
            self.swallow_release = true;
            return true;
        }
        for (index, tab) in self.visible_tabs().into_iter().enumerate() {
            if Self::in_rect(mouse, (xo - 30, yo + 3 + 27 * index as i32, 35, 27)) {
                click_sound(button, false);
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
    pub(super) fn open_overlay(&mut self, collection: usize, (button_x, button_y): (i32, i32)) {
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
    pub(super) fn overlay_clicked(&mut self, mouse: (f64, f64), button: i32, shift: bool) -> bool {
        let hit = self.overlay.as_ref().and_then(|overlay| {
            overlay
                .buttons
                .iter()
                .find(|b| Self::in_rect(mouse, (b.x, b.y, 24, 24)))
                .map(|b| (b.recipe, b.craftable))
        });
        match (button, hit) {
            (0, Some((recipe, craftable))) => {
                crate::ui_sound::play_down_sound();
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
    pub(super) fn place(&mut self, collection: usize, shift: bool) -> bool {
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

    pub(super) fn try_place(&mut self, id: i32, craftable: bool, shift: bool) -> bool {
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
    pub(super) fn search_changed(&mut self) {
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
}

/// `AbstractWidget.mouseClicked`: a widget clicks for the primary button, and a recipe button
/// for the secondary one too (`isValidClickButton`).
fn click_sound(button: i32, secondary_too: bool) {
    if button == 0 || (secondary_too && button == 1) {
        crate::ui_sound::play_down_sound();
    }
}
