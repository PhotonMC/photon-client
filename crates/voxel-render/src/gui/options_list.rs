//! The settings screens' shell (`OptionsSubScreen`) and the scrolling list of
//! options in it (`OptionsList`): a title over the list, Done under it, and
//! in the list headers and rows of one wide or two narrow option widgets.

use std::any::Any;

use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::cycle::CycleButton;
use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, Settings, place_in_frame};
use crate::gui::option_text::on_off;
use crate::gui::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::gui::slider::{InputType, OptionSlider, SliderValues};
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
}

impl<S: SliderValues + 'static> Control for OptionSlider<S>
where
    S::Value: 'static,
{
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

/// `OptionsList`.
pub struct OptionsList {
    arena: Arena,
    place: Place,
    entries: Vec<Entry>,
    scroll: f64,
    /// Whether the mouse holds the scroll bar.
    scrolling: bool,
    focus: Option<Focus>,
    /// Whether the mouse holds the focused widget (`isDragging`).
    dragging: bool,
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
            dragging: false,
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
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let place = self.place;
        place.draw_background(frame, atlas, self.scroll);
        // The hover of a widget needs the mouse inside the cut.
        let inside = place.contains((f64::from(mouse.0), f64::from(mouse.1)));
        let mouse = if inside { mouse } else { (i32::MIN, i32::MIN) };
        let mark = frame.mark();
        for entry in &self.entries {
            if entry.y + entry.height < place.y || entry.y > place.bottom() {
                continue;
            }
            match &entry.content {
                Content::Header { text, .. } => text.draw(frame, atlas, font, &self.arena, false),
                Content::Row(children) => {
                    for child in children {
                        child.control.draw(frame, atlas, font, &self.arena, mouse);
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
            if changed_entry && (top < self.place.y || bottom > self.place.bottom()) {
                self.scroll_to_entry(entry);
            }
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
            return (self.scrolling, None);
        };
        let mut changed = None;
        if let Content::Row(children) = &mut self.entries[entry].content {
            let child = &mut children[child];
            if child.control.pressed(lang, &self.arena, mouse, shift) {
                changed = Some(child.tag);
            }
        }
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
    size: (i32, i32),
    in_world: bool,
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
        self.list.draw(frame, atlas, font, mouse);
        let highlighted = self.done.contains(&self.arena, mouse);
        self.done.draw(frame, atlas, font, &self.arena, highlighted);
    }

    /// A press of the primary button: the list takes it if it is on it,
    /// else Done.
    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> SubScreenInput {
        let (took, changed) = self.list.clicked(lang, mouse, shift);
        if took {
            return SubScreenInput {
                done: false,
                changed,
            };
        }
        let at = (mouse.0 as i32, mouse.1 as i32);
        SubScreenInput {
            done: self.done.is_active() && self.done.contains(&self.arena, at),
            changed: None,
        }
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) -> Option<u8> {
        self.list.dragged(lang, mouse, dy)
    }

    pub fn released(&mut self, lang: &Lang) {
        self.list.released(lang);
    }
}
