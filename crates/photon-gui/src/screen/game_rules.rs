//! The in-world game rules screen (`InWorldGameRulesScreen` over `AbstractGameRulesScreen`): a
//! title and a search over a list of the rules the server named, each under its category with an
//! on/off button or a number box, and Done and Cancel. The list shows "Retrieving game rules..."
//! until the server's first answer.
//!
//! Not handled: the arrow keys between the list's controls.

mod arrows;
mod draw;
mod input;

use std::collections::BTreeSet;
use std::time::Instant;

use photon_data::game_rules::{GameRule, rule};
use photon_game::text::{Component, Lang, Run, Style};

use crate::FontRuns;
use crate::chat::split;
use crate::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::widgets::edit_box::EditBox;
use crate::widgets::scroll_area::Place;
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;

/// `HeaderAndFooterLayout(this, 12 + 9 + 15, 33)`.
const HEADER_HEIGHT: i32 = 36;
const FOOTER_HEIGHT: i32 = 33;
/// `AbstractSelectionList.getRowWidth` and the rule list's item height.
const ROW_WIDTH: i32 = 220;
const ITEM_HEIGHT: i32 = 24;
/// The search box: `EditBox(font, 200, 15)`.
const SEARCH_WIDTH: i32 = 200;
const SEARCH_HEIGHT: i32 = 15;
/// The control at the right of a row (`create(10, 5, 44, 20, ...)`).
const CONTROL_WIDTH: i32 = 44;
const CONTROL_HEIGHT: i32 = 20;
/// The width a rule's name wraps at, and a description in its tooltip.
const LABEL_WIDTH: i32 = 170;
const DESCRIPTION_WIDTH: i32 = 150;
/// `EditBox.setTextColor` of a number that is valid and one that is not.
const VALID_COLOUR: u32 = 0xFFE0_E0E0;
const INVALID_COLOUR: u32 = 0xFFFF_0000;
/// `ChatFormatting.YELLOW` and `GRAY`.
const YELLOW: u32 = 0xFFFF55;
/// A scroll of the wheel moves the list by this much (`itemHeight / 2`).
const SCROLL_RATE: f64 = (ITEM_HEIGHT / 2) as f64;

/// What the screen asks of the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameRulesEvent {
    /// The screen closes without sending anything.
    Close,
    /// Done: the entries that changed go to the server (none, and nothing is sent), and the
    /// screen closes.
    Apply(Vec<(String, String)>),
    /// Escape or Cancel with changes pending: the discard question opens.
    ConfirmDiscard,
}

/// A rule's value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Value {
    Bool(bool),
    Int(i32),
}

impl Value {
    fn serialize(self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
        }
    }
}

/// `GameRule.deserialize`: `true` or `false`, or a whole number within the rule's bounds.
fn deserialize(rule: &GameRule, text: &str) -> Option<Value> {
    match rule.bounds {
        None => match text {
            "true" => Some(Value::Bool(true)),
            "false" => Some(Value::Bool(false)),
            _ => None,
        },
        Some((low, high)) => {
            // `StringReader.readInt` takes digits, dots and minuses, and the rest is left over.
            let number = text
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.' || c == '-');
            let value = text.parse::<i32>().ok().filter(|_| number)?;
            (low..=high).contains(&value).then_some(Value::Int(value))
        }
    }
}

/// The default of a rule, as a value.
fn default_value(rule: &GameRule) -> Value {
    deserialize(rule, rule.default).unwrap_or(Value::Bool(false))
}

/// A rule the server named, with what the screen says of it.
struct Entry {
    rule: &'static GameRule,
    /// The value the server gave (none when it was not one the rule takes).
    initial: Option<Value>,
    current: Value,
    lines: Vec<Vec<Run>>,
    tooltip: Vec<Vec<Run>>,
    /// What the search looks through, lower case: the short id, the name, the category's label
    /// and the description.
    haystack: [String; 4],
}

/// The control of a row.
enum Control {
    /// A `CycleButton` of on and off.
    Toggle(Widget),
    /// An `EditBox` of a number, and the place of its field.
    Input {
        rect: usize,
        edit: EditBox,
        valid: bool,
    },
}

enum Row {
    /// A category's label (`CategoryRuleEntry`).
    Category(Vec<Run>),
    /// A rule's label and control, and the entry it belongs to.
    Rule {
        entry: usize,
        serial: u32,
        control: Control,
    },
}

/// Where the focus is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Search,
    Done,
    Cancel,
    /// The control of the rule at this index of the entries.
    Control(usize),
}

pub struct GameRulesScreen {
    arena: Arena,
    size: (i32, i32),
    title: Widget,
    search_rect: usize,
    search: EditBox,
    search_hint: Vec<Run>,
    last_search: String,
    focused_at: Instant,
    focus: Option<Focus>,
    loading: Option<Loading>,
    done: Widget,
    cancel: Widget,
    entries: Vec<Entry>,
    rows: Vec<Row>,
    place: Place,
    scroll: f64,
    scrolling: bool,
    received: bool,
    /// `invalidEntries`: the rows whose number is not one the rule takes. A row that a new search
    /// replaced stays in it.
    invalid: BTreeSet<u32>,
    next_serial: u32,
    on: [Vec<Run>; 2],
    categories: Vec<(&'static str, Vec<Run>)>,
}

/// `LoadingDotsWidget`: the text it says and where it stands.
struct Loading {
    text: Vec<Run>,
    rect: usize,
}

fn plain(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(&Component::translatable(key), &Style::default())
}

impl GameRulesScreen {
    /// `init`: the title, the search box and the buttons, and the loading text where the list
    /// will be.
    pub fn new(font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        let (width, height) = size;
        let mut arena = Arena::default();
        let title_runs = plain(lang, "editGamerule.title");
        let title_rect = arena.add(font.width_of_runs(&title_runs), 9);
        let search_rect = arena.add(SEARCH_WIDTH, SEARCH_HEIGHT);
        let message = plain(lang, "editGamerule.inGame.downloadingGamerules");
        let loading = Loading {
            rect: arena.add(font.width_of_runs(&message), 2 + 9 + 6 + 9 + 2),
            text: message,
        };
        let mut button = |key: &str| Widget {
            rect: arena.add(150, 20),
            active: true,
            kind: Kind::Button {
                label: plain(lang, key),
            },
        };
        let (done, cancel) = (button("gui.done"), button("gui.cancel"));
        let hint = Component::translatable("gui.game_rule.search")
            .with_colour("gray")
            .italic();
        let mut screen = Self {
            title: Widget {
                rect: title_rect,
                active: false,
                kind: Kind::Text { text: title_runs },
            },
            search_rect,
            search: EditBox::new(SEARCH_WIDTH - 8, 32),
            search_hint: lang.runs(&hint, &Style::default()),
            last_search: String::new(),
            focused_at: Instant::now(),
            focus: Some(Focus::Search),
            loading: Some(loading),
            done,
            cancel,
            entries: Vec::new(),
            rows: Vec::new(),
            place: Place {
                y: HEADER_HEIGHT,
                width,
                height: height - HEADER_HEIGHT - FOOTER_HEIGHT,
                in_world: true,
            },
            scroll: 0.0,
            scrolling: false,
            received: false,
            invalid: BTreeSet::new(),
            next_serial: 0,
            on: [plain(lang, "options.on"), plain(lang, "options.off")],
            categories: Vec::new(),
            size,
            arena,
        };
        screen.arrange();
        screen
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The header's title and search box and the footer's buttons in their frames, and the
    /// loading text in the contents' (`HeaderAndFooterLayout.arrangeElements`).
    fn arrange(&mut self) {
        let (width, height) = self.size;
        let centred = Settings::default().align_horizontally(0.5);
        let mut header = Grid::new();
        header.row_spacing = 4;
        header.add(Item::Leaf(self.title.rect), (0, 0), (1, 1), centred);
        header.add(Item::Leaf(self.search_rect), (1, 0), (1, 1), centred);
        header.arrange(&mut self.arena);
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(header)),
            (0, 0, width, HEADER_HEIGHT),
            (0.5, 0.5),
        );
        let leaves = [self.done.rect, self.cancel.rect];
        let mut footer = horizontal(&self.arena, &leaves, 8);
        footer.arrange(&mut self.arena);
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(footer)),
            (0, height - FOOTER_HEIGHT, width, FOOTER_HEIGHT),
            (0.5, 0.5),
        );
        if let Some(loading) = &self.loading {
            let rect = &mut self.arena.0[loading.rect];
            let content_height = rect.height;
            rect.x = (width - rect.width) / 2;
            rect.y = (HEADER_HEIGHT + 30).min(height - FOOTER_HEIGHT - content_height);
        }
    }

    /// `Screen.resize`: the widgets laid out again, the list sized to the screen.
    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
        self.place.width = size.0;
        self.place.height = size.1 - HEADER_HEIGHT - FOOTER_HEIGHT;
        self.arrange();
        self.set_scroll(self.scroll);
    }

    /// `onGameRuleValuesUpdated`: the first answer fills the list; later ones are ignored.
    pub fn values_received(&mut self, values: &[(String, String)], font: &Font, lang: &Lang) {
        if self.received {
            return;
        }
        self.received = true;
        for (name, text) in values {
            let Some(rule) = rule(name) else { continue };
            let initial = deserialize(rule, text);
            self.entries.push(entry_of(rule, initial, (font, lang)));
        }
        self.categories = categories_of(&self.entries, lang);
        self.loading = None;
        self.update_search(font);
    }

    pub(super) fn content_height(&self) -> i32 {
        self.rows.len() as i32 * ITEM_HEIGHT + 4
    }

    pub(super) fn max_scroll(&self) -> f64 {
        f64::from(self.place.max_scroll(self.content_height()))
    }

    fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, self.max_scroll());
        self.position_controls();
    }

    pub(super) fn row_left(&self) -> i32 {
        self.size.0 / 2 - ROW_WIDTH / 2
    }

    /// The top of the row `index`.
    pub(super) fn row_y(&self, index: usize) -> i32 {
        self.place.y + 2 - self.scroll as i32 + index as i32 * ITEM_HEIGHT
    }

    /// Where each row's control stands: against the right edge of the row's content.
    fn position_controls(&mut self) {
        let left = self.row_left() + 2 + (ROW_WIDTH - 4) - CONTROL_WIDTH - 1;
        for index in 0..self.rows.len() {
            let y = self.row_y(index) + 2;
            let rect = match &self.rows[index] {
                Row::Rule {
                    control: Control::Toggle(widget),
                    ..
                } => widget.rect,
                Row::Rule {
                    control: Control::Input { rect, .. },
                    ..
                } => *rect,
                Row::Category(_) => continue,
            };
            let rect = &mut self.arena.0[rect];
            rect.x = left;
            rect.y = y;
        }
    }

    /// `RuleList.populateChildren`: the rows of the rules that match the search, under their
    /// categories, both in the order of their description ids.
    fn update_search(&mut self, font: &Font) {
        let filter = self.search.value().to_lowercase();
        let mut order: Vec<usize> = (0..self.entries.len())
            .filter(|&index| {
                let haystack = &self.entries[index].haystack;
                haystack.iter().any(|text| text.contains(&filter))
            })
            .collect();
        order.sort_by_key(|&index| {
            (
                self.entries[index].rule.category,
                self.entries[index].rule.name,
            )
        });
        let mut rows = Vec::new();
        let mut current = None;
        for index in order {
            let category = self.entries[index].rule.category;
            if current != Some(category) {
                current = Some(category);
                let label = self.categories.iter().find(|(name, _)| *name == category);
                rows.push(Row::Category(
                    label.map(|(_, runs)| runs.clone()).unwrap_or_default(),
                ));
            }
            rows.push(self.row_of(index, font));
        }
        self.last_search = self.search.value();
        self.rows = rows;
        self.scroll = 0.0;
        self.position_controls();
    }

    /// The row of a rule, its control made from the value the rule has now.
    fn row_of(&mut self, index: usize, font: &Font) -> Row {
        let serial = self.next_serial;
        self.next_serial += 1;
        let entry = &self.entries[index];
        let control = match entry.current {
            Value::Bool(value) => Control::Toggle(Widget {
                rect: self.arena.add(CONTROL_WIDTH, CONTROL_HEIGHT),
                active: true,
                kind: Kind::Button {
                    label: self.on[usize::from(!value)].clone(),
                },
            }),
            Value::Int(value) => {
                let mut edit = EditBox::new(CONTROL_WIDTH - 8, 32);
                edit.set_value(&value.to_string(), font);
                Control::Input {
                    rect: self.arena.add(CONTROL_WIDTH, CONTROL_HEIGHT),
                    edit,
                    valid: true,
                }
            }
        };
        Row::Rule {
            entry: index,
            serial,
            control,
        }
    }

    /// Whether a rule differs from what the server gave (`hasGameRuleChanged`).
    fn changed(entry: &Entry) -> bool {
        entry
            .initial
            .is_some_and(|initial| initial != entry.current)
    }

    pub fn has_pending_changes(&self) -> bool {
        self.entries.iter().any(Self::changed)
    }

    /// `onDone`: the changed rules, by name and value.
    fn changes(&self) -> Vec<(String, String)> {
        self.entries
            .iter()
            .filter(|entry| Self::changed(entry))
            .map(|entry| (entry.rule.name.to_owned(), entry.current.serialize()))
            .collect()
    }

    /// `onClose`: the question when something changed, else the screen closes.
    pub fn close_requested(&self) -> GameRulesEvent {
        if self.has_pending_changes() {
            GameRulesEvent::ConfirmDiscard
        } else {
            GameRulesEvent::Close
        }
    }

    /// Done cannot be pressed while a number is not one its rule takes.
    fn refresh_done(&mut self) {
        self.done.active = self.invalid.is_empty();
    }
}

/// A rule the server named: its texts, as the screen builds them in `RuleList.addEntry`.
fn entry_of(
    rule: &'static GameRule,
    initial: Option<Value>,
    (font, lang): (&Font, &Lang),
) -> Entry {
    let short = rule.name.strip_prefix("minecraft:").unwrap_or(rule.name);
    let key = format!("gamerule.{}", rule.name.replace(':', "."));
    let name = lang.runs(&Component::translatable(key.clone()), &Style::default());
    let description_key = format!("{key}.description");
    let description = lang
        .get(&description_key)
        .map(|_| lang.runs(&Component::translatable(description_key), &Style::default()));
    let yellow = Style {
        colour: Some(YELLOW),
        ..Style::default()
    };
    let id = vec![Run {
        text: short.to_owned(),
        style: yellow,
    }];
    let default = lang.runs(
        &Component::translatable_with(
            "editGamerule.default",
            vec![Component::literal(rule.default)],
        )
        .with_colour("gray"),
        &Style::default(),
    );
    let mut tooltip = vec![id];
    if let Some(description) = &description {
        tooltip.extend(split(description, DESCRIPTION_WIDTH, font));
    }
    tooltip.push(default);
    let plain_text = |runs: &[Run]| runs.iter().map(|run| run.text.as_str()).collect::<String>();
    let category_key = format!("gamerule.category.minecraft.{}", rule.category);
    Entry {
        rule,
        initial,
        current: initial.unwrap_or_else(|| default_value(rule)),
        lines: split(&name, LABEL_WIDTH, font),
        haystack: [
            short.to_lowercase(),
            plain_text(&name).to_lowercase(),
            lang.text(&category_key).to_lowercase(),
            description
                .as_deref()
                .map(plain_text)
                .unwrap_or_default()
                .to_lowercase(),
        ],
        tooltip,
    }
}

/// The label of each category the entries have: bold and yellow.
fn categories_of(entries: &[Entry], lang: &Lang) -> Vec<(&'static str, Vec<Run>)> {
    let style = Style {
        colour: Some(YELLOW),
        bold: Some(true),
        ..Style::default()
    };
    let mut seen: Vec<(&'static str, Vec<Run>)> = Vec::new();
    for entry in entries {
        if seen.iter().all(|(name, _)| *name != entry.rule.category) {
            let key = format!("gamerule.category.minecraft.{}", entry.rule.category);
            let label = lang.runs(&Component::translatable(key), &style);
            seen.push((entry.rule.category, label));
        }
    }
    seen
}
