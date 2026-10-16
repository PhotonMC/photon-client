use super::*;
use photon_core::ids::ItemId;

impl GenericScreen {
    /// The clicks the name box asked for since the last call.
    pub(in super::super) fn take_queued(&mut self) -> Vec<ScreenAction> {
        std::mem::take(&mut self.extras.queued)
    }

    /// Whether the stonecutter shows its recipes: an input and entries for it.
    pub(super) fn stone_display(menu: &ContainerMenu) -> bool {
        menu.slots.first().is_some_and(Option::is_some) && !menu.options.is_empty()
    }

    /// The stonecutter's entries shown: twelve from the first one.
    pub(super) fn stone_shown(&self) -> std::ops::Range<usize> {
        self.extras.start_index..(self.extras.start_index + 12).min(self.extras.options_count)
    }

    /// Where an entry is drawn, as the cell's corner.
    pub(super) fn stone_cell(&self, index: usize) -> (i32, i32) {
        let place = (index - self.extras.start_index) as i32;
        (52 + place % 4 * 16, 14 + place / 4 * 18 + 2)
    }

    /// `StonecutterScreen.mouseClicked`: a recipe pressed, or the scroll bar
    /// grabbed. The button pressed, if a recipe was.
    pub(in super::super) fn stone_click(
        &mut self,
        mouse: (f64, f64),
        menu: &ContainerMenu,
    ) -> Option<i32> {
        if self.kind != MenuKind::Stonecutter {
            return None;
        }
        self.extras.scrolling = false;
        if !Self::stone_display(menu) {
            return None;
        }
        let (mx, my) = (
            mouse.0 - f64::from(self.layout.left()),
            mouse.1 - f64::from(self.layout.top()),
        );
        for index in self.stone_shown() {
            let (x, y) = self.stone_cell(index);
            let (xx, yy) = (mx - f64::from(x), my - f64::from(y - 2));
            if (0.0..16.0).contains(&xx) && (0.0..18.0).contains(&yy) {
                return Some(index as i32);
            }
        }
        if (119.0..131.0).contains(&mx) && (9.0..63.0).contains(&my) {
            self.extras.scrolling = true;
        }
        None
    }

    /// `StonecutterScreen.mouseDragged`.
    pub(in super::super) fn stone_drag(&mut self, mouse: (f64, f64)) {
        if self.kind != MenuKind::Stonecutter || !self.extras.scrolling || self.stone_rows() <= 0 {
            return;
        }
        let top = f64::from(self.layout.top() + 14);
        let scroll = ((mouse.1 - top - 7.5) / (54.0 - 15.0)).clamp(0.0, 1.0) as f32;
        self.stone_scroll_to(scroll);
    }

    /// `StonecutterScreen.mouseScrolled`.
    pub(in super::super) fn stone_wheel(&mut self, y: f64) {
        let rows = self.stone_rows();
        if self.kind == MenuKind::Stonecutter && rows > 0 {
            let scroll = (self.extras.scroll_offs - (y / f64::from(rows)) as f32).clamp(0.0, 1.0);
            self.stone_scroll_to(scroll);
        }
    }

    /// `StonecutterScreen.mouseReleased`.
    pub(in super::super) fn stone_release(&mut self) {
        self.extras.scrolling = false;
    }

    /// The rows past the three shown.
    pub(super) fn stone_rows(&self) -> i32 {
        if self.extras.options_count > 12 {
            (self.extras.options_count as i32 + 3) / 4 - 3
        } else {
            0
        }
    }

    pub(super) fn stone_scroll_to(&mut self, scroll: f32) {
        self.extras.scroll_offs = scroll;
        let rows = self.stone_rows() as f32;
        self.extras.start_index = ((scroll * rows + 0.5) as usize) * 4;
    }

    /// `StonecutterScreen.extractBackground`'s recipe list and scroll bar.
    pub(super) fn stone_draw(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
    ) {
        // Any change of the input slot, its count too, starts the list over
        // (`containerChanged` runs on each change of the input container).
        let input = key_of(menu, 0);
        if input != self.extras.stone_input {
            self.extras.stone_input = input;
            self.extras.scroll_offs = 0.0;
            self.extras.start_index = 0;
        }
        self.extras.options_count = menu.options.len();
        if !Self::stone_display(menu) {
            draw.frame.sprite(
                draw.atlas,
                "container/stonecutter/scroller_disabled",
                left + 119,
                top + 15,
                12,
                15,
            );
            return;
        }
        let active = menu.options.len() > 12;
        let scroller = if active {
            "container/stonecutter/scroller"
        } else {
            "container/stonecutter/scroller_disabled"
        };
        let travel = (41.0 * self.extras.scroll_offs) as i32;
        draw.frame
            .sprite(draw.atlas, scroller, left + 119, top + 15 + travel, 12, 15);
        let selected = i32::from(menu.data.first().copied().unwrap_or(-1));
        let (mx, my) = (mouse.0 - f64::from(left), mouse.1 - f64::from(top));
        let mut hovered = None;
        for index in self.stone_shown() {
            let (x, y) = self.stone_cell(index);
            let over = mx >= f64::from(x)
                && my >= f64::from(y)
                && mx < f64::from(x + 16)
                && my < f64::from(y + 18);
            let sprite = if index as i32 == selected {
                "container/stonecutter/recipe_selected"
            } else if over {
                "container/stonecutter/recipe_highlighted"
            } else {
                "container/stonecutter/recipe"
            };
            draw.frame
                .sprite(draw.atlas, sprite, left + x, top + y - 1, 16, 18);
            if let Some(stack) = Self::option_stack(menu.options[index]) {
                draw.frame.item(draw.item, &stack, left + x, top + y);
                if over {
                    hovered = Some(stack);
                }
            }
        }
        if let Some(stack) = hovered {
            let lines = (draw.tooltip)(&stack, false);
            self.extras.pending_tooltip = Some(lines);
        }
    }

    /// What an entry shows, as a stack of one.
    pub(super) fn option_stack(item: Option<ItemId>) -> Option<photon_world::ClientItemStackState> {
        item.map(|item| photon_world::ClientItemStackState::new(1, item))
    }

    /// Whether the loom shows its patterns: a banner and a dye, room for
    /// another layer and something to offer.
    pub(super) fn loom_display(menu: &ContainerMenu) -> bool {
        let has = |slot: usize| menu.slots.get(slot).is_some_and(Option::is_some);
        has(0) && has(1) && menu.banner_layers < 6 && !menu.patterns.is_empty()
    }

    /// The rows past the four shown.
    pub(super) fn loom_rows(&self) -> i32 {
        ((self.extras.loom_count as i32 + 3) / 4 - 4).max(0)
    }

    /// `LoomScreen.mouseClicked`: a pattern pressed (its index), or the scroll
    /// bar grabbed.
    pub(in super::super) fn loom_click(
        &mut self,
        mouse: (f64, f64),
        menu: &ContainerMenu,
    ) -> Option<i32> {
        if self.kind != MenuKind::Loom {
            return None;
        }
        self.extras.loom_scrolling = false;
        if !Self::loom_display(menu) {
            return None;
        }
        let (mx, my) = (
            mouse.0 - f64::from(self.layout.left()),
            mouse.1 - f64::from(self.layout.top()),
        );
        let first = self.extras.loom_row * 4;
        let shown = first..(first + 16).min(menu.patterns.len());
        for index in shown {
            let place = (index - first) as i32;
            let (x, y) = (60 + place % 4 * 14, 13 + place / 4 * 14);
            if (0.0..14.0).contains(&(mx - f64::from(x)))
                && (0.0..14.0).contains(&(my - f64::from(y)))
            {
                return Some(index as i32);
            }
        }
        if (119.0..131.0).contains(&mx) && (9.0..65.0).contains(&my) {
            self.extras.loom_scrolling = true;
        }
        None
    }

    pub(super) fn loom_scroll_to(&mut self, offs: f32) {
        self.extras.loom_offs = offs;
        let rows = self.loom_rows() as f32;
        self.extras.loom_row = (offs * rows + 0.5) as usize;
    }

    /// `LoomScreen.mouseDragged`.
    pub(in super::super) fn loom_drag(&mut self, mouse: (f64, f64)) {
        if self.kind != MenuKind::Loom || !self.extras.loom_scrolling || self.loom_rows() <= 0 {
            return;
        }
        let top = f64::from(self.layout.top() + 13);
        let offs = ((mouse.1 - top - 7.5) / (56.0 - 15.0)).clamp(0.0, 1.0) as f32;
        self.loom_scroll_to(offs);
    }

    /// `LoomScreen.mouseScrolled`.
    pub(in super::super) fn loom_wheel(&mut self, y: f64) {
        let rows = self.loom_rows();
        if self.kind == MenuKind::Loom && rows > 0 {
            let offs = (self.extras.loom_offs - (y / f64::from(rows)) as f32).clamp(0.0, 1.0);
            self.loom_scroll_to(offs);
        }
    }

    /// `LoomScreen.mouseReleased`.
    pub(in super::super) fn loom_release(&mut self) {
        self.extras.loom_scrolling = false;
    }

    /// `LoomScreen.extractBackground`'s scroll bar, pattern buttons and mark.
    pub(super) fn loom_draw(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
    ) {
        self.extras.loom_count = menu.patterns.len();
        if self.extras.loom_row * 4 >= menu.patterns.len().max(1) {
            self.extras.loom_row = 0;
            self.extras.loom_offs = 0.0;
        }
        if menu.banner_layers >= 6 {
            draw.frame.sprite(
                draw.atlas,
                "container/loom/error",
                left + 143 - 5,
                top + 57 - 5,
                26,
                26,
            );
        }
        let display = Self::loom_display(menu);
        let scroller = if display && menu.patterns.len() > 16 {
            "container/loom/scroller"
        } else {
            "container/loom/scroller_disabled"
        };
        let travel = (41.0 * self.extras.loom_offs) as i32;
        draw.frame
            .sprite(draw.atlas, scroller, left + 119, top + 13 + travel, 12, 15);
        if !display {
            return;
        }
        let selected = if menu.patterns.len() == 1 {
            0
        } else {
            i32::from(menu.data.first().copied().unwrap_or(-1))
        };
        let dye = menu
            .slots
            .get(1)
            .and_then(Option::as_ref)
            .and_then(|stack| {
                photon_data::item::registry::ITEMS
                    .get(usize::try_from(stack.item_registry_id().0).ok()?)
            })
            .and_then(|item| item.name.strip_prefix("minecraft:")?.strip_suffix("_dye"))
            .unwrap_or("white");
        let first = self.extras.loom_row * 4;
        let (mx, my) = (mouse.0 - f64::from(left), mouse.1 - f64::from(top));
        let mut tip = None;
        for index in first..(first + 16).min(menu.patterns.len()) {
            let place = (index - first) as i32;
            let (x, y) = (60 + place % 4 * 14, 13 + place / 4 * 14);
            let over = mx >= f64::from(x)
                && my >= f64::from(y)
                && mx < f64::from(x + 14)
                && my < f64::from(y + 14);
            let sprite = if index as i32 == selected {
                "container/loom/pattern_selected"
            } else if over {
                "container/loom/pattern_highlighted"
            } else {
                "container/loom/pattern"
            };
            draw.frame
                .sprite(draw.atlas, sprite, left + x, top + y, 14, 14);
            let pattern = usize::try_from(menu.patterns[index])
                .ok()
                .and_then(|id| self.titles.patterns.get(id));
            if let Some((asset, key)) = pattern {
                // The banner's texture from its top left: 21 by 40 of the 64
                // by 64, drawn small over grey.
                let (px, py) = (left + x + 4, top + y + 2);
                draw.frame
                    .fill(draw.atlas, (px, py, px + 5, py + 10), 0xFF47_4F52);
                draw.frame.blit_region(
                    draw.atlas,
                    GuiBlend::Translucent,
                    &format!("banner/{asset}"),
                    (64, 64),
                    (0, 1, 21, 40),
                    (px, py, 5, 10),
                );
                if over && index as i32 != selected {
                    tip = Some(self.titles.lang.text(&format!("{key}.{dye}")));
                }
            }
        }
        if let Some(text) = tip {
            let lines = vec![vec![photon_game::text::Run {
                text,
                style: photon_game::text::Style::default(),
            }]];
            self.extras.pending_tooltip = Some(lines);
        }
    }

    /// The enchanting table's three rows: whether the player can pay for each.
    pub(super) fn enchant_can_afford(menu: &ContainerMenu, row: usize) -> bool {
        let cost = i32::from(menu.data.get(row).copied().unwrap_or(0));
        let lapis = menu
            .slots
            .get(1)
            .and_then(Option::as_ref)
            .map_or(0, photon_world::ClientItemStackState::item_count);
        menu.creative()
            || (lapis > row as i32 && menu.xp_level > row as i32 && menu.xp_level >= cost)
    }

    /// `EnchantmentScreen.mouseClicked`: the row pressed, if the menu would
    /// take the press (`clickMenuButton`).
    pub(in super::super) fn enchant_click(
        &self,
        mouse: (f64, f64),
        menu: &ContainerMenu,
    ) -> Option<i32> {
        if self.kind != MenuKind::Enchantment {
            return None;
        }
        let (mx, my) = (
            mouse.0 - f64::from(self.layout.left() + 60),
            mouse.1 - f64::from(self.layout.top()),
        );
        (0..3).find_map(|row| {
            let (xx, yy) = (mx, my - f64::from(14 + 19 * row as i32));
            let hit = (0.0..108.0).contains(&xx) && (0.0..19.0).contains(&yy);
            let cost = i32::from(menu.data.get(row).copied().unwrap_or(0));
            let has_item = menu.slots.first().is_some_and(Option::is_some);
            (hit && cost > 0 && has_item && Self::enchant_can_afford(menu, row))
                .then_some(row as i32)
        })
    }
}
