//! The screens of the containers the server opens: `ContainerScreen` (the
//! chest family), `DispenserScreen`, `HopperScreen` and `ShulkerBoxScreen`,
//! each `AbstractContainerScreen` over a generic menu. The clicks themselves
//! run in [`voxel_game::container_menu`].

mod transform;

use super::container::{ContainerLayout, ContainerMouse, SlotClick, slot_key_clicks};
use super::{KeyMods, KeyResult, Screen, ScreenAction, ScreenDraw};
use crate::gui::edit_box::Clipboard;
use crate::gui::font::Font;
use crate::gui::frame::GuiBlend;
use voxel_game::container_menu::{ClickType, ContainerMenu, MenuKind, Smelter};

/// The size of the screen's image for a menu.
fn image(kind: MenuKind) -> (i32, i32) {
    match kind {
        MenuKind::Chest(rows) => (176, 114 + rows as i32 * 18),
        MenuKind::Dispenser | MenuKind::Crafting | MenuKind::Furnace(_) => (176, 166),
        MenuKind::Hopper => (176, 133),
        MenuKind::ShulkerBox | MenuKind::Inventory => (176, 167),
        _ => (176, 166),
    }
}

/// Where a slot sits in the image: the menu's own slots as its constructor
/// places them, then the player's 27 and 9 below them.
pub fn slot_position(kind: MenuKind, index: usize) -> (i32, i32) {
    let top = kind.top_slots();
    if matches!(kind, MenuKind::Furnace(_)) && index < top {
        return [(56, 17), (56, 53), (116, 35)][index];
    }
    if let Some(position) = transform::slot_position(kind, index) {
        return position;
    }
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
    /// The font the anvil's name box measures with.
    pub font: std::sync::Arc<Font>,
    /// `container.repair.cost` and `container.repair.expensive`, for the anvil.
    pub repair_cost: String,
    pub repair_expensive: String,
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
    /// What only the menus that transform their items keep.
    extras: transform::Extras,
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
            extras: transform::Extras::new(),
        }
    }

    /// The crafting table's recipe book button, where `CraftingScreen` puts it.
    fn recipe_button(&self) -> Option<(i32, i32)> {
        match self.kind {
            MenuKind::Crafting => Some((self.layout.left() + 5, self.layout.height / 2 - 49)),
            MenuKind::Furnace(_) => Some((self.layout.left() + 20, self.layout.height / 2 - 49)),
            _ => None,
        }
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

    /// The screen's texture, over the image's rectangle.
    fn blit_background(&self, draw: &mut ScreenDraw<'_, '_>) {
        let (left, top) = (self.layout.left(), self.layout.top());
        let (width, height) = image(self.kind);
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
                    MenuKind::Furnace(Smelter::Furnace) => "container/furnace",
                    MenuKind::Furnace(Smelter::BlastFurnace) => "container/blast_furnace",
                    MenuKind::Furnace(Smelter::Smoker) => "container/smoker",
                    MenuKind::ShulkerBox => "container/shulker_box",
                    other => transform::texture(other).unwrap_or("container/hopper"),
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
    }

    /// `AbstractFurnaceScreen.extractBackground`: the flame, as much of it as
    /// burns, and the arrow, as much as has cooked.
    fn furnace_progress(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        smelter: Smelter,
    ) {
        let (left, top) = (self.layout.left(), self.layout.top());
        let folder = match smelter {
            Smelter::Furnace => "furnace",
            Smelter::BlastFurnace => "blast_furnace",
            Smelter::Smoker => "smoker",
        };
        let data = |index: usize| i32::from(menu.data.get(index).copied().unwrap_or(0));
        if data(0) > 0 {
            let duration = if data(1) == 0 { 200 } else { data(1) };
            let lit = (data(0) as f32 / duration as f32).clamp(0.0, 1.0);
            let height = (lit * 13.0).ceil() as i32 + 1;
            draw.frame.blit_part(
                draw.atlas,
                GuiBlend::Translucent,
                &format!("container/{folder}/lit_progress"),
                (14, 14),
                (0, 14 - height),
                left + 56,
                top + 36 + 14 - height,
                14,
                height,
            );
        }
        let (current, total) = (data(2), data(3));
        let cooked = if total != 0 && current != 0 {
            (current as f32 / total as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let width = (cooked * 24.0).ceil() as i32;
        draw.frame.blit_part(
            draw.atlas,
            GuiBlend::Translucent,
            &format!("container/{folder}/burn_progress"),
            (24, 16),
            (0, 0),
            left + 79,
            top + 34,
            width,
            16,
        );
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
        clipboard: &mut dyn Clipboard,
    ) -> KeyResult {
        if let Some(result) = self.name_key(
            key,
            mods.control || mods.command,
            mods.shift,
            menu,
            clipboard,
        ) {
            return result;
        }
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

    fn char_typed(&mut self, character: char) {
        self.name_char(character);
    }

    fn drain_actions(&mut self) -> Vec<ScreenAction> {
        self.take_queued()
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
        self.blit_background(draw);
        if let MenuKind::Furnace(smelter) = self.kind {
            self.furnace_progress(draw, menu, smelter);
        }
        self.draw_extras(draw, menu, (left, top));
        if let Some((bx, by)) = self.recipe_button() {
            let button = if self.on_recipe_button(mouse) {
                "recipe_book/button_highlighted"
            } else {
                "recipe_book/button"
            };
            draw.frame.sprite(draw.atlas, button, bx, by, 20, 18);
        }
        // The dispenser centres its title; the others keep it at the corner.
        let (label_x, label_y) = transform::title_label(self.kind);
        let title_x = if matches!(self.kind, MenuKind::Dispenser | MenuKind::Furnace(_)) {
            (width - draw.font.width(&self.titles.title)) / 2
        } else {
            label_x
        };
        draw.frame.text(
            draw.atlas,
            draw.font,
            &self.titles.title,
            left + title_x,
            top + label_y,
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
            self.mouse.draw_slot(
                draw,
                menu,
                index,
                (left + x, top + y),
                transform::empty_icon(kind, index),
            );
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
            font: std::sync::Arc::new(Font::default()),
            repair_cost: String::new(),
            repair_expensive: String::new(),
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
