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
}

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
        // Any change of the input slot, its count too, starts the list over
        // (`containerChanged` runs on each change of the input container).
        let input = key_of(menu, 0);
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
            self.extras.pending_tooltip = Some(lines);
        }
    }

    /// What an entry shows, as a stack of one.
    fn option_stack(item: i32) -> Option<voxel_world::ClientItemStackState> {
        (item >= 0).then(|| voxel_world::ClientItemStackState::new(1, item))
    }

    /// Whether the loom shows its patterns: a banner and a dye, room for
    /// another layer and something to offer.
    fn loom_display(menu: &ContainerMenu) -> bool {
        let has = |slot: usize| menu.slots.get(slot).is_some_and(Option::is_some);
        has(0) && has(1) && menu.banner_layers < 6 && !menu.patterns.is_empty()
    }

    /// The rows past the four shown.
    fn loom_rows(&self) -> i32 {
        ((self.extras.loom_count as i32 + 3) / 4 - 4).max(0)
    }

    /// `LoomScreen.mouseClicked`: a pattern pressed (its index), or the scroll
    /// bar grabbed.
    pub(super) fn loom_click(&mut self, mouse: (f64, f64), menu: &ContainerMenu) -> Option<i32> {
        if self.kind != MenuKind::Loom {
            return None;
        }
        self.extras.loom_scrolling = false;
        if !Self::loom_display(menu) {
            return None;
        }
        let (mx, my) = (
            mouse.0 - f64::from(self.layout.left()),
            mouse.1 - f64::from(self.layout.top()),
        );
        let first = self.extras.loom_row * 4;
        let shown = first..(first + 16).min(menu.patterns.len());
        for index in shown {
            let place = (index - first) as i32;
            let (x, y) = (60 + place % 4 * 14, 13 + place / 4 * 14);
            if (0.0..14.0).contains(&(mx - f64::from(x)))
                && (0.0..14.0).contains(&(my - f64::from(y)))
            {
                return Some(index as i32);
            }
        }
        if (119.0..131.0).contains(&mx) && (9.0..65.0).contains(&my) {
            self.extras.loom_scrolling = true;
        }
        None
    }

    fn loom_scroll_to(&mut self, offs: f32) {
        self.extras.loom_offs = offs;
        let rows = self.loom_rows() as f32;
        self.extras.loom_row = (offs * rows + 0.5) as usize;
    }

    /// `LoomScreen.mouseDragged`.
    pub(super) fn loom_drag(&mut self, mouse: (f64, f64)) {
        if self.kind != MenuKind::Loom || !self.extras.loom_scrolling || self.loom_rows() <= 0 {
            return;
        }
        let top = f64::from(self.layout.top() + 13);
        let offs = ((mouse.1 - top - 7.5) / (56.0 - 15.0)).clamp(0.0, 1.0) as f32;
        self.loom_scroll_to(offs);
    }

    /// `LoomScreen.mouseScrolled`.
    pub(super) fn loom_wheel(&mut self, y: f64) {
        let rows = self.loom_rows();
        if self.kind == MenuKind::Loom && rows > 0 {
            let offs = (self.extras.loom_offs - (y / f64::from(rows)) as f32).clamp(0.0, 1.0);
            self.loom_scroll_to(offs);
        }
    }

    /// `LoomScreen.mouseReleased`.
    pub(super) fn loom_release(&mut self) {
        self.extras.loom_scrolling = false;
    }

    /// `LoomScreen.extractBackground`'s scroll bar, pattern buttons and mark.
    fn loom_draw(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
    ) {
        self.extras.loom_count = menu.patterns.len();
        if self.extras.loom_row * 4 >= menu.patterns.len().max(1) {
            self.extras.loom_row = 0;
            self.extras.loom_offs = 0.0;
        }
        if menu.banner_layers >= 6 {
            draw.frame.sprite(
                draw.atlas,
                "container/loom/error",
                left + 143 - 5,
                top + 57 - 5,
                26,
                26,
            );
        }
        let display = Self::loom_display(menu);
        let scroller = if display && menu.patterns.len() > 16 {
            "container/loom/scroller"
        } else {
            "container/loom/scroller_disabled"
        };
        let travel = (41.0 * self.extras.loom_offs) as i32;
        draw.frame
            .sprite(draw.atlas, scroller, left + 119, top + 13 + travel, 12, 15);
        if !display {
            return;
        }
        let selected = if menu.patterns.len() == 1 {
            0
        } else {
            i32::from(menu.data.first().copied().unwrap_or(-1))
        };
        let dye = menu
            .slots
            .get(1)
            .and_then(Option::as_ref)
            .and_then(|stack| {
                voxel_data::item::registry::ITEMS
                    .get(usize::try_from(stack.item_registry_id()).ok()?)
            })
            .and_then(|item| item.name.strip_prefix("minecraft:")?.strip_suffix("_dye"))
            .unwrap_or("white");
        let first = self.extras.loom_row * 4;
        let (mx, my) = (mouse.0 - f64::from(left), mouse.1 - f64::from(top));
        let mut tip = None;
        for index in first..(first + 16).min(menu.patterns.len()) {
            let place = (index - first) as i32;
            let (x, y) = (60 + place % 4 * 14, 13 + place / 4 * 14);
            let over = mx >= f64::from(x)
                && my >= f64::from(y)
                && mx < f64::from(x + 14)
                && my < f64::from(y + 14);
            let sprite = if index as i32 == selected {
                "container/loom/pattern_selected"
            } else if over {
                "container/loom/pattern_highlighted"
            } else {
                "container/loom/pattern"
            };
            draw.frame
                .sprite(draw.atlas, sprite, left + x, top + y, 14, 14);
            let pattern = usize::try_from(menu.patterns[index])
                .ok()
                .and_then(|id| self.titles.patterns.get(id));
            if let Some((asset, key)) = pattern {
                // The banner's texture from its top left: 21 by 40 of the 64
                // by 64, drawn small over grey.
                let (px, py) = (left + x + 4, top + y + 2);
                draw.frame
                    .fill(draw.atlas, (px, py, px + 5, py + 10), 0xFF47_4F52);
                draw.frame.blit_region(
                    draw.atlas,
                    GuiBlend::Translucent,
                    &format!("banner/{asset}"),
                    (64, 64),
                    (0, 1, 21, 40),
                    (px, py, 5, 10),
                );
                if over && index as i32 != selected {
                    tip = Some(self.titles.lang.text(&format!("{key}.{dye}")));
                }
            }
        }
        if let Some(text) = tip {
            let lines = vec![vec![voxel_game::text::Run {
                text,
                style: voxel_game::text::Style::default(),
            }]];
            self.extras.pending_tooltip = Some(lines);
        }
    }

    /// The enchanting table's three rows: whether the player can pay for each.
    fn enchant_can_afford(menu: &ContainerMenu, row: usize) -> bool {
        let cost = i32::from(menu.data.get(row).copied().unwrap_or(0));
        let lapis = menu
            .slots
            .get(1)
            .and_then(Option::as_ref)
            .map_or(0, voxel_world::ClientItemStackState::item_count);
        menu.creative()
            || (lapis > row as i32 && menu.xp_level > row as i32 && menu.xp_level >= cost)
    }

    /// `EnchantmentScreen.mouseClicked`: the row pressed, if the menu would
    /// take the press (`clickMenuButton`).
    pub(super) fn enchant_click(&self, mouse: (f64, f64), menu: &ContainerMenu) -> Option<i32> {
        if self.kind != MenuKind::Enchantment {
            return None;
        }
        let (mx, my) = (
            mouse.0 - f64::from(self.layout.left() + 60),
            mouse.1 - f64::from(self.layout.top()),
        );
        (0..3).find_map(|row| {
            let (xx, yy) = (mx, my - f64::from(14 + 19 * row as i32));
            let hit = (0.0..108.0).contains(&xx) && (0.0..19.0).contains(&yy);
            let cost = i32::from(menu.data.get(row).copied().unwrap_or(0));
            let has_item = menu.slots.first().is_some_and(Option::is_some);
            (hit && cost > 0 && has_item && Self::enchant_can_afford(menu, row))
                .then_some(row as i32)
        })
    }

    /// `EnchantmentScreen.extractBackground`'s three rows, and the tooltip of
    /// the hovered one (`extractRenderState`).
    #[expect(
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    fn enchant_draw(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
    ) {
        const WORDS: [&str; 62] = [
            "the",
            "elder",
            "scrolls",
            "klaatu",
            "berata",
            "niktu",
            "xyzzy",
            "bless",
            "curse",
            "light",
            "darkness",
            "fire",
            "air",
            "earth",
            "water",
            "hot",
            "dry",
            "cold",
            "wet",
            "ignite",
            "snuff",
            "embiggen",
            "twist",
            "shorten",
            "stretch",
            "fiddle",
            "destroy",
            "imbue",
            "galvanize",
            "enchant",
            "free",
            "limited",
            "range",
            "of",
            "towards",
            "inside",
            "sphere",
            "cube",
            "self",
            "other",
            "ball",
            "mental",
            "physical",
            "grow",
            "shrink",
            "demon",
            "elemental",
            "spirit",
            "animal",
            "creature",
            "beast",
            "humanoid",
            "undead",
            "fresh",
            "stale",
            "phnglui",
            "mglwnafh",
            "cthulhu",
            "rlyeh",
            "wgahnagl",
            "fhtagn",
            "baguette",
        ];
        let data = |index: usize| i32::from(menu.data.get(index).copied().unwrap_or(0));
        // `EnchantmentNames.initSeed`: the same names each frame.
        let mut random = voxel_core::random::RandomSource::create(i64::from(data(3)));
        let mut tip = None;
        for row in 0..3_usize {
            let x = left + 60;
            let y = top + 14 + 19 * row as i32;
            let cost = data(row);
            if cost == 0 {
                draw.frame.sprite(
                    draw.atlas,
                    "container/enchanting_table/enchantment_slot_disabled",
                    x,
                    y,
                    108,
                    19,
                );
                continue;
            }
            let cost_text = cost.to_string();
            let text_width = 86 - draw.font.width(&cost_text);
            // `EnchantmentNames.getRandomName`, cut to the width.
            let count = random.next_int_bound(2).unwrap_or(0) + 3;
            let name = (0..count)
                .map(|_| WORDS[random.next_int_bound(WORDS.len() as i32).unwrap_or(0) as usize])
                .collect::<Vec<_>>()
                .join(" ");
            let alt = self.titles.alt_font.clone();
            let mut shown = String::new();
            for character in name.chars() {
                shown.push(character);
                if alt.width(&shown) > text_width {
                    shown.pop();
                    break;
                }
            }
            let level = format!("container/enchanting_table/level_{}", row + 1);
            let affordable = Self::enchant_can_afford(menu, row);
            let (name_colour, cost_colour) = if !affordable {
                draw.frame.sprite(
                    draw.atlas,
                    "container/enchanting_table/enchantment_slot_disabled",
                    x,
                    y,
                    108,
                    19,
                );
                draw.frame.sprite(
                    draw.atlas,
                    &format!("{level}_disabled"),
                    x + 1,
                    y + 1,
                    16,
                    16,
                );
                (0xFF34_2F25, 0xFF40_7F10)
            } else {
                let (xx, yy) = (mouse.0 - f64::from(x), mouse.1 - f64::from(y));
                let over = (0.0..108.0).contains(&xx) && (0.0..19.0).contains(&yy);
                let (sprite, colour) = if over {
                    ("enchantment_slot_highlighted", 0xFFFF_FF80)
                } else {
                    ("enchantment_slot", 0xFF68_5E4A)
                };
                draw.frame.sprite(
                    draw.atlas,
                    &format!("container/enchanting_table/{sprite}"),
                    x,
                    y,
                    108,
                    19,
                );
                draw.frame.sprite(draw.atlas, &level, x + 1, y + 1, 16, 16);
                (colour, 0xFF80_FF20)
            };
            draw.frame
                .text(draw.atlas, &alt, &shown, x + 20, y + 2, name_colour);
            draw.frame.text_with_shadow(
                draw.atlas,
                draw.font,
                &cost_text,
                x + 20 + 86 - draw.font.width(&cost_text),
                y + 9,
                cost_colour,
            );
            // The tooltip of the row under the mouse.
            let (xx, yy) = (mouse.0 - f64::from(x), mouse.1 - f64::from(y));
            if tip.is_none() && (0.0..108.0).contains(&xx) && (0.0..17.0).contains(&yy) {
                tip = Some(row);
            }
        }
        if let Some(row) = tip {
            self.enchant_tooltip(menu, row);
        }
    }

    /// `EnchantmentScreen.extractRenderState`'s tooltip over a row: the clue,
    /// and what it costs.
    fn enchant_tooltip(&mut self, menu: &ContainerMenu, row: usize) {
        use voxel_game::text::{Component, Run, Style};
        let data = |index: usize| i32::from(menu.data.get(index).copied().unwrap_or(0));
        let cost = data(row);
        let clue = usize::try_from(data(4 + row)).ok();
        let level = data(7 + row);
        let Some(enchantment) = clue.and_then(|id| self.titles.enchantments.get(id)) else {
            return;
        };
        if cost <= 0 || level < 0 {
            return;
        }
        let lang = &self.titles.lang;
        // `Enchantment.getFullname`: the name in grey, red for a curse, and
        // the level unless the enchantment has one level only.
        let mut full = Component::literal(enchantment.name.clone())
            .with_colour(if enchantment.curse { "red" } else { "gray" });
        if level != 1 || enchantment.max_level != 1 {
            full = full
                .append(Component::literal(" "))
                .append(Component::translatable(format!(
                    "enchantment.level.{level}"
                )));
        }
        let clue_line =
            Component::translatable_with("container.enchant.clue", vec![full]).with_colour("white");
        let line = |text: String, colour: &str| {
            vec![Run {
                text,
                style: Style::coloured(colour),
            }]
        };
        let mut lines = vec![lang.runs(&clue_line, &Style::default())];
        if !menu.creative() {
            lines.push(Vec::new());
            let gold = menu
                .slots
                .get(1)
                .and_then(Option::as_ref)
                .map_or(0, voxel_world::ClientItemStackState::item_count);
            let row_cost = row as i32 + 1;
            if menu.xp_level < cost {
                lines.push(line(
                    lang.text("container.enchant.level.requirement")
                        .replace("%s", &cost.to_string()),
                    "red",
                ));
            } else {
                let lapis = if row_cost == 1 {
                    lang.text("container.enchant.lapis.one")
                } else {
                    lang.text("container.enchant.lapis.many")
                        .replace("%s", &row_cost.to_string())
                };
                lines.push(line(lapis, if gold >= row_cost { "gray" } else { "red" }));
                let levels = if row_cost == 1 {
                    lang.text("container.enchant.level.one")
                } else {
                    lang.text("container.enchant.level.many")
                        .replace("%s", &row_cost.to_string())
                };
                lines.push(line(levels, "gray"));
            }
        }
        self.extras.pending_tooltip = Some(lines);
    }

    /// `AnvilScreen.onNameChanged`: with an item in the slot, the box's text
    /// is the name to send, or none at all when it is what the item is
    /// called already; `AnvilMenu.setItemName` tells whether that is news.
    fn name_changed(&mut self) {
        let Some((hover, custom)) = &self.extras.named_item else {
            return;
        };
        let value = self.extras.name.value();
        let name = if !custom && value == *hover {
            String::new()
        } else {
            value
        };
        if name.chars().count() <= 50 && self.extras.item_name.as_ref() != Some(&name) {
            self.extras.item_name = Some(name.clone());
            self.extras.queued.push(ScreenAction::Rename(name));
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
        if self.kind == MenuKind::Anvil && self.extras.named_item.is_some() {
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
            MenuKind::Loom => self.loom_draw(draw, menu, mouse, (left, top)),
            MenuKind::Enchantment => self.enchant_draw(draw, menu, mouse, (left, top)),
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
            let item = menu.slots.first().and_then(Option::as_ref);
            let name = item
                .map(|stack| {
                    (draw.tooltip)(stack, false)
                        .first()
                        .map(|line| line.iter().map(|run| run.text.as_str()).collect::<String>())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            self.extras.named_item = item.map(|stack| {
                let custom = voxel_game::item_values::Values::new(
                    stack.item_registry_id(),
                    stack.patch_bytes(),
                )
                .has(voxel_data::item::component_types::CUSTOM_NAME);
                (name.clone(), custom)
            });
            let font = self.titles.font.clone();
            self.extras.name.set_value(&name, &font);
            self.extras.focused_at = Instant::now();
            // `setValue` tells the responder, which sends the name.
            self.name_changed();
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
                    self.titles
                        .repair_cost
                        .replace("%1$s", &cost.to_string())
                        .replace("%s", &cost.to_string()),
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
