use super::*;

impl CreativeScreen {
    /// A screen opened on a GUI of this size, on the tab last used, with the
    /// item tags the server last sent (`updateCreativeTags`).
    pub fn new(
        width: i32,
        height: i32,
        tab: usize,
        assets: CreativeAssets,
        tags: Arc<ItemTags>,
    ) -> Self {
        Self {
            layout: ContainerLayout::new(width, height, IMAGE),
            menu: CreativeMenu::new(tab, assets.trees.registries().clone()),
            hovered: None,
            mouse: ContainerMouse::default(),
            scrolling: false,
            has_clicked_outside: false,
            // `new EditBox(font, x + 82, y + 6, 80, 9, ...)` with no border,
            // fifty characters long.
            search: EditBox::new(80, 50).with_colour(0xFFFF_FFFF),
            ignore_text_input: false,
            focused_at: std::time::Instant::now(),
            assets,
            tags,
            visible_tags: Vec::new(),
        }
    }

    /// The tab the screen is on.
    pub fn tab(&self) -> usize {
        self.menu.tab()
    }

    /// The menu, which the clicks run in.
    pub fn menu_mut(&mut self) -> &mut CreativeMenu {
        &mut self.menu
    }

    /// A tab chosen from a script: a click on its button.
    pub fn select_tab_index(&mut self, tab: usize) {
        if tab < CREATIVE_TABS.len() {
            self.select_tab(tab);
        }
    }

    pub(super) fn inventory_tab(&self) -> bool {
        self.menu.kind() == TabKind::Inventory
    }

    pub(super) fn view<'a>(&'a self, inventory: &'a ContainerMenu) -> View<'a> {
        View {
            creative: &self.menu,
            inventory,
        }
    }

    pub(super) fn slot_count(&self) -> usize {
        if self.inventory_tab() {
            INVENTORY_SLOTS
        } else {
            PICKER_SLOTS
        }
    }

    /// `getHoveredSlot`.
    pub(super) fn hovered_slot(&self, mouse: (f64, f64)) -> Option<usize> {
        let inventory_tab = self.inventory_tab();
        self.layout.hovered(mouse, self.slot_count(), |index| {
            slot_position(inventory_tab, index)
        })
    }

    /// `checkTabClicked`, for the mouse relative to the image.
    pub(super) fn tab_clicked(&self, tab: usize, (xm, ym): (f64, f64)) -> bool {
        let (x, y) = (f64::from(tab_x(tab)), f64::from(tab_y(tab)));
        xm >= x && xm <= x + 26.0 && ym >= y && ym <= y + 32.0
    }

    /// `hasClickedOutside`: outside the image, and not on the tab. The
    /// reference compares the mouse itself with the tab's place in the
    /// image, and so does this.
    pub(super) fn clicked_outside(&mut self, mouse: (f64, f64)) -> bool {
        self.has_clicked_outside =
            self.layout.outside(mouse) && !self.tab_clicked(self.menu.tab(), mouse);
        self.has_clicked_outside
    }

    /// `insideScrollbar`.
    pub(super) fn inside_scrollbar(&self, (x, y): (f64, f64)) -> bool {
        let (x0, y0) = (self.layout.left() + 175, self.layout.top() + 18);
        x >= f64::from(x0)
            && y >= f64::from(y0)
            && x < f64::from(x0 + 14)
            && y < f64::from(y0 + 112)
            && CREATIVE_TABS[self.menu.tab()].can_scroll
    }

    /// `canScroll`.
    pub(super) fn can_scroll(&self) -> bool {
        CREATIVE_TABS[self.menu.tab()].can_scroll && self.menu.can_scroll()
    }

    pub(super) fn select_tab(&mut self, tab: usize) {
        let old = self.menu.tab();
        self.mouse.quick_craft_slots.clear();
        self.menu.select_tab(tab);
        if CREATIVE_TABS[tab].kind == TabKind::Search {
            self.focused_at = std::time::Instant::now();
            if old != tab {
                self.search.set_value("", &self.assets.font);
            }
            self.refresh_search_results();
        } else {
            self.search.set_value("", &self.assets.font);
        }
    }

    /// `refreshSearchResults`: everything for an empty box, otherwise what
    /// the search trees find: the tag tree for a search that starts with `#`,
    /// which also names the tags the tooltips list.
    pub(super) fn refresh_search_results(&mut self) {
        self.visible_tags.clear();
        let text = self.search.value();
        if text.is_empty() {
            self.menu.select_tab(self.menu.tab());
            return;
        }
        let found = if let Some(term) = text.strip_prefix('#') {
            self.visible_tags = visible_tags(&self.tags, term);
            tag_search(&self.tags, &term.to_lowercase())
        } else {
            self.assets.trees.index().search(&text.to_lowercase())
        };
        self.menu.show_items(&found);
    }

    /// The screen's clicks as `slotClicked` gets them.
    pub(super) fn actions(&self, slot_clicks: Vec<SlotClick>) -> Vec<ScreenAction> {
        let inventory_tab = self.inventory_tab();
        slot_clicks
            .into_iter()
            .map(|click| {
                ScreenAction::Creative(Click {
                    target: usize::try_from(click.slot)
                        .ok()
                        .map(|slot| target_of(inventory_tab, slot)),
                    slot_id: click.slot,
                    button: click.button,
                    input: click.kind,
                    outside: self.has_clicked_outside,
                })
            })
            .collect()
    }

    /// `keyPressed` in the search tab: a number key swaps with the hovered
    /// slot, the edit keys edit the search box, escape closes and nothing
    /// else does anything.
    pub(super) fn search_key_pressed(
        &mut self,
        key: glfw::Key,
        mods: KeyMods,
        inventory: &ContainerMenu,
        clipboard: &mut dyn Clipboard,
    ) -> KeyResult {
        let view = self.view(inventory);
        let numeric = matches!(
            key,
            glfw::Key::Num0
                | glfw::Key::Num1
                | glfw::Key::Num2
                | glfw::Key::Num3
                | glfw::Key::Num4
                | glfw::Key::Num5
                | glfw::Key::Num6
                | glfw::Key::Num7
                | glfw::Key::Num8
                | glfw::Key::Num9
        );
        // A hovered empty slot of the window takes no swap.
        let quick_swap = self.hovered.is_none_or(|slot| {
            !matches!(view.target(slot), Target::Window(_)) || view.stack(slot).is_some()
        });
        if numeric && quick_swap {
            let clicks = slot_key_clicks(key, mods.control, self.hovered, &view);
            let actions = self.actions(clicks);
            if !actions.is_empty() {
                self.ignore_text_input = true;
                return KeyResult::Consumed(actions);
            }
        }
        let before = self.search.value();
        if let Some(edit) = edit_key(key)
            && self.search.key_pressed(
                edit,
                mods.control || mods.command,
                mods.shift,
                &self.assets.font,
                clipboard,
            )
        {
            if before != self.search.value() {
                self.refresh_search_results();
            }
            return KeyResult::Consumed(Vec::new());
        }
        if key == glfw::Key::Escape {
            return KeyResult::Close;
        }
        // The focused box swallows every other key.
        KeyResult::Consumed(Vec::new())
    }

    /// `extractTabButton`: the button and the tab's icon.
    pub(super) fn build_tab(&self, draw: &mut ScreenDraw<'_, '_>, tab: usize) {
        let entry = &CREATIVE_TABS[tab];
        let selected = tab == self.menu.tab();
        let x = self.layout.left() + tab_x(tab);
        let y = self.layout.top() - if entry.top { 28 } else { -(IMAGE.1 - 4) };
        let sprite = format!(
            "container/creative_inventory/tab_{}_{}_{}",
            if entry.top { "top" } else { "bottom" },
            if selected { "selected" } else { "unselected" },
            entry.column + 1
        );
        draw.frame.sprite(draw.atlas, &sprite, x, y, 26, 32);
        let icon = table_stack(&entry.icon, self.assets.trees.registries());
        draw.frame.item(
            draw.item,
            &icon,
            x + 13 - 8,
            y + 16 - 8 + if entry.top { 1 } else { -1 },
        );
    }

    /// The image of the tab: its background, and its scroll bar if it has
    /// one.
    pub(super) fn build_image(&self, draw: &mut ScreenDraw<'_, '_>) {
        let (left, top) = (self.layout.left(), self.layout.top());
        let selected = self.menu.tab();
        let background = CREATIVE_TABS[selected]
            .background
            .trim_start_matches("minecraft:textures/gui/")
            .trim_end_matches(".png");
        draw.frame.blit_texture(
            draw.atlas,
            background,
            (256, 256),
            (0, 0),
            left,
            top,
            IMAGE.0,
            IMAGE.1,
        );
        if CREATIVE_TABS[selected].can_scroll {
            let sprite = if self.can_scroll() {
                "container/creative_inventory/scroller"
            } else {
                "container/creative_inventory/scroller_disabled"
            };
            let travel = ((112 - 17) as f32 * self.menu.scroll()) as i32;
            draw.frame
                .sprite(draw.atlas, sprite, left + 175, top + 18 + travel, 12, 15);
        }
    }

    /// The hovered stack's tooltip when nothing is carried, else the name of
    /// the tab under the mouse (`checkTabHovering`).
    pub(super) fn build_tooltip(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        inventory: &ContainerMenu,
        mouse: (f64, f64),
    ) {
        let selected = self.menu.tab();
        let titles = &self.assets.titles;
        let view = self.view(inventory);
        let hovered_stack = self.hovered.and_then(|slot| {
            view.stack(slot)
                .cloned()
                .map(|stack| (stack, matches!(view.target(slot), Target::Window(_))))
        });
        if let Some((stack, creative_slot)) = hovered_stack
            && (inventory.carried.is_none()
                || super::super::bundle_tooltip::shows_with_item_in_hand(&stack))
        {
            let mut lines = (draw.tooltip)(&stack, creative_slot);
            let kind = CREATIVE_TABS[selected].kind;
            // Each tag the search names that the stack is in goes after its
            // name; the reference walks a hash set, so their order is its own.
            if !lines.is_empty() && kind == TabKind::Search && creative_slot {
                for tag in &self.visible_tags {
                    if self.tags.contains(stack.item_registry_id().0, tag) {
                        lines.insert(
                            1,
                            vec![Run {
                                text: format!("#{tag}"),
                                style: Style::coloured("dark_purple"),
                            }],
                        );
                    }
                }
            }
            // `getTooltipFromContainerItem`: the tabs a stack is in follow its
            // name, except for a stack in a single category's own window.
            if !lines.is_empty() && !(kind == TabKind::Category && creative_slot) {
                let mut at = 1;
                for tab in shown_tabs() {
                    if CREATIVE_TABS[tab].kind != TabKind::Search
                        && tab_contains(tab, &stack, self.assets.trees.registries())
                    {
                        lines.insert(
                            at,
                            vec![Run {
                                text: titles[tab].clone(),
                                style: Style::coloured("blue"),
                            }],
                        );
                        at += 1;
                    }
                }
            }
            super::super::bundle_tooltip::stack_tooltip(&self.layout, draw, &stack, &lines, mouse);
            return;
        }
        let (xm, ym) = self.layout.relative(mouse);
        let hovered_tab = shown_tabs().into_iter().find(|&tab| {
            let (x, y) = (tab_x(tab), tab_y(tab));
            xm >= f64::from(x + 3 - 1)
                && xm < f64::from(x + 3 + 21 + 1)
                && ym >= f64::from(y + 3 - 1)
                && ym < f64::from(y + 3 + 27 + 1)
        });
        if let Some(tab) = hovered_tab {
            let line = vec![Run {
                text: titles[tab].clone(),
                style: Style::default(),
            }];
            self.layout.tooltip(draw, &[line], mouse);
        }
    }
}
