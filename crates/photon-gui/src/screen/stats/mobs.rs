//! The mobs tab (`MobsStatisticsList`): each kind of mob the player killed or was killed by, with
//! the two counts.

use std::collections::HashMap;

use super::list::{ListView, PADDING};
use crate::FrameRuns;
use crate::widgets::scroll_area::Place;
use photon_core::ids::EntityTypeId;
use photon_data::stat::StatKey;
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `9 * 4`: the name, and a line each for the kills and the deaths.
const ROW_HEIGHT: i32 = 36;

struct Row {
    name: Vec<Run>,
    kills: Vec<Run>,
    killed_by: Vec<Run>,
    has_kills: bool,
    was_killed_by: bool,
}

pub(super) struct MobsList {
    rows: Vec<Row>,
    view: ListView,
}

/// The id of a stat type by its name.
fn type_id(name: &str) -> i32 {
    photon_data::stat::type_names()
        .iter()
        .position(|candidate| *candidate == name)
        .unwrap_or(0) as i32
}

/// The count of the statistic of a stat type for an entity type.
fn count(stats: &HashMap<StatKey, i32>, stat_type: i32, entity: i32) -> i32 {
    stats
        .get(&StatKey {
            stat_type,
            value: entity,
        })
        .copied()
        .unwrap_or(0)
}

impl MobsList {
    /// `MobsStatisticsList`: the entity types in the order of the registry with either count.
    pub(super) fn new(lang: &Lang, stats: &HashMap<StatKey, i32>) -> Self {
        let (killed, killed_by) = (type_id("minecraft:killed"), type_id("minecraft:killed_by"));
        let literal = |text: String| Component::literal(text);
        let mut rows = Vec::new();
        for id in 0.. {
            let Some(name) = photon_data::entity_type::name(EntityTypeId(id)) else {
                break;
            };
            let (kills, deaths) = (count(stats, killed, id), count(stats, killed_by, id));
            if kills == 0 && deaths == 0 {
                continue;
            }
            // `EntityType.getDescription`: `entity.minecraft.pig`.
            let mob = Component::translatable(format!("entity.{}", name.replace(':', ".")));
            let runs = |component: &Component| lang.runs(component, &Style::default());
            let kills_text = if kills == 0 {
                Component::translatable_with("stat_type.minecraft.killed.none", vec![mob.clone()])
            } else {
                Component::translatable_with(
                    "stat_type.minecraft.killed",
                    vec![literal(kills.to_string()), mob.clone()],
                )
            };
            let killed_by_text = if deaths == 0 {
                Component::translatable_with(
                    "stat_type.minecraft.killed_by.none",
                    vec![mob.clone()],
                )
            } else {
                Component::translatable_with(
                    "stat_type.minecraft.killed_by",
                    vec![mob.clone(), literal(deaths.to_string())],
                )
            };
            rows.push(Row {
                name: runs(&mob),
                kills: runs(&kills_text),
                killed_by: runs(&killed_by_text),
                has_kills: kills != 0,
                was_killed_by: deaths != 0,
            });
        }
        Self {
            rows,
            view: ListView::new(ROW_HEIGHT),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub(super) fn place(&mut self, place: Place) {
        self.view.place(place, self.rows.len());
    }

    pub(super) fn scroll_by(&mut self, delta: f64) {
        self.view.scroll_by(delta, self.rows.len());
    }

    /// A press: a row under it is selected, and the list has the focus. Whether the list took it.
    pub(super) fn clicked(&mut self, mouse: (i32, i32)) -> bool {
        let rows = self.rows.len();
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

    /// `ObjectSelectionList.nextFocusPath` by Tab: the selected entry, else the first (the last
    /// going backwards), takes the focus. Whether the list had an entry to give it to.
    pub(super) fn focus_entry(&mut self, forward: bool) -> bool {
        let rows = self.rows.len();
        self.view.focus_entry(forward, rows)
    }

    /// The rows that show, then the scroll bar.
    pub(super) fn draw(&self, frame: &mut GuiFrame, (atlas, font): (&GuiAtlas, &Font)) {
        let view = &self.view;
        let mark = frame.mark();
        for (index, row) in self.rows.iter().enumerate() {
            let y = view.row_y(index);
            if !view.shows(y) {
                continue;
            }
            view.draw_selection(frame, atlas, index);
            let (x, top) = (view.row_left() + PADDING, y + PADDING + 1);
            let shade = |on: bool| if on { 0xFFBA_BABA } else { 0xFF80_8080 };
            frame.styled_text(atlas, font, &row.name, x + 2, top, 0xFFFF_FFFF, true);
            frame.styled_text(
                atlas,
                font,
                &row.kills,
                x + 12,
                top + 9,
                shade(row.has_kills),
                true,
            );
            frame.styled_text(
                atlas,
                font,
                &row.killed_by,
                x + 12,
                top + 18,
                shade(row.was_killed_by),
                true,
            );
        }
        frame.clip_from(
            mark,
            (0, view.place.y, view.place.width, view.place.bottom()),
        );
        view.draw_scrollbar(frame, atlas, self.rows.len());
    }
}
