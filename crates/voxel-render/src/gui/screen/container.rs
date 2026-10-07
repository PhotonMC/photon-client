//! What the container screens share: `AbstractContainerScreen`'s mouse
//! handling (clicks, drags, double clicks) over a view of a menu, the slot
//! keys, and drawing a slot and the carried stack.

use super::ScreenDraw;
use voxel_game::menus::container::{ClickType, max_stack_size};
use voxel_world::ClientItemStackState as Stack;

/// `MouseHandler`'s double click window.
const DOUBLE_CLICK: std::time::Duration = std::time::Duration::from_millis(250);

/// A click the screen asks the menu for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotClick {
    pub slot: i32,
    pub button: i32,
    pub kind: ClickType,
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
    #[expect(
        clippy::too_many_arguments,
        reason = "the reference method's parameters, in its order"
    )]
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

impl ContainerMouse {
    /// `extractCarriedItem`: the stack at the mouse, less what a drag over
    /// more than one slot would leave.
    pub(crate) fn draw_carried(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
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
            draw.frame.item_with_count(
                draw.atlas, draw.font, draw.item, carried, count, shown, x, y,
            );
        }
    }

    /// `extractSlot`: a drag's preview count, an empty slot's icon, or the
    /// stack and its count.
    pub(crate) fn draw_slot(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
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
            draw.frame.sprite(draw.atlas, icon, x, y, 16, 16);
            return;
        }
        if preview {
            draw.frame
                .fill(draw.atlas, (x, y, x + 16, y + 16), 0x80FF_FFFF);
        }
        if let Some((stack, count)) = shown {
            draw.frame
                .item_with_count(draw.atlas, draw.font, draw.item, &stack, count, text, x, y);
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

/// Where a container screen's image sits in the GUI and what the mouse is
/// over: `AbstractContainerScreen`'s `leftPos`, `topPos` and slot hit tests.
#[derive(Debug, Clone, Copy)]
pub struct ContainerLayout {
    pub width: i32,
    pub height: i32,
    /// The image's size (`imageWidth`, `imageHeight`).
    pub image: (i32, i32),
    /// A `leftPos` the recipe book moved the image to.
    pub left_override: Option<i32>,
}

impl ContainerLayout {
    pub fn new(width: i32, height: i32, image: (i32, i32)) -> Self {
        Self {
            width,
            height,
            image,
            left_override: None,
        }
    }

    /// `leftPos`.
    pub fn left(&self) -> i32 {
        self.left_override
            .unwrap_or((self.width - self.image.0) / 2)
    }

    /// `topPos`.
    pub fn top(&self) -> i32 {
        (self.height - self.image.1) / 2
    }

    /// The mouse relative to the image's corner.
    pub fn relative(&self, (x, y): (f64, f64)) -> (f64, f64) {
        (x - f64::from(self.left()), y - f64::from(self.top()))
    }

    /// `getHoveredSlot`: the first of `count` slots, placed in the image by
    /// `position`, within a unit of the mouse.
    pub fn hovered(
        &self,
        mouse: (f64, f64),
        count: usize,
        position: impl Fn(usize) -> (i32, i32),
    ) -> Option<usize> {
        let (xm, ym) = self.relative(mouse);
        (0..count).find(|&index| {
            let (sx, sy) = position(index);
            xm >= f64::from(sx - 1)
                && xm < f64::from(sx + 17)
                && ym >= f64::from(sy - 1)
                && ym < f64::from(sy + 17)
        })
    }

    /// `hasClickedOutside`: the mouse is off the image.
    pub fn outside(&self, (x, y): (f64, f64)) -> bool {
        let (left, top) = (f64::from(self.left()), f64::from(self.top()));
        x < left
            || y < top
            || x >= left + f64::from(self.image.0)
            || y >= top + f64::from(self.image.1)
    }

    /// `extractTransparentBackground`: the world dimmed behind the screen.
    pub fn background(&self, draw: &mut ScreenDraw<'_, '_>) {
        draw.frame.fill_gradient(
            draw.atlas,
            (0, 0, self.width, self.height),
            0xC010_1010,
            0xD010_1010,
        );
    }

    /// The hovered slot's highlight, behind or in front of its stack.
    pub fn highlight(&self, draw: &mut ScreenDraw<'_, '_>, (x, y): (i32, i32), front: bool) {
        let sprite = if front {
            "container/slot_highlight_front"
        } else {
            "container/slot_highlight_back"
        };
        draw.frame.sprite(
            draw.atlas,
            sprite,
            self.left() + x - 4,
            self.top() + y - 4,
            24,
            24,
        );
    }

    /// A stack's tooltip beside the mouse.
    pub fn tooltip(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        lines: &[Vec<voxel_game::text::Run>],
        mouse: (f64, f64),
    ) {
        draw.frame.tooltip(
            draw.atlas,
            draw.font,
            lines,
            (mouse.0 as i32, mouse.1 as i32),
            (self.width, self.height),
        );
    }
}
