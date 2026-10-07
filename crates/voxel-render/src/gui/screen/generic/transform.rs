//! What the screens of the menus that transform their items add to the
//! generic container screen: where their slots are, their textures, the
//! error marks and previews over the image, and the anvil's name box.
//!
//! Not drawn: the cartography table's map (the client has no map data), the
//! smithing table's armour stand and the enchanting table's book.

use std::time::Instant;

use super::{GenericScreen, GuiBlend, KeyResult, MenuKind, ScreenAction, ScreenDraw};
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
        MenuKind::Brewing => &[(56, 51), (79, 58), (102, 51), (79, 17), (17, 17)],
        MenuKind::Beacon => &[(136, 110)],
        MenuKind::Crafter => &[
            (26, 17),
            (44, 17),
            (62, 17),
            (26, 35),
            (44, 35),
            (62, 35),
            (26, 53),
            (44, 53),
            (62, 53),
        ],
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
        MenuKind::Brewing => "container/brewing_stand",
        MenuKind::Beacon => "container/beacon",
        MenuKind::Crafter => "container/crafter",
        MenuKind::Merchant => "container/villager",
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
        MenuKind::Crafting => (29, 6),
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
        (MenuKind::Brewing, 0..=2) => Some("container/slot/potion"),
        (MenuKind::Brewing, 4) => Some("container/slot/brewing_fuel"),
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
    stone_input: Option<(i32, i32)>,
    /// The loom's pattern list: its scroll position, first row shown, whether
    /// the bar is grabbed and how many patterns it offers.
    loom_offs: f32,
    loom_row: usize,
    loom_scrolling: bool,
    loom_count: usize,
    name: EditBox,
    focused_at: Instant,
    filled_for: Option<Option<(i32, i32)>>,
    /// `AnvilMenu.itemName`: the name the server was last sent, none yet.
    /// `setTooltipForNextFrame`: the tooltip a menu's own widgets asked for,
    /// which is drawn last.
    pub(super) pending_tooltip: Option<Vec<Vec<voxel_game::text::Run>>>,
    item_name: Option<String>,
    /// The item in the name slot: its hover name, and whether it has a
    /// custom name; none when the slot is empty (which also makes the box
    /// uneditable).
    named_item: Option<(String, bool)>,
    queued: Vec<ScreenAction>,
    /// The smithing table's three slot backgrounds that cycle their icons.
    cycling: [CyclingIcons; 3],
}

/// `CyclingSlotBackground`: the icons an empty slot shows in turn, a new one every 30 ticks and
/// faded in over the first four.
#[derive(Debug, Clone, Default)]
struct CyclingIcons {
    icons: &'static [&'static str],
    tick: i32,
    index: usize,
}

impl CyclingIcons {
    fn tick(&mut self, icons: &'static [&'static str]) {
        if self.icons != icons {
            self.icons = icons;
            self.index = 0;
        }
        if !self.icons.is_empty() {
            self.tick += 1;
            if self.tick % 30 == 0 {
                self.index = (self.index + 1) % self.icons.len();
            }
        }
    }

    fn draw(&self, draw: &mut ScreenDraw<'_, '_>, (x, y): (i32, i32)) {
        if self.icons.is_empty() {
            return;
        }
        let transition = self.icons.len() > 1 && self.tick >= 30;
        let alpha = if transition {
            ((self.tick % 30) as f32 + draw.partial).min(4.0) / 4.0
        } else {
            1.0
        };
        if alpha < 1.0 {
            let previous = (self.index + self.icons.len() - 1) % self.icons.len();
            self.icon(draw, self.icons[previous], 1.0 - alpha, (x, y));
        }
        self.icon(draw, self.icons[self.index], alpha, (x, y));
    }

    /// `blitSprite` in `ARGB.white(alpha)`.
    fn icon(&self, draw: &mut ScreenDraw<'_, '_>, name: &str, alpha: f32, (x, y): (i32, i32)) {
        let channel = (alpha * 255.0).floor() as u32;
        draw.frame.sprite_tinted(
            draw.atlas,
            name,
            (x, y, 16, 16),
            channel << 24 | 0x00FF_FFFF,
        );
    }
}

/// The icons the empty slots of the smithing table cycle through, by template (`SmithingTemplateItem`).
const TEMPLATE_ICONS: [&str; 2] = [
    "container/slot/smithing_template_armor_trim",
    "container/slot/smithing_template_netherite_upgrade",
];
const TRIM_BASE: [&str; 4] = [
    "container/slot/helmet",
    "container/slot/chestplate",
    "container/slot/leggings",
    "container/slot/boots",
];
const TRIM_ADDITION: [&str; 7] = [
    "container/slot/ingot",
    "container/slot/redstone_dust",
    "container/slot/lapis_lazuli",
    "container/slot/quartz",
    "container/slot/diamond",
    "container/slot/emerald",
    "container/slot/amethyst_shard",
];
const UPGRADE_BASE: [&str; 11] = [
    "container/slot/helmet",
    "container/slot/sword",
    "container/slot/chestplate",
    "container/slot/pickaxe",
    "container/slot/leggings",
    "container/slot/axe",
    "container/slot/boots",
    "container/slot/hoe",
    "container/slot/shovel",
    "container/slot/nautilus_armor",
    "container/slot/spear",
];
const UPGRADE_ADDITION: [&str; 1] = ["container/slot/ingot"];

impl Extras {
    pub(super) fn new() -> Self {
        Self {
            scroll_offs: 0.0,
            start_index: 0,
            scrolling: false,
            options_count: 0,
            stone_input: None,
            loom_offs: 0.0,
            loom_row: 0,
            loom_scrolling: false,
            loom_count: 0,
            // `new EditBox(font, x + 62, y + 24, 103, 12, ...)`, unbordered.
            name: EditBox::new(103, 50).with_colour(0xFFFF_FFFF),
            focused_at: Instant::now(),
            filled_for: None,
            pending_tooltip: None,
            item_name: None,
            named_item: None,
            queued: Vec::new(),
            cycling: Default::default(),
        }
    }

    /// `SmithingScreen.containerTick`: the icons of the three slots, for the template in the first.
    pub(super) fn tick_smithing(&mut self, menu: &ContainerMenu) {
        let template =
            menu.slots.first().and_then(Option::as_ref).map(|stack| {
                voxel_data::item::registry::ITEMS[stack.item_registry_id() as usize].name
            });
        let (base, addition): (&'static [&'static str], &'static [&'static str]) = match template {
            Some("minecraft:netherite_upgrade_smithing_template") => {
                (&UPGRADE_BASE, &UPGRADE_ADDITION)
            }
            Some(name) if name.ends_with("_armor_trim_smithing_template") => {
                (&TRIM_BASE, &TRIM_ADDITION)
            }
            _ => (&[], &[]),
        };
        self.cycling[0].tick(&TEMPLATE_ICONS);
        self.cycling[1].tick(base);
        self.cycling[2].tick(addition);
    }

    /// The icons of the empty smithing slots, behind everything else the screen draws.
    pub(super) fn draw_smithing_icons(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
    ) {
        for (index, icons) in self.cycling.iter().enumerate() {
            let empty = menu.slots.get(index).is_none_or(Option::is_none);
            if let (true, Some((x, y))) = (empty, slot_position(MenuKind::Smithing, index)) {
                icons.draw(draw, (left + x, top + y));
            }
        }
    }
}

/// `BrewingStandScreen.extractBackground`: the fuel bar, the arrow down the brew and its bubbles,
/// from the brewing time and the fuel the server sent.
fn brewing_progress(
    draw: &mut ScreenDraw<'_, '_>,
    (ticks, fuel): (i32, i32),
    (left, top): (i32, i32),
) {
    const BUBBLE_LENGTHS: [i32; 7] = [29, 24, 20, 16, 11, 6, 0];
    let fuel_length = ((18 * fuel + 20 - 1) / 20).clamp(0, 18);
    if fuel_length > 0 {
        draw.frame.sprite_cut(
            draw.atlas,
            "container/brewing_stand/fuel_length",
            (18, 4),
            (0, 0),
            left + 60,
            top + 44,
            fuel_length,
            4,
        );
    }
    if ticks > 0 {
        let length = (28.0_f32 * (1.0_f32 - ticks as f32 / 400.0_f32)) as i32;
        if length > 0 {
            draw.frame.sprite_cut(
                draw.atlas,
                "container/brewing_stand/brew_progress",
                (9, 28),
                (0, 0),
                left + 97,
                top + 16,
                9,
                length,
            );
        }
        let length = BUBBLE_LENGTHS[(ticks / 2 % 7) as usize];
        if length > 0 {
            draw.frame.sprite_cut(
                draw.atlas,
                "container/brewing_stand/bubbles",
                (12, 29),
                (0, 29 - length),
                left + 63,
                top + 14 + 29 - length,
                12,
                length,
            );
        }
    }
}

mod generic_screen_anvil;
mod generic_screen_name;
mod generic_screen_stone;
