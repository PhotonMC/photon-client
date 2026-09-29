//! The survival inventory screen: `AbstractContainerScreen` over the
//! player's own menu, as `InventoryScreen` lays it out with its recipe book
//! closed. The screen turns mouse and key input into menu clicks and draws
//! the menu; the clicks themselves run in [`crate::inventory_menu`].
//!
//! Not drawn: the player's model in its box, item tooltips and the recipe
//! book; its button is drawn and takes the click, but does not open it.

use crate::inventory_menu::{ClickType, InventoryMenu, SLOT_COUNT, max_stack_size};
use crate::render::gui::{GuiAtlas, GuiFrame, ItemDrawer, PlayerDrawer, TooltipLines};
use crate::world::ClientItemStackState as Stack;

const IMAGE_WIDTH: i32 = 176;
const IMAGE_HEIGHT: i32 = 166;
/// `MouseHandler`'s double click window.
const DOUBLE_CLICK: std::time::Duration = std::time::Duration::from_millis(250);

/// Where each menu slot sits in the screen's image: `InventoryMenu`'s
/// constructor.
pub fn slot_position(index: usize) -> (i32, i32) {
    match index {
        0 => (154, 28),
        1..5 => (
            98 + ((index - 1) % 2) as i32 * 18,
            18 + ((index - 1) / 2) as i32 * 18,
        ),
        5..9 => (8, 8 + (index - 5) as i32 * 18),
        9..36 => (
            8 + ((index - 9) % 9) as i32 * 18,
            84 + ((index - 9) / 9) as i32 * 18,
        ),
        36..45 => (8 + (index - 36) as i32 * 18, 142),
        _ => (77, 62),
    }
}

/// The icon an empty slot shows: the armour slots' and the off hand's.
pub(crate) fn empty_icon(index: usize) -> Option<&'static str> {
    match index {
        5 => Some("container/slot/helmet"),
        6 => Some("container/slot/chestplate"),
        7 => Some("container/slot/leggings"),
        8 => Some("container/slot/boots"),
        45 => Some("container/slot/shield"),
        _ => None,
    }
}

/// A click the screen asks the menu for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotClick {
    pub slot: i32,
    pub button: i32,
    pub kind: ClickType,
}

/// What a key did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyResult {
    Clicks(Vec<SlotClick>),
    Close,
}

/// What `AbstractContainerScreen`'s mouse handling reads from a menu.
pub trait SlotView {
    fn stack(&self, slot: usize) -> Option<&Stack>;
    fn carried(&self) -> Option<&Stack>;
    fn can_item_quick_replace(&self, slot: usize, stack: &Stack, ignore_size: bool) -> bool;
    fn may_place(&self, slot: usize, stack: &Stack) -> bool;
    fn slot_max_for(&self, slot: usize, stack: &Stack) -> i32;
    /// `AbstractContainerMenu.canDragTo`.
    fn can_drag_to(&self, slot: usize) -> bool;
    /// `AbstractContainerMenu.canTakeItemForPickAll`.
    fn can_take_for_pick_all(&self, slot: usize) -> bool;
    fn slot_count(&self) -> usize;
    /// `Slot.isSameInventory`.
    fn same_container(&self, a: usize, b: usize) -> bool;
}

impl SlotView for InventoryMenu {
    fn stack(&self, slot: usize) -> Option<&Stack> {
        self.slots[slot].as_ref()
    }

    fn carried(&self) -> Option<&Stack> {
        self.carried.as_ref()
    }

    fn can_item_quick_replace(&self, slot: usize, stack: &Stack, ignore_size: bool) -> bool {
        InventoryMenu::can_item_quick_replace(self, slot, stack, ignore_size)
    }

    fn may_place(&self, slot: usize, stack: &Stack) -> bool {
        InventoryMenu::may_place(self, slot, stack)
    }

    fn slot_max_for(&self, slot: usize, stack: &Stack) -> i32 {
        InventoryMenu::slot_max_for(self, slot, stack)
    }

    fn can_drag_to(&self, _slot: usize) -> bool {
        true
    }

    fn can_take_for_pick_all(&self, slot: usize) -> bool {
        slot != 0
    }

    fn slot_count(&self) -> usize {
        SLOT_COUNT
    }

    fn same_container(&self, a: usize, b: usize) -> bool {
        container_of(a) == container_of(b)
    }
}

/// `AbstractContainerScreen`'s mouse state: the drag, the double click and
/// what a press leaves for its release. Where the slots are, and which one
/// the mouse is on, is the screen's.
#[derive(Debug, Clone)]
pub struct ContainerMouse {
    last_click_slot: Option<usize>,
    pub(crate) is_quick_crafting: bool,
    quick_crafting_type: i32,
    quick_crafting_button: i32,
    pub(crate) quick_craft_slots: Vec<usize>,
    pub(crate) quick_crafting_remainder: i32,
    skip_next_release: bool,
    double_click: bool,
    last_quick_moved: Option<Stack>,
    /// `MouseHandler`'s last click: when, and which button.
    last_click: Option<(std::time::Instant, i32)>,
}

impl Default for ContainerMouse {
    fn default() -> Self {
        Self {
            last_click_slot: None,
            is_quick_crafting: false,
            quick_crafting_type: 0,
            quick_crafting_button: 0,
            quick_craft_slots: Vec::new(),
            quick_crafting_remainder: 0,
            skip_next_release: true,
            double_click: false,
            last_quick_moved: None,
            last_click: None,
        }
    }
}

impl ContainerMouse {
    /// The double click check `MouseHandler` makes before a press.
    pub fn note_click(&mut self, now: std::time::Instant, button: i32) -> bool {
        let double = self
            .last_click
            .is_some_and(|(time, last)| now.duration_since(time) < DOUBLE_CLICK && last == button);
        self.last_click = Some((now, button));
        double
    }

    /// `mouseClicked`, after the widgets did not take the press.
    #[allow(clippy::too_many_arguments)]
    pub fn clicked(
        &mut self,
        double: bool,
        slot: Option<usize>,
        outside: bool,
        button: i32,
        shift: bool,
        cloning: bool,
        menu: &impl SlotView,
    ) -> Vec<SlotClick> {
        let mut clicks = Vec::new();
        self.double_click = self.last_click_slot == slot && double;
        self.skip_next_release = false;
        if button == 0 || button == 1 || cloning {
            let slot_id = if outside {
                -999
            } else {
                slot.map_or(-1, |slot| slot as i32)
            };
            if slot_id != -1 && !self.is_quick_crafting {
                if menu.carried().is_none() {
                    let quick = slot_id != -999 && shift;
                    let kind = if cloning {
                        ClickType::Clone
                    } else if quick {
                        self.last_quick_moved = slot.and_then(|slot| menu.stack(slot).cloned());
                        ClickType::QuickMove
                    } else if slot_id == -999 {
                        ClickType::Throw
                    } else {
                        ClickType::Pickup
                    };
                    clicks.push(SlotClick {
                        slot: slot_id,
                        button,
                        kind,
                    });
                    self.skip_next_release = true;
                } else {
                    self.is_quick_crafting = true;
                    self.quick_crafting_button = button;
                    self.quick_craft_slots.clear();
                    self.quick_crafting_type = button;
                }
            }
        }
        self.last_click_slot = slot;
        clicks
    }

    /// `mouseDragged`: a drag gathers the slots it passes.
    pub fn dragged(&mut self, slot: Option<usize>, menu: &impl SlotView) {
        let Some(slot) = slot else {
            return;
        };
        if let Some(carried) = menu.carried()
            && self.should_add_slot_to_quick_craft(slot, carried, menu)
            && !self.quick_craft_slots.contains(&slot)
        {
            self.quick_craft_slots.push(slot);
            self.recalculate_quick_craft_remaining(menu);
        }
    }

    /// `mouseReleased`.
    pub fn released(
        &mut self,
        slot: Option<usize>,
        outside: bool,
        button: i32,
        shift: bool,
        cloning: bool,
        menu: &impl SlotView,
    ) -> Vec<SlotClick> {
        let mut clicks = Vec::new();
        let slot_id = if outside {
            -999
        } else {
            slot.map_or(-1, |slot| slot as i32)
        };
        if self.double_click
            && button == 0
            && slot.is_some_and(|slot| menu.can_take_for_pick_all(slot))
        {
            let slot = slot.expect("checked above");
            if shift {
                if let Some(moved) = &self.last_quick_moved {
                    for target in 0..menu.slot_count() {
                        if menu.stack(target).is_some()
                            && menu.same_container(target, slot)
                            && menu.can_item_quick_replace(target, moved, true)
                        {
                            clicks.push(SlotClick {
                                slot: target as i32,
                                button,
                                kind: ClickType::QuickMove,
                            });
                        }
                    }
                }
            } else {
                clicks.push(SlotClick {
                    slot: slot_id,
                    button,
                    kind: ClickType::PickupAll,
                });
            }
            self.double_click = false;
        } else {
            if self.is_quick_crafting && self.quick_crafting_button != button {
                self.is_quick_crafting = false;
                self.quick_craft_slots.clear();
                self.skip_next_release = true;
                return clicks;
            }
            if self.skip_next_release {
                self.skip_next_release = false;
                return clicks;
            }
            if self.is_quick_crafting && !self.quick_craft_slots.is_empty() {
                // `quickCraftToSlots`: the start, each slot, the end.
                let kind = self.quick_crafting_type;
                clicks.push(SlotClick {
                    slot: -999,
                    button: kind << 2,
                    kind: ClickType::QuickCraft,
                });
                for &slot in &self.quick_craft_slots {
                    clicks.push(SlotClick {
                        slot: slot as i32,
                        button: 1 | kind << 2,
                        kind: ClickType::QuickCraft,
                    });
                }
                clicks.push(SlotClick {
                    slot: -999,
                    button: 2 | kind << 2,
                    kind: ClickType::QuickCraft,
                });
            } else if menu.carried().is_some() && cloning {
                clicks.push(SlotClick {
                    slot: slot_id,
                    button,
                    kind: ClickType::Clone,
                });
            } else if menu.carried().is_some() {
                let quick = slot_id != -999 && shift;
                if quick {
                    self.last_quick_moved = slot.and_then(|slot| menu.stack(slot).cloned());
                }
                clicks.push(SlotClick {
                    slot: slot_id,
                    button,
                    kind: if quick {
                        ClickType::QuickMove
                    } else {
                        ClickType::Pickup
                    },
                });
            }
        }
        self.is_quick_crafting = false;
        clicks
    }

    fn should_add_slot_to_quick_craft(
        &self,
        slot: usize,
        carried: &Stack,
        menu: &impl SlotView,
    ) -> bool {
        self.is_quick_crafting
            && (carried.item_count() > self.quick_craft_slots.len() as i32
                || self.quick_crafting_type == 2)
            && menu.can_item_quick_replace(slot, carried, true)
            && menu.may_place(slot, carried)
            && menu.can_drag_to(slot)
    }

    pub(crate) fn place_count(&self, carried: &Stack) -> i32 {
        match self.quick_crafting_type {
            0 => (carried.item_count() as f32 / self.quick_craft_slots.len() as f32).floor() as i32,
            1 => 1,
            2 => max_stack_size(carried),
            _ => carried.item_count(),
        }
    }

    pub(crate) fn recalculate_quick_craft_remaining(&mut self, menu: &impl SlotView) {
        let Some(carried) = menu.carried() else {
            return;
        };
        if !self.is_quick_crafting {
            return;
        }
        if self.quick_crafting_type == 2 {
            self.quick_crafting_remainder = max_stack_size(carried);
            return;
        }
        self.quick_crafting_remainder = carried.item_count();
        for &slot in &self.quick_craft_slots {
            let carry = menu.stack(slot).map_or(0, Stack::item_count);
            let max = max_stack_size(carried).min(menu.slot_max_for(slot, carried));
            let count = (self.place_count(carried) + carry).min(max);
            self.quick_crafting_remainder -= count - carry;
        }
    }

    /// The count, and its text if it is not the number, of the stack the
    /// mouse carries: less what a drag over more than one slot would leave.
    pub(crate) fn carried_count(&self, carried: &Stack) -> (i32, Option<&'static str>) {
        let (mut count, mut text) = (carried.item_count(), None);
        if self.is_quick_crafting && self.quick_craft_slots.len() > 1 {
            count = self.quick_crafting_remainder;
            if count <= 0 {
                text = Some("0");
            }
        }
        (count, text)
    }

    /// `extractSlot`'s drag preview for a slot: what a drag would put in it,
    /// and the yellow count when that is the limit. `None` when the drag is
    /// not over it; a slot the drag cannot fill leaves the drag.
    pub(crate) fn slot_preview(
        &mut self,
        index: usize,
        menu: &impl SlotView,
    ) -> Option<SlotPreview> {
        if !(self.is_quick_crafting && self.quick_craft_slots.contains(&index)) {
            return None;
        }
        let carried = menu.carried()?;
        if self.quick_craft_slots.len() == 1 {
            return Some(SlotPreview::Hidden);
        }
        if menu.can_item_quick_replace(index, carried, true) {
            let max = max_stack_size(carried).min(menu.slot_max_for(index, carried));
            let carry = menu.stack(index).map_or(0, Stack::item_count);
            let mut count = self.place_count(carried) + carry;
            let mut limit = None;
            if count > max {
                count = max;
                limit = Some(max.to_string());
            }
            Some(SlotPreview::Fill {
                stack: carried.clone(),
                count,
                limit,
            })
        } else {
            self.quick_craft_slots.retain(|&slot| slot != index);
            self.recalculate_quick_craft_remaining(menu);
            None
        }
    }
}

/// A slot under a drag.
pub(crate) enum SlotPreview {
    /// The only slot the drag is over: nothing is drawn in it.
    Hidden,
    Fill {
        stack: Stack,
        count: i32,
        limit: Option<String>,
    },
}

/// The survival inventory screen's own state between events.
#[derive(Debug, Clone)]
pub struct InventoryScreen {
    width: i32,
    height: i32,
    hovered: Option<usize>,
    mouse: ContainerMouse,
    /// `InventoryScreen.buttonClicked`: the recipe book button took the
    /// press, so the release is swallowed.
    button_clicked: bool,
}

impl InventoryScreen {
    /// A screen opened on a GUI of this size.
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            hovered: None,
            mouse: ContainerMouse::default(),
            button_clicked: false,
        }
    }

    /// The GUI was resized: `Screen.resize` lays the screen out again.
    pub fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    fn left(&self) -> i32 {
        (self.width - IMAGE_WIDTH) / 2
    }

    fn top(&self) -> i32 {
        (self.height - IMAGE_HEIGHT) / 2
    }

    fn recipe_button(&self) -> (i32, i32) {
        (self.left() + 104, self.height / 2 - 22)
    }

    fn on_recipe_button(&self, x: f64, y: f64) -> bool {
        let (bx, by) = self.recipe_button();
        x >= f64::from(bx) && y >= f64::from(by) && x < f64::from(bx + 20) && y < f64::from(by + 18)
    }

    /// `getHoveredSlot`: the first slot within a unit of the mouse.
    fn hovered_slot(&self, x: f64, y: f64) -> Option<usize> {
        let (xm, ym) = (x - f64::from(self.left()), y - f64::from(self.top()));
        (0..SLOT_COUNT).find(|&index| {
            let (sx, sy) = slot_position(index);
            xm >= f64::from(sx - 1)
                && xm < f64::from(sx + 17)
                && ym >= f64::from(sy - 1)
                && ym < f64::from(sy + 17)
        })
    }

    fn clicked_outside(&self, x: f64, y: f64) -> bool {
        let (left, top) = (f64::from(self.left()), f64::from(self.top()));
        x < left
            || y < top
            || x >= left + f64::from(IMAGE_WIDTH)
            || y >= top + f64::from(IMAGE_HEIGHT)
    }

    /// `mouseClicked`, with `MouseHandler`'s double click check first.
    pub fn mouse_clicked(
        &mut self,
        x: f64,
        y: f64,
        button: i32,
        shift: bool,
        now: std::time::Instant,
        menu: &InventoryMenu,
    ) -> Vec<SlotClick> {
        let double = self.mouse.note_click(now, button);
        // The recipe book button is a widget, which takes the press first.
        if self.on_recipe_button(x, y) {
            self.button_clicked = true;
            return Vec::new();
        }
        let slot = self.hovered_slot(x, y);
        let outside = self.clicked_outside(x, y);
        self.mouse
            .clicked(double, slot, outside, button, shift, false, menu)
    }

    /// `mouseDragged`: a drag gathers the slots it passes.
    pub fn mouse_dragged(&mut self, x: f64, y: f64, menu: &InventoryMenu) {
        let slot = self.hovered_slot(x, y);
        self.mouse.dragged(slot, menu);
    }

    /// `mouseReleased`.
    pub fn mouse_released(
        &mut self,
        x: f64,
        y: f64,
        button: i32,
        shift: bool,
        menu: &InventoryMenu,
    ) -> Vec<SlotClick> {
        if self.button_clicked {
            self.button_clicked = false;
            return Vec::new();
        }
        let slot = self.hovered_slot(x, y);
        let outside = self.clicked_outside(x, y);
        self.mouse
            .released(slot, outside, button, shift, false, menu)
    }

    /// `keyPressed`: the inventory key and escape close the screen; the
    /// hotbar keys and the off hand key swap with the hovered slot; the drop
    /// key throws from it, all of it with control.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        control: bool,
        menu: &InventoryMenu,
    ) -> KeyResult {
        if matches!(key, glfw::Key::Escape | glfw::Key::E) {
            return KeyResult::Close;
        }
        KeyResult::Clicks(slot_key_clicks(key, control, self.hovered, menu))
    }

    /// `extractRenderState`: the dimmed world, the image, the recipe book
    /// button, the title, the slots with the hovered one lit, then the
    /// carried stack at the mouse.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &crate::render::font::Font,
        title: &str,
        menu: &InventoryMenu,
        mouse: (f64, f64),
        draw: &mut ItemDrawer<'_>,
        player: &mut PlayerDrawer<'_>,
        tooltip: &mut TooltipLines<'_>,
    ) {
        let (left, top) = (self.left(), self.top());
        // `extractTransparentBackground`.
        frame.fill_gradient(
            atlas,
            (0, 0, self.width, self.height),
            0xC010_1010,
            0xD010_1010,
        );
        frame.blit_texture(
            atlas,
            "container/inventory",
            (256, 256),
            (0, 0),
            left,
            top,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
        );
        let (bx, by) = self.recipe_button();
        let button = if self.on_recipe_button(mouse.0, mouse.1) {
            "recipe_book/button_highlighted"
        } else {
            "recipe_book/button"
        };
        frame.sprite(atlas, button, bx, by, 20, 18);
        frame.entity(&player(
            (left + 26, top + 8, left + 75, top + 78),
            30.0,
            0.0625,
            (mouse.0 as f32, mouse.1 as f32),
        ));
        frame.text(atlas, font, title, left + 97, top + 6, 0xFF40_4040);
        self.hovered = self.hovered_slot(mouse.0, mouse.1);
        if let Some(slot) = self.hovered {
            let (x, y) = slot_position(slot);
            frame.sprite(
                atlas,
                "container/slot_highlight_back",
                left + x - 4,
                top + y - 4,
                24,
                24,
            );
        }
        for index in 0..SLOT_COUNT {
            self.build_slot(frame, atlas, font, menu, index, draw);
        }
        if let Some(slot) = self.hovered {
            let (x, y) = slot_position(slot);
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
            .draw_carried(frame, atlas, font, draw, menu.carried.as_ref(), mouse);
        // The hovered stack's tooltip, when nothing is carried.
        if menu.carried.is_none()
            && let Some(stack) = self.hovered.and_then(|slot| menu.slots[slot].as_ref())
        {
            let lines = tooltip(stack, false);
            frame.tooltip(
                atlas,
                font,
                &lines,
                (mouse.0 as i32, mouse.1 as i32),
                (self.width, self.height),
            );
        }
    }

    fn build_slot(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &crate::render::font::Font,
        menu: &InventoryMenu,
        index: usize,
        draw: &mut ItemDrawer<'_>,
    ) {
        let (x, y) = slot_position(index);
        let (x, y) = (self.left() + x, self.top() + y);
        self.mouse.draw_slot(
            frame,
            atlas,
            font,
            draw,
            menu,
            index,
            (x, y),
            empty_icon(index),
        );
    }
}

impl ContainerMouse {
    /// `extractCarriedItem`: the stack at the mouse, less what a drag over
    /// more than one slot would leave.
    pub(crate) fn draw_carried(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &crate::render::font::Font,
        draw: &mut ItemDrawer<'_>,
        carried: Option<&Stack>,
        mouse: (f64, f64),
    ) {
        let Some(carried) = carried else {
            return;
        };
        let (count, text) = self.carried_count(carried);
        let (x, y) = (mouse.0 as i32 - 8, mouse.1 as i32 - 8);
        if count > 0 || text.is_some() {
            let shown = text.map(|text| (text, 0xFFFF_FF55));
            frame.item_with_count(atlas, font, draw, carried, count, shown, x, y);
        }
    }

    /// `extractSlot`: a drag's preview count, an empty slot's icon, or the
    /// stack and its count.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_slot(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &crate::render::font::Font,
        draw: &mut ItemDrawer<'_>,
        menu: &impl SlotView,
        index: usize,
        (x, y): (i32, i32),
        empty_icon: Option<&str>,
    ) {
        let mut shown = menu
            .stack(index)
            .map(|stack| (stack.clone(), stack.item_count()));
        let mut text = None;
        let mut preview = false;
        let yellow;
        match self.slot_preview(index, menu) {
            Some(SlotPreview::Hidden) => return,
            Some(SlotPreview::Fill {
                stack,
                count,
                limit,
            }) => {
                preview = true;
                if let Some(limit) = limit {
                    yellow = limit;
                    text = Some((yellow.as_str(), 0xFFFF_FF55));
                }
                shown = Some((stack, count)).filter(|&(_, count)| count > 0);
            }
            None => {}
        }
        if shown.is_none()
            && let Some(icon) = empty_icon
        {
            frame.sprite(atlas, icon, x, y, 16, 16);
            return;
        }
        if preview {
            frame.fill(atlas, (x, y, x + 16, y + 16), 0x80FF_FFFF);
        }
        if let Some((stack, count)) = shown {
            frame.item_with_count(atlas, font, draw, &stack, count, text, x, y);
        }
    }
}

/// `AbstractContainerScreen.keyPressed`'s slot keys: a hotbar key or the
/// off hand key swaps with the hovered slot when nothing is carried, the
/// drop key throws from it, all of it with control.
pub(crate) fn slot_key_clicks(
    key: glfw::Key,
    control: bool,
    hovered: Option<usize>,
    menu: &impl SlotView,
) -> Vec<SlotClick> {
    let mut clicks = Vec::new();
    if menu.carried().is_none()
        && let Some(slot) = hovered
    {
        let button = match key {
            glfw::Key::F => Some(40),
            glfw::Key::Num1 => Some(0),
            glfw::Key::Num2 => Some(1),
            glfw::Key::Num3 => Some(2),
            glfw::Key::Num4 => Some(3),
            glfw::Key::Num5 => Some(4),
            glfw::Key::Num6 => Some(5),
            glfw::Key::Num7 => Some(6),
            glfw::Key::Num8 => Some(7),
            glfw::Key::Num9 => Some(8),
            _ => None,
        };
        if let Some(button) = button {
            clicks.push(SlotClick {
                slot: slot as i32,
                button,
                kind: ClickType::Swap,
            });
            return clicks;
        }
    }
    if let Some(slot) = hovered
        && menu.stack(slot).is_some()
        && key == glfw::Key::Q
    {
        clicks.push(SlotClick {
            slot: slot as i32,
            button: i32::from(control),
            kind: ClickType::Throw,
        });
    }
    clicks
}

/// Which container a menu slot belongs to: the result, the crafting grid,
/// or the player's inventory.
fn container_of(index: usize) -> u8 {
    match index {
        0 => 0,
        1..5 => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn centre(screen: &InventoryScreen, slot: usize) -> (f64, f64) {
        let (x, y) = slot_position(slot);
        (
            f64::from(screen.left() + x + 8),
            f64::from(screen.top() + y + 8),
        )
    }

    #[test]
    fn a_click_picks_up_on_press_and_places_on_release() {
        let mut screen = InventoryScreen::new(427, 240);
        let mut menu = InventoryMenu::default();
        menu.slots[36] = Some(Stack::new(64, 1));
        let now = std::time::Instant::now();
        let (x, y) = centre(&screen, 36);
        let press = screen.mouse_clicked(x, y, 0, false, now, &menu);
        assert_eq!(
            press,
            vec![SlotClick {
                slot: 36,
                button: 0,
                kind: ClickType::Pickup
            }]
        );
        // The release after a pick up is skipped.
        assert!(screen.mouse_released(x, y, 0, false, &menu).is_empty());
        menu.carried = menu.slots[36].take();
        let (x, y) = centre(&screen, 10);
        let later = now + std::time::Duration::from_secs(1);
        assert!(
            screen
                .mouse_clicked(x, y, 0, false, later, &menu)
                .is_empty()
        );
        let release = screen.mouse_released(x, y, 0, false, &menu);
        assert_eq!(
            release,
            vec![SlotClick {
                slot: 10,
                button: 0,
                kind: ClickType::Pickup
            }]
        );
    }

    #[test]
    fn a_drag_over_slots_sends_the_quick_craft_steps() {
        let mut screen = InventoryScreen::new(427, 240);
        let mut menu = InventoryMenu::default();
        menu.carried = Some(Stack::new(64, 1));
        let now = std::time::Instant::now();
        let (x, y) = centre(&screen, 10);
        assert!(screen.mouse_clicked(x, y, 1, false, now, &menu).is_empty());
        screen.mouse_dragged(x, y, &menu);
        let (x, y) = centre(&screen, 11);
        screen.mouse_dragged(x, y, &menu);
        let clicks = screen.mouse_released(x, y, 1, false, &menu);
        let steps: Vec<(i32, i32)> = clicks
            .iter()
            .map(|click| (click.slot, click.button))
            .collect();
        assert_eq!(steps, vec![(-999, 4), (10, 5), (11, 5), (-999, 6)]);
        assert!(
            clicks
                .iter()
                .all(|click| click.kind == ClickType::QuickCraft)
        );
    }

    #[test]
    fn a_double_click_gathers() {
        let mut screen = InventoryScreen::new(427, 240);
        let mut menu = InventoryMenu::default();
        menu.slots[20] = Some(Stack::new(5, 1));
        let now = std::time::Instant::now();
        let (x, y) = centre(&screen, 20);
        screen.mouse_clicked(x, y, 0, false, now, &menu);
        screen.mouse_released(x, y, 0, false, &menu);
        menu.carried = menu.slots[20].take();
        screen.mouse_clicked(
            x,
            y,
            0,
            false,
            now + std::time::Duration::from_millis(100),
            &menu,
        );
        let clicks = screen.mouse_released(x, y, 0, false, &menu);
        assert_eq!(
            clicks,
            vec![SlotClick {
                slot: 20,
                button: 0,
                kind: ClickType::PickupAll
            }]
        );
    }
}
