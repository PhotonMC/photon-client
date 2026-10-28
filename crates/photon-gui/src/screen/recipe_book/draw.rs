use super::*;
use crate::FrameRuns;
use photon_core::ids::ItemId;

impl RecipeBookPanel {
    /// What the search reads of each collection: the tooltip lines of its
    /// recipes' results and their ids (`SessionSearchTrees.updateRecipes`).
    pub(super) fn search_texts(&mut self, draw: &ScreenDraw<'_, '_>) {
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
                        photon_game::tooltip::search_lines(
                            &stack,
                            &self.assets.lang,
                            &self.assets.registries,
                        )
                        .into_iter()
                        .map(|line| line.to_lowercase()),
                    );
                    if let Some(item) = usize::try_from(item.0)
                        .ok()
                        .and_then(|item| photon_data::item::registry::ITEMS.get(item))
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
                    photon_game::menus::creative::query_matches(&text.lines, &text.ids, &query)
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
    pub(super) fn start_tab_animations(&mut self, book: &RecipeBook) {
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
    pub(super) fn report_seen(&mut self, book: &RecipeBook) {
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

    pub(super) fn draw_panel(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        mouse: (f64, f64),
        pages: usize,
    ) {
        let (xo, yo) = self.origin();
        draw.frame.blit_texture(
            draw.atlas,
            "recipe_book",
            (256, 256),
            (1, 1),
            (xo, yo, PANEL.0, PANEL.1),
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
            (xo + 25, yo + 13, 81, 14),
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

    pub(super) fn tooltip(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        mouse: (f64, f64),
        lines: Vec<Vec<Run>>,
    ) {
        draw.frame.tooltip(
            draw.atlas,
            draw.font,
            &lines,
            (mouse.0 as i32, mouse.1 as i32),
            self.size,
        );
    }

    pub(super) fn draw_tabs(&self, draw: &mut ScreenDraw<'_, '_>, (xo, yo): (i32, i32)) {
        let icons: &[&[&str]] = match self.kind {
            BookKind::Crafting { .. } => &[
                &["iron_axe", "golden_sword"],
                &["bricks"],
                &["lava_bucket", "apple"],
                &["redstone"],
            ],
            BookKind::Furnace(photon_game::menus::container::Smelter::Furnace) => {
                &[&["porkchop"], &["stone"], &["lava_bucket", "emerald"]]
            }
            BookKind::Furnace(photon_game::menus::container::Smelter::BlastFurnace) => {
                &[&["redstone_ore"], &["iron_shovel", "golden_leggings"]]
            }
            BookKind::Furnace(photon_game::menus::container::Smelter::Smoker) => &[&["porkchop"]],
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
                if let Some(id) = photon_data::item::registry::index(&format!("minecraft:{name}")) {
                    let stack = Stack::new(1, ItemId(id as i32));
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
    pub(super) fn squeeze_button(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        mark: photon_render::gui::FrameMark,
        collection: i32,
        (x, y): (i32, i32),
    ) {
        if let Some(time) = self.squeezing.1.get(&collection) {
            let squeeze = 1.0 + 0.1 * (time / 15.0 * std::f32::consts::PI).sin();
            draw.frame
                .scale_about(mark, (squeeze, squeeze), ((x + 8) as f32, (y + 12) as f32));
        }
    }

    pub(super) fn draw_recipes(
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
            let results: Vec<Vec<(ItemId, i32)>> = offered
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
}
