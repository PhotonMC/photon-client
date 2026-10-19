//! The items tab (`ItemStatisticsList`): a header of six buttons that sort by a column, then a row
//! for each item with something counted, with its six numbers.

use std::collections::{BTreeSet, HashMap};

use super::list::{ListView, PADDING};
use crate::FrameRuns;
use crate::widgets::scroll_area::Place;
use photon_data::stat::{Formatter, StatKey};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame, ItemDrawer};

const ROW_HEIGHT: i32 = 22;
const BUTTON_SIZE: i32 = 18;
/// The stat type of each column: the blocks mined, then the items broken, crafted, used, picked up
/// and dropped.
const COLUMNS: [&str; 6] = [
    "minecraft:mined",
    "minecraft:broken",
    "minecraft:crafted",
    "minecraft:used",
    "minecraft:picked_up",
    "minecraft:dropped",
];
/// The sprite on each column's button.
const ICONS: [&str; 6] = [
    "statistics/block_mined",
    "statistics/item_broken",
    "statistics/item_crafted",
    "statistics/item_used",
    "statistics/item_picked_up",
    "statistics/item_dropped",
];

/// `getColumnX`: where the numbers of a column end, from the row's content.
fn column_x(column: usize) -> i32 {
    75 + 40 * column as i32
}

/// One item with something counted.
struct Row {
    /// The item's registry id.
    item: i32,
    /// The block a block item places, by its registry id: the one the "mined" column counts.
    block: Option<i32>,
}

pub(super) struct ItemsList {
    /// The header is the first row of the list, then the items.
    rows: Vec<Row>,
    view: ListView,
    types: [i32; 6],
    none: Vec<Run>,
    /// `sortColumn` and `sortOrder`: the column sorted by and which way (-1 down, 1 up).
    sort: Option<(usize, i32)>,
    /// The button the last press was on, which keeps the focus.
    focused: Option<usize>,
}

fn type_id(name: &str) -> i32 {
    photon_data::stat::type_names()
        .iter()
        .position(|candidate| *candidate == name)
        .unwrap_or(0) as i32
}

fn count(stats: &HashMap<StatKey, i32>, stat_type: i32, value: i32) -> i32 {
    stats
        .get(&StatKey { stat_type, value })
        .copied()
        .unwrap_or(0)
}

impl ItemsList {
    /// `ItemStatisticsList`: the items counted by any of the item columns, and the items of the
    /// blocks mined. The reference keeps them in a set that has no order; here they are by id.
    pub(super) fn new(lang: &Lang, stats: &HashMap<StatKey, i32>) -> Self {
        let types = COLUMNS.map(type_id);
        let air = photon_data::item::registry::index("minecraft:air");
        let mut items = BTreeSet::new();
        for (id, _) in photon_data::item::registry::ITEMS.iter().enumerate() {
            if types[1..]
                .iter()
                .any(|stat_type| count(stats, *stat_type, id as i32) > 0)
            {
                items.insert(id);
            }
        }
        for (id, block) in photon_data::block::blocks::BLOCKS.iter().enumerate() {
            if count(stats, types[0], id as i32) > 0
                && let Some(item) = photon_data::item::registry::index(block.item_name())
            {
                items.insert(item);
            }
        }
        items.remove(&air.unwrap_or(usize::MAX));
        let rows = items
            .into_iter()
            .map(|item| Row {
                item: item as i32,
                block: photon_data::item::registry::ITEMS[item]
                    .block
                    .and_then(photon_data::block::registry::block_index)
                    .map(|block| block as i32),
            })
            .collect();
        Self {
            rows,
            view: ListView::new(ROW_HEIGHT),
            types,
            none: lang.runs(&Component::translatable("stats.none"), &Style::default()),
            sort: None,
            focused: None,
        }
    }

    /// Whether any item is counted: the header is there only then.
    pub(super) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The list's rows: the header, and the items.
    fn len(&self) -> usize {
        self.rows.len() + usize::from(!self.rows.is_empty())
    }

    pub(super) fn place(&mut self, place: Place) {
        let rows = self.len();
        self.view.place(place, rows);
    }

    pub(super) fn scroll_by(&mut self, delta: f64) {
        let rows = self.len();
        self.view.scroll_by(delta, rows);
    }

    /// Where the sort button of a column is, for the header row at `y`.
    fn button_rect(&self, column: usize, y: i32) -> (i32, i32) {
        (
            self.view.row_left() + PADDING + column_x(column) - BUTTON_SIZE,
            y + PADDING + 1,
        )
    }

    /// The column whose button is at the mouse.
    fn button_at(&self, mouse: (i32, i32)) -> Option<usize> {
        if self.rows.is_empty() {
            return None;
        }
        let y = self.view.row_y(0);
        (0..6).find(|column| {
            let (x, top) = self.button_rect(*column, y);
            mouse.0 >= x
                && mouse.0 < x + BUTTON_SIZE
                && mouse.1 >= top
                && mouse.1 < top + BUTTON_SIZE
        })
    }

    /// A press on the header: the column's button sorts (`sortByColumn`): by it descending, then
    /// ascending, then not at all. The rows are sorted by the numbers held now.
    pub(super) fn clicked(&mut self, mouse: (i32, i32), stats: &HashMap<StatKey, i32>) {
        self.focused = self.button_at(mouse);
        let Some(column) = self.focused else {
            return;
        };
        self.sort = match self.sort {
            Some((held, -1)) if held == column => Some((column, 1)),
            Some((held, _)) if held == column => None,
            _ => Some((column, -1)),
        };
        let Some((column, order)) = self.sort else {
            return;
        };
        let types = self.types;
        let key = |row: &Row| -> i32 {
            if column == 0 {
                row.block.map_or(-1, |block| count(stats, types[0], block))
            } else {
                count(stats, types[column], row.item)
            }
        };
        self.rows.sort_by(|a, b| {
            let (one, two) = (key(a), key(b));
            let ordering = if one == two {
                a.item.cmp(&b.item)
            } else {
                one.cmp(&two)
            };
            if order < 0 {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    /// `ItemRow.extractContent`: the item in its slot, then the six numbers (a dash where an item
    /// is no block and so has none mined).
    fn draw_row(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        (row, y, shaded): (&Row, i32, bool),
        (stats, item): (&HashMap<StatKey, i32>, &mut ItemDrawer<'_>),
    ) {
        let x = self.view.row_left() + PADDING;
        let content_y = y + PADDING;
        let text_y = content_y + (ROW_HEIGHT - 2 * PADDING) / 2 - 9 / 2;
        frame.sprite(atlas, "container/slot", x, content_y, 18, 18);
        let stack = photon_world::ClientItemStackState::new(1, photon_core::ids::ItemId(row.item));
        frame.item(item, &stack, x + 1, content_y + 1);
        let colour = if shaded { 0xFFFF_FFFF } else { 0xFFBA_BABA };
        for column in 0..6 {
            let value = if column == 0 {
                row.block.map(|block| count(stats, self.types[0], block))
            } else {
                Some(count(stats, self.types[column], row.item))
            };
            let at = x + column_x(column);
            if let Some(number) = value {
                let text = photon_game::stats::format(Formatter::Default, number);
                let left = at - font.width(&text);
                frame.text_with_shadow(atlas, font, &text, left, text_y, colour);
            } else {
                let left = at - crate::FontRuns::width_of_runs(font, &self.none);
                frame.styled_text(atlas, font, &self.none, left, text_y, colour, true);
            }
        }
    }

    /// The header and the rows that show, then the scroll bar.
    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        (stats, item): (&HashMap<StatKey, i32>, &mut ItemDrawer<'_>),
        mouse: (i32, i32),
    ) {
        let view = &self.view;
        let mark = frame.mark();
        if !self.rows.is_empty() {
            self.draw_header(frame, atlas, mouse);
        }
        for (index, row) in self.rows.iter().enumerate() {
            let y = view.row_y(index + 1);
            if view.shows(y) {
                self.draw_row(
                    frame,
                    (atlas, font),
                    (row, y, (index + 1) % 2 == 0),
                    (stats, item),
                );
            }
        }
        frame.clip_from(
            mark,
            (0, view.place.y, view.place.width, view.place.bottom()),
        );
        view.draw_scrollbar(frame, atlas, self.len());
    }

    /// `HeaderEntry.extractContent`: the six buttons, and the arrow on the column sorted by.
    fn draw_header(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, mouse: (i32, i32)) {
        let y = self.view.row_y(0);
        if !self.view.shows(y) {
            return;
        }
        let hovered = self.button_at(mouse);
        for (column, icon) in ICONS.iter().enumerate() {
            let (x, top) = self.button_rect(column, y);
            let background = if hovered == Some(column) || self.focused == Some(column) {
                "container/slot"
            } else {
                "statistics/header"
            };
            frame.sprite(atlas, background, x, top, BUTTON_SIZE, BUTTON_SIZE);
            frame.sprite(atlas, icon, x, top, BUTTON_SIZE, BUTTON_SIZE);
        }
        if let Some((column, order)) = self.sort {
            let sprite = if order == 1 {
                "statistics/sort_up"
            } else {
                "statistics/sort_down"
            };
            let x = self.view.row_left() + PADDING + column_x(column) - 36;
            frame.sprite(atlas, sprite, x, y + PADDING + 1, BUTTON_SIZE, BUTTON_SIZE);
        }
    }
}
