//! The settings screens' shell (`OptionsSubScreen`) and the scrolling list of
//! options in it (`OptionsList`): a title over the list, Done under it, and
//! in the list headers and rows of one wide or two narrow option widgets.

use std::any::Any;

use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::cycle::{CycleButton, Tip};
use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, Settings, place_in_frame};
use crate::gui::navigation::{Candidate, Direction, Rectangle, next_in_direction};
use crate::gui::option_text::on_off;
use crate::gui::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::gui::slider::{InputType, OptionSlider, SliderKey, SliderTip, SliderValues};
use crate::gui::tooltip::{Holder, Pending};
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame};

/// `HeaderAndFooterLayout(this)`: the default header and footer heights.
const HEADER_AND_FOOTER: i32 = 33;
/// `OptionsList.DEFAULT_ITEM_HEIGHT`.
const ITEM_HEIGHT: i32 = 25;
/// `getRowWidth`, and the width of a wide widget.
const ROW_WIDTH: i32 = 310;
/// The width of each of the two narrow widgets, and the step between them.
const SMALL_WIDTH: i32 = 150;
const COLUMN_STEP: i32 = 160;
/// `Entry.CONTENT_PADDING`.
const PADDING: i32 = 2;
/// `ScrollbarSettings.scrollRate`: half of an item's height.
const SCROLL_RATE: f64 = (ITEM_HEIGHT / 2) as f64;

/// A widget of an option, whatever it is made of.
pub trait Control {
    /// `isMouseOver`: the widget is active and the mouse is on it.
    fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool;
    /// The press of the primary button that landed on the widget: whether
    /// the option's value changed.
    fn pressed(&mut self, lang: &Lang, arena: &Arena, mouse: (f64, f64), shift: bool) -> bool;
    /// The mouse moved with the primary button down: whether the option's
    /// value changed.
    fn dragged(&mut self, lang: &Lang, arena: &Arena, x: f64) -> bool;
    /// The primary button let go.
    fn released(&mut self, lang: &Lang);
    fn set_focused(&mut self, focused: bool, input: InputType);
    fn set_active(&mut self, active: bool);
    /// `active`: an inactive widget takes no focus.
    fn is_active(&self) -> bool;
    /// A key the focused widget is given (`keyPressed`): whether it took it,
    /// and whether the option's value changed.
    fn key_pressed(
        &mut self,
        lang: &Lang,
        arena: &Arena,
        key: SliderKey,
        shift: bool,
    ) -> (bool, bool);
    fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    );
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// The widget's tooltip, if it has one.
    fn tooltip(&self) -> Option<&Holder> {
        None
    }
}

impl<S: SliderValues + 'static> Control for OptionSlider<S>
where
    S::Value: 'static,
{
    fn tooltip(&self) -> Option<&Holder> {
        Some(OptionSlider::tooltip(self))
    }

    fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool {
        OptionSlider::is_mouse_over(self, arena, mouse)
    }

    fn pressed(&mut self, lang: &Lang, arena: &Arena, mouse: (f64, f64), _shift: bool) -> bool {
        self.clicked(lang, arena, mouse).is_some()
    }

    fn dragged(&mut self, lang: &Lang, arena: &Arena, x: f64) -> bool {
        OptionSlider::dragged(self, lang, arena, x).is_some()
    }

    fn released(&mut self, lang: &Lang) {
        OptionSlider::released(self, lang);
    }

    fn set_focused(&mut self, focused: bool, input: InputType) {
        OptionSlider::set_focused(self, focused, input);
    }

    fn set_active(&mut self, active: bool) {
        OptionSlider::set_active(self, active);
    }

    fn is_active(&self) -> bool {
        OptionSlider::is_active(self)
    }

    fn key_pressed(
        &mut self,
        lang: &Lang,
        arena: &Arena,
        key: SliderKey,
        _shift: bool,
    ) -> (bool, bool) {
        let (taken, value) = OptionSlider::key_pressed(self, lang, arena, key);
        (taken, value.is_some())
    }

    fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    ) {
        OptionSlider::draw(self, frame, atlas, font, arena, mouse);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<V: Clone + PartialEq + 'static> Control for CycleButton<V> {
    fn tooltip(&self) -> Option<&Holder> {
        Some(CycleButton::tooltip(self))
    }

    fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool {
        CycleButton::is_mouse_over(self, arena, mouse)
    }

    fn pressed(&mut self, lang: &Lang, _arena: &Arena, _mouse: (f64, f64), shift: bool) -> bool {
        self.press(lang, shift);
        true
    }

    fn dragged(&mut self, _lang: &Lang, _arena: &Arena, _x: f64) -> bool {
        false
    }

    fn released(&mut self, _lang: &Lang) {}

    fn set_focused(&mut self, focused: bool, _input: InputType) {
        CycleButton::set_focused(self, focused);
    }

    fn set_active(&mut self, active: bool) {
        CycleButton::set_active(self, active);
    }

    fn is_active(&self) -> bool {
        CycleButton::is_active(self)
    }

    fn key_pressed(
        &mut self,
        lang: &Lang,
        _arena: &Arena,
        key: SliderKey,
        shift: bool,
    ) -> (bool, bool) {
        // `AbstractButton.keyPressed`: Enter and Space press it.
        let pressed = key == SliderKey::Select && self.selected(lang, shift).is_some();
        (pressed, pressed)
    }

    fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    ) {
        CycleButton::draw(self, frame, atlas, font, arena, mouse);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Makes the widget for the place `rect` of the list's arena.
pub type Build = Box<dyn FnOnce(&Lang, usize) -> Box<dyn Control>>;

/// A widget of an option and what a screen calls it by, to find it again
/// (`findOption`).
struct Child {
    tag: u8,
    rect: usize,
    control: Box<dyn Control>,
}

enum Content {
    /// `HeaderEntry`: a line of text and the space above it.
    Header { text: Widget, padding_top: i32 },
    /// `Entry`: one or two option widgets.
    Row(Vec<Child>),
}

struct Entry {
    y: i32,
    height: i32,
    content: Content,
}

/// The widget that has the focus: its entry, then its place in it.
type Focus = (usize, usize);

/// `FocusNavigationEvent`: Tab (forward or back) or an arrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    Tab(bool),
    Arrow(Direction),
}

/// `OptionsList`.
pub struct OptionsList {
    arena: Arena,
    place: Place,
    entries: Vec<Entry>,
    scroll: f64,
    /// Whether the mouse holds the scroll bar.
    scrolling: bool,
    focus: Option<Focus>,
    /// Whether the screen's focus is on the list, which a click on an entry
    /// gives it without any widget of it taking the focus.
    screen_focused: bool,
    /// Whether the mouse holds the focused widget (`isDragging`).
    dragging: bool,
    /// What the window last took in.
    input: InputType,
}

impl OptionsList {
    /// A list with no entries, for a screen `width` wide whose list stands
    /// `y` down and is `height` tall.
    pub fn new(width: i32, (y, height): (i32, i32), in_world: bool) -> Self {
        Self {
            arena: Arena::default(),
            place: Place {
                y,
                width,
                height,
                in_world,
            },
            entries: Vec::new(),
            scroll: 0.0,
            scrolling: false,
            focus: None,
            screen_focused: false,
            dragging: false,
            input: InputType::Mouse,
        }
    }

    /// `addHeader`: a line of text, with room above it unless it is first.
    pub fn add_header(&mut self, font: &Font, lang: &Lang, key: &str) {
        let padding_top = if self.entries.is_empty() { 0 } else { 9 * 2 };
        let text = lang.runs(&Component::translatable(key), &Style::default());
        let rect = self.arena.add(font.width_of_runs(&text), 9);
        self.push(Entry {
            y: 0,
            height: padding_top + 9 + 4,
            content: Content::Header {
                text: Widget {
                    rect,
                    active: false,
                    kind: Kind::Text { text },
                },
                padding_top,
            },
        });
    }

    /// `addBig`: one wide widget.
    pub fn add_big(&mut self, lang: &Lang, tag: u8, build: Build) {
        let rect = self.arena.add(ROW_WIDTH, 20);
        let control = build(lang, rect);
        self.push(Entry {
            y: 0,
            height: ITEM_HEIGHT,
            content: Content::Row(vec![Child { tag, rect, control }]),
        });
    }

    /// `addSmall`: the widgets two to a row, the last row with one if they
    /// are odd.
    pub fn add_small(&mut self, lang: &Lang, builds: Vec<(u8, Build)>) {
        let mut builds = builds.into_iter();
        while let Some((tag, build)) = builds.next() {
            let mut children = Vec::new();
            for (tag, build) in std::iter::once((tag, build)).chain(builds.next()) {
                let rect = self.arena.add(SMALL_WIDTH, 20);
                children.push(Child {
                    tag,
                    rect,
                    control: build(lang, rect),
                });
            }
            self.push(Entry {
                y: 0,
                height: ITEM_HEIGHT,
                content: Content::Row(children),
            });
        }
    }

    fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
        self.reposition();
    }

    /// `findOption`: the widget an option was added with.
    pub fn find(&self, tag: u8) -> Option<&dyn Control> {
        self.children()
            .find(|child| child.tag == tag)
            .map(|child| &*child.control)
    }

    pub fn find_mut(&mut self, tag: u8) -> Option<&mut dyn Control> {
        for entry in &mut self.entries {
            if let Content::Row(children) = &mut entry.content {
                for child in children {
                    if child.tag == tag {
                        return Some(&mut *child.control);
                    }
                }
            }
        }
        None
    }

    fn children(&self) -> impl Iterator<Item = &Child> {
        self.entries.iter().flat_map(|entry| match &entry.content {
            Content::Row(children) => children.iter(),
            Content::Header { .. } => [].iter(),
        })
    }

    /// `getRowLeft`.
    fn row_left(&self) -> i32 {
        self.place.width / 2 - ROW_WIDTH / 2
    }

    /// `contentHeight`.
    fn max_scroll(&self) -> i32 {
        self.place.max_scroll(self.content_height())
    }

    /// `contentHeight`.
    fn content_height(&self) -> i32 {
        self.entries.iter().map(|entry| entry.height).sum::<i32>() + 4
    }

    pub fn scroll_amount(&self) -> f64 {
        self.scroll
    }

    /// `setScrollAmount`: kept between the top and the end, the entries
    /// moved with it (the widgets with them).
    pub fn set_scroll_amount(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, f64::from(self.max_scroll()));
        self.reposition();
    }

    /// `repositionEntries`, and the widgets where `extractContent` puts them.
    fn reposition(&mut self) {
        let mut y = self.place.y + PADDING - self.scroll as i32;
        let x = self.place.width / 2 - 155;
        for entry in &mut self.entries {
            entry.y = y;
            y += entry.height;
            match &entry.content {
                Content::Header { text, padding_top } => {
                    let rect = &mut self.arena.0[text.rect];
                    (rect.x, rect.y) = (x, entry.y + PADDING + padding_top);
                }
                Content::Row(children) => {
                    for (index, child) in children.iter().enumerate() {
                        let rect = &mut self.arena.0[child.rect];
                        (rect.x, rect.y) = (x + COLUMN_STEP * index as i32, entry.y + PADDING);
                    }
                }
            }
        }
    }

    /// `updateSizeAndPosition`: a screen of another size.
    pub fn resize(&mut self, width: i32, (y, height): (i32, i32)) {
        self.place.width = width;
        self.place.y = y;
        self.place.height = height;
        self.reposition();
        if let Some((entry, _)) = self.focus {
            self.scroll_to_entry(entry);
        }
        self.set_scroll_amount(self.scroll);
    }

    /// `scrollToEntry`.
    fn scroll_to_entry(&mut self, entry: usize) {
        let top_delta = self.entries[entry].y - self.place.y - PADDING;
        if top_delta < 0 {
            self.set_scroll_amount(self.scroll + f64::from(top_delta));
        }
        let bottom_delta =
            self.place.bottom() - self.entries[entry].y - self.entries[entry].height - PADDING;
        if bottom_delta < 0 {
            self.set_scroll_amount(self.scroll - f64::from(bottom_delta));
        }
    }

    /// `scrollBarX`.
    fn scrollbar_x(&self) -> i32 {
        self.row_left() + ROW_WIDTH + SCROLLBAR_WIDTH + 2
    }

    /// The menu list's background, scrolled with the list, the widgets in
    /// it cut to the list, the lines above and below, then the scroll bar.
    /// The tooltip a widget asks for, to draw over everything.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
    ) -> Option<Pending> {
        let place = self.place;
        place.draw_background(frame, atlas, self.scroll);
        // The hover of a widget needs the mouse inside the cut.
        let inside = place.contains((f64::from(mouse.0), f64::from(mouse.1)));
        let mouse = if inside { mouse } else { (i32::MIN, i32::MIN) };
        let mark = frame.mark();
        let mut pending: Option<Pending> = None;
        for (entry_index, entry) in self.entries.iter().enumerate() {
            if entry.y + entry.height < place.y || entry.y > place.bottom() {
                continue;
            }
            match &entry.content {
                Content::Header { text, .. } => text.draw(frame, atlas, font, &self.arena, false),
                Content::Row(children) => {
                    for (child_index, child) in children.iter().enumerate() {
                        child.control.draw(frame, atlas, font, &self.arena, mouse);
                        let Some(holder) = child.control.tooltip() else {
                            continue;
                        };
                        let rect = self.arena.0[child.rect];
                        let hovered = inside
                            && mouse.0 >= rect.x
                            && mouse.0 < rect.x + rect.width
                            && mouse.1 >= rect.y
                            && mouse.1 < rect.y + rect.height;
                        let focused = self.focus == Some((entry_index, child_index));
                        let widget = Rectangle {
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height,
                        };
                        if let Some(asked) =
                            holder.refresh(frame.millis, (hovered, focused), self.input, widget)
                            && (pending.is_none() || focused)
                        {
                            pending = Some(asked);
                        }
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
    fn entry_at(&self, (x, y): (f64, f64)) -> Option<usize> {
        let (x, y) = (x as i32, y as i32);
        let left = self.row_left();
        self.entries.iter().position(|entry| {
            x >= left && x < left + ROW_WIDTH && y >= entry.y && y < entry.y + entry.height
        })
    }

    /// `ContainerEventHandler.setFocused` down the list: the widget that had
    /// the focus loses it, and an entry that sticks out of the list is
    /// scrolled into it (`setSelected`).
    fn focus_on(&mut self, focus: Option<Focus>, input: InputType) {
        let old = self.focus;
        if old == focus {
            return;
        }
        if let Some((entry, child)) = old
            && let Content::Row(children) = &mut self.entries[entry].content
        {
            children[child].control.set_focused(false, input);
        }
        self.focus = focus;
        if let Some((entry, child)) = focus {
            if let Content::Row(children) = &mut self.entries[entry].content {
                children[child].control.set_focused(true, input);
            }
            let changed_entry = !matches!(old, Some((before, _)) if before == entry);
            let top = self.entries[entry].y + PADDING;
            let bottom = self.entries[entry].y + self.entries[entry].height - PADDING;
            let keyboard = matches!(input, InputType::KeyboardArrow | InputType::KeyboardTab);
            if changed_entry && (keyboard || top < self.place.y || bottom > self.place.bottom()) {
                self.scroll_to_entry(entry);
            }
        }
    }

    /// Whether the screen's focus is on the list.
    pub fn screen_focused(&self) -> bool {
        self.screen_focused
    }

    /// How many widgets an entry can give the focus to (a header's one is
    /// a text, which never takes it).
    fn widgets_in(&self, entry: usize) -> usize {
        match &self.entries[entry].content {
            Content::Row(children) => children.len(),
            Content::Header { .. } => 0,
        }
    }

    fn is_active(&self, (entry, child): Focus) -> bool {
        matches!(&self.entries[entry].content,
            Content::Row(children) if children[child].control.is_active())
    }

    /// The first active widget of an entry among `order`.
    fn first_active(&self, entry: usize, order: impl Iterator<Item = usize>) -> Option<Focus> {
        order
            .map(|child| (entry, child))
            .find(|&at| self.is_active(at))
    }

    /// `ContainerObjectSelectionList.nextFocusPath`: where the focus would
    /// go, given the widget that has it (if any).
    pub fn next_focus(&self, nav: Nav) -> Option<Focus> {
        if self.entries.is_empty() {
            return None;
        }
        match nav {
            Nav::Tab(forward) => self.tab_focus(forward),
            Nav::Arrow(direction) => self.arrow_focus(direction),
        }
    }

    /// Tab: on along the entry's widgets, then the entries after (or
    /// before) it, a header having none.
    fn tab_focus(&self, forward: bool) -> Option<Focus> {
        if let Some((entry, child)) = self.focus {
            let count = self.widgets_in(entry);
            let found = if forward {
                self.first_active(entry, child + 1..count)
            } else {
                self.first_active(entry, (0..child).rev())
            };
            if found.is_some() {
                return found;
            }
        }
        let entries: Vec<usize> = match (self.focus, forward) {
            (Some((entry, _)), true) => (entry + 1..self.entries.len()).collect(),
            (Some((entry, _)), false) => (0..entry).rev().collect(),
            (None, true) => (0..self.entries.len()).collect(),
            (None, false) => (0..self.entries.len()).rev().collect(),
        };
        entries.into_iter().find_map(|entry| {
            let count = self.widgets_in(entry);
            if forward {
                self.first_active(entry, 0..count)
            } else {
                self.first_active(entry, (0..count).rev())
            }
        })
    }

    /// An arrow: left and right along the entry; up and down to the entry
    /// above or below, at the same place in it (or the last one it has).
    fn arrow_focus(&self, direction: Direction) -> Option<Focus> {
        let horizontal = matches!(direction, Direction::Left | Direction::Right);
        if let (true, Some((entry, child))) = (horizontal, self.focus) {
            let count = self.widgets_in(entry);
            return if direction == Direction::Right {
                self.first_active(entry, child + 1..count)
            } else {
                self.first_active(entry, (0..child).rev())
            };
        }
        // With no focus left enters at the end of the first entry, right at
        // its start.
        let (index, direction) = match (self.focus, direction) {
            (Some((_, child)), direction) => (child, direction),
            (None, Direction::Left) => (usize::MAX, Direction::Down),
            (None, Direction::Right) => (0, Direction::Down),
            (None, direction) => (0, direction),
        };
        let down = direction == Direction::Down;
        let mut entry = self.focus.map(|(entry, _)| entry);
        loop {
            entry = match (entry, down) {
                (None, true) => Some(0),
                (None, false) => self.entries.len().checked_sub(1),
                (Some(entry), true) => Some(entry + 1),
                (Some(entry), false) => entry.checked_sub(1),
            }
            .filter(|entry| *entry < self.entries.len());
            let entry = entry?;
            let count = self.widgets_in(entry);
            if count > 0 {
                let at = (entry, index.min(count - 1));
                if self.is_active(at) {
                    return Some(at);
                }
            }
        }
    }

    /// A key for the widget that has the focus: whether it took it, and the
    /// tag of the widget whose option changed.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> (bool, Option<u8>) {
        let Some((entry, child)) = self.focus else {
            return (false, None);
        };
        let Content::Row(children) = &mut self.entries[entry].content else {
            return (false, None);
        };
        let child = &mut children[child];
        let (taken, changed) = child.control.key_pressed(lang, &self.arena, key, shift);
        (taken, changed.then_some(child.tag))
    }

    /// The focus moves by a key: the widget before loses it, the new one
    /// takes it and the list shows it.
    pub fn move_focus(&mut self, focus: Option<Focus>, input: InputType) {
        self.focus_on(None, input);
        self.focus_on(focus, input);
        self.screen_focused = focus.is_some();
        self.input = input;
    }

    /// The list's own rectangle, which arrow navigation looks at.
    pub fn rectangle(&self) -> Rectangle {
        Rectangle {
            x: 0,
            y: self.place.y,
            width: self.place.width,
            height: self.place.height,
        }
    }

    /// A press of the primary button at `mouse`: whether the list took it
    /// (it landed on a widget or the scroll bar), and the tag of the widget
    /// whose option changed.
    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> (bool, Option<u8>) {
        if !self.place.contains(mouse) {
            return (false, None);
        }
        self.scrolling = self.max_scroll() > 0
            && mouse.0 >= f64::from(self.scrollbar_x())
            && mouse.0 <= f64::from(self.scrollbar_x() + SCROLLBAR_WIDTH)
            && mouse.1 >= f64::from(self.place.y)
            && mouse.1 < f64::from(self.place.bottom());
        let hit = self.entry_at(mouse).and_then(|entry| {
            let Content::Row(children) = &self.entries[entry].content else {
                return None;
            };
            children
                .iter()
                .position(|child| child.control.is_mouse_over(&self.arena, mouse))
                .map(|child| (entry, child))
        });
        let Some((entry, child)) = hit else {
            // An entry (or the bar) takes the click, so the list takes the
            // screen's focus, though no widget of it has taken any.
            let took = self.scrolling || self.entry_at(mouse).is_some();
            self.screen_focused |= took;
            return (took, None);
        };
        self.screen_focused = true;
        let mut changed = None;
        if let Content::Row(children) = &mut self.entries[entry].content {
            let child = &mut children[child];
            if child.control.pressed(lang, &self.arena, mouse, shift) {
                changed = Some(child.tag);
            }
        }
        self.input = InputType::Mouse;
        self.focus_on(Some((entry, child)), InputType::Mouse);
        self.dragging = true;
        (true, changed)
    }

    /// The mouse moved to `(x, y)`, by `dy` down, with the primary button
    /// down.
    pub fn dragged(&mut self, lang: &Lang, (x, y): (f64, f64), dy: f64) -> Option<u8> {
        if self.scrolling {
            let to = self
                .place
                .dragged_scroll(self.scroll, y, dy, self.content_height());
            self.set_scroll_amount(to);
        }
        if self.dragging
            && let Some((entry, child)) = self.focus
            && let Content::Row(children) = &mut self.entries[entry].content
        {
            let child = &mut children[child];
            if child.control.dragged(lang, &self.arena, x) {
                return Some(child.tag);
            }
        }
        None
    }

    /// The primary button let go.
    pub fn released(&mut self, lang: &Lang) {
        self.scrolling = false;
        if self.dragging {
            self.dragging = false;
            if let Some((entry, child)) = self.focus
                && let Content::Row(children) = &mut self.entries[entry].content
            {
                children[child].control.released(lang);
            }
        }
    }

    /// `AbstractScrollArea.mouseScrolled`: the wheel moves the list the
    /// mouse is over.
    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) -> bool {
        if !self.place.contains(mouse) {
            return false;
        }
        self.set_scroll_amount(self.scroll - scroll_y * SCROLL_RATE);
        true
    }
}

/// Builders of the widgets the settings lists are made of.
/// The value of a cycle button of a few named values: the key of its name.
pub type Named = &'static str;

pub fn translated(key: &Named) -> Component {
    Component::translatable(*key)
}

pub fn toggle(value: &bool) -> Component {
    on_off(*value)
}

pub fn counted(key: &str, value: i32) -> Component {
    Component::translatable_with(key, vec![Component::literal(value.to_string())])
}

/// `CommonComponents.optionStatus(caption, false)`.
pub fn off_composed(caption: &Component) -> Component {
    Component::translatable_with("options.off.composed", vec![caption.clone()])
}

/// A cycle button of an option.
pub fn cycle<V: Clone + PartialEq + 'static>(
    key: &'static str,
    values: Vec<V>,
    text: fn(&V) -> Component,
    initial: V,
) -> Build {
    Box::new(move |lang, rect| {
        Box::new(CycleButton::new(
            lang,
            rect,
            Component::translatable(key),
            values,
            text,
            &initial,
        ))
    })
}

pub fn boolean(key: &'static str, initial: bool) -> Build {
    cycle(key, vec![true, false], toggle, initial)
}

pub fn named(key: &'static str, values: &[Named], initial: Named) -> Build {
    cycle(key, values.to_vec(), translated, initial)
}

/// A slider of an option.
pub fn slider<S>(
    key: &'static str,
    values: S,
    text: fn(&Component, &S::Value) -> Component,
    initial: S::Value,
) -> Build
where
    S: SliderValues + 'static,
    S::Value: 'static,
{
    Box::new(move |lang, rect| {
        Box::new(OptionSlider::new(
            lang,
            rect,
            values,
            Component::translatable(key),
            text,
            initial,
        ))
    })
}

/// The lines of a tooltip that says one thing whatever the value
/// (`cachedConstantTooltip`).
pub fn constant_tip<V>(key: &'static str) -> Tip<V> {
    Box::new(move |_| vec![Some(Component::translatable(key))])
}

/// A cycle button of an option with a tooltip.
pub fn cycle_tip<V: Clone + PartialEq + 'static>(
    key: &'static str,
    values: Vec<V>,
    text: fn(&V) -> Component,
    initial: V,
    tip: Tip<V>,
) -> Build {
    Box::new(move |lang, rect| {
        Box::new(
            CycleButton::new(
                lang,
                rect,
                Component::translatable(key),
                values,
                text,
                &initial,
            )
            .with_tip(lang, tip),
        )
    })
}

/// A cycle button of on and off with a tooltip that says one thing.
pub fn boolean_tip(key: &'static str, initial: bool, tip: &'static str) -> Build {
    cycle_tip(key, vec![true, false], toggle, initial, constant_tip(tip))
}

/// A cycle button of named values whose tooltip is the value's own
/// (`<value>.tooltip`).
pub fn named_tip(key: &'static str, values: &[Named], initial: Named) -> Build {
    cycle_tip(
        key,
        values.to_vec(),
        translated,
        initial,
        Box::new(|value: &Named| vec![Some(Component::translatable(format!("{value}.tooltip")))]),
    )
}

/// A slider of an option with a tooltip.
pub fn slider_tip<S>(
    key: &'static str,
    values: S,
    text: fn(&Component, &S::Value) -> Component,
    initial: S::Value,
    tip: SliderTip<S::Value>,
) -> Build
where
    S: SliderValues + 'static,
    S::Value: 'static,
{
    Box::new(move |lang, rect| {
        Box::new(
            OptionSlider::new(
                lang,
                rect,
                values,
                Component::translatable(key),
                text,
                initial,
            )
            .with_tip(lang, tip),
        )
    })
}

/// A button of the list that opens another screen (`Button` in an
/// `OptionsList`): a press is its "change".
pub struct ButtonControl {
    widget: Widget,
    focused: bool,
}

impl Control for ButtonControl {
    fn is_mouse_over(&self, arena: &Arena, mouse: (f64, f64)) -> bool {
        self.widget.is_active()
            && self
                .widget
                .contains(arena, (mouse.0 as i32, mouse.1 as i32))
    }

    fn pressed(&mut self, _lang: &Lang, _arena: &Arena, _mouse: (f64, f64), _shift: bool) -> bool {
        true
    }

    fn dragged(&mut self, _lang: &Lang, _arena: &Arena, _x: f64) -> bool {
        false
    }

    fn released(&mut self, _lang: &Lang) {}

    fn set_focused(&mut self, focused: bool, _input: InputType) {
        self.focused = focused;
    }

    fn set_active(&mut self, active: bool) {
        self.widget.active = active;
    }

    fn is_active(&self) -> bool {
        self.widget.is_active()
    }

    fn key_pressed(
        &mut self,
        _lang: &Lang,
        _arena: &Arena,
        key: SliderKey,
        _shift: bool,
    ) -> (bool, bool) {
        let pressed = key == SliderKey::Select && self.widget.is_active();
        (pressed, pressed)
    }

    fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        mouse: (i32, i32),
    ) {
        let highlighted = self.widget.contains(arena, mouse) || self.focused;
        self.widget.draw(frame, atlas, font, arena, highlighted);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A button with a label that a press of is reported to the screen.
pub fn button(key: &'static str) -> Build {
    Box::new(move |lang, rect| {
        Box::new(ButtonControl {
            widget: Widget {
                rect,
                active: true,
                kind: Kind::Button {
                    label: lang.runs(&Component::translatable(key), &Style::default()),
                },
            },
            focused: false,
        })
    })
}

/// What an input to a settings screen asks for.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SubScreenInput {
    /// "Done", or Escape: back to the screen before.
    pub done: bool,
    /// The tag of the widget whose option changed.
    pub changed: Option<u8>,
}

/// `OptionsSubScreen`: the title, the list and Done.
pub struct SubScreen {
    arena: Arena,
    title: Widget,
    /// A second line in the header, and whether it is shown: once it has
    /// been added it keeps its place.
    warning: Option<(Widget, bool)>,
    done: Widget,
    pub list: OptionsList,
    /// Whether Done has the focus (the list keeps its own).
    done_focused: bool,
    size: (i32, i32),
    in_world: bool,
}

/// Where the focus goes on the screen: a widget of the list, or Done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    List(Focus),
    Done,
}

impl SubScreen {
    /// `init` of a screen `size` big with the title `key` and an empty
    /// list, which `fill` adds the options to.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        key: &str,
        in_world: bool,
        fill: impl FnOnce(&mut OptionsList),
    ) -> Self {
        let (width, height) = size;
        let mut arena = Arena::default();
        let text = lang.runs(&Component::translatable(key), &Style::default());
        let title = Widget {
            rect: arena.add(font.width_of_runs(&text), 9),
            active: false,
            kind: Kind::Text { text },
        };
        let done_label: Vec<Run> =
            lang.runs(&Component::translatable("gui.done"), &Style::default());
        let done = Widget {
            rect: arena.add(200, 20),
            active: true,
            kind: Kind::Button { label: done_label },
        };
        let mut screen = Self {
            arena,
            title,
            warning: None,
            done,
            list: OptionsList::new(
                width,
                (HEADER_AND_FOOTER, height - 2 * HEADER_AND_FOOTER),
                in_world,
            ),
            done_focused: false,
            size,
            in_world,
        };
        fill(&mut screen.list);
        screen.arrange();
        screen
    }

    /// `repositionElements`: the title in the header, Done in the footer,
    /// the list between.
    fn arrange(&mut self) {
        let (width, height) = self.size;
        // The header: the title, and the line under it if there is one.
        let centred = Settings::default().align_horizontally(0.5);
        let mut header = Grid::new();
        header.row_spacing = 2;
        header.add(Item::Leaf(self.title.rect), (0, 0), (1, 1), centred);
        if let Some((warning, _)) = &self.warning {
            header.add(Item::Leaf(warning.rect), (1, 0), (1, 1), centred);
        }
        header.arrange(&mut self.arena);
        let frame = (
            0,
            0,
            width.max(header.width),
            HEADER_AND_FOOTER.max(header.height),
        );
        place_in_frame(
            &mut self.arena,
            Item::Grid(Box::new(header)),
            frame,
            (0.5, 0.5),
        );
        place_in_frame(
            &mut self.arena,
            Item::Leaf(self.done.rect),
            (0, height - HEADER_AND_FOOTER, width, HEADER_AND_FOOTER),
            (0.5, 0.5),
        );
        self.list
            .resize(width, (HEADER_AND_FOOTER, height - 2 * HEADER_AND_FOOTER));
    }

    /// A line of words in the header under the title, `line` being the
    /// runs and how wide they are, shown or hidden (`restartWarning`).
    /// Hidden, it still has its place.
    pub fn set_header_line(&mut self, (runs, width): (&[Run], i32), show: bool) {
        match &mut self.warning {
            Some((_, shown)) if *shown == show => return,
            Some((_, shown)) => *shown = show,
            None if show => {
                let rect = self.arena.add(width, 9);
                let widget = Widget {
                    rect,
                    active: false,
                    kind: Kind::Text {
                        text: runs.to_vec(),
                    },
                };
                self.warning = Some((widget, true));
            }
            None => return,
        }
        self.arrange();
    }

    /// The window changed its GUI size: the layout is made again and the
    /// list keeps what it holds.
    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
        self.arrange();
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size;
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        if let Some((warning, true)) = &self.warning {
            warning.draw(frame, atlas, font, &self.arena, false);
        }
        let pending = self.list.draw(frame, atlas, font, mouse);
        let highlighted = self.done.contains(&self.arena, mouse) || self.done_focused;
        self.done.draw(frame, atlas, font, &self.arena, highlighted);
        if let Some(pending) = pending {
            pending.draw(frame, atlas, font, mouse, self.size);
        }
    }

    /// A press of the primary button: the list takes it if it is on it,
    /// else Done.
    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> SubScreenInput {
        let (took, changed) = self.list.clicked(lang, mouse, shift);
        if took {
            self.done_focused = false;
            return SubScreenInput {
                done: false,
                changed,
            };
        }
        let at = (mouse.0 as i32, mouse.1 as i32);
        let done = self.done.is_active() && self.done.contains(&self.arena, at);
        if done {
            self.apply(Target::Done, InputType::Mouse);
        }
        SubScreenInput {
            done,
            changed: None,
        }
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) -> Option<u8> {
        self.list.dragged(lang, mouse, dy)
    }

    /// The focus moves to `target`; what had it lets go.
    fn apply(&mut self, target: Target, input: InputType) {
        self.done_focused = false;
        match target {
            Target::List(focus) => self.list.move_focus(Some(focus), input),
            Target::Done => {
                self.list.move_focus(None, input);
                self.done_focused = true;
            }
        }
    }

    fn clear_focus(&mut self) {
        self.done_focused = false;
        self.list.move_focus(None, InputType::None);
    }

    /// `handleTabNavigation` of the screen, whose widgets in order are the
    /// title, the list and Done: the focused widget's own next first.
    fn tab_target(&self, forward: bool) -> Option<Target> {
        if self.list.screen_focused()
            && let Some(focus) = self.list.next_focus(Nav::Tab(forward))
        {
            return Some(Target::List(focus));
        }
        let at = if self.list.screen_focused() {
            Some(0)
        } else {
            self.done_focused.then_some(1)
        };
        let start = match (at, forward) {
            (Some(at), true) => at + 1,
            (Some(at), false) => at,
            (None, true) => 0,
            (None, false) => 2,
        };
        let order: Vec<usize> = if forward {
            (start..2).collect()
        } else {
            (0..start).rev().collect()
        };
        order.into_iter().find_map(|child| match child {
            0 => self.list.next_focus(Nav::Tab(forward)).map(Target::List),
            _ => (self.done.is_active() && !self.done_focused).then_some(Target::Done),
        })
    }

    /// Tab (and Shift+Tab): the focus moves on, and past the last widget
    /// round to the first.
    pub fn tab(&mut self, forward: bool) {
        let mut target = self.tab_target(forward);
        if target.is_none() {
            self.clear_focus();
            target = self.tab_target(forward);
        }
        if let Some(target) = target {
            self.apply(target, InputType::KeyboardTab);
        }
    }

    /// `handleArrowNavigation` of the screen: the list's own step, else the
    /// widget that lies next to the focused one.
    fn arrow(&mut self, direction: Direction) {
        let list_step = self.list.next_focus(Nav::Arrow(direction));
        if self.list.screen_focused()
            && let Some(focus) = list_step
        {
            self.apply(Target::List(focus), InputType::KeyboardArrow);
            return;
        }
        let rectangle = |rect: usize| {
            let rect = self.arena.0[rect];
            Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }
        };
        // The title and the line under it never take the focus.
        let mut children = vec![Candidate {
            rectangle: rectangle(self.title.rect),
            focusable: false,
        }];
        if let Some((warning, _)) = &self.warning {
            children.push(Candidate {
                rectangle: rectangle(warning.rect),
                focusable: false,
            });
        }
        let list_at = children.len();
        children.push(Candidate {
            rectangle: self.list.rectangle(),
            focusable: list_step.is_some(),
        });
        children.push(Candidate {
            rectangle: rectangle(self.done.rect),
            focusable: self.done.is_active(),
        });
        let focused = if self.list.screen_focused() {
            Some(list_at)
        } else {
            self.done_focused.then_some(list_at + 1)
        };
        let container = Rectangle {
            x: 0,
            y: 0,
            width: self.size.0,
            height: self.size.1,
        };
        match next_in_direction(&children, focused, direction, container) {
            Some(at) if at == list_at => {
                if let Some(focus) = list_step {
                    self.apply(Target::List(focus), InputType::KeyboardArrow);
                }
            }
            Some(_) => self.apply(Target::Done, InputType::KeyboardArrow),
            None => {}
        }
    }

    /// `Screen.keyPressed` for a key other than Tab and Escape: the focused
    /// widget takes it, or an arrow moves the focus.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> SubScreenInput {
        if self.list.screen_focused() {
            let (taken, changed) = self.list.key_pressed(lang, key, shift);
            if taken {
                return SubScreenInput {
                    done: false,
                    changed,
                };
            }
        } else if self.done_focused && key == SliderKey::Select && self.done.is_active() {
            return SubScreenInput {
                done: true,
                changed: None,
            };
        }
        match key {
            SliderKey::Left => self.arrow(Direction::Left),
            SliderKey::Right => self.arrow(Direction::Right),
            SliderKey::Up => self.arrow(Direction::Up),
            SliderKey::Down => self.arrow(Direction::Down),
            SliderKey::Select => {}
        }
        SubScreenInput::default()
    }

    pub fn released(&mut self, lang: &Lang) {
        self.list.released(lang);
    }
}
