//! The general tab (`GeneralStatisticsList`): every custom statistic, by its name, with its number.

use std::collections::HashMap;

use super::list::{ListView, PADDING};
use crate::FrameRuns;
use crate::widgets::scroll_area::Place;
use photon_data::stat::{self, Formatter, StatKey};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

const ROW_HEIGHT: i32 = 14;
/// The stat type of the custom statistics (`Stats.CUSTOM`).
const CUSTOM_TYPE: &str = "minecraft:custom";

struct Entry {
    /// The custom statistic's id (its place in the registry).
    id: i32,
    name: Vec<Run>,
    formatter: Formatter,
}

pub(super) struct GeneralList {
    entries: Vec<Entry>,
    view: ListView,
    custom_type: i32,
}

impl GeneralList {
    /// `GeneralStatisticsList`: the statistics sorted by their names in the language.
    pub(super) fn new(lang: &Lang) -> Self {
        let custom_type = stat::type_names()
            .iter()
            .position(|name| *name == CUSTOM_TYPE)
            .unwrap_or(0) as i32;
        let mut keyed: Vec<(Vec<u16>, Entry)> = stat::custom_names()
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let key = format!("stat.{}", name.replace(':', "."));
                let id = index as i32;
                let entry = Entry {
                    id,
                    name: lang.runs(&Component::translatable(&key), &Style::default()),
                    formatter: stat::custom_formatter(id).unwrap_or(Formatter::Default),
                };
                (lang.text(&key).encode_utf16().collect(), entry)
            })
            .collect();
        // `Comparator.comparing(I18n.get(key))`: `String.compareTo`, by UTF-16 unit.
        keyed.sort_by(|a, b| a.0.cmp(&b.0));
        Self {
            entries: keyed.into_iter().map(|(_, entry)| entry).collect(),
            view: ListView::new(ROW_HEIGHT),
            custom_type,
        }
    }

    pub(super) fn place(&mut self, place: Place) {
        self.view.place(place, self.entries.len());
    }

    pub(super) fn scroll_by(&mut self, delta: f64) {
        self.view.scroll_by(delta, self.entries.len());
    }

    /// A press: an entry under it is selected, and the list has the focus. Whether the list took it.
    pub(super) fn clicked(&mut self, mouse: (i32, i32)) -> bool {
        let rows = self.entries.len();
        let Some(index) = self.view.entry_at(mouse, rows) else {
            return false;
        };
        self.view.focused = true;
        self.view.select(index, rows);
        true
    }

    /// Another child of the screen took the press: the focus leaves the list.
    pub(super) fn unfocus(&mut self) {
        self.view.focused = false;
    }

    /// An up or down arrow in the focused list; whether the focus stayed in it.
    pub(super) fn arrow_step(&mut self, down: bool) -> bool {
        let rows = self.entries.len();
        self.view.arrow_step(down, rows)
    }

    /// A left or right arrow: nothing to move to, so the list lets go.
    pub(super) fn clear(&mut self) {
        self.view.clear();
    }

    /// Whether the list has the focus.
    pub(super) fn is_focused(&self) -> bool {
        self.view.focused
    }

    /// `ObjectSelectionList.nextFocusPath` by Tab: the selected entry, else the first (the last
    /// going backwards), takes the focus. Whether the list had an entry to give it to.
    pub(super) fn focus_entry(&mut self, forward: bool) -> bool {
        let rows = self.entries.len();
        self.view.focus_entry(forward, rows)
    }

    /// The entries that show, then the scroll bar.
    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        stats: &HashMap<StatKey, i32>,
    ) {
        let view = &self.view;
        let mark = frame.mark();
        for (index, entry) in self.entries.iter().enumerate() {
            let y = view.row_y(index);
            if !view.shows(y) {
                continue;
            }
            view.draw_selection(frame, atlas, index);
            let text_y = y + PADDING + (ROW_HEIGHT - 2 * PADDING) / 2 - 9 / 2;
            let colour = if index % 2 == 0 {
                0xFFFF_FFFF
            } else {
                0xFFBA_BABA
            };
            let left = view.row_left() + PADDING + 2;
            frame.styled_text(atlas, font, &entry.name, (left, text_y), colour, true);
            let key = StatKey {
                stat_type: self.custom_type,
                value: entry.id,
            };
            let number = stats.get(&key).copied().unwrap_or(0);
            let value = photon_game::stats::format(entry.formatter, number);
            let x = view.row_right() - PADDING - font.width(&value) - 4;
            frame.text_with_shadow(atlas, font, &value, x, text_y, colour);
        }
        frame.clip_from(
            mark,
            (0, view.place.y, view.place.width, view.place.bottom()),
        );
        view.draw_scrollbar(frame, atlas, self.entries.len());
    }
}
