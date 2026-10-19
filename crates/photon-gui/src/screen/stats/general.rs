//! The general tab (`GeneralStatisticsList`): every custom statistic, by its name, with its number.

use std::collections::HashMap;

use crate::FrameRuns;
use crate::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use photon_data::stat::StatKey;
use photon_data::stat::{self, Formatter};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `ObjectSelectionList`'s row height and the width of a row.
const ROW_HEIGHT: i32 = 14;
const ROW_WIDTH: i32 = 280;
/// `AbstractSelectionList.Entry`'s padding at each side of the content, and the list's under its
/// top.
const PADDING: i32 = 2;
/// The stat type of the custom statistics, in the registry's order (`minecraft:custom`).
const CUSTOM_TYPE: &str = "minecraft:custom";

struct Entry {
    /// The custom statistic's id (its place in the registry).
    id: i32,
    name: Vec<Run>,
    formatter: Formatter,
}

pub(super) struct GeneralList {
    entries: Vec<Entry>,
    place: Place,
    scroll: f64,
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
            place: Place {
                y: 0,
                width: 0,
                height: 0,
                in_world: true,
            },
            scroll: 0.0,
            custom_type,
        }
    }

    /// `updateSizeAndPosition`: the list's place, and the scroll held to what it can be there.
    pub(super) fn place(&mut self, place: Place) {
        self.place = place;
        self.scroll = self
            .scroll
            .clamp(0.0, f64::from(self.place.max_scroll(self.content_height())));
    }

    /// `AbstractSelectionList.contentHeight`.
    fn content_height(&self) -> i32 {
        self.entries.len() as i32 * ROW_HEIGHT + 4
    }

    /// `AbstractScrollArea.mouseScrolled`: the rate is half a row.
    pub(super) fn scroll_by(&mut self, delta: f64) {
        let max = f64::from(self.place.max_scroll(self.content_height()));
        self.scroll = (self.scroll - delta * f64::from(ROW_HEIGHT / 2)).clamp(0.0, max);
    }

    /// The entries that show, then the scroll bar.
    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        stats: &HashMap<StatKey, i32>,
    ) {
        let place = self.place;
        let row_left = place.width / 2 - ROW_WIDTH / 2;
        let row_right = row_left + ROW_WIDTH;
        let mark = frame.mark();
        let top = place.y + 2 - self.scroll as i32;
        for (index, entry) in self.entries.iter().enumerate() {
            let y = top + index as i32 * ROW_HEIGHT;
            if y + ROW_HEIGHT < place.y || y > place.bottom() {
                continue;
            }
            let content_y = y + PADDING;
            let text_y = content_y + (ROW_HEIGHT - 2 * PADDING) / 2 - 9 / 2;
            let colour = if index % 2 == 0 {
                0xFFFF_FFFF
            } else {
                0xFFBA_BABA
            };
            frame.styled_text(
                atlas,
                font,
                &entry.name,
                row_left + PADDING + 2,
                text_y,
                colour,
                true,
            );
            let key = StatKey {
                stat_type: self.custom_type,
                value: entry.id,
            };
            let value =
                photon_game::stats::format(entry.formatter, stats.get(&key).copied().unwrap_or(0));
            let x = row_right - PADDING - font.width(&value) - 4;
            frame.text_with_shadow(atlas, font, &value, x, text_y, colour);
        }
        frame.clip_from(mark, (0, place.y, place.width, place.bottom()));
        place.draw_scrollbar(
            frame,
            atlas,
            row_right + SCROLLBAR_WIDTH + 2,
            self.scroll,
            self.content_height(),
        );
    }
}
