//! The survival inventory screen: `AbstractContainerScreen` over the
//! player's own menu, as `InventoryScreen` lays it out with its recipe book
//! closed. The screen turns mouse and key input into menu clicks and draws
//! the menu; the clicks themselves run in [`voxel_game::container_menu`].
//!
//! Not drawn: the recipe book; its button is drawn and takes the click, but
//! does not open it.

use super::container::{ContainerLayout, ContainerMouse, SlotClick, SlotView, slot_key_clicks};
use super::{KeyMods, KeyResult, Screen, ScreenAction, ScreenDraw};
use crate::gui::edit_box::Clipboard;
use voxel_game::container_menu::{ClickType, ContainerMenu, SLOT_COUNT};
use voxel_world::ClientItemStackState as Stack;

const IMAGE: (i32, i32) = (176, 166);

/// Where each menu slot sits in the screen's image: `ContainerMenu`'s
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

impl SlotView for ContainerMenu {
    fn stack(&self, slot: usize) -> Option<&Stack> {
        self.slots[slot].as_ref()
    }

    fn carried(&self) -> Option<&Stack> {
        self.carried.as_ref()
    }

    fn can_item_quick_replace(&self, slot: usize, stack: &Stack, ignore_size: bool) -> bool {
        ContainerMenu::can_item_quick_replace(self, slot, stack, ignore_size)
    }

    fn may_place(&self, slot: usize, stack: &Stack) -> bool {
        ContainerMenu::may_place(self, slot, stack)
    }

    fn slot_max_for(&self, slot: usize, stack: &Stack) -> i32 {
        ContainerMenu::slot_max_for(self, slot, stack)
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

/// The survival inventory screen's own state between events.
#[derive(Debug, Clone)]
pub struct InventoryScreen {
    layout: ContainerLayout,
    /// The title over the crafting grid (`container.crafting`).
    title: String,
    hovered: Option<usize>,
    mouse: ContainerMouse,
    /// `InventoryScreen.buttonClicked`: the recipe book button took the
    /// press, so the release is swallowed.
    button_clicked: bool,
}

impl InventoryScreen {
    /// A screen opened on a GUI of this size.
    pub fn new(width: i32, height: i32, title: String) -> Self {
        Self {
            layout: ContainerLayout::new(width, height, IMAGE),
            title,
            hovered: None,
            mouse: ContainerMouse::default(),
            button_clicked: false,
        }
    }

    fn recipe_button(&self) -> (i32, i32) {
        (self.layout.left() + 104, self.layout.height / 2 - 22)
    }

    fn on_recipe_button(&self, (x, y): (f64, f64)) -> bool {
        let (bx, by) = self.recipe_button();
        x >= f64::from(bx) && y >= f64::from(by) && x < f64::from(bx + 20) && y < f64::from(by + 18)
    }

    fn hovered_slot(&self, mouse: (f64, f64)) -> Option<usize> {
        self.layout.hovered(mouse, SLOT_COUNT, slot_position)
    }

    fn actions(clicks: Vec<SlotClick>) -> Vec<ScreenAction> {
        clicks.into_iter().map(ScreenAction::Container).collect()
    }
}

impl Screen for InventoryScreen {
    fn resize(&mut self, width: i32, height: i32) {
        self.layout = ContainerLayout::new(width, height, IMAGE);
    }

    fn mouse_clicked(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        now: std::time::Instant,
        menu: &ContainerMenu,
    ) -> Vec<ScreenAction> {
        let double = self.mouse.note_click(now, button);
        // The recipe book button is a widget, which takes the press first.
        if self.on_recipe_button(mouse) {
            self.button_clicked = true;
            return Vec::new();
        }
        let slot = self.hovered_slot(mouse);
        let outside = self.layout.outside(mouse);
        Self::actions(
            self.mouse
                .clicked(double, slot, outside, button, shift, false, menu),
        )
    }

    fn mouse_dragged(&mut self, mouse: (f64, f64), menu: &ContainerMenu) {
        let slot = self.hovered_slot(mouse);
        self.mouse.dragged(slot, menu);
    }

    fn mouse_released(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        menu: &ContainerMenu,
    ) -> Vec<ScreenAction> {
        if self.button_clicked {
            self.button_clicked = false;
            return Vec::new();
        }
        let slot = self.hovered_slot(mouse);
        let outside = self.layout.outside(mouse);
        Self::actions(
            self.mouse
                .released(slot, outside, button, shift, false, menu),
        )
    }

    /// `keyPressed`: the inventory key and escape close the screen; the
    /// hotbar keys and the off hand key swap with the hovered slot; the drop
    /// key throws from it, all of it with control.
    fn key_pressed(
        &mut self,
        key: glfw::Key,
        mods: KeyMods,
        menu: &ContainerMenu,
        _clipboard: &mut dyn Clipboard,
    ) -> KeyResult {
        if matches!(key, glfw::Key::Escape | glfw::Key::E) {
            return KeyResult::Close;
        }
        KeyResult::Actions(Self::actions(slot_key_clicks(
            key,
            mods.control,
            self.hovered,
            menu,
        )))
    }

    fn scripted_click(&mut self, slot: i32, button: i32, kind: ClickType) -> ScreenAction {
        ScreenAction::Container(SlotClick { slot, button, kind })
    }

    /// `extractRenderState`: the dimmed world, the image, the recipe book
    /// button, the player, the title, the slots with the hovered one lit,
    /// the carried stack at the mouse, then the hovered stack's tooltip.
    fn build(&mut self, draw: &mut ScreenDraw<'_, '_>, menu: &ContainerMenu, mouse: (f64, f64)) {
        let (left, top) = (self.layout.left(), self.layout.top());
        self.layout.background(draw);
        draw.frame.blit_texture(
            draw.atlas,
            "container/inventory",
            (256, 256),
            (0, 0),
            left,
            top,
            IMAGE.0,
            IMAGE.1,
        );
        let (bx, by) = self.recipe_button();
        let button = if self.on_recipe_button(mouse) {
            "recipe_book/button_highlighted"
        } else {
            "recipe_book/button"
        };
        draw.frame.sprite(draw.atlas, button, bx, by, 20, 18);
        let model = (draw.player)(
            (left + 26, top + 8, left + 75, top + 78),
            30.0,
            0.0625,
            (mouse.0 as f32, mouse.1 as f32),
        );
        draw.frame.entity(&model);
        draw.frame.text(
            draw.atlas,
            draw.font,
            &self.title,
            left + 97,
            top + 6,
            0xFF40_4040,
        );
        self.hovered = self.hovered_slot(mouse);
        if let Some(slot) = self.hovered {
            self.layout.highlight(draw, slot_position(slot), false);
        }
        for index in 0..SLOT_COUNT {
            let (x, y) = slot_position(index);
            self.mouse
                .draw_slot(draw, menu, index, (left + x, top + y), empty_icon(index));
        }
        if let Some(slot) = self.hovered {
            self.layout.highlight(draw, slot_position(slot), true);
        }
        self.mouse.draw_carried(draw, menu.carried.as_ref(), mouse);
        // The hovered stack's tooltip, when nothing is carried.
        if menu.carried.is_none()
            && let Some(stack) = self.hovered.and_then(|slot| menu.slots[slot].as_ref())
        {
            let lines = (draw.tooltip)(stack, false);
            self.layout.tooltip(draw, &lines, mouse);
        }
    }
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
            f64::from(screen.layout.left() + x + 8),
            f64::from(screen.layout.top() + y + 8),
        )
    }

    #[test]
    fn a_click_picks_up_on_press_and_places_on_release() {
        let mut screen = InventoryScreen::new(427, 240, String::new());
        let mut menu = ContainerMenu::default();
        menu.slots[36] = Some(Stack::new(64, 1));
        let now = std::time::Instant::now();
        let (x, y) = centre(&screen, 36);
        let press = screen.mouse_clicked((x, y), 0, false, now, &menu);
        assert_eq!(
            press,
            vec![ScreenAction::Container(SlotClick {
                slot: 36,
                button: 0,
                kind: ClickType::Pickup
            })]
        );
        // The release after a pick up is skipped.
        assert!(screen.mouse_released((x, y), 0, false, &menu).is_empty());
        menu.carried = menu.slots[36].take();
        let (x, y) = centre(&screen, 10);
        let later = now + std::time::Duration::from_secs(1);
        assert!(
            screen
                .mouse_clicked((x, y), 0, false, later, &menu)
                .is_empty()
        );
        let release = screen.mouse_released((x, y), 0, false, &menu);
        assert_eq!(
            release,
            vec![ScreenAction::Container(SlotClick {
                slot: 10,
                button: 0,
                kind: ClickType::Pickup
            })]
        );
    }

    #[test]
    fn a_drag_over_slots_sends_the_quick_craft_steps() {
        let mut screen = InventoryScreen::new(427, 240, String::new());
        let mut menu = ContainerMenu::default();
        menu.carried = Some(Stack::new(64, 1));
        let now = std::time::Instant::now();
        let (x, y) = centre(&screen, 10);
        assert!(
            screen
                .mouse_clicked((x, y), 1, false, now, &menu)
                .is_empty()
        );
        screen.mouse_dragged((x, y), &menu);
        let (x, y) = centre(&screen, 11);
        screen.mouse_dragged((x, y), &menu);
        let clicks = screen.mouse_released((x, y), 1, false, &menu);
        let clicks: Vec<SlotClick> = clicks
            .into_iter()
            .map(|action| match action {
                ScreenAction::Container(click) => click,
                other => panic!("not a container click: {other:?}"),
            })
            .collect();
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
        let mut screen = InventoryScreen::new(427, 240, String::new());
        let mut menu = ContainerMenu::default();
        menu.slots[20] = Some(Stack::new(5, 1));
        let now = std::time::Instant::now();
        let (x, y) = centre(&screen, 20);
        screen.mouse_clicked((x, y), 0, false, now, &menu);
        screen.mouse_released((x, y), 0, false, &menu);
        menu.carried = menu.slots[20].take();
        screen.mouse_clicked(
            (x, y),
            0,
            false,
            now + std::time::Duration::from_millis(100),
            &menu,
        );
        let clicks = screen.mouse_released((x, y), 0, false, &menu);
        assert_eq!(
            clicks,
            vec![ScreenAction::Container(SlotClick {
                slot: 20,
                button: 0,
                kind: ClickType::PickupAll
            })]
        );
    }
}
