//! The debug options screen (`DebugOptionsScreen`, opened with F3+F6): a search over the debug
//! screen's entries, which are listed under their groups with three buttons each (off, in the
//! overlay, always), and the two profiles.
//!
//! Not drawn: the tooltip over the name of an entry that reduced debug info hides.

mod draw;

use std::time::Instant;

use crate::FontRuns;
use photon_game::text::{Component, Lang, Run, Style};

use crate::chat::split;
use crate::hud::debug::{Category, EntryList, Profile, Status, listed_entries};
use crate::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::widgets::edit_box::EditBox;
use crate::widgets::scroll_area::Place;
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;

/// The header's height, as `HeaderAndFooterLayout(this, 61, 33)` has it, and the footer's.
const HEADER_HEIGHT: i32 = 61;
const FOOTER_HEIGHT: i32 = 33;
/// `OptionList`'s rows: `getRowWidth` and the item height.
const ROW_WIDTH: i32 = 350;
const ITEM_HEIGHT: i32 = 20;
/// The buttons of an entry, each as wide as this and 16 tall.
const BUTTON_WIDTH: i32 = 60;
const BUTTON_HEIGHT: i32 = 16;
/// The colours the buttons' values and the warning take: always, in the overlay, off, and a value
/// that is not the current one.
const ALWAYS_COLOUR: u32 = 0xDF_5050;
const OVERLAY_COLOUR: u32 = 0xFF_FF55;
const NEVER_COLOUR: u32 = 0xFF_FFFF;
const UNSELECTED_COLOUR: u32 = 0xBA_BABA;
/// The name of an entry reduced debug info hides.
const DISALLOWED_COLOUR: u32 = 0xFF80_8080;
/// A scroll of the wheel moves the list by this much (`itemHeight / 2`).
const SCROLL_RATE: f64 = (ITEM_HEIGHT / 2) as f64;

/// The statuses a row's buttons set, in the order they stand in: off, in the overlay, always.
const BUTTON_STATUSES: [Status; 3] = [Status::Never, Status::InOverlay, Status::AlwaysOn];

/// A row of the list.
enum Row {
    /// A group's label (`CategoryEntry`).
    Category(Vec<Run>),
    /// An entry (`OptionEntry`): its name, whether reduced debug info allows it, and its buttons.
    Entry {
        id: &'static str,
        name: Vec<Run>,
        allowed: bool,
        buttons: [Widget; 3],
    },
}

/// A text of the language in one colour, or the one it has.
fn styled(lang: &Lang, key: &str, colour: Option<u32>) -> Vec<Run> {
    let style = Style {
        colour,
        ..Style::default()
    };
    lang.runs(&Component::translatable(key), &style)
}

/// The label of each button (off, in the overlay, always), as the value it is and as one that is not.
fn value_texts(lang: &Lang) -> [[Vec<Run>; 2]; 3] {
    [
        ("options.off", NEVER_COLOUR),
        ("debug.entry.overlay", OVERLAY_COLOUR),
        ("debug.entry.always", ALWAYS_COLOUR),
    ]
    .map(|(key, colour)| {
        [
            styled(lang, key, Some(colour)),
            styled(lang, key, Some(UNSELECTED_COLOUR)),
        ]
    })
}

/// The footer's buttons: the two profiles (the one in use cannot be pressed) and Done.
fn footer_buttons(arena: &mut Arena, lang: &Lang, entries: &EntryList) -> ([Widget; 2], Widget) {
    let mut button = |key: &str, wide: i32| Widget {
        rect: arena.add(wide, 20),
        active: true,
        kind: Kind::Button {
            label: styled(lang, key, None),
        },
    };
    let profiles = Profile::ALL.map(|profile| {
        let mut widget = button(profile.translation_key(), 120);
        widget.active = !entries.is_using_profile(profile);
        widget
    });
    (profiles, button("gui.done", 60))
}

pub struct DebugOptionsScreen {
    arena: Arena,
    title: Widget,
    search_rect: usize,
    search: EditBox,
    focused_at: Instant,
    subtitle: Vec<Vec<Run>>,
    subtitle_rect: usize,
    profiles: [Widget; 2],
    done: Widget,
    entries: EntryList,
    reduced: bool,
    last_search: String,
    rows: Vec<Row>,
    /// The three values' texts: the label of a button that is the current value, by button, and
    /// that of one that is not.
    values: [[Vec<Run>; 2]; 3],
    categories: [Vec<Run>; 2],
    place: Place,
    scroll: f64,
    size: (i32, i32),
}

impl DebugOptionsScreen {
    /// `init` over the entries `entries` holds, with reduced debug info or not.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        entries: &EntryList,
        reduced: bool,
    ) -> Self {
        let (width, height) = size;
        let styled = |key: &str, colour: Option<u32>| styled(lang, key, colour);
        let title_runs = styled("debug.options.title", None);
        let subtitle = split(
            &styled("debug.options.warning", Some(ALWAYS_COLOUR)),
            ROW_WIDTH,
            font,
        );
        let mut arena = Arena::default();
        let spacer = arena.add(ROW_WIDTH / 3, 1);
        let title_rect = arena.add(font.width_of_runs(&title_runs), 9);
        let search_rect = arena.add(ROW_WIDTH / 3, 20);
        let widest = subtitle.iter().map(|line| font.width_of_runs(line)).max();
        let subtitle_rect = arena.add(widest.unwrap_or(0), subtitle.len() as i32 * 9);
        let (profiles, done) = footer_buttons(&mut arena, lang, entries);
        let mut screen = Self {
            title: Widget {
                rect: title_rect,
                active: false,
                kind: Kind::Text { text: title_runs },
            },
            search_rect,
            // `EditBox.getInnerWidth` of the box.
            search: EditBox::new(ROW_WIDTH / 3 - 8, 32),
            focused_at: Instant::now(),
            subtitle,
            subtitle_rect,
            profiles,
            done,
            entries: entries.clone(),
            reduced,
            last_search: String::new(),
            rows: Vec::new(),
            values: value_texts(lang),
            categories: [
                styled(Category::ScreenText.translation_key(), None),
                styled(Category::Renderer.translation_key(), None),
            ],
            place: Place {
                y: HEADER_HEIGHT,
                width,
                height: height - HEADER_HEIGHT - FOOTER_HEIGHT,
                in_world: true,
            },
            scroll: 0.0,
            size,
            arena,
        };
        screen.arrange(spacer);
        screen.update_search();
        screen
    }

    /// The header's title row and subtitle and the footer's buttons, placed in their frames.
    fn arrange(&mut self, spacer: usize) {
        let (width, height) = self.size;
        let centred = Settings::default().align_horizontally(0.5);
        let middle = Settings {
            y_align: 0.5,
            ..Settings::default()
        };
        let mut title_row = Grid::new();
        title_row.column_spacing = 8;
        title_row.add(Item::Leaf(spacer), (0, 0), (1, 1), Settings::default());
        title_row.add(Item::Leaf(self.title.rect), (0, 1), (1, 1), middle);
        title_row.add(
            Item::Leaf(self.search_rect),
            (0, 2),
            (1, 1),
            Settings::default(),
        );
        let mut header = Grid::new();
        header.row_spacing = 8;
        header.add(Item::Grid(Box::new(title_row)), (0, 0), (1, 1), centred);
        header.add(Item::Leaf(self.subtitle_rect), (1, 0), (1, 1), centred);
        header.arrange(&mut self.arena);
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(header)),
            (0, 0, width, HEADER_HEIGHT),
            (0.5, 0.5),
        );
        let leaves = [self.profiles[0].rect, self.profiles[1].rect, self.done.rect];
        let mut footer = horizontal(&self.arena, &leaves, 8);
        footer.arrange(&mut self.arena);
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(footer)),
            (0, height - FOOTER_HEIGHT, width, FOOTER_HEIGHT),
            (0.5, 0.5),
        );
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// `OptionList.updateSearch`: the rows for the entries whose id holds the search text, each
    /// group's label before its first.
    fn update_search(&mut self) {
        let filter = self.search.value();
        let mut rows = Vec::new();
        let mut current = None;
        for entry in listed_entries() {
            if !entry.id.contains(filter.as_str()) {
                continue;
            }
            if current != Some(entry.category) {
                current = Some(entry.category);
                let index = usize::from(entry.category == Category::Renderer);
                rows.push(Row::Category(self.categories[index].clone()));
            }
            let allowed = !self.reduced || entry.allowed_when_reduced;
            let style = Style {
                italic: (!allowed).then_some(true),
                ..Style::default()
            };
            let name = vec![Run {
                text: entry.id.to_owned(),
                style,
            }];
            let buttons = std::array::from_fn(|_| Widget {
                rect: self.arena.add(BUTTON_WIDTH, BUTTON_HEIGHT),
                active: true,
                kind: Kind::Button { label: Vec::new() },
            });
            rows.push(Row::Entry {
                id: entry.id,
                name,
                allowed,
                buttons,
            });
        }
        self.last_search = filter;
        self.rows = rows;
        self.refresh_rows();
        self.set_scroll(self.scroll);
    }

    /// `OptionEntry.refreshEntry`: the button of the current status is the one that cannot be
    /// pressed, and each says its value in the colour that tells.
    fn refresh_rows(&mut self) {
        for row in &mut self.rows {
            let Row::Entry { id, buttons, .. } = row else {
                continue;
            };
            let current = self.entries.status(id);
            for (index, button) in buttons.iter_mut().enumerate() {
                let selected = BUTTON_STATUSES[index] == current;
                button.active = !selected;
                button.kind = Kind::Button {
                    label: self.values[index][usize::from(!selected)].clone(),
                };
            }
        }
    }

    pub(super) fn content_height(&self) -> i32 {
        self.rows.len() as i32 * ITEM_HEIGHT + 4
    }

    fn max_scroll(&self) -> f64 {
        f64::from(self.place.max_scroll(self.content_height()))
    }

    fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, self.max_scroll());
        self.position_buttons();
    }

    /// Where each entry's buttons stand (`OptionEntry.extractContent` sets them as it draws):
    /// against the right edge of the row's content.
    fn position_buttons(&mut self) {
        let left = self.row_left() + 2 + (ROW_WIDTH - 4) - 3 * BUTTON_WIDTH;
        for index in 0..self.rows.len() {
            let y = self.row_y(index) + 2;
            let Row::Entry { buttons, .. } = &self.rows[index] else {
                continue;
            };
            for (slot, button) in buttons.iter().enumerate() {
                let rect = &mut self.arena.0[button.rect];
                rect.x = left + slot as i32 * BUTTON_WIDTH;
                rect.y = y;
            }
        }
    }

    /// The wheel over the list (`AbstractSelectionList.mouseScrolled`).
    pub fn scrolled(&mut self, mouse: (f64, f64), dy: f64) {
        if self.place.contains(mouse) {
            self.set_scroll(self.scroll - dy * SCROLL_RATE);
        }
    }

    /// A character typed into the search box (it has the focus at first).
    pub fn char_typed(&mut self, character: char, font: &Font) {
        if self.search.char_typed(character, font) && self.search.value() != self.last_search {
            self.update_search();
        }
    }

    /// The text typed into the search box, a character at a time.
    pub fn type_search(&mut self, text: &str, font: &Font) {
        for character in text.chars() {
            self.char_typed(character, font);
        }
    }

    fn row_left(&self) -> i32 {
        self.size.0 / 2 - ROW_WIDTH / 2
    }

    /// The top of the row `index`.
    fn row_y(&self, index: usize) -> i32 {
        self.place.y + 2 - self.scroll as i32 + index as i32 * ITEM_HEIGHT
    }
}
