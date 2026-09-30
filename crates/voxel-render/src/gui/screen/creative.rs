//! The creative inventory screen: `CreativeModeInventoryScreen` over a
//! [`CreativeMenu`]. The screen lays the window, the hotbar, the tabs and
//! the scroll bar out, turns mouse and key input into clicks for the menu,
//! and draws them; what a click does is the menu's.
//!
//! Not drawn: the effects panel.

use std::sync::Arc;

use super::container::{ContainerLayout, ContainerMouse, SlotClick, SlotView, slot_key_clicks};
use super::inventory::empty_icon;
use super::{KeyMods, KeyResult, Screen, ScreenAction, ScreenDraw};
use crate::gui::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::font::Font;
use voxel_data::item::creative_tabs::{CREATIVE_TABS, TabKind};
use voxel_game::creative_menu::{
    Click, CreativeMenu, SearchTrees, Target, shown_tabs, tab_contains, table_stack,
};
use voxel_game::inventory_menu::{ClickType, InventoryMenu, max_stack_size};
use voxel_game::text::{Run, Style};
use voxel_world::ClientItemStackState as Stack;

const IMAGE: (i32, i32) = (195, 136);
/// The window's slots, then the hotbar's, in the picker's layout.
const PICKER_SLOTS: usize = 54;
/// The player menu's slots, then the bin, in the inventory tab's layout.
const INVENTORY_SLOTS: usize = 47;
const BIN: usize = 46;

/// What every creative screen shares with the rest of the client: the
/// search trees, the font the search box measures with, and each tab's
/// title in the client's language.
#[derive(Clone)]
pub struct CreativeAssets {
    pub trees: Arc<SearchTrees>,
    pub font: Arc<Font>,
    pub titles: Arc<[String]>,
}

/// The screen's own state between events.
#[derive(Clone)]
pub struct CreativeScreen {
    layout: ContainerLayout,
    menu: CreativeMenu,
    hovered: Option<usize>,
    mouse: ContainerMouse,
    scrolling: bool,
    /// What `hasClickedOutside` last found: `slotClicked` reads it.
    has_clicked_outside: bool,
    /// The search tab's box.
    search: EditBox,
    /// `ignoreTextInput`: the key that opened the search is not typed.
    ignore_text_input: bool,
    /// When the box took the focus, which its cursor blinks from.
    focused_at: std::time::Instant,
    assets: CreativeAssets,
}

/// What the mouse handling reads from the picker and the player's menu.
struct View<'a> {
    creative: &'a CreativeMenu,
    inventory: &'a InventoryMenu,
}

impl View<'_> {
    fn target(&self, slot: usize) -> Target {
        target_of(self.creative.kind() == TabKind::Inventory, slot)
    }
}

/// The screen's slot index as what it is in a tab's layout.
fn target_of(inventory_tab: bool, slot: usize) -> Target {
    if inventory_tab {
        if slot == BIN {
            Target::Destroy
        } else {
            Target::Inventory(slot)
        }
    } else if slot < 45 {
        Target::Window(slot)
    } else {
        Target::Hotbar(slot - 45)
    }
}

impl SlotView for View<'_> {
    fn stack(&self, slot: usize) -> Option<&Stack> {
        match self.target(slot) {
            Target::Window(slot) => self.creative.window_stack(slot),
            Target::Hotbar(slot) => self.inventory.slots[36 + slot].as_ref(),
            Target::Inventory(slot) => self.inventory.slots[slot].as_ref(),
            Target::Destroy => None,
        }
    }

    fn carried(&self) -> Option<&Stack> {
        self.inventory.carried.as_ref()
    }

    fn can_item_quick_replace(&self, slot: usize, stack: &Stack, ignore_size: bool) -> bool {
        match self.stack(slot) {
            Some(present) if present.is_same_item_same_components(stack) => {
                present.item_count() + if ignore_size { 0 } else { stack.item_count() }
                    <= max_stack_size(stack)
            }
            Some(_) => false,
            None => true,
        }
    }

    fn may_place(&self, slot: usize, stack: &Stack) -> bool {
        match self.target(slot) {
            Target::Inventory(slot) => self.inventory.may_place(slot, stack),
            _ => true,
        }
    }

    fn slot_max_for(&self, slot: usize, stack: &Stack) -> i32 {
        match self.target(slot) {
            Target::Inventory(slot) => self.inventory.slot_max_for(slot, stack),
            Target::Hotbar(slot) => self.inventory.slot_max_for(36 + slot, stack),
            _ => 99.min(max_stack_size(stack)),
        }
    }

    fn can_drag_to(&self, slot: usize) -> bool {
        !matches!(self.target(slot), Target::Window(_) | Target::Destroy)
    }

    fn can_take_for_pick_all(&self, slot: usize) -> bool {
        !matches!(self.target(slot), Target::Window(_) | Target::Destroy)
    }

    fn slot_count(&self) -> usize {
        if self.creative.kind() == TabKind::Inventory {
            INVENTORY_SLOTS
        } else {
            PICKER_SLOTS
        }
    }

    fn same_container(&self, a: usize, b: usize) -> bool {
        let container = |slot: usize| match self.target(slot) {
            Target::Window(_) | Target::Destroy => 0,
            Target::Hotbar(_) => 1,
            Target::Inventory(0) => 2,
            Target::Inventory(1..5) => 3,
            Target::Inventory(_) => 1,
        };
        container(a) == container(b)
    }
}

impl CreativeScreen {
    /// A screen opened on a GUI of this size, on the tab last used.
    pub fn new(width: i32, height: i32, tab: usize, assets: CreativeAssets) -> Self {
        Self {
            layout: ContainerLayout::new(width, height, IMAGE),
            menu: CreativeMenu::new(tab),
            hovered: None,
            mouse: ContainerMouse::default(),
            scrolling: false,
            has_clicked_outside: false,
            // `new EditBox(font, x + 82, y + 6, 80, 9, ...)` with no border,
            // fifty characters long.
            search: EditBox::new(80, 50),
            ignore_text_input: false,
            focused_at: std::time::Instant::now(),
            assets,
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

    fn inventory_tab(&self) -> bool {
        self.menu.kind() == TabKind::Inventory
    }

    fn view<'a>(&'a self, inventory: &'a InventoryMenu) -> View<'a> {
        View {
            creative: &self.menu,
            inventory,
        }
    }

    fn slot_count(&self) -> usize {
        if self.inventory_tab() {
            INVENTORY_SLOTS
        } else {
            PICKER_SLOTS
        }
    }

    /// `getHoveredSlot`.
    fn hovered_slot(&self, mouse: (f64, f64)) -> Option<usize> {
        let inventory_tab = self.inventory_tab();
        self.layout.hovered(mouse, self.slot_count(), |index| {
            slot_position(inventory_tab, index)
        })
    }

    /// `checkTabClicked`, for the mouse relative to the image.
    fn tab_clicked(&self, tab: usize, (xm, ym): (f64, f64)) -> bool {
        let (x, y) = (f64::from(tab_x(tab)), f64::from(tab_y(tab)));
        xm >= x && xm <= x + 26.0 && ym >= y && ym <= y + 32.0
    }

    /// `hasClickedOutside`: outside the image, and not on the tab. The
    /// reference compares the mouse itself with the tab's place in the
    /// image, and so does this.
    fn clicked_outside(&mut self, mouse: (f64, f64)) -> bool {
        self.has_clicked_outside =
            self.layout.outside(mouse) && !self.tab_clicked(self.menu.tab(), mouse);
        self.has_clicked_outside
    }

    /// `insideScrollbar`.
    fn inside_scrollbar(&self, (x, y): (f64, f64)) -> bool {
        let (x0, y0) = (self.layout.left() + 175, self.layout.top() + 18);
        x >= f64::from(x0)
            && y >= f64::from(y0)
            && x < f64::from(x0 + 14)
            && y < f64::from(y0 + 112)
            && CREATIVE_TABS[self.menu.tab()].can_scroll
    }

    /// `canScroll`.
    fn can_scroll(&self) -> bool {
        CREATIVE_TABS[self.menu.tab()].can_scroll && self.menu.can_scroll()
    }

    fn select_tab(&mut self, tab: usize) {
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
    /// the search trees find. A tag search (`#`) finds nothing: the client
    /// does not hold the items' tags.
    fn refresh_search_results(&mut self) {
        let text = self.search.value();
        if text.is_empty() {
            self.menu.select_tab(self.menu.tab());
            return;
        }
        let query = text.to_lowercase();
        let found = if query.starts_with('#') {
            Vec::new()
        } else {
            self.assets.trees.index().search(&query)
        };
        self.menu.show_items(&found);
    }

    /// The screen's clicks as `slotClicked` gets them.
    fn actions(&self, slot_clicks: Vec<SlotClick>) -> Vec<ScreenAction> {
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
    fn search_key_pressed(
        &mut self,
        key: glfw::Key,
        mods: KeyMods,
        inventory: &InventoryMenu,
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
                return KeyResult::Actions(actions);
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
            return KeyResult::Actions(Vec::new());
        }
        if key == glfw::Key::Escape {
            return KeyResult::Close;
        }
        // The focused box swallows every other key.
        KeyResult::Actions(Vec::new())
    }

    /// `extractTabButton`: the button and the tab's icon.
    fn build_tab(&self, draw: &mut ScreenDraw<'_, '_>, tab: usize) {
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
        let icon = table_stack(entry.icon);
        draw.frame.item(
            draw.item,
            &icon,
            x + 13 - 8,
            y + 16 - 8 + if entry.top { 1 } else { -1 },
        );
    }

    /// The image of the tab: its background, and its scroll bar if it has
    /// one.
    fn build_image(&self, draw: &mut ScreenDraw<'_, '_>) {
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
    fn build_tooltip(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        inventory: &InventoryMenu,
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
        if inventory.carried.is_none()
            && let Some((stack, creative_slot)) = hovered_stack
        {
            let mut lines = (draw.tooltip)(&stack, creative_slot);
            let kind = CREATIVE_TABS[selected].kind;
            // `getTooltipFromContainerItem`: the tabs a stack is in follow its
            // name, except for a stack in a single category's own window.
            if !lines.is_empty() && !(kind == TabKind::Category && creative_slot) {
                let mut at = 1;
                for tab in shown_tabs() {
                    if CREATIVE_TABS[tab].kind != TabKind::Search && tab_contains(tab, &stack) {
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
            self.layout.tooltip(draw, &lines, mouse);
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

impl Screen for CreativeScreen {
    /// `resize`: the layout again, keeping the row at the top.
    fn resize(&mut self, width: i32, height: i32) {
        let row = self.menu.row_index_for_scroll(self.menu.scroll());
        self.layout = ContainerLayout::new(width, height, IMAGE);
        let scroll = self.menu.scroll_for_row_index(row);
        self.menu.set_scroll(scroll);
    }

    fn mouse_clicked(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        now: std::time::Instant,
        inventory: &InventoryMenu,
    ) -> Vec<ScreenAction> {
        let double = self.mouse.note_click(now, button);
        if button == 0 {
            let relative = self.layout.relative(mouse);
            if shown_tabs()
                .into_iter()
                .any(|tab| self.tab_clicked(tab, relative))
            {
                return Vec::new();
            }
            if !self.inventory_tab() && self.inside_scrollbar(mouse) {
                self.scrolling = self.can_scroll();
                return Vec::new();
            }
        }
        let slot = self.hovered_slot(mouse);
        let outside = self.clicked_outside(mouse);
        let view = View {
            creative: &self.menu,
            inventory,
        };
        let clicks = self
            .mouse
            .clicked(double, slot, outside, button, shift, button == 2, &view);
        self.actions(clicks)
    }

    /// `mouseReleased`: a tab is chosen when its button is let go.
    fn mouse_released(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        inventory: &InventoryMenu,
    ) -> Vec<ScreenAction> {
        if button == 0 {
            let relative = self.layout.relative(mouse);
            self.scrolling = false;
            if let Some(tab) = shown_tabs()
                .into_iter()
                .find(|&tab| self.tab_clicked(tab, relative))
            {
                self.select_tab(tab);
                return Vec::new();
            }
        }
        let slot = self.hovered_slot(mouse);
        let outside = self.clicked_outside(mouse);
        let view = View {
            creative: &self.menu,
            inventory,
        };
        let clicks = self
            .mouse
            .released(slot, outside, button, shift, button == 2, &view);
        self.actions(clicks)
    }

    /// `mouseDragged`: the scroll bar follows, or a drag gathers slots.
    fn mouse_dragged(&mut self, mouse: (f64, f64), inventory: &InventoryMenu) {
        if self.scrolling {
            let y0 = f64::from(self.layout.top() + 18);
            let y1 = y0 + 112.0;
            let scroll =
                (((mouse.1 - y0) as f32 - 7.5) / ((y1 - y0) as f32 - 15.0)).clamp(0.0, 1.0);
            self.menu.set_scroll(scroll);
            return;
        }
        let slot = self.hovered_slot(mouse);
        let view = View {
            creative: &self.menu,
            inventory,
        };
        self.mouse.dragged(slot, &view);
    }

    /// `mouseScrolled`: the wheel scrolls the rows.
    fn mouse_scrolled(&mut self, y: f64) {
        if self.can_scroll() {
            let scroll = self.menu.subtract_input_from_scroll(self.menu.scroll(), y);
            self.menu.set_scroll(scroll);
        }
    }

    /// `keyPressed`. Outside the search tab the chat key opens the search
    /// tab, the inventory key closes the screen and the slot keys act on
    /// the hovered slot; in it [`Self::search_key_pressed`] reads the key.
    fn key_pressed(
        &mut self,
        key: glfw::Key,
        mods: KeyMods,
        inventory: &InventoryMenu,
        clipboard: &mut dyn Clipboard,
    ) -> KeyResult {
        self.ignore_text_input = false;
        if self.menu.kind() == TabKind::Search {
            return self.search_key_pressed(key, mods, inventory, clipboard);
        }
        if key == glfw::Key::T {
            self.ignore_text_input = true;
            if let Some(tab) = CREATIVE_TABS
                .iter()
                .position(|tab| tab.kind == TabKind::Search)
            {
                self.select_tab(tab);
            }
            return KeyResult::Actions(Vec::new());
        }
        if matches!(key, glfw::Key::Escape | glfw::Key::E) {
            return KeyResult::Close;
        }
        let clicks = slot_key_clicks(key, mods.control, self.hovered, &self.view(inventory));
        KeyResult::Actions(self.actions(clicks))
    }

    /// `keyReleased`.
    fn key_released(&mut self) {
        self.ignore_text_input = false;
    }

    /// `charTyped`: a character into the search box.
    fn char_typed(&mut self, character: char) {
        if self.ignore_text_input || self.menu.kind() != TabKind::Search {
            return;
        }
        let before = self.search.value();
        if self.search.char_typed(character, &self.assets.font) && before != self.search.value() {
            self.refresh_search_results();
        }
    }

    fn scripted_click(&mut self, slot: i32, button: i32, input: ClickType) -> ScreenAction {
        self.has_clicked_outside = slot == -999;
        ScreenAction::Creative(Click {
            target: usize::try_from(slot)
                .ok()
                .filter(|&slot| slot < self.slot_count())
                .map(|slot| target_of(self.inventory_tab(), slot)),
            slot_id: slot,
            button,
            input,
            outside: self.has_clicked_outside,
        })
    }

    /// `extractRenderState`: the dimmed world, the tabs, the image, the scroll
    /// bar, the slots with the hovered one lit, the title, the carried stack
    /// at the mouse, then a tooltip.
    fn build(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        inventory: &InventoryMenu,
        mouse: (f64, f64),
    ) {
        let (left, top) = (self.layout.left(), self.layout.top());
        let selected = self.menu.tab();
        self.layout.background(draw);
        for tab in shown_tabs() {
            if tab != selected {
                self.build_tab(draw, tab);
            }
        }
        self.build_image(draw);
        self.build_tab(draw, selected);
        if self.menu.kind() == TabKind::Search {
            self.search.draw(
                draw.frame,
                draw.atlas,
                draw.font,
                (left + 82, top + 6),
                self.focused_at,
            );
        }
        let inventory_tab = self.inventory_tab();
        if inventory_tab {
            let model = (draw.player)(
                (left + 73, top + 6, left + 105, top + 49),
                20.0,
                0.0625,
                (mouse.0 as f32, mouse.1 as f32),
            );
            draw.frame.entity(&model);
        }
        self.hovered = self.hovered_slot(mouse);
        if let Some(slot) = self.hovered {
            self.layout
                .highlight(draw, slot_position(inventory_tab, slot), false);
        }
        for index in 0..self.slot_count() {
            let (x, y) = slot_position(inventory_tab, index);
            let icon = match target_of(inventory_tab, index) {
                Target::Inventory(slot) => empty_icon(slot),
                _ => None,
            };
            let view = View {
                creative: &self.menu,
                inventory,
            };
            self.mouse
                .draw_slot(draw, &view, index, (left + x, top + y), icon);
        }
        if CREATIVE_TABS[selected].show_title {
            draw.frame.text(
                draw.atlas,
                draw.font,
                &self.assets.titles[selected],
                left + 8,
                top + 6,
                0xFF40_4040,
            );
        }
        if let Some(slot) = self.hovered {
            self.layout
                .highlight(draw, slot_position(inventory_tab, slot), true);
        }
        self.mouse
            .draw_carried(draw, inventory.carried.as_ref(), mouse);
        self.build_tooltip(draw, inventory, mouse);
    }

    fn as_creative_mut(&mut self) -> Option<&mut CreativeScreen> {
        Some(self)
    }
}

/// Where a slot sits in the image: the picker's constructor, or
/// `selectTab`'s layout of the inventory tab.
fn slot_position(inventory_tab: bool, index: usize) -> (i32, i32) {
    if !inventory_tab {
        return if index < 45 {
            (9 + (index % 9) as i32 * 18, 18 + (index / 9) as i32 * 18)
        } else {
            (9 + (index - 45) as i32 * 18, 112)
        };
    }
    match index {
        0..5 => (-2000, -2000),
        5..9 => {
            let position = index - 5;
            (
                54 + (position / 2) as i32 * 54,
                6 + (position % 2) as i32 * 27,
            )
        }
        45 => (35, 20),
        BIN => (173, 112),
        _ => {
            let position = index - 9;
            let x = 9 + (position % 9) as i32 * 18;
            let y = if index >= 36 {
                112
            } else {
                54 + (position / 9) as i32 * 18
            };
            (x, y)
        }
    }
}

/// `getTabX`.
fn tab_x(tab: usize) -> i32 {
    let tab = &CREATIVE_TABS[tab];
    if tab.aligned_right {
        IMAGE.0 - 27 * (7 - tab.column) + 1
    } else {
        27 * tab.column
    }
}

/// `getTabY`.
fn tab_y(tab: usize) -> i32 {
    if CREATIVE_TABS[tab].top { -32 } else { IMAGE.1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_game::creative_menu::default_tab;

    /// A screen with no language, no font and no titles.
    fn test_screen(tab: usize) -> CreativeScreen {
        let assets = CreativeAssets {
            trees: Arc::new(SearchTrees::new(
                Arc::new(voxel_game::text::Lang::default()),
                Arc::new(voxel_game::registries::Registries::default()),
            )),
            font: Arc::new(Font::default()),
            titles: Arc::from(Vec::new()),
        };
        CreativeScreen::new(427, 240, tab, assets)
    }

    fn centre(screen: &CreativeScreen, slot: usize) -> (f64, f64) {
        let (x, y) = slot_position(screen.inventory_tab(), slot);
        (
            f64::from(screen.layout.left() + x + 8),
            f64::from(screen.layout.top() + y + 8),
        )
    }

    #[test]
    fn a_window_slot_click_takes_a_stack_on_press() {
        let mut screen = test_screen(default_tab());
        let inventory = InventoryMenu::default();
        let mouse = centre(&screen, 0);
        let clicks = screen.mouse_clicked(mouse, 0, false, std::time::Instant::now(), &inventory);
        assert_eq!(
            clicks,
            vec![ScreenAction::Creative(Click {
                target: Some(Target::Window(0)),
                slot_id: 0,
                button: 0,
                input: ClickType::Pickup,
                outside: false,
            })]
        );
    }

    #[test]
    fn a_tab_is_chosen_when_its_button_is_released() {
        let mut screen = test_screen(default_tab());
        let inventory = InventoryMenu::default();
        let other = shown_tabs()
            .into_iter()
            .find(|&tab| tab != default_tab() && CREATIVE_TABS[tab].top)
            .expect("another top tab");
        let x = f64::from(screen.layout.left() + tab_x(other) + 13);
        let y = f64::from(screen.layout.top() - 16);
        let now = std::time::Instant::now();
        assert!(
            screen
                .mouse_clicked((x, y), 0, false, now, &inventory)
                .is_empty()
        );
        assert_eq!(screen.tab(), default_tab());
        assert!(
            screen
                .mouse_released((x, y), 0, false, &inventory)
                .is_empty()
        );
        assert_eq!(screen.tab(), other);
    }

    #[test]
    fn the_scroll_bar_follows_a_drag() {
        let mut screen = test_screen(default_tab());
        let inventory = InventoryMenu::default();
        let x = f64::from(screen.layout.left() + 180);
        let y = f64::from(screen.layout.top() + 25);
        let now = std::time::Instant::now();
        screen.mouse_clicked((x, y), 0, false, now, &inventory);
        screen.mouse_dragged((x, f64::from(screen.layout.top() + 200)), &inventory);
        assert_eq!(screen.menu.scroll(), 1.0);
        screen.mouse_released((x, y), 0, false, &inventory);
        screen.mouse_scrolled(1.0);
        assert!(screen.menu.scroll() < 1.0);
    }

    #[test]
    fn the_inventory_tab_puts_armour_and_the_bin_where_the_reference_does() {
        let inventory_tab = CREATIVE_TABS
            .iter()
            .position(|tab| tab.kind == TabKind::Inventory)
            .expect("an inventory tab");
        assert!(test_screen(inventory_tab).inventory_tab());
        assert_eq!(slot_position(true, 5), (54, 6));
        assert_eq!(slot_position(true, 6), (54, 33));
        assert_eq!(slot_position(true, 7), (108, 6));
        assert_eq!(slot_position(true, 45), (35, 20));
        assert_eq!(slot_position(true, 36), (9, 112));
        assert_eq!(slot_position(true, 9), (9, 54));
        assert_eq!(slot_position(true, BIN), (173, 112));
    }
}
