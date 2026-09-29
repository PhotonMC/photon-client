//! The creative inventory screen: `CreativeModeInventoryScreen` over a
//! [`CreativeMenu`]. The screen lays the window, the hotbar, the tabs and
//! the scroll bar out, turns mouse and key input into clicks for the menu,
//! and draws them; what a click does is the menu's.
//!
//! Not drawn: the player's model on the inventory tab, item tooltips, the
//! search box's text and the effects panel.

use crate::creative_menu::{Click, CreativeMenu, Target, shown_tabs, table_stack};
use crate::creative_tabs_table::{CREATIVE_TABS, TabKind};
use crate::inventory_menu::{ClickType, InventoryMenu, max_stack_size};
use crate::render::gui::{GuiAtlas, GuiFrame, ItemDrawer};
use crate::render::inventory_screen::{
    ContainerMouse, SlotClick, SlotView, empty_icon, slot_key_clicks,
};
use crate::world::ClientItemStackState as Stack;

const IMAGE_WIDTH: i32 = 195;
const IMAGE_HEIGHT: i32 = 136;
/// The window's slots, then the hotbar's, in the picker's layout.
const PICKER_SLOTS: usize = 54;
/// The player menu's slots, then the bin, in the inventory tab's layout.
const INVENTORY_SLOTS: usize = 47;
const BIN: usize = 46;

/// What a key did.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyResult {
    Clicks(Vec<Click>),
    Close,
}

/// The screen's own state between events.
#[derive(Debug, Clone)]
pub struct CreativeScreen {
    width: i32,
    height: i32,
    menu: CreativeMenu,
    hovered: Option<usize>,
    mouse: ContainerMouse,
    scrolling: bool,
    /// What `hasClickedOutside` last found: `slotClicked` reads it.
    has_clicked_outside: bool,
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
    pub fn new(width: i32, height: i32, tab: usize) -> Self {
        Self {
            width,
            height,
            menu: CreativeMenu::new(tab),
            hovered: None,
            mouse: ContainerMouse::default(),
            scrolling: false,
            has_clicked_outside: false,
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

    /// `resize`: the layout again, keeping the row at the top.
    pub fn resize(&mut self, width: i32, height: i32) {
        let row = self.menu.row_index_for_scroll(self.menu.scroll());
        self.width = width;
        self.height = height;
        let scroll = self.menu.scroll_for_row_index(row);
        self.menu.set_scroll(scroll);
    }

    fn left(&self) -> i32 {
        (self.width - IMAGE_WIDTH) / 2
    }

    fn top(&self) -> i32 {
        (self.height - IMAGE_HEIGHT) / 2
    }

    fn inventory_tab(&self) -> bool {
        self.menu.kind() == TabKind::Inventory
    }

    fn slot_count(&self) -> usize {
        if self.inventory_tab() {
            INVENTORY_SLOTS
        } else {
            PICKER_SLOTS
        }
    }

    /// Where a slot sits in the image: the picker's constructor, or
    /// `selectTab`'s layout of the inventory tab.
    fn slot_position(&self, index: usize) -> (i32, i32) {
        if !self.inventory_tab() {
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

    /// `getHoveredSlot`: the first slot within a unit of the mouse.
    fn hovered_slot(&self, x: f64, y: f64) -> Option<usize> {
        let (xm, ym) = (x - f64::from(self.left()), y - f64::from(self.top()));
        (0..self.slot_count()).find(|&index| {
            let (sx, sy) = self.slot_position(index);
            xm >= f64::from(sx - 1)
                && xm < f64::from(sx + 17)
                && ym >= f64::from(sy - 1)
                && ym < f64::from(sy + 17)
        })
    }

    /// `getTabX`.
    fn tab_x(&self, tab: usize) -> i32 {
        let tab = &CREATIVE_TABS[tab];
        if tab.aligned_right {
            IMAGE_WIDTH - 27 * (7 - tab.column) + 1
        } else {
            27 * tab.column
        }
    }

    /// `getTabY`.
    fn tab_y(&self, tab: usize) -> i32 {
        if CREATIVE_TABS[tab].top {
            -32
        } else {
            IMAGE_HEIGHT
        }
    }

    /// `checkTabClicked`.
    fn tab_clicked(&self, tab: usize, xm: f64, ym: f64) -> bool {
        let (x, y) = (f64::from(self.tab_x(tab)), f64::from(self.tab_y(tab)));
        xm >= x && xm <= x + 26.0 && ym >= y && ym <= y + 32.0
    }

    /// `hasClickedOutside`: outside the image, and not on the tab. The
    /// reference compares the mouse itself with the tab's place in the
    /// image, and so does this.
    fn clicked_outside(&mut self, x: f64, y: f64) -> bool {
        let (left, top) = (f64::from(self.left()), f64::from(self.top()));
        let outside = x < left
            || y < top
            || x >= left + f64::from(IMAGE_WIDTH)
            || y >= top + f64::from(IMAGE_HEIGHT);
        self.has_clicked_outside = outside && !self.tab_clicked(self.menu.tab(), x, y);
        self.has_clicked_outside
    }

    /// `insideScrollbar`.
    fn inside_scrollbar(&self, x: f64, y: f64) -> bool {
        let (x0, y0) = (self.left() + 175, self.top() + 18);
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
        self.mouse.quick_craft_slots.clear();
        self.menu.select_tab(tab);
    }

    /// A tab chosen from a script: a click on its button.
    pub fn select_tab_index(&mut self, tab: usize) {
        if tab < CREATIVE_TABS.len() {
            self.select_tab(tab);
        }
    }

    /// A click a script asks for: on a slot of the screen's layout, or -999
    /// outside it.
    pub fn scripted_click(&mut self, slot: i32, button: i32, input: ClickType) -> Click {
        self.has_clicked_outside = slot == -999;
        Click {
            target: usize::try_from(slot)
                .ok()
                .filter(|&slot| slot < self.slot_count())
                .map(|slot| target_of(self.inventory_tab(), slot)),
            slot_id: slot,
            button,
            input,
            outside: self.has_clicked_outside,
        }
    }

    /// The screen's clicks as `slotClicked` gets them.
    fn clicks(&self, slot_clicks: Vec<SlotClick>) -> Vec<Click> {
        let inventory_tab = self.inventory_tab();
        slot_clicks
            .into_iter()
            .map(|click| Click {
                target: usize::try_from(click.slot)
                    .ok()
                    .map(|slot| target_of(inventory_tab, slot)),
                slot_id: click.slot,
                button: click.button,
                input: click.kind,
                outside: self.has_clicked_outside,
            })
            .collect()
    }

    /// `mouseClicked`, with `MouseHandler`'s double click check first.
    pub fn mouse_clicked(
        &mut self,
        x: f64,
        y: f64,
        button: i32,
        shift: bool,
        now: std::time::Instant,
        inventory: &InventoryMenu,
    ) -> Vec<Click> {
        let double = self.mouse.note_click(now, button);
        if button == 0 {
            let (xm, ym) = (x - f64::from(self.left()), y - f64::from(self.top()));
            if shown_tabs()
                .into_iter()
                .any(|tab| self.tab_clicked(tab, xm, ym))
            {
                return Vec::new();
            }
            if !self.inventory_tab() && self.inside_scrollbar(x, y) {
                self.scrolling = self.can_scroll();
                return Vec::new();
            }
        }
        let slot = self.hovered_slot(x, y);
        let outside = self.clicked_outside(x, y);
        let view = View {
            creative: &self.menu,
            inventory,
        };
        let clicks = self
            .mouse
            .clicked(double, slot, outside, button, shift, button == 2, &view);
        self.clicks(clicks)
    }

    /// `mouseDragged`: the scroll bar follows, or a drag gathers slots.
    pub fn mouse_dragged(&mut self, x: f64, y: f64, inventory: &InventoryMenu) {
        if self.scrolling {
            let y0 = f64::from(self.top() + 18);
            let y1 = y0 + 112.0;
            let scroll = (((y - y0) as f32 - 7.5) / ((y1 - y0) as f32 - 15.0)).clamp(0.0, 1.0);
            self.menu.set_scroll(scroll);
            return;
        }
        let slot = self.hovered_slot(x, y);
        let view = View {
            creative: &self.menu,
            inventory,
        };
        self.mouse.dragged(slot, &view);
    }

    /// `mouseReleased`: a tab is chosen when its button is let go.
    pub fn mouse_released(
        &mut self,
        x: f64,
        y: f64,
        button: i32,
        shift: bool,
        inventory: &InventoryMenu,
    ) -> Vec<Click> {
        if button == 0 {
            let (xm, ym) = (x - f64::from(self.left()), y - f64::from(self.top()));
            self.scrolling = false;
            if let Some(tab) = shown_tabs()
                .into_iter()
                .find(|&tab| self.tab_clicked(tab, xm, ym))
            {
                self.select_tab(tab);
                return Vec::new();
            }
        }
        let slot = self.hovered_slot(x, y);
        let outside = self.clicked_outside(x, y);
        let view = View {
            creative: &self.menu,
            inventory,
        };
        let clicks = self
            .mouse
            .released(slot, outside, button, shift, button == 2, &view);
        self.clicks(clicks)
    }

    /// `mouseScrolled`: the wheel scrolls the rows.
    pub fn mouse_scrolled(&mut self, y: f64) {
        if self.can_scroll() {
            let scroll = self.menu.subtract_input_from_scroll(self.menu.scroll(), y);
            self.menu.set_scroll(scroll);
        }
    }

    /// `keyPressed`. Outside the search tab the chat key opens the search
    /// tab, the inventory key closes the screen and the slot keys act on
    /// the hovered slot; in it only escape closes, and a number key swaps.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        control: bool,
        inventory: &InventoryMenu,
    ) -> KeyResult {
        let search = self.menu.kind() == TabKind::Search;
        if key == glfw::Key::Escape || (!search && key == glfw::Key::E) {
            return KeyResult::Close;
        }
        if !search && key == glfw::Key::T {
            if let Some(tab) = CREATIVE_TABS
                .iter()
                .position(|tab| tab.kind == TabKind::Search)
            {
                self.select_tab(tab);
            }
            return KeyResult::Clicks(Vec::new());
        }
        let view = View {
            creative: &self.menu,
            inventory,
        };
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
        if search {
            // A hovered empty slot of the window takes no swap.
            let quick_swap = self.hovered.is_none_or(|slot| {
                !matches!(view.target(slot), Target::Window(_)) || view.stack(slot).is_some()
            });
            if numeric && quick_swap {
                let clicks = slot_key_clicks(key, control, self.hovered, &view);
                return KeyResult::Clicks(self.clicks(clicks));
            }
            return KeyResult::Clicks(Vec::new());
        }
        let clicks = slot_key_clicks(key, control, self.hovered, &view);
        KeyResult::Clicks(self.clicks(clicks))
    }

    /// `extractRenderState`: the dimmed world, the tabs, the image, the scroll
    /// bar, the slots with the hovered one lit, the title, then the carried
    /// stack at the mouse.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &crate::render::font::Font,
        titles: &[String],
        inventory: &InventoryMenu,
        mouse: (f64, f64),
        draw: &mut ItemDrawer<'_>,
    ) {
        let (left, top) = (self.left(), self.top());
        let selected = self.menu.tab();
        // `extractTransparentBackground`.
        frame.fill_gradient(
            atlas,
            (0, 0, self.width, self.height),
            0xC010_1010,
            0xD010_1010,
        );
        for tab in shown_tabs() {
            if tab != selected {
                self.build_tab(frame, atlas, draw, tab);
            }
        }
        let background = CREATIVE_TABS[selected]
            .background
            .trim_start_matches("minecraft:textures/gui/")
            .trim_end_matches(".png");
        frame.blit_texture(
            atlas,
            background,
            (256, 256),
            (0, 0),
            left,
            top,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
        );
        if CREATIVE_TABS[selected].can_scroll {
            let sprite = if self.can_scroll() {
                "container/creative_inventory/scroller"
            } else {
                "container/creative_inventory/scroller_disabled"
            };
            let travel = ((112 - 17) as f32 * self.menu.scroll()) as i32;
            frame.sprite(atlas, sprite, left + 175, top + 18 + travel, 12, 15);
        }
        self.build_tab(frame, atlas, draw, selected);
        self.hovered = self.hovered_slot(mouse.0, mouse.1);
        if let Some(slot) = self.hovered {
            let (x, y) = self.slot_position(slot);
            frame.sprite(
                atlas,
                "container/slot_highlight_back",
                left + x - 4,
                top + y - 4,
                24,
                24,
            );
        }
        let inventory_tab = self.inventory_tab();
        for index in 0..self.slot_count() {
            let (x, y) = self.slot_position(index);
            let icon = match target_of(inventory_tab, index) {
                Target::Inventory(slot) => empty_icon(slot),
                _ => None,
            };
            let view = View {
                creative: &self.menu,
                inventory,
            };
            self.mouse.draw_slot(
                frame,
                atlas,
                font,
                draw,
                &view,
                index,
                (left + x, top + y),
                icon,
            );
        }
        if CREATIVE_TABS[selected].show_title {
            frame.text(
                atlas,
                font,
                &titles[selected],
                left + 8,
                top + 6,
                0xFF40_4040,
            );
        }
        if let Some(slot) = self.hovered {
            let (x, y) = self.slot_position(slot);
            frame.sprite(
                atlas,
                "container/slot_highlight_front",
                left + x - 4,
                top + y - 4,
                24,
                24,
            );
        }
        self.mouse
            .draw_carried(frame, atlas, font, draw, inventory.carried.as_ref(), mouse);
    }

    /// `extractTabButton`: the button and the tab's icon.
    fn build_tab(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        draw: &mut ItemDrawer<'_>,
        tab: usize,
    ) {
        let entry = &CREATIVE_TABS[tab];
        let selected = tab == self.menu.tab();
        let x = self.left() + self.tab_x(tab);
        let y = self.top() - if entry.top { 28 } else { -(IMAGE_HEIGHT - 4) };
        let sprite = format!(
            "container/creative_inventory/tab_{}_{}_{}",
            if entry.top { "top" } else { "bottom" },
            if selected { "selected" } else { "unselected" },
            entry.column + 1
        );
        frame.sprite(atlas, &sprite, x, y, 26, 32);
        let icon = table_stack(entry.icon);
        frame.item(
            draw,
            icon.item_registry_id(),
            x + 13 - 8,
            y + 16 - 8 + if entry.top { 1 } else { -1 },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creative_menu::default_tab;

    fn centre(screen: &CreativeScreen, slot: usize) -> (f64, f64) {
        let (x, y) = screen.slot_position(slot);
        (
            f64::from(screen.left() + x + 8),
            f64::from(screen.top() + y + 8),
        )
    }

    #[test]
    fn a_window_slot_click_takes_a_stack_on_press() {
        let mut screen = CreativeScreen::new(427, 240, default_tab());
        let inventory = InventoryMenu::default();
        let (x, y) = centre(&screen, 0);
        let clicks = screen.mouse_clicked(x, y, 0, false, std::time::Instant::now(), &inventory);
        assert_eq!(
            clicks,
            vec![Click {
                target: Some(Target::Window(0)),
                slot_id: 0,
                button: 0,
                input: ClickType::Pickup,
                outside: false,
            }]
        );
    }

    #[test]
    fn a_tab_is_chosen_when_its_button_is_released() {
        let mut screen = CreativeScreen::new(427, 240, default_tab());
        let inventory = InventoryMenu::default();
        let other = shown_tabs()
            .into_iter()
            .find(|&tab| tab != default_tab() && CREATIVE_TABS[tab].top)
            .expect("another top tab");
        let x = f64::from(screen.left() + screen.tab_x(other) + 13);
        let y = f64::from(screen.top() - 16);
        let now = std::time::Instant::now();
        assert!(
            screen
                .mouse_clicked(x, y, 0, false, now, &inventory)
                .is_empty()
        );
        assert_eq!(screen.tab(), default_tab());
        assert!(screen.mouse_released(x, y, 0, false, &inventory).is_empty());
        assert_eq!(screen.tab(), other);
    }

    #[test]
    fn the_scroll_bar_follows_a_drag() {
        let mut screen = CreativeScreen::new(427, 240, default_tab());
        let inventory = InventoryMenu::default();
        let x = f64::from(screen.left() + 180);
        let y = f64::from(screen.top() + 25);
        let now = std::time::Instant::now();
        screen.mouse_clicked(x, y, 0, false, now, &inventory);
        screen.mouse_dragged(x, f64::from(screen.top() + 200), &inventory);
        assert_eq!(screen.menu.scroll(), 1.0);
        screen.mouse_released(x, y, 0, false, &inventory);
        screen.mouse_scrolled(1.0);
        assert!(screen.menu.scroll() < 1.0);
    }

    #[test]
    fn the_inventory_tab_puts_armour_and_the_bin_where_the_reference_does() {
        let inventory_tab = CREATIVE_TABS
            .iter()
            .position(|tab| tab.kind == TabKind::Inventory)
            .expect("an inventory tab");
        let screen = CreativeScreen::new(427, 240, inventory_tab);
        assert_eq!(screen.slot_position(5), (54, 6));
        assert_eq!(screen.slot_position(6), (54, 33));
        assert_eq!(screen.slot_position(7), (108, 6));
        assert_eq!(screen.slot_position(45), (35, 20));
        assert_eq!(screen.slot_position(36), (9, 112));
        assert_eq!(screen.slot_position(9), (9, 54));
        assert_eq!(screen.slot_position(BIN), (173, 112));
    }
}
