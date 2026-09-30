//! The screens of the containers the server opens: `ContainerScreen` (the
//! chest family), `DispenserScreen`, `HopperScreen` and `ShulkerBoxScreen`,
//! each `AbstractContainerScreen` over a generic menu. The clicks themselves
//! run in [`voxel_game::container_menu`].

mod transform;

use super::container::{ContainerLayout, ContainerMouse, SlotClick, slot_key_clicks};
use super::recipe_book::{RecipeAssets, RecipeBookPanel};
use super::{KeyMods, KeyResult, Screen, ScreenAction, ScreenDraw};
use crate::gui::edit_box::Clipboard;
use crate::gui::font::Font;
use crate::gui::frame::GuiBlend;
use voxel_game::container_menu::{ClickType, ContainerMenu, MenuKind, Smelter};
use voxel_game::recipe_book::BookKind;

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
    /// The banner patterns the loom can offer, by registry id: the texture
    /// name and the translation key of each.
    pub patterns: std::sync::Arc<Vec<(String, String)>>,
    /// The client's language, for the loom's button tooltips.
    pub lang: std::sync::Arc<voxel_game::text::Lang>,
    /// The font the enchanting table writes its names in.
    pub alt_font: std::sync::Arc<Font>,
    /// Each enchantment's name, by registry id, for the clue it gives.
    pub enchantments: std::sync::Arc<Vec<EnchantText>>,
    /// What the recipe book writes.
    pub recipe: std::sync::Arc<RecipeAssets>,
}

/// An enchantment as the clue names it (`Enchantment.getFullname`): its name,
/// whether it is a curse and its highest level.
#[derive(Debug, Clone, Default)]
pub struct EnchantText {
    pub name: String,
    pub curse: bool,
    pub max_level: i32,
}

/// A generic container screen's own state between events.
#[derive(Debug, Clone)]
pub struct GenericScreen {
    kind: MenuKind,
    layout: ContainerLayout,
    titles: Titles,
    hovered: Option<usize>,
    mouse: ContainerMouse,
    /// The recipe book, for the menus that have one.
    book: Option<RecipeBookPanel>,
    /// What only the menus that transform their items keep.
    extras: transform::Extras,
}

impl GenericScreen {
    /// A screen for a menu, opened on a GUI of this size.
    pub fn new(kind: MenuKind, width: i32, height: i32, titles: Titles) -> Self {
        // `CraftingScreen` and `AbstractFurnaceScreen` place the toggle button.
        let button = if kind == MenuKind::Crafting {
            (5, -49)
        } else {
            (20, -49)
        };
        let book = BookKind::of_menu(kind).map(|book| {
            RecipeBookPanel::new(book, 176, button, titles.recipe.clone(), (width, height))
        });
        Self {
            kind,
            layout: ContainerLayout::new(width, height, image(kind)),
            titles,
            hovered: None,
            mouse: ContainerMouse::default(),
            book,
            extras: transform::Extras::new(),
        }
    }

    /// `AbstractRecipeBookScreen.widthTooNarrow`: the open book covers the
    /// screen, which takes no click or hover.
    fn book_covers(&self) -> bool {
        self.book
            .as_ref()
            .is_some_and(RecipeBookPanel::blocks_screen)
    }

    /// `hasClickedOutside`: off the image, and off the book's panel and tabs.
    fn outside(&self, mouse: (f64, f64)) -> bool {
        self.layout.outside(mouse)
            && !self.book.as_ref().is_some_and(|book| {
                book.claims(
                    mouse,
                    (self.layout.left(), self.layout.top()),
                    image(self.kind).1,
                )
            })
    }

    fn hovered_slot(&self, mouse: (f64, f64)) -> Option<usize> {
        if self.book_covers() {
            return None;
        }
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

    /// The clicks as actions, with what the recipe book answers them with.
    fn actions(&mut self, clicks: Vec<SlotClick>) -> Vec<ScreenAction> {
        let mut actions = Vec::new();
        for click in clicks {
            if let Some(book) = &mut self.book {
                book.slot_clicked(click.slot);
                actions.extend(book.drain_actions());
            }
            actions.push(ScreenAction::Container(click));
        }
        actions
    }
}

impl Screen for GenericScreen {
    fn resize(&mut self, width: i32, height: i32) {
        self.layout = ContainerLayout::new(width, height, image(self.kind));
        if let Some(book) = &mut self.book {
            book.resize((width, height));
            self.layout.left_override = Some(book.left_pos());
        }
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
        if let Some(entry) = self.stone_click(mouse, menu) {
            // `clickMenuButton`: one that is not the chosen one, and is an entry.
            let chosen = i32::from(menu.data.first().copied().unwrap_or(-1));
            return if entry == chosen {
                Vec::new()
            } else {
                vec![ScreenAction::Button(entry)]
            };
        }
        if let Some(entry) = self.loom_click(mouse, menu) {
            return vec![ScreenAction::Button(entry)];
        }
        if let Some(row) = self.enchant_click(mouse, menu) {
            return vec![ScreenAction::Button(row)];
        }
        if let Some(book) = &mut self.book
            && book.mouse_clicked(mouse, button, shift, self.layout.left())
        {
            return book.drain_actions();
        }
        if self.book_covers() {
            return Vec::new();
        }
        let slot = self.hovered_slot(mouse);
        let outside = self.outside(mouse);
        let clicks = self.mouse.clicked(
            double,
            slot,
            outside,
            button,
            shift,
            button == 2 && menu.creative(),
            menu,
        );
        self.actions(clicks)
    }

    fn mouse_dragged(&mut self, mouse: (f64, f64), menu: &ContainerMenu) {
        self.stone_drag(mouse);
        self.loom_drag(mouse);
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
        self.stone_release();
        self.loom_release();
        if self
            .book
            .as_mut()
            .is_some_and(RecipeBookPanel::take_swallowed_release)
        {
            return Vec::new();
        }
        if self.book_covers() {
            return Vec::new();
        }
        let slot = self.hovered_slot(mouse);
        let outside = self.outside(mouse);
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
    /// hotbar keys and the off hand key swap with the hovered slot, the drop
    /// key throws from it.
    fn key_pressed(
        &mut self,
        key: glfw::Key,
        mods: KeyMods,
        menu: &ContainerMenu,
        clipboard: &mut dyn Clipboard,
    ) -> KeyResult {
        if let Some(book) = &mut self.book
            && book.key_pressed(key, (mods.control || mods.command, mods.shift), clipboard)
        {
            return KeyResult::Actions(Vec::new());
        }
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
        let clicks = slot_key_clicks(key, mods.control, self.hovered, menu);
        KeyResult::Actions(self.actions(clicks))
    }

    fn key_released(&mut self) {
        if let Some(book) = &mut self.book {
            book.key_released();
        }
    }

    fn mouse_scrolled(&mut self, y: f64) {
        self.stone_wheel(y);
        self.loom_wheel(y);
    }

    fn char_typed(&mut self, character: char) {
        if let Some(book) = &mut self.book
            && book.char_typed(character)
        {
            return;
        }
        self.name_char(character);
    }

    fn drain_actions(&mut self) -> Vec<ScreenAction> {
        let mut actions = self.take_queued();
        if let Some(book) = &mut self.book {
            actions.extend(book.drain_actions());
        }
        actions
    }

    fn scripted_click(&mut self, slot: i32, button: i32, kind: ClickType) -> ScreenAction {
        ScreenAction::Container(SlotClick { slot, button, kind })
    }

    /// `extractRenderState`: the dimmed world, the image, the labels, the
    /// slots with the hovered one lit, the carried stack at the mouse, then
    /// the hovered stack's tooltip.
    fn build(&mut self, draw: &mut ScreenDraw<'_, '_>, menu: &ContainerMenu, mouse: (f64, f64)) {
        if let Some(book) = &mut self.book {
            book.sync(draw.book);
            self.layout.left_override = Some(book.left_pos());
        }
        let (left, top) = (self.layout.left(), self.layout.top());
        let (width, height) = image(self.kind);
        self.layout.background(draw);
        self.blit_background(draw);
        if let MenuKind::Furnace(smelter) = self.kind {
            self.furnace_progress(draw, menu, smelter);
        }
        if self.book_covers() {
            // A book with no room beside the screen covers it: only the
            // background shows, and the book over it.
            if let Some(book) = &mut self.book {
                book.draw(draw, menu, mouse, left);
            }
            self.mouse.draw_carried(draw, menu.carried.as_ref(), mouse);
            return;
        }
        self.draw_extras(draw, menu, mouse, (left, top));
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
        if let Some(book) = &mut self.book {
            book.draw(draw, menu, mouse, left);
            book.draw_ghost(draw, menu, (left, top), &|slot| slot_position(kind, slot));
        }
        self.mouse.draw_carried(draw, menu.carried.as_ref(), mouse);
        if let Some(lines) = self.extras.pending_tooltip.take() {
            self.layout.tooltip(draw, &lines, mouse);
        }
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
            patterns: std::sync::Arc::default(),
            lang: std::sync::Arc::default(),
            alt_font: std::sync::Arc::new(Font::default()),
            enchantments: std::sync::Arc::default(),
            recipe: std::sync::Arc::default(),
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
