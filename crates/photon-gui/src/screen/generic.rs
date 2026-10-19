//! The screens of the containers the server opens: `ContainerScreen` (the
//! chest family), `DispenserScreen`, `HopperScreen` and `ShulkerBoxScreen`,
//! each `AbstractContainerScreen` over a generic menu. The clicks themselves
//! run in [`photon_game::menus::container`].

mod beacon;
mod merchant;
mod transform;

use super::container::{ContainerLayout, ContainerMouse, SlotClick, slot_key_clicks};
use super::recipe_book::{RecipeAssets, RecipeBookPanel};
use super::{KeyMods, KeyResult, Screen, ScreenAction, ScreenDraw};
use crate::widgets::edit_box::Clipboard;
use photon_game::menus::container::{ClickType, ContainerMenu, MenuKind, MountFamily, Smelter};
use photon_game::menus::recipe_book::BookKind;
use photon_render::gui::font::Font;
use photon_render::gui::frame::GuiBlend;

/// The size of the screen's image for a menu.
fn image(kind: MenuKind) -> (i32, i32) {
    match kind {
        MenuKind::Chest(rows) => (176, 114 + rows as i32 * 18),
        MenuKind::Dispenser | MenuKind::Crafting | MenuKind::Furnace(_) => (176, 166),
        MenuKind::Hopper => (176, 133),
        MenuKind::ShulkerBox | MenuKind::Inventory => (176, 167),
        MenuKind::Beacon => (230, 219),
        MenuKind::Merchant => (276, 166),
        _ => (176, 166),
    }
}

/// Where a slot sits in the image: the menu's own slots as its constructor
/// places them, then the player's 27 and 9 below them.
pub fn slot_position(kind: MenuKind, index: usize) -> (i32, i32) {
    let top = kind.top_slots();
    // `CrafterMenu`'s result slot comes after the player's slots.
    if kind == MenuKind::Crafter && index == 45 {
        return (134, 35);
    }
    // `HorseInventoryMenu`: the saddle, the armour, then the chest's three rows.
    if let MenuKind::Mount(mount) = kind
        && index < top
    {
        return match index {
            0 => (8, 18),
            1 => (8, 36),
            _ => {
                let (column, row) = (
                    (index - 2) % usize::from(mount.columns),
                    (index - 2) / usize::from(mount.columns),
                );
                (80 + column as i32 * 18, 18 + row as i32 * 18)
            }
        };
    }
    if kind == MenuKind::Merchant && index < 3 {
        return [(136, 37), (162, 37), (220, 37)][index];
    }
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
    let (inventory_left, inventory_top) = match kind {
        MenuKind::Chest(rows) => (8, rows as i32 * 18 + 31),
        MenuKind::Hopper => (8, 51),
        MenuKind::Beacon => (36, 137),
        MenuKind::Merchant => (108, 84),
        _ => (8, 84),
    };
    let own = index - top;
    if own < 27 {
        (
            inventory_left + (own % 9) as i32 * 18,
            inventory_top + (own / 9) as i32 * 18,
        )
    } else {
        (inventory_left + (own - 27) as i32 * 18, inventory_top + 58)
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
    pub lang: std::sync::Arc<photon_game::text::Lang>,
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
    /// The beacon's buttons, for its menu.
    beacon: Option<beacon::BeaconPanel>,
    /// The trading screen's list, for the merchant's menu.
    merchant: Option<merchant::MerchantPanel>,
    /// `AbstractMountInventoryScreen.xMouse` and `yMouse`: the mouse of the frame before, which
    /// the background turns the mount's model by.
    last_mouse: (f64, f64),
}

impl Screen for GenericScreen {
    fn tick(&mut self, menu: &ContainerMenu) {
        if let Some(panel) = &mut self.beacon {
            panel.tick(menu);
        }
        if self.kind == MenuKind::Smithing {
            self.extras.tick_smithing(menu);
        }
    }

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
        let origin = (self.layout.left(), self.layout.top());
        if let Some(panel) = &mut self.merchant
            && let Some(actions) = panel.click(mouse, origin, menu, button == 0)
        {
            return actions;
        }
        if button == 0
            && let Some(panel) = &mut self.beacon
            && let Some(actions) = panel.click(mouse, origin, menu)
        {
            return actions;
        }
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
        let slot = self.hovered_slot(mouse, menu);
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
        self.actions(clicks, menu)
    }

    fn mouse_dragged(&mut self, mouse: (f64, f64), menu: &ContainerMenu) {
        let top = self.layout.top();
        if let Some(panel) = &mut self.merchant {
            panel.drag(mouse, top, menu);
        }
        self.stone_drag(mouse);
        self.loom_drag(mouse);
        let slot = self.hovered_slot(mouse, menu);
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
        if let Some(panel) = &mut self.merchant {
            panel.release();
        }
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
        let slot = self.hovered_slot(mouse, menu);
        let outside = self.outside(mouse);
        let clicks = self.mouse.released(
            slot,
            outside,
            button,
            shift,
            button == 2 && menu.creative(),
            menu,
        );
        self.actions(clicks, menu)
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
            return KeyResult::Consumed(Vec::new());
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
        KeyResult::Actions(self.actions(clicks, menu))
    }

    fn key_released(&mut self) {
        if let Some(book) = &mut self.book {
            book.key_released();
        }
    }

    fn mouse_scrolled(&mut self, y: f64, menu: &ContainerMenu) {
        if self.mouse.bundle.scrolled(y, self.hovered, menu) {
            return;
        }
        if let Some(panel) = &mut self.merchant {
            panel.wheel(y);
        }
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
        let mut actions = self.mouse.bundle.drain();
        actions.extend(self.take_queued());
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
        if let MenuKind::Mount(mount) = self.kind {
            self.mount_background(draw, menu, mount);
        }
        if let Some(panel) = &self.merchant {
            panel.background(draw, menu, (left, top));
        }
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
        if let Some(panel) = &self.merchant {
            panel.buttons(draw, (left, top), mouse);
        }
        self.draw_labels(draw, menu, mouse, (left, top), (width, height));
        let previous = self.hovered;
        self.hovered = self.hovered_slot(mouse, menu);
        self.mouse
            .bundle
            .hover_changed(previous, self.hovered, menu);
        let kind = self.kind;
        if kind == MenuKind::Smithing {
            self.extras.draw_smithing_icons(draw, menu, (left, top));
        }
        if let Some(slot) = self.hovered {
            self.layout
                .highlight(draw, slot_position(kind, slot), false);
        }
        self.draw_slots(draw, menu, (left, top));
        if let Some(slot) = self.hovered {
            self.layout.highlight(draw, slot_position(kind, slot), true);
        }
        if let Some(panel) = &mut self.merchant {
            let deprecated = self.titles.lang.text("merchant.deprecated");
            if let Some(lines) = panel.contents(draw, menu, (left, top), mouse, &deprecated) {
                self.extras.pending_tooltip = Some(lines);
            }
        }
        if let Some(book) = &mut self.book {
            book.draw(draw, menu, mouse, left);
            book.draw_ghost(draw, menu, (left, top), &|slot| slot_position(kind, slot));
        }
        self.mouse.draw_carried(draw, menu.carried.as_ref(), mouse);
        self.draw_crafter_redstone(draw, menu);
        if let Some(lines) = self.extras.pending_tooltip.take()
            && draw.widget_tooltips
        {
            self.layout.tooltip(draw, &lines, mouse);
        }
        let hovered = self
            .hovered
            .and_then(|slot| menu.slots.get(slot))
            .and_then(Option::as_ref);
        let shown = hovered.is_some_and(|stack| {
            super::bundle_tooltip::item_tooltip(
                &self.layout,
                draw,
                stack,
                menu.carried.is_some(),
                mouse,
            )
        });
        if !shown && let Some(book) = &self.book {
            book.ghost_tooltip(draw, menu, self.hovered, mouse);
        }
        self.last_mouse = mouse;
    }
}

mod book;

mod draw;

#[cfg(test)]
mod tests {
    use super::*;
    use photon_world::ClientItemStackState as Stack;

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
        menu.slots[30] = Some(Stack::new(5, photon_core::ids::ItemId(1)));
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

    #[test]
    fn a_click_on_an_empty_crafter_slot_switches_it_before_the_click_is_made() {
        let kind = MenuKind::Crafter;
        let mut menu = ContainerMenu::for_menu(kind, 4, Vec::new(), None, 0);
        menu.data = vec![0, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let click = |slot, kind| SlotClick {
            slot,
            button: 0,
            kind,
        };
        let toggle = |menu: &ContainerMenu, slot, kind| {
            GenericScreen::crafter_toggle(MenuKind::Crafter, &click(slot, kind), menu)
        };
        // An empty open slot is switched off, a switched-off one on.
        assert_eq!(
            toggle(&menu, 0, ClickType::Pickup),
            Some(ScreenAction::SlotState {
                slot: 0,
                enabled: false
            })
        );
        assert_eq!(
            toggle(&menu, 1, ClickType::Pickup),
            Some(ScreenAction::SlotState {
                slot: 1,
                enabled: true
            })
        );
        // Not while something is carried, nor over an item, nor on the player's slots.
        menu.carried = Some(Stack::new(1, photon_core::ids::ItemId(1)));
        assert_eq!(toggle(&menu, 0, ClickType::Pickup), None);
        menu.carried = None;
        menu.slots[0] = Some(Stack::new(1, photon_core::ids::ItemId(1)));
        assert_eq!(toggle(&menu, 0, ClickType::Pickup), None);
        assert_eq!(toggle(&menu, 12, ClickType::Pickup), None);
        // A number key switches a switched-off slot on when the hotbar slot holds something.
        assert_eq!(toggle(&menu, 1, ClickType::Swap), None);
        menu.slots[36] = Some(Stack::new(1, photon_core::ids::ItemId(1)));
        assert_eq!(
            GenericScreen::crafter_toggle(
                MenuKind::Crafter,
                &SlotClick {
                    slot: 1,
                    button: 0,
                    kind: ClickType::Swap
                },
                &menu
            ),
            Some(ScreenAction::SlotState {
                slot: 1,
                enabled: true
            })
        );
    }
}
