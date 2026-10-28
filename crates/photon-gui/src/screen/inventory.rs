//! The survival inventory screen: `AbstractContainerScreen` over the
//! player's own menu, as `InventoryScreen` lays it out with its recipe book
//! closed. The screen turns mouse and key input into menu clicks and draws
//! the menu; the clicks themselves run in [`photon_game::menus::container`].
//!
//! The recipe book beside it is [`super::recipe_book`]'s.

use super::bundle_tooltip::item_tooltip;
use super::container::{ContainerLayout, ContainerMouse, SlotClick, SlotView, slot_key_clicks};
use super::recipe_book::{RecipeAssets, RecipeBookPanel};
use super::{KeyMods, KeyResult, Screen, ScreenAction, ScreenDraw};
use crate::widgets::edit_box::Clipboard;
use photon_game::menus::container::{ClickType, ContainerMenu, SLOT_COUNT};
use photon_game::menus::recipe_book::BookKind;
use photon_world::ClientItemStackState as Stack;

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
    fn inventory_slot(&self, index: usize) -> Option<usize> {
        self.slot_of_inventory(index)
    }

    fn is_fake(&self, slot: usize) -> bool {
        self.is_fake_slot(slot)
    }

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
        !self.is_result(slot)
    }

    fn slot_count(&self) -> usize {
        self.slots.len()
    }

    fn same_container(&self, a: usize, b: usize) -> bool {
        self.same_inventory(a, b)
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
    book: RecipeBookPanel,
}

impl InventoryScreen {
    /// A screen opened on a GUI of this size.
    pub fn new(
        width: i32,
        height: i32,
        title: String,
        recipe: std::sync::Arc<RecipeAssets>,
    ) -> Self {
        Self {
            layout: ContainerLayout::new(width, height, IMAGE),
            title,
            hovered: None,
            mouse: ContainerMouse::default(),
            book: RecipeBookPanel::new(
                BookKind::Crafting { size: 2 },
                IMAGE.0,
                (104, -22),
                recipe,
                (width, height),
            ),
        }
    }

    /// The slots under the hovered one's highlight and the highlight over it.
    fn draw_slots(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
    ) {
        let previous = self.hovered;
        self.hovered = self.hovered_slot(mouse);
        self.mouse
            .bundle
            .hover_changed(previous, self.hovered, menu);
        if let Some(slot) = self.hovered {
            self.layout.highlight(draw, slot_position(slot), false);
        }
        for index in 0..SLOT_COUNT {
            let (x, y) = slot_position(index);
            self.mouse
                .draw_slot(draw, menu, index, &self.layout, (x, y), empty_icon(index));
        }
        if let Some(slot) = self.hovered {
            self.layout.highlight(draw, slot_position(slot), true);
        }
    }

    fn hovered_slot(&self, mouse: (f64, f64)) -> Option<usize> {
        if self.book.blocks_screen() {
            return None;
        }
        self.layout.hovered(mouse, SLOT_COUNT, slot_position)
    }

    /// The clicks as actions, with what the recipe book answers them with.
    fn actions(&mut self, clicks: Vec<SlotClick>) -> Vec<ScreenAction> {
        let mut actions = Vec::new();
        for click in clicks {
            self.book.slot_clicked(click.slot);
            actions.extend(self.book.drain_actions());
            actions.push(ScreenAction::Container(click));
        }
        actions
    }
}

impl Screen for InventoryScreen {
    fn resize(&mut self, width: i32, height: i32) {
        self.layout = ContainerLayout::new(width, height, IMAGE);
        self.book.resize((width, height));
        self.layout.left_override = Some(self.book.left_pos());
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
        // The recipe book's widgets take the press first.
        if self
            .book
            .mouse_clicked(mouse, button, shift, self.layout.left())
        {
            return self.book.drain_actions();
        }
        if self.book.blocks_screen() {
            return Vec::new();
        }
        let slot = self.hovered_slot(mouse);
        let outside = self.layout.outside(mouse)
            && !self
                .book
                .claims(mouse, (self.layout.left(), self.layout.top()), IMAGE.1);
        let clicks = self.mouse.clicked(
            double,
            (slot, outside),
            button,
            shift,
            button == 2 && menu.creative(),
            menu,
        );
        self.actions(clicks)
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
        if self.book.take_swallowed_release() || self.book.blocks_screen() {
            return Vec::new();
        }
        let slot = self.hovered_slot(mouse);
        let outside = self.layout.outside(mouse)
            && !self
                .book
                .claims(mouse, (self.layout.left(), self.layout.top()), IMAGE.1);
        let clicks = self.mouse.released(
            slot,
            outside,
            button,
            shift,
            button == 2 && menu.creative(),
            menu,
        );
        self.actions(clicks)
    }

    /// `keyPressed`: the inventory key and escape close the screen; the
    /// hotbar keys and the off hand key swap with the hovered slot; the drop
    /// key throws from it, all of it with control.
    fn key_pressed(
        &mut self,
        key: glfw::Key,
        mods: KeyMods,
        menu: &ContainerMenu,
        clipboard: &mut dyn Clipboard,
    ) -> KeyResult {
        if self
            .book
            .key_pressed(key, (mods.control || mods.command, mods.shift), clipboard)
        {
            return KeyResult::Consumed(Vec::new());
        }
        if matches!(key, glfw::Key::Escape | glfw::Key::E) {
            return KeyResult::Close;
        }
        let clicks = slot_key_clicks(key, mods.control, self.hovered, menu);
        KeyResult::Actions(self.actions(clicks))
    }

    fn key_released(&mut self) {
        self.book.key_released();
    }

    fn char_typed(&mut self, character: char) {
        self.book.char_typed(character);
    }

    fn mouse_scrolled(&mut self, y: f64, menu: &ContainerMenu) {
        self.mouse.bundle.scrolled(y, self.hovered, menu);
    }

    fn drain_actions(&mut self) -> Vec<ScreenAction> {
        let mut actions = self.mouse.bundle.drain();
        actions.extend(self.book.drain_actions());
        actions
    }

    fn scripted_click(&mut self, slot: i32, button: i32, kind: ClickType) -> ScreenAction {
        ScreenAction::Container(SlotClick { slot, button, kind })
    }

    /// `extractRenderState`: the dimmed world, the image, the recipe book
    /// button, the player, the title, the slots with the hovered one lit,
    /// the carried stack at the mouse, then the hovered stack's tooltip.
    fn build(&mut self, draw: &mut ScreenDraw<'_, '_>, menu: &ContainerMenu, mouse: (f64, f64)) {
        self.book.sync(draw.book);
        self.layout.left_override = Some(self.book.left_pos());
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
        let model = (draw.player)(
            (left + 26, top + 8, left + 75, top + 78),
            30.0,
            0.0625,
            (mouse.0 as f32, mouse.1 as f32),
        );
        draw.frame.figure(&model);
        // `InventoryScreen.extractRenderState`: the effects come after the background stage (the
        // dimmed world, the image and the player) and before the rest of the screen.
        let effects_tooltip = super::effects::draw(draw, &self.layout, mouse);
        if self.book.blocks_screen() {
            // A book with no room beside the screen covers it: only the
            // background shows, and the book over it.
            self.book.draw(draw, menu, mouse, left);
            self.mouse.draw_carried(draw, menu.carried.as_ref(), mouse);
            super::effects::draw_tooltip(draw, &self.layout, effects_tooltip, mouse);
            return;
        }
        draw.frame.text(
            draw.atlas,
            draw.font,
            &self.title,
            left + 97,
            top + 6,
            0xFF40_4040,
        );
        self.draw_slots(draw, menu, mouse);
        self.book.draw(draw, menu, mouse, left);
        self.book
            .draw_ghost(draw, menu, (left, top), &slot_position);
        self.mouse.draw_carried(draw, menu.carried.as_ref(), mouse);
        // The hovered stack's tooltip, when nothing is carried (or it is a bundle's).
        let hovered = self.hovered.and_then(|slot| menu.slots[slot].as_ref());
        let shown = hovered.is_some_and(|stack| {
            item_tooltip(&self.layout, draw, stack, menu.carried.is_some(), mouse)
        });
        if !shown {
            self.book.ghost_tooltip(draw, menu, self.hovered, mouse);
        }
        super::effects::draw_tooltip(draw, &self.layout, effects_tooltip, mouse);
    }

    fn shows_active_effects(&self) -> bool {
        super::effects::can_see(&self.layout)
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
        let mut screen = InventoryScreen::new(427, 240, String::new(), std::sync::Arc::default());
        let mut menu = ContainerMenu::default();
        menu.slots[36] = Some(Stack::new(64, photon_core::ids::ItemId(1)));
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
        let mut screen = InventoryScreen::new(427, 240, String::new(), std::sync::Arc::default());
        let mut menu = ContainerMenu::default();
        menu.carried = Some(Stack::new(64, photon_core::ids::ItemId(1)));
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
        let mut screen = InventoryScreen::new(427, 240, String::new(), std::sync::Arc::default());
        let mut menu = ContainerMenu::default();
        menu.slots[20] = Some(Stack::new(5, photon_core::ids::ItemId(1)));
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

    struct NoClip;
    impl Clipboard for NoClip {
        fn get(&mut self) -> String {
            String::new()
        }
        fn set(&mut self, _text: &str) {}
    }

    fn press(screen: &mut InventoryScreen, key: glfw::Key, menu: &ContainerMenu) -> KeyResult {
        screen.key_pressed(key, KeyMods::default(), menu, &mut NoClip)
    }

    #[test]
    fn a_letter_the_screen_has_no_use_for_is_not_consumed() {
        let mut screen = InventoryScreen::new(427, 240, String::new(), std::sync::Arc::default());
        let menu = ContainerMenu::default();
        assert_eq!(
            press(&mut screen, glfw::Key::H, &menu),
            KeyResult::Actions(Vec::new())
        );
        assert_eq!(press(&mut screen, glfw::Key::E, &menu), KeyResult::Close);
    }

    #[test]
    fn a_hotbar_swap_is_not_consumed() {
        let mut screen = InventoryScreen::new(427, 240, String::new(), std::sync::Arc::default());
        let mut menu = ContainerMenu::default();
        menu.slots[20] = Some(Stack::new(5, photon_core::ids::ItemId(1)));
        screen.hovered = Some(20);
        let result = press(&mut screen, glfw::Key::Num1, &menu);
        assert!(matches!(result, KeyResult::Actions(actions) if !actions.is_empty()));
    }
}
