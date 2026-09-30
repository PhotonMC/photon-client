//! What the screens of the menus that transform their items add to the
//! generic container screen: where their slots are, their textures, the
//! error marks and previews over the image, and the anvil's name box.
//!
//! Not drawn: the cartography table's map (the client has no map data), the
//! smithing table's armour stand and the enchanting table's book.

use std::time::Instant;

use super::{GenericScreen, KeyResult, MenuKind, ScreenAction, ScreenDraw};
use crate::gui::edit_box::{Clipboard, EditBox, edit_key};
use voxel_game::container_menu::ContainerMenu;

/// Where a slot sits in the image, for the menus this module knows; `None`
/// for the rest.
pub(super) fn slot_position(kind: MenuKind, index: usize) -> Option<(i32, i32)> {
    let top = kind.top_slots();
    if index >= top {
        return None;
    }
    let slots: &[(i32, i32)] = match kind {
        MenuKind::Stonecutter => &[(20, 33), (143, 33)],
        MenuKind::Grindstone => &[(49, 19), (49, 40), (129, 34)],
        MenuKind::Cartography => &[(15, 15), (15, 52), (145, 39)],
        MenuKind::Loom => &[(13, 26), (33, 26), (23, 45), (143, 57)],
        MenuKind::Smithing => &[(8, 48), (26, 48), (44, 48), (98, 48)],
        MenuKind::Anvil => &[(27, 47), (76, 47), (134, 47)],
        MenuKind::Enchantment => &[(15, 47), (35, 47)],
        _ => return None,
    };
    Some(slots[index])
}

/// The screen's background texture.
pub(super) fn texture(kind: MenuKind) -> Option<&'static str> {
    Some(match kind {
        MenuKind::Stonecutter => "container/stonecutter",
        MenuKind::Grindstone => "container/grindstone",
        MenuKind::Cartography => "container/cartography_table",
        MenuKind::Loom => "container/loom",
        MenuKind::Smithing => "container/smithing",
        MenuKind::Anvil => "container/anvil",
        MenuKind::Enchantment => "container/enchanting_table",
        _ => return None,
    })
}

/// The corner of the title label, `(titleLabelX, titleLabelY)`; the x is
/// `None` where the title is centred.
pub(super) fn title_label(kind: MenuKind) -> (i32, i32) {
    match kind {
        MenuKind::Stonecutter => (8, 5),
        MenuKind::Cartography | MenuKind::Loom => (8, 4),
        MenuKind::Smithing => (44, 15),
        MenuKind::Anvil => (60, 6),
        _ => (8, 6),
    }
}

/// The icon an empty slot shows.
pub(super) fn empty_icon(kind: MenuKind, index: usize) -> Option<&'static str> {
    match (kind, index) {
        (MenuKind::Loom, 0) => Some("container/slot/banner"),
        (MenuKind::Loom, 1) => Some("container/slot/dye"),
        (MenuKind::Loom, 2) => Some("container/slot/banner_pattern"),
        (MenuKind::Enchantment, 1) => Some("container/slot/lapis_lazuli"),
        // The template slot cycles two icons; the first is shown.
        (MenuKind::Smithing, 0) => Some("container/slot/smithing_template_armor_trim"),
        _ => None,
    }
}

/// Which item a slot holds, to see it change.
fn key_of(menu: &ContainerMenu, slot: usize) -> Option<(i32, i32)> {
    menu.slots
        .get(slot)
        .and_then(Option::as_ref)
        .map(|stack| (stack.item_registry_id(), stack.item_count()))
}

/// The anvil's state: the name box, what it last told the server, and the
/// item it was last filled for.
#[derive(Debug, Clone)]
pub(super) struct Extras {
    /// The stonecutter's scroll position, from 0 to 1, and the first entry
    /// shown (a multiple of four).
    scroll_offs: f32,
    start_index: usize,
    scrolling: bool,
    options_count: usize,
    stone_input: Option<i32>,
    name: EditBox,
    focused_at: Instant,
    filled_for: Option<Option<(i32, i32)>>,
    last_sent: String,
    queued: Vec<ScreenAction>,
}

impl Extras {
    pub(super) fn new() -> Self {
        Self {
            scroll_offs: 0.0,
            start_index: 0,
            scrolling: false,
            options_count: 0,
            stone_input: None,
            // `new EditBox(font, x + 62, y + 24, 103, 12, ...)`, unbordered.
            name: EditBox::new(103, 50),
            focused_at: Instant::now(),
            filled_for: None,
            last_sent: String::new(),
            queued: Vec::new(),
        }
    }
}

impl GenericScreen {
    /// The clicks the name box asked for since the last call.
    pub(super) fn take_queued(&mut self) -> Vec<ScreenAction> {
        std::mem::take(&mut self.extras.queued)
    }

    /// Whether the stonecutter shows its recipes: an input and entries for it.
    fn stone_display(menu: &ContainerMenu) -> bool {
        menu.slots.first().is_some_and(Option::is_some) && !menu.options.is_empty()
    }

    /// The stonecutter's entries shown: twelve from the first one.
    fn stone_shown(&self) -> std::ops::Range<usize> {
        self.extras.start_index..(self.extras.start_index + 12).min(self.extras.options_count)
    }

    /// Where an entry is drawn, as the cell's corner.
    fn stone_cell(&self, index: usize) -> (i32, i32) {
        let place = (index - self.extras.start_index) as i32;
        (52 + place % 4 * 16, 14 + place / 4 * 18 + 2)
    }

    /// `StonecutterScreen.mouseClicked`: a recipe pressed, or the scroll bar
    /// grabbed. The button pressed, if a recipe was.
    pub(super) fn stone_click(&mut self, mouse: (f64, f64), menu: &ContainerMenu) -> Option<i32> {
        if self.kind != MenuKind::Stonecutter {
            return None;
        }
        self.extras.scrolling = false;
        if !Self::stone_display(menu) {
            return None;
        }
        let (mx, my) = (
            mouse.0 - f64::from(self.layout.left()),
            mouse.1 - f64::from(self.layout.top()),
        );
        for index in self.stone_shown() {
            let (x, y) = self.stone_cell(index);
            let (xx, yy) = (mx - f64::from(x), my - f64::from(y - 2));
            if (0.0..16.0).contains(&xx) && (0.0..18.0).contains(&yy) {
                return Some(index as i32);
            }
        }
        if (119.0..131.0).contains(&mx) && (9.0..63.0).contains(&my) {
            self.extras.scrolling = true;
        }
        None
    }

    /// `StonecutterScreen.mouseDragged`.
    pub(super) fn stone_drag(&mut self, mouse: (f64, f64)) {
        if self.kind != MenuKind::Stonecutter || !self.extras.scrolling || self.stone_rows() <= 0 {
            return;
        }
        let top = f64::from(self.layout.top() + 14);
        let scroll = ((mouse.1 - top - 7.5) / (54.0 - 15.0)).clamp(0.0, 1.0) as f32;
        self.stone_scroll_to(scroll);
    }

    /// `StonecutterScreen.mouseScrolled`.
    pub(super) fn stone_wheel(&mut self, y: f64) {
        let rows = self.stone_rows();
        if self.kind == MenuKind::Stonecutter && rows > 0 {
            let scroll = (self.extras.scroll_offs - (y / f64::from(rows)) as f32).clamp(0.0, 1.0);
            self.stone_scroll_to(scroll);
        }
    }

    /// `StonecutterScreen.mouseReleased`.
    pub(super) fn stone_release(&mut self) {
        self.extras.scrolling = false;
    }

    /// The rows past the three shown.
    fn stone_rows(&self) -> i32 {
        if self.extras.options_count > 12 {
            (self.extras.options_count as i32 + 3) / 4 - 3
        } else {
            0
        }
    }

    fn stone_scroll_to(&mut self, scroll: f32) {
        self.extras.scroll_offs = scroll;
        let rows = self.stone_rows() as f32;
        self.extras.start_index = ((scroll * rows + 0.5) as usize) * 4;
    }

    /// `StonecutterScreen.extractBackground`'s recipe list and scroll bar.
    fn stone_draw(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
    ) {
        // A new input starts the list over.
        let input = menu
            .slots
            .first()
            .and_then(Option::as_ref)
            .map(|stack| stack.item_registry_id());
        if input != self.extras.stone_input {
            self.extras.stone_input = input;
            self.extras.scroll_offs = 0.0;
            self.extras.start_index = 0;
        }
        self.extras.options_count = menu.options.len();
        if !Self::stone_display(menu) {
            draw.frame.sprite(
                draw.atlas,
                "container/stonecutter/scroller_disabled",
                left + 119,
                top + 15,
                12,
                15,
            );
            return;
        }
        let active = menu.options.len() > 12;
        let scroller = if active {
            "container/stonecutter/scroller"
        } else {
            "container/stonecutter/scroller_disabled"
        };
        let travel = (41.0 * self.extras.scroll_offs) as i32;
        draw.frame
            .sprite(draw.atlas, scroller, left + 119, top + 15 + travel, 12, 15);
        let selected = i32::from(menu.data.first().copied().unwrap_or(-1));
        let (mx, my) = (mouse.0 - f64::from(left), mouse.1 - f64::from(top));
        let mut hovered = None;
        for index in self.stone_shown() {
            let (x, y) = self.stone_cell(index);
            let over = mx >= f64::from(x)
                && my >= f64::from(y)
                && mx < f64::from(x + 16)
                && my < f64::from(y + 18);
            let sprite = if index as i32 == selected {
                "container/stonecutter/recipe_selected"
            } else if over {
                "container/stonecutter/recipe_highlighted"
            } else {
                "container/stonecutter/recipe"
            };
            draw.frame
                .sprite(draw.atlas, sprite, left + x, top + y - 1, 16, 18);
            if let Some(stack) = Self::option_stack(menu.options[index]) {
                draw.frame.item(draw.item, &stack, left + x, top + y);
                if over {
                    hovered = Some(stack);
                }
            }
        }
        if let Some(stack) = hovered {
            let lines = (draw.tooltip)(&stack, false);
            self.layout.tooltip(draw, &lines, mouse);
        }
    }

    /// What an entry shows, as a stack of one.
    fn option_stack(item: i32) -> Option<voxel_world::ClientItemStackState> {
        (item >= 0).then(|| voxel_world::ClientItemStackState::new(1, item))
    }

    /// `AnvilScreen.onNameChanged`: a name that differs from what the server
    /// has is sent.
    fn name_changed(&mut self) {
        let value = self.extras.name.value();
        if value != self.extras.last_sent && value.chars().count() <= 50 {
            self.extras.last_sent.clone_from(&value);
            self.extras.queued.push(ScreenAction::Rename(value));
        }
    }

    /// A key for the anvil's name box, when it can take input.
    pub(super) fn name_key(
        &mut self,
        key: glfw::Key,
        control: bool,
        shift: bool,
        menu: &ContainerMenu,
        clipboard: &mut dyn Clipboard,
    ) -> Option<KeyResult> {
        if self.kind != MenuKind::Anvil || menu.slots.first().is_none_or(Option::is_none) {
            return None;
        }
        if key == glfw::Key::Escape {
            return Some(KeyResult::Close);
        }
        if let Some(edit) = edit_key(key) {
            let font = self.titles.font.clone();
            self.extras
                .name
                .key_pressed(edit, control, shift, &font, clipboard);
            self.name_changed();
        }
        // The box takes every key while it can, so the inventory key is typed.
        Some(KeyResult::Actions(Vec::new()))
    }

    /// A character typed into the anvil's name box.
    pub(super) fn name_char(&mut self, character: char) {
        if self.kind == MenuKind::Anvil {
            let font = self.titles.font.clone();
            if self.extras.name.char_typed(character, &font) {
                self.name_changed();
            }
        }
    }

    /// The marks, previews and boxes of the menu over its image.
    pub(super) fn draw_extras(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
    ) {
        let has = |slot: usize| menu.slots.get(slot).is_some_and(Option::is_some);
        let data = |index: usize| i32::from(menu.data.get(index).copied().unwrap_or(0));
        match self.kind {
            MenuKind::Grindstone => {
                if (has(0) || has(1)) && !has(2) {
                    draw.frame.sprite(
                        draw.atlas,
                        "container/grindstone/error",
                        left + 92,
                        top + 31,
                        28,
                        21,
                    );
                }
            }
            MenuKind::Smithing => {
                if data(0) > 0 {
                    draw.frame.sprite(
                        draw.atlas,
                        "container/smithing/error",
                        left + 65,
                        top + 46,
                        28,
                        21,
                    );
                }
            }
            MenuKind::Cartography => self.cartography_preview(draw, menu, (left, top)),
            MenuKind::Anvil => self.anvil_extras(draw, menu, (left, top)),
            MenuKind::Stonecutter => self.stone_draw(draw, menu, mouse, (left, top)),
            _ => {}
        }
    }

    /// `CartographyTableScreen.extractBackground` without the map drawn.
    fn cartography_preview(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
    ) {
        let item = |slot: usize| {
            menu.slots
                .get(slot)
                .and_then(Option::as_ref)
                .and_then(|stack| {
                    voxel_data::item::registry::ITEMS
                        .get(usize::try_from(stack.item_registry_id()).ok()?)
                })
                .map_or("", |item| item.name)
        };
        let extra = item(1);
        let sprite = |draw: &mut ScreenDraw<'_, '_>, name: &str, x: i32, y: i32, w: i32, h: i32| {
            draw.frame.sprite(
                draw.atlas,
                &format!("container/cartography_table/{name}"),
                left + x,
                top + y,
                w,
                h,
            );
        };
        if extra == "minecraft:paper" {
            sprite(draw, "scaled_map", 67, 13, 66, 66);
        } else if extra == "minecraft:map" {
            sprite(draw, "duplicated_map", 83, 13, 50, 66);
            sprite(draw, "duplicated_map", 67, 29, 50, 66);
        } else if extra == "minecraft:glass_pane" {
            sprite(draw, "map", 67, 13, 66, 66);
            sprite(draw, "locked", 118, 60, 10, 14);
        } else {
            sprite(draw, "map", 67, 13, 66, 66);
        }
    }

    /// The anvil's name box, its error mark and what the repair costs.
    fn anvil_extras(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
    ) {
        let has = |slot: usize| menu.slots.get(slot).is_some_and(Option::is_some);
        let editable = has(0);
        // The box is filled with the item's name when the item changes.
        let key = key_of(menu, 0);
        if self.extras.filled_for != Some(key) {
            self.extras.filled_for = Some(key);
            let name = menu
                .slots
                .first()
                .and_then(Option::as_ref)
                .map(|stack| {
                    (draw.tooltip)(stack, false)
                        .first()
                        .map(|line| line.iter().map(|run| run.text.as_str()).collect::<String>())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let font = self.titles.font.clone();
            self.extras.name.set_value(&name, &font);
            self.extras.last_sent.clone_from(&name);
            self.extras.focused_at = Instant::now();
        }
        let field = if editable {
            "container/anvil/text_field"
        } else {
            "container/anvil/text_field_disabled"
        };
        draw.frame
            .sprite(draw.atlas, field, left + 59, top + 20, 110, 16);
        if (has(0) || has(1)) && !has(2) {
            draw.frame.sprite(
                draw.atlas,
                "container/anvil/error",
                left + 99,
                top + 45,
                28,
                21,
            );
        }
        if editable {
            self.extras.name.draw(
                draw.frame,
                draw.atlas,
                draw.font,
                (left + 62, top + 24),
                self.extras.focused_at,
            );
        }
        // The cost of taking the result.
        let cost = i32::from(menu.data.first().copied().unwrap_or(0));
        if cost > 0 {
            let (text, colour) = if cost >= 40 && !menu.creative() {
                (self.titles.repair_expensive.clone(), 0xFFFF_6060)
            } else if has(2) {
                let colour = if menu.can_pick_up_result() {
                    0xFF80_FF20
                } else {
                    0xFFFF_6060
                };
                (
                    self.titles.repair_cost.replace("%s", &cost.to_string()),
                    colour,
                )
            } else {
                return;
            };
            let x = 176 - 8 - draw.font.width(&text) - 2;
            draw.frame.fill(
                draw.atlas,
                (left + x - 2, top + 67, left + 176 - 8, top + 79),
                0x4F00_0000,
            );
            draw.frame
                .text_with_shadow(draw.atlas, draw.font, &text, left + x, top + 69, colour);
        }
    }
}
