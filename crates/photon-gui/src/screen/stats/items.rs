//! The items tab (`ItemStatisticsList`): a header of six buttons that sort by a column, then a row
//! for each item with something counted, with its six numbers.

use std::collections::{BTreeSet, HashMap};

use super::list::{ListView, PADDING};
use super::navigation::Arrow;
use crate::FrameRuns;
use crate::tooltip::{Holder, Pending};
use crate::widgets::navigation::Rectangle;
use crate::widgets::scroll_area::Place;
use crate::widgets::slider::InputType;
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
    /// What the last press was on, which keeps the focus.
    focus: Focus,
    /// The tooltip of each sort button: the name of what it counts.
    tips: Vec<Holder>,
}

/// The widget of the list that has the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    None,
    /// The sort button of a column in the header.
    Button(usize),
    /// The item's slot in a row (by its place among the items).
    Slot(usize),
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
            focus: Focus::None,
            tips: COLUMNS
                .iter()
                .map(|name| {
                    let key = format!("stat_type.{}", name.replace(':', "."));
                    Holder::of(lang, &Component::translatable(key))
                })
                .collect(),
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

    /// The column whose button is at the mouse. A button is hovered only inside the list (the
    /// scissor), but pressed wherever it is.
    fn button_at(&self, mouse: (i32, i32), hover: bool) -> Option<usize> {
        let place = self.view.place;
        if self.rows.is_empty() || hover && (mouse.1 < place.y || mouse.1 >= place.bottom()) {
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

    /// Enter or Space on the widget with the focus: a sort button sounds and sorts (as a press
    /// does); a slot does nothing.
    pub(super) fn press_focused(&mut self, stats: &HashMap<StatKey, i32>) {
        if let (true, Focus::Button(column)) = (self.view.focused, self.focus) {
            crate::ui_sound::play_down_sound();
            self.sort_by(column, stats);
        }
    }

    /// `AbstractWidget.extractTooltipForNextRenderPass` of the sort buttons: the tooltip a button
    /// under the pointer, or focused by the keyboard, asks for once it has waited.
    pub(super) fn tooltip(
        &self,
        mouse: (i32, i32),
        millis: u64,
        input: InputType,
    ) -> Option<Pending> {
        let y = self.view.row_y(0);
        let shown = !self.rows.is_empty() && self.view.shows(y);
        let hovered = if shown {
            self.button_at(mouse, true)
        } else {
            None
        };
        let mut asked = None;
        for (column, tip) in self.tips.iter().enumerate() {
            let (x, top) = self.button_rect(column, y);
            let rectangle = Rectangle {
                x,
                y: top,
                width: BUTTON_SIZE,
                height: BUTTON_SIZE,
            };
            let focused = shown && self.view.focused && self.focus == Focus::Button(column);
            let state = (hovered == Some(column), focused);
            if let Some(pending) = tip.refresh(millis, state, input, rectangle) {
                asked = Some(pending);
            }
        }
        asked
    }

    /// Whether the list has the focus inside it.
    pub(super) fn is_focused(&self) -> bool {
        self.view.focused
    }

    /// An arrow with the focus in the list (`ContainerObjectSelectionList.nextFocusPath`).
    /// Whether the list kept it: past the last entry downwards, above the header and to the right
    /// of a slot the screen takes over; left of the first widget nothing moves.
    pub(super) fn arrow(&mut self, arrow: Arrow) -> bool {
        let slots = self.len().saturating_sub(1);
        let target = if self.view.focused {
            self.moved_focus(arrow, slots)
        } else {
            Some(Self::first_focus(arrow, slots))
        };
        let Some(target) = target else {
            return false;
        };
        self.focus = target;
        self.view.focused = true;
        let entry = match target {
            Focus::Slot(item) => item + 1,
            _ => 0,
        };
        let rows = self.len();
        self.view.select(entry, rows);
        true
    }

    /// Where an arrow puts the focus when the list has none: right and down start at the first
    /// sort button, left at the last, up at the last item.
    fn first_focus(arrow: Arrow, slots: usize) -> Focus {
        match arrow {
            Arrow::Down | Arrow::Right => Focus::Button(0),
            Arrow::Left => Focus::Button(ICONS.len() - 1),
            Arrow::Up => slots.checked_sub(1).map_or(Focus::Button(0), Focus::Slot),
        }
    }

    /// Where an arrow moves the focus the list has; `None` when the screen is to take the arrow.
    fn moved_focus(&self, arrow: Arrow, slots: usize) -> Option<Focus> {
        match (self.focus, arrow) {
            (Focus::Button(column), Arrow::Left) => Some(Focus::Button(column.saturating_sub(1))),
            (Focus::Button(column), Arrow::Right) => {
                Some(Focus::Button((column + 1).min(ICONS.len() - 1)))
            }
            // A slot has nothing to its right: the screen takes the arrow.
            (Focus::Slot(_), Arrow::Right) => None,
            (focus, Arrow::Down) => Self::below(focus, slots),
            // Nothing above the header: the screen takes the arrow.
            (Focus::Button(_), Arrow::Up) => None,
            (Focus::Slot(0), Arrow::Up) => Some(Focus::Button(0)),
            (Focus::Slot(item), Arrow::Up) => Some(Focus::Slot(item - 1)),
            (focus, _) => Some(focus),
        }
    }

    /// The widget under the focused one (the column index of a button is not kept).
    fn below(focus: Focus, slots: usize) -> Option<Focus> {
        match focus {
            Focus::Button(_) if slots > 0 => Some(Focus::Slot(0)),
            Focus::Slot(item) if item + 1 < slots => Some(Focus::Slot(item + 1)),
            _ => None,
        }
    }

    /// A step of Tab through the widgets of the list, in the order of their entries: the sort
    /// buttons of the header, then the slot of each item. An unfocused list takes the first (the
    /// last going backwards). Whether it took the step; a step past the end is the screen's.
    pub(super) fn tab_step(&mut self, forward: bool) -> bool {
        let buttons = ICONS.len();
        let Some(next) = self.next_widget(forward, buttons) else {
            return false;
        };
        let (focus, entry) = if next < buttons {
            (Focus::Button(next), 0)
        } else {
            (Focus::Slot(next - buttons), next - buttons + 1)
        };
        self.focus = focus;
        self.view.focused = true;
        let rows = self.len();
        self.view.select(entry, rows);
        true
    }

    /// The place among the buttons and slots that a step of Tab goes to, if there is one.
    fn next_widget(&self, forward: bool, buttons: usize) -> Option<usize> {
        let total = buttons + self.len().saturating_sub(1);
        let at = match (self.view.focused, self.focus) {
            (true, Focus::Button(column)) => Some(column),
            (true, Focus::Slot(item)) => Some(buttons + item),
            _ => None,
        };
        let next = match (at, forward) {
            (Some(at), true) => Some(at + 1),
            (Some(at), false) => at.checked_sub(1),
            (None, true) => Some(0),
            (None, false) => total.checked_sub(1),
        };
        next.filter(|&next| next < total)
    }

    /// Another child of the screen took the press: the focus leaves the list, and with it the
    /// selection (`ContainerObjectSelectionList.setFocused`).
    pub(super) fn unfocus(&mut self) {
        self.focus = Focus::None;
        self.view.selected = None;
        self.view.focused = false;
    }

    /// The item slot whose rectangle holds the point.
    fn slot_at(&self, mouse: (i32, i32), index: usize) -> bool {
        let (x, y) = (
            self.view.row_left() + PADDING,
            self.view.row_y(index + 1) + PADDING,
        );
        mouse.0 >= x && mouse.0 < x + 18 && mouse.1 >= y && mouse.1 < y + 18
    }

    /// The widget at the mouse, with its entry: a header button or an item's slot.
    fn widget_at(&self, mouse: (i32, i32)) -> Option<(usize, Focus)> {
        let entry = self.view.entry_at(mouse, self.len())?;
        let focus = match entry {
            0 => Focus::Button(self.button_at(mouse, false)?),
            index if self.slot_at(mouse, index - 1) => Focus::Slot(index - 1),
            _ => return None,
        };
        Some((entry, focus))
    }

    /// A press: a header button sorts, a slot is focused; both choose their entry. Whether the
    /// list took the press (a press it does not take leaves the focus as it was).
    pub(super) fn clicked(&mut self, mouse: (i32, i32), stats: &HashMap<StatKey, i32>) -> bool {
        let Some((entry, focus)) = self.widget_at(mouse) else {
            return false;
        };
        self.focus = focus;
        self.view.focused = true;
        let rows = self.len();
        self.view.select(entry, rows);
        // Both are widgets: `AbstractWidget.mouseClicked` sounds before the button sorts.
        crate::ui_sound::play_down_sound();
        if let Focus::Button(column) = focus {
            self.sort_by(column, stats);
        }
        true
    }

    /// `sortByColumn`: by the column descending, then ascending, then not at all, the rows sorted
    /// by the numbers held now (`ItemRowComparator`: equal numbers by item id).
    fn sort_by(&mut self, column: usize, stats: &HashMap<StatKey, i32>) {
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
        (index, row, y, shaded): (usize, &Row, i32, bool),
        (stats, item): (&HashMap<StatKey, i32>, &mut ItemDrawer<'_>),
    ) {
        let x = self.view.row_left() + PADDING;
        let content_y = y + PADDING;
        let text_y = content_y + (ROW_HEIGHT - 2 * PADDING) / 2 - 9 / 2;
        frame.sprite(atlas, "container/slot", x, content_y, 18, 18);
        let stack = photon_world::ClientItemStackState::new(1, photon_core::ids::ItemId(row.item));
        frame.item(item, &stack, x + 1, content_y + 1);
        if self.focus == Focus::Slot(index) {
            outline(frame, atlas, (x, content_y, 18, 18));
        }
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

    /// `ItemDisplayWidget.extractTooltip`: the stack of the slot the mouse is over and where its
    /// tooltip goes (below and right of the slot's corner).
    pub(super) fn hovered_stack(
        &self,
        mouse: (i32, i32),
    ) -> Option<(photon_world::ClientItemStackState, (i32, i32))> {
        let (entry, Focus::Slot(index)) = self.widget_at(mouse)? else {
            return None;
        };
        let place = self.view.place;
        if entry == 0 || mouse.1 < place.y || mouse.1 >= place.bottom() {
            return None;
        }
        let row = self.rows.get(index)?;
        let at = (
            self.view.row_left() + PADDING + 18,
            self.view.row_y(index + 1) + PADDING + 18,
        );
        let stack = photon_world::ClientItemStackState::new(1, photon_core::ids::ItemId(row.item));
        Some((stack, at))
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
                    (index, row, y, (index + 1) % 2 == 0),
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
        let hovered = self.button_at(mouse, true);
        for (column, icon) in ICONS.iter().enumerate() {
            let (x, top) = self.button_rect(column, y);
            let background = if hovered == Some(column) || self.focus == Focus::Button(column) {
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

/// `GuiGraphicsExtractor.outline` in white.
fn outline(frame: &mut GuiFrame, atlas: &GuiAtlas, (x, y, width, height): (i32, i32, i32, i32)) {
    let white = 0xFFFF_FFFF;
    frame.fill(atlas, (x, y, x + width, y + 1), white);
    frame.fill(atlas, (x, y + height - 1, x + width, y + height), white);
    frame.fill(atlas, (x, y + 1, x + 1, y + height - 1), white);
    frame.fill(
        atlas,
        (x + width - 1, y + 1, x + width, y + height - 1),
        white,
    );
}
