//! `KeyBindsList`: a heading for each category, then a row for each of its key mappings with
//! the mapping's name, the button that waits for its key and the button that resets it.

use crate::{FontRuns, FrameRuns};
use photon_data::keys::{InputKey, mappings};
use photon_game::options::key_display_name;
use photon_game::text::{Component, Lang, Run, Style};

use crate::layout::Arena;
use crate::options::list::Nav;
use crate::tooltip::{Holder, Pending};
use crate::widgets::navigation::{Direction, Rectangle};
use crate::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::widgets::slider::InputType;
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `KeyBindsList.ITEM_HEIGHT`.
const ITEM_HEIGHT: i32 = 20;
/// `getRowWidth`.
const ROW_WIDTH: i32 = 340;
/// `FocusableTextWidget.DEFAULT_PADDING`.
const TEXT_PADDING: i32 = 4;
/// `ScrollbarSettings.scrollRate`: half of an item's height.
const SCROLL_RATE: f64 = (ITEM_HEIGHT / 2) as f64;

/// What a row is made of.
pub(super) enum Row {
    /// `CategoryEntry`: the category's name, a text that takes the focus.
    Category { text: Vec<Run>, width: i32 },
    /// `KeyEntry`: the mapping (by its place in `keyMappings`), its name, and its buttons.
    Key {
        mapping: usize,
        name: Vec<Run>,
        change: Widget,
        reset: Widget,
        collision: bool,
        tip: Holder,
    },
}

/// What the rows are refreshed from: each mapping's key and default, the mapping waiting for
/// its key, and the system's names of keys.
pub(super) struct Binds<'a> {
    pub keys: &'a [InputKey],
    pub defaults: &'a [InputKey],
    pub selected: Option<usize>,
    pub names: fn(InputKey) -> Option<String>,
}

/// The widget that has the focus: its row, then its place in it.
pub(super) type Focus = (usize, usize);

pub(super) struct KeyBindsList {
    arena: Arena,
    pub(super) place: Place,
    pub(super) rows: Vec<Row>,
    scroll: f64,
    /// Whether the mouse holds the scroll bar.
    scrolling: bool,
    pub(super) focus: Option<Focus>,
    /// Whether the screen's focus is on the list.
    pub(super) screen_focused: bool,
    pub(super) input: InputType,
}

/// `Arrays.sort` of the mappings (`KeyMapping.compareTo`): by category, then by `order`, then
/// by translated name as Java compares strings.
fn sorted(lang: &Lang) -> Vec<usize> {
    let categories = photon_data::keys::categories();
    let rows: Vec<_> = mappings().collect();
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| {
        let (a, b) = (&rows[a], &rows[b]);
        let category = |name| categories.iter().position(|known| *known == name);
        category(a.category)
            .cmp(&category(b.category))
            .then(a.order.cmp(&b.order))
            .then_with(|| {
                let (a, b) = (lang.text(a.name), lang.text(b.name));
                a.encode_utf16().cmp(b.encode_utf16())
            })
    });
    order
}

impl KeyBindsList {
    /// The list of every mapping, for a screen `width` wide whose list stands `y` down and is
    /// `height` tall.
    pub(super) fn new(
        font: &Font,
        lang: &Lang,
        (width, (y, height)): (i32, (i32, i32)),
        in_world: bool,
        binds: &Binds,
    ) -> Self {
        let all: Vec<_> = mappings().collect();
        let mut arena = Arena::default();
        let mut rows = Vec::new();
        let mut previous = None;
        for mapping in sorted(lang) {
            let row = &all[mapping];
            if previous != Some(row.category) {
                previous = Some(row.category);
                let label = Component::translatable(category_key(row.category));
                let text = lang.runs(&label, &Style::default());
                let width = font.width_of_runs(&text) + TEXT_PADDING * 2;
                rows.push(Row::Category { text, width });
            }
            let button = |arena: &mut Arena, width| Widget {
                rect: arena.add(width, 20),
                active: true,
                kind: Kind::Button { label: Vec::new() },
            };
            let reset_label = lang.runs(
                &Component::translatable("controls.reset"),
                &Style::default(),
            );
            let mut reset = button(&mut arena, 50);
            reset.kind = Kind::Button { label: reset_label };
            rows.push(Row::Key {
                mapping,
                name: lang.runs(&Component::translatable(row.name), &Style::default()),
                change: button(&mut arena, 75),
                reset,
                collision: false,
                tip: Holder::default(),
            });
        }
        let mut list = Self {
            arena,
            place: Place {
                y,
                width,
                height,
                in_world,
            },
            rows,
            scroll: 0.0,
            scrolling: false,
            focus: None,
            screen_focused: false,
            input: InputType::Mouse,
        };
        list.refresh(lang, binds);
        list.reposition();
        list
    }

    /// `refreshEntries`: each row's key, whether it can be reset, whether its key is another's
    /// too (with the tooltip that names those), and the mapping waiting for its key.
    pub(super) fn refresh(&mut self, lang: &Lang, binds: &Binds) {
        let all: Vec<_> = mappings().collect();
        for row in &mut self.rows {
            let Row::Key {
                mapping,
                change,
                reset,
                collision,
                tip,
                ..
            } = row
            else {
                continue;
            };
            let key = binds.keys[*mapping];
            let is_default = |index: usize| binds.keys[index] == binds.defaults[index];
            let mut message = key_display_name(key, (binds.names)(key), lang);
            reset.active = !is_default(*mapping);
            let mut clashes = Component::empty();
            *collision = false;
            if key != InputKey::Keysym(-1) {
                for (other, row) in all.iter().enumerate() {
                    if other != *mapping
                        && binds.keys[other] == key
                        && (!is_default(other) || !is_default(*mapping))
                    {
                        if *collision {
                            clashes = clashes.append(Component::literal(", "));
                        }
                        *collision = true;
                        clashes = clashes.append(Component::translatable(row.name));
                    }
                }
            }
            if *collision {
                message = Component::literal("[ ")
                    .append(message.with_colour("white"))
                    .append(Component::literal(" ]"))
                    .with_colour("yellow");
                let words = Component::translatable_with(
                    "controls.keybinds.duplicateKeybinds",
                    vec![clashes],
                );
                tip.set(Some(lang.runs(&words, &Style::default())));
            } else {
                tip.set(None);
            }
            if binds.selected == Some(*mapping) {
                let marked = Style {
                    colour: photon_game::text::named_colour("white"),
                    underlined: Some(true),
                    ..Style::default()
                };
                message = Component::literal("> ")
                    .append(message.with_style(&marked))
                    .append(Component::literal(" <"))
                    .with_colour("yellow");
            }
            change.kind = Kind::Button {
                label: lang.runs(&message, &Style::default()),
            };
        }
    }

    /// `getRowLeft`.
    fn row_left(&self) -> i32 {
        self.place.width / 2 - ROW_WIDTH / 2
    }

    /// `scrollBarX`.
    fn scrollbar_x(&self) -> i32 {
        self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2
    }

    /// `contentHeight`.
    fn content_height(&self) -> i32 {
        self.rows.len() as i32 * ITEM_HEIGHT + 4
    }

    /// The top of a row (`repositionEntries`).
    fn row_y(&self, row: usize) -> i32 {
        self.place.y + 2 - self.scroll as i32 + row as i32 * ITEM_HEIGHT
    }

    pub(super) fn scroll_amount(&self) -> f64 {
        self.scroll
    }

    /// `setScrollAmount`: kept between the top and the end.
    pub(super) fn set_scroll_amount(&mut self, scroll: f64) {
        let max = self.place.max_scroll(self.content_height());
        self.scroll = scroll.clamp(0.0, f64::from(max));
        self.reposition();
    }

    /// Where `KeyEntry.extractContent` puts the buttons: Reset left of the scroll bar, the key's
    /// button left of that.
    fn reposition(&mut self) {
        let reset_x = self.scrollbar_x() - 50 - 10;
        let change_x = reset_x - 5 - 75;
        for index in 0..self.rows.len() {
            let y = self.row_y(index);
            if let Row::Key { change, reset, .. } = &self.rows[index] {
                let (change, reset) = (change.rect, reset.rect);
                (self.arena.0[reset].x, self.arena.0[reset].y) = (reset_x, y);
                (self.arena.0[change].x, self.arena.0[change].y) = (change_x, y);
            }
        }
    }

    /// `updateSizeAndPosition`.
    pub(super) fn resize(&mut self, width: i32, (y, height): (i32, i32)) {
        (self.place.width, self.place.y, self.place.height) = (width, y, height);
        self.reposition();
        if let Some((row, _)) = self.focus {
            self.scroll_to_row(row);
        }
        self.set_scroll_amount(self.scroll);
    }

    /// `scrollToEntry`.
    fn scroll_to_row(&mut self, row: usize) {
        let top_delta = self.row_y(row) - self.place.y - 2;
        if top_delta < 0 {
            self.set_scroll_amount(self.scroll + f64::from(top_delta));
        }
        let bottom_delta = self.place.bottom() - self.row_y(row) - ITEM_HEIGHT - 2;
        if bottom_delta < 0 {
            self.set_scroll_amount(self.scroll - f64::from(bottom_delta));
        }
    }

    /// A category's text: where it stands (centred, at the foot of its row) and how big.
    fn category_rect(&self, row: usize, width: i32) -> Rectangle {
        Rectangle {
            x: self.place.width / 2 - width / 2,
            y: self.row_y(row) + 2 + ITEM_HEIGHT - 4 - (9 + TEXT_PADDING * 2),
            width,
            height: 9 + TEXT_PADDING * 2,
        }
    }

    /// The list's background, the rows cut to it, the lines above and below, then the scroll
    /// bar; the tooltip a button asks for, to draw over everything.
    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
    ) -> Option<Pending> {
        let place = self.place;
        place.draw_background(frame, atlas, self.scroll);
        let inside = place.contains((f64::from(mouse.0), f64::from(mouse.1)));
        let mouse = if inside { mouse } else { (i32::MIN, i32::MIN) };
        let mark = frame.mark();
        let mut pending = None;
        for (index, row) in self.rows.iter().enumerate() {
            let y = self.row_y(index);
            if y + ITEM_HEIGHT < place.y || y > place.bottom() {
                continue;
            }
            let focused = |child| self.focus == Some((index, child));
            match row {
                Row::Category { text, width } => {
                    let rect = self.category_rect(index, *width);
                    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
                    if focused(0) {
                        frame.fill(atlas, (x + 1, y, x + w, y + h), 0xFF00_0000);
                        // `outline`: top, bottom, left, right.
                        for side in [
                            (x, y, x + w, y + 1),
                            (x, y + h - 1, x + w, y + h),
                            (x, y + 1, x + 1, y + h - 1),
                            (x + w - 1, y + 1, x + w, y + h - 1),
                        ] {
                            frame.fill(atlas, side, 0xFFFF_FFFF);
                        }
                    }
                    let text_x = x + w / 2 - font.width_of_runs(text) / 2;
                    frame.styled_text(
                        atlas,
                        font,
                        text,
                        text_x,
                        y + TEXT_PADDING,
                        0xFFFF_FFFF,
                        true,
                    );
                }
                Row::Key {
                    name,
                    change,
                    reset,
                    collision,
                    tip,
                    ..
                } => {
                    for (child, widget) in [(1, reset), (0, change)] {
                        let lit = widget.contains(&self.arena, mouse) || focused(child);
                        widget.draw(frame, atlas, font, &self.arena, lit);
                    }
                    let name_x = self.row_left() + 2;
                    frame.styled_text(atlas, font, name, name_x, y + 6, 0xFFFF_FFFF, true);
                    let change_rect = self.arena.0[change.rect];
                    if *collision {
                        let left = change_rect.x - 6;
                        frame.fill(atlas, (left, y + 1, left + 3, y + 18), 0xFFFF_FF00);
                    }
                    let widget = Rectangle {
                        x: change_rect.x,
                        y: change_rect.y,
                        width: change_rect.width,
                        height: change_rect.height,
                    };
                    let hovered = change.contains(&self.arena, mouse);
                    if let Some(asked) =
                        tip.refresh(frame.millis, (hovered, focused(0)), self.input, widget)
                        && (pending.is_none() || focused(0))
                    {
                        pending = Some(asked);
                    }
                }
            }
        }
        frame.clip_from(mark, (0, place.y, place.width, place.bottom()));
        place.draw_edges(
            frame,
            atlas,
            (self.scroll, self.content_height()),
            self.scrollbar_x(),
        );
        pending
    }

    /// `getEntryAtPosition`.
    fn row_at(&self, (x, y): (f64, f64)) -> Option<usize> {
        let (x, y) = (x as i32, y as i32);
        let left = self.row_left();
        (0..self.rows.len()).find(|&row| {
            let top = self.row_y(row);
            x >= left && x < left + ROW_WIDTH && y >= top && y < top + ITEM_HEIGHT
        })
    }

    /// Whether a widget of a row is on the point and would take a press.
    fn widget_at(&self, row: usize, (x, y): (f64, f64)) -> Option<usize> {
        let at = (x as i32, y as i32);
        match &self.rows[row] {
            Row::Category { width, .. } => {
                let rect = self.category_rect(row, *width);
                let inside = at.0 >= rect.x
                    && at.1 >= rect.y
                    && at.0 < rect.x + rect.width
                    && at.1 < rect.y + rect.height;
                inside.then_some(0)
            }
            Row::Key { change, reset, .. } => [change, reset]
                .iter()
                .position(|widget| widget.is_active() && widget.contains(&self.arena, at)),
        }
    }

    /// A press of the primary button: whether the list took it, and the widget it pressed.
    pub(super) fn clicked(&mut self, mouse: (f64, f64)) -> (bool, Option<Focus>) {
        if !self.place.contains(mouse) {
            return (false, None);
        }
        let bar = f64::from(self.scrollbar_x());
        self.scrolling = self.place.max_scroll(self.content_height()) > 0
            && mouse.0 >= bar
            && mouse.0 <= bar + f64::from(SCROLLBAR_WIDTH)
            && mouse.1 >= f64::from(self.place.y)
            && mouse.1 < f64::from(self.place.bottom());
        let hit = self
            .row_at(mouse)
            .and_then(|row| self.widget_at(row, mouse).map(|child| (row, child)));
        let Some(hit) = hit else {
            let took = self.scrolling || self.row_at(mouse).is_some();
            self.screen_focused |= took;
            return (took, None);
        };
        self.screen_focused = true;
        self.input = InputType::Mouse;
        self.focus_on(Some(hit), InputType::Mouse);
        (true, Some(hit))
    }

    /// The mouse moved, by `dy` down, with the primary button down: the bar it holds follows.
    pub(super) fn dragged(&mut self, (_, y): (f64, f64), dy: f64) {
        if self.scrolling {
            let to = self
                .place
                .dragged_scroll(self.scroll, y, dy, self.content_height());
            self.set_scroll_amount(to);
        }
    }

    pub(super) fn released(&mut self) {
        self.scrolling = false;
    }

    /// `mouseScrolled`.
    pub(super) fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        if self.place.contains(mouse) {
            self.set_scroll_amount(self.scroll - scroll_y * SCROLL_RATE);
        }
    }

    /// The list's own rectangle, which arrow navigation looks at.
    pub(super) fn rectangle(&self) -> Rectangle {
        Rectangle {
            x: 0,
            y: self.place.y,
            width: self.place.width,
            height: self.place.height,
        }
    }
}

mod focus;

/// `Category.label`: `key.category.<namespace>.<path>`.
fn category_key(id: &str) -> String {
    format!("key.category.{}", id.replace(':', "."))
}
