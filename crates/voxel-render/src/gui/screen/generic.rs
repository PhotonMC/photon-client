//! The screens of the containers the server opens: `ContainerScreen` (the
//! chest family), `DispenserScreen`, `HopperScreen` and `ShulkerBoxScreen`,
//! each `AbstractContainerScreen` over a generic menu. The clicks themselves
//! run in [`voxel_game::container_menu`].

use super::container::{ContainerLayout, ContainerMouse, SlotClick, slot_key_clicks};
use super::{KeyMods, KeyResult, Screen, ScreenAction, ScreenDraw};
use crate::gui::edit_box::Clipboard;
use voxel_game::container_menu::{ClickType, ContainerMenu, MenuKind};

/// The size of the screen's image for a menu.
fn image(kind: MenuKind) -> (i32, i32) {
    match kind {
        MenuKind::Chest(rows) => (176, 114 + rows as i32 * 18),
        MenuKind::Dispenser | MenuKind::Crafting => (176, 166),
        MenuKind::Hopper => (176, 133),
        MenuKind::ShulkerBox | MenuKind::Inventory => (176, 167),
    }
}

/// Where a slot sits in the image: the menu's own slots as its constructor
/// places them, then the player's 27 and 9 below them.
pub fn slot_position(kind: MenuKind, index: usize) -> (i32, i32) {
    let top = kind.top_slots();
    if kind == MenuKind::Crafting && index < top {
        return if index == 0 {
            (124, 35)
        } else {
            (
                30 + ((index - 1) % 3) as i32 * 18,
                17 + ((index - 1) / 3) as i32 * 18,
            )
        };
    }
    if index < top {
        let (column, row) = ((index % 9) as i32, (index / 9) as i32);
        return match kind {
            MenuKind::Dispenser => (62 + (index % 3) as i32 * 18, 17 + (index / 3) as i32 * 18),
            MenuKind::Hopper => (44 + index as i32 * 18, 20),
            _ => (8 + column * 18, 18 + row * 18),
        };
    }
    // `addStandardInventorySlots`: three rows, then the hotbar 58 lower.
    let inventory_top = match kind {
        MenuKind::Chest(rows) => rows as i32 * 18 + 31,
        MenuKind::Hopper => 51,
        _ => 84,
    };
    let own = index - top;
    if own < 27 {
        (
            8 + (own % 9) as i32 * 18,
            inventory_top + (own / 9) as i32 * 18,
        )
    } else {
        (8 + (own - 27) as i32 * 18, inventory_top + 58)
    }
}

/// What the screen draws of its text: the menu's title and the label over the
/// player's inventory, in the client's language.
#[derive(Debug, Clone)]
pub struct Titles {
    pub title: String,
    pub inventory: String,
}

/// A generic container screen's own state between events.
#[derive(Debug, Clone)]
pub struct GenericScreen {
    kind: MenuKind,
    layout: ContainerLayout,
    titles: Titles,
    hovered: Option<usize>,
    mouse: ContainerMouse,
    /// The recipe book button of the crafting table took the press, so the
    /// release is swallowed; the book itself is not drawn.
    button_clicked: bool,
}

impl GenericScreen {
    /// A screen for a menu, opened on a GUI of this size.
    pub fn new(kind: MenuKind, width: i32, height: i32, titles: Titles) -> Self {
        Self {
            kind,
            layout: ContainerLayout::new(width, height, image(kind)),
            titles,
            hovered: None,
            mouse: ContainerMouse::default(),
            button_clicked: false,
        }
    }

    /// The crafting table's recipe book button, where `CraftingScreen` puts it.
    fn recipe_button(&self) -> Option<(i32, i32)> {
        (self.kind == MenuKind::Crafting)
            .then(|| (self.layout.left() + 5, self.layout.height / 2 - 49))
    }

    fn on_recipe_button(&self, (x, y): (f64, f64)) -> bool {
        self.recipe_button().is_some_and(|(bx, by)| {
            x >= f64::from(bx)
                && y >= f64::from(by)
                && x < f64::from(bx + 20)
                && y < f64::from(by + 18)
        })
    }

    fn hovered_slot(&self, mouse: (f64, f64)) -> Option<usize> {
        let kind = self.kind;
        self.layout
            .hovered(mouse, kind.slot_count(), |index| slot_position(kind, index))
    }

    fn actions(clicks: Vec<SlotClick>) -> Vec<ScreenAction> {
        clicks.into_iter().map(ScreenAction::Container).collect()
    }
}

impl Screen for GenericScreen {
    fn resize(&mut self, width: i32, height: i32) {
        self.layout = ContainerLayout::new(width, height, image(self.kind));
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
        if std::mem::take(&mut self.button_clicked) {
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
    /// hotbar keys and the off hand key swap with the hovered slot, the drop
    /// key throws from it.
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

    /// `extractRenderState`: the dimmed world, the image, the labels, the
    /// slots with the hovered one lit, the carried stack at the mouse, then
    /// the hovered stack's tooltip.
    fn build(&mut self, draw: &mut ScreenDraw<'_, '_>, menu: &ContainerMenu, mouse: (f64, f64)) {
        let (left, top) = (self.layout.left(), self.layout.top());
        let (width, height) = image(self.kind);
        self.layout.background(draw);
        match self.kind {
            MenuKind::Chest(rows) => {
                let upper = rows as i32 * 18 + 17;
                let texture = "container/generic_54";
                draw.frame.blit_texture(
                    draw.atlas,
                    texture,
                    (256, 256),
                    (0, 0),
                    left,
                    top,
                    width,
                    upper,
                );
                draw.frame.blit_texture(
                    draw.atlas,
                    texture,
                    (256, 256),
                    (0, 126),
                    left,
                    top + upper,
                    width,
                    96,
                );
            }
            other => {
                let texture = match other {
                    MenuKind::Dispenser => "container/dispenser",
                    MenuKind::Hopper => "container/hopper",
                    MenuKind::Crafting => "container/crafting_table",
                    _ => "container/shulker_box",
                };
                draw.frame.blit_texture(
                    draw.atlas,
                    texture,
                    (256, 256),
                    (0, 0),
                    left,
                    top,
                    width,
                    height,
                );
            }
        }
        if let Some((bx, by)) = self.recipe_button() {
            let button = if self.on_recipe_button(mouse) {
                "recipe_book/button_highlighted"
            } else {
                "recipe_book/button"
            };
            draw.frame.sprite(draw.atlas, button, bx, by, 20, 18);
        }
        // The dispenser centres its title; the others keep it at the corner.
        let title_x = if self.kind == MenuKind::Dispenser {
            (width - draw.font.width(&self.titles.title)) / 2
        } else {
            8
        };
        draw.frame.text(
            draw.atlas,
            draw.font,
            &self.titles.title,
            left + title_x,
            top + 6,
            0xFF40_4040,
        );
        draw.frame.text(
            draw.atlas,
            draw.font,
            &self.titles.inventory,
            left + 8,
            top + height - 94,
            0xFF40_4040,
        );
        self.hovered = self.hovered_slot(mouse);
        let kind = self.kind;
        if let Some(slot) = self.hovered {
            self.layout
                .highlight(draw, slot_position(kind, slot), false);
        }
        for index in 0..kind.slot_count() {
            let (x, y) = slot_position(kind, index);
            self.mouse
                .draw_slot(draw, menu, index, (left + x, top + y), None);
        }
        if let Some(slot) = self.hovered {
            self.layout.highlight(draw, slot_position(kind, slot), true);
        }
        self.mouse.draw_carried(draw, menu.carried.as_ref(), mouse);
        if menu.carried.is_none()
            && let Some(stack) = self
                .hovered
                .and_then(|slot| menu.slots.get(slot))
                .and_then(Option::as_ref)
        {
            let lines = (draw.tooltip)(stack, false);
            self.layout.tooltip(draw, &lines, mouse);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_world::ClientItemStackState as Stack;

    fn titles() -> Titles {
        Titles {
            title: "Chest".to_owned(),
            inventory: "Inventory".to_owned(),
        }
    }

    #[test]
    fn a_chests_slots_follow_its_constructor() {
        let chest = MenuKind::Chest(3);
        assert_eq!(slot_position(chest, 0), (8, 18));
        assert_eq!(slot_position(chest, 26), (152, 54));
        // The player's rows start 13 below the last chest row, the hotbar 58
        // under them.
        assert_eq!(slot_position(chest, 27), (8, 85));
        assert_eq!(slot_position(chest, 54), (8, 143));
        assert_eq!(image(chest), (176, 168));
        assert_eq!(slot_position(MenuKind::Hopper, 4), (116, 20));
        assert_eq!(slot_position(MenuKind::Hopper, 5), (8, 51));
        assert_eq!(slot_position(MenuKind::Dispenser, 4), (80, 35));
        assert_eq!(slot_position(MenuKind::ShulkerBox, 27), (8, 84));
        assert_eq!(slot_position(MenuKind::ShulkerBox, 63 - 1), (152, 142));
    }

    #[test]
    fn a_click_in_the_chest_names_the_menu_slot() {
        let kind = MenuKind::Chest(3);
        let mut screen = GenericScreen::new(kind, 427, 240, titles());
        let mut menu = ContainerMenu::for_menu(kind, 4, Vec::new(), None, 0);
        menu.slots[30] = Some(Stack::new(5, 1));
        let (x, y) = slot_position(kind, 30);
        let mouse = (
            f64::from(screen.layout.left() + x + 8),
            f64::from(screen.layout.top() + y + 8),
        );
        let press = screen.mouse_clicked(mouse, 0, false, std::time::Instant::now(), &menu);
        assert_eq!(
            press,
            vec![ScreenAction::Container(SlotClick {
                slot: 30,
                button: 0,
                kind: ClickType::Pickup
            })]
        );
    }
}
