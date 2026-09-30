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
    name: EditBox,
    focused_at: Instant,
    filled_for: Option<Option<(i32, i32)>>,
    last_sent: String,
    queued: Vec<ScreenAction>,
}

impl Extras {
    pub(super) fn new() -> Self {
        Self {
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
