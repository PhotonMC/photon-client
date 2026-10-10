//! The screen of a notice or confirmation dialog (`SimpleDialogScreen`): the title with the
//! warning button beside it, the body in a scrolling box, and the dialog's own buttons in a row
//! at the bottom, over the menu background.

use std::cell::Cell;

use photon_game::dialog::{ActionButton, Body, Dialog, PlainMessage};
use photon_game::text::{Component, Lang, Run, Style};

use crate::gui::chat::split;
use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, Settings, horizontal, place_in_frame};
use crate::gui::tooltip::{Holder, Pending};
use crate::gui::widgets::navigation::Rectangle;
use crate::gui::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::gui::widgets::slider::InputType;
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame, ItemDrawer};

/// `HeaderAndFooterLayout.DEFAULT_HEADER_AND_FOOTER_HEIGHT` and its content margin.
const BAR: i32 = 33;
const CONTENT_MARGIN: i32 = 30;
/// `FocusableTextWidget.DEFAULT_PADDING`.
const PADDING: i32 = 4;
/// `ScrollableLayout`'s space either side of the content: the bar and its gap.
const RESERVE: i32 = 4 + SCROLLBAR_WIDTH;
const SCROLL_RATE: f64 = 10.0;
const WARNING_SIZE: i32 = 20;

/// What the player did on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// One of the dialog's own buttons, by its place in `main_actions`.
    Button(usize),
    /// The warning button: the screen about custom screens opens.
    Warning,
    /// The screen is closed (`Screen.onClose`).
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Warning,
    Part(usize),
    Button(usize),
}

/// The item an item body shows.
struct Shown {
    id: i32,
    count: i32,
    decorations: bool,
}

/// A focusable part of the body: wrapped text, or an item's box.
struct Part {
    rect: usize,
    lines: Option<Vec<Vec<Run>>>,
    item: Option<Shown>,
}

/// `ScrollableLayout.Container`: where the box stands.
#[derive(Debug, Clone, Copy)]
struct Container {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub struct DialogScreen {
    arena: Arena,
    size: (i32, i32),
    in_world: bool,
    can_close: bool,
    title: Widget,
    warning: Widget,
    warning_tooltip: Holder,
    parts: Vec<Part>,
    buttons: Vec<(Widget, Holder)>,
    container: Container,
    content_height: i32,
    scroll: f64,
    dragging: bool,
    focus: Option<Focus>,
    input: Cell<InputType>,
}

/// `FocusableTextWidget`: `message` wrapped to the width less its padding, as tall as its lines.
fn text_part(
    arena: &mut Arena,
    (font, lang): (&Font, &Lang),
    message: &PlainMessage,
) -> (usize, Part) {
    let runs = lang.runs(&message.contents, &Style::default());
    let lines = if runs.iter().all(|run| run.text.is_empty()) {
        Vec::new()
    } else {
        split(&runs, message.width - 2 * PADDING, font)
    };
    let rect = arena.add(message.width, lines.len() as i32 * 9 + 2 * PADDING);
    (
        rect,
        Part {
            rect,
            lines: Some(lines),
            item: None,
        },
    )
}

fn centred() -> Settings {
    Settings::default().align_horizontally(0.5)
}

fn middle() -> Settings {
    Settings {
        x_align: 0.5,
        y_align: 0.5,
        ..Settings::default()
    }
}

/// `DialogBodyHandlers.createBodyElement` for each body, in a column spaced by 10.
fn body_layout(
    arena: &mut Arena,
    (font, lang): (&Font, &Lang),
    bodies: &[Body],
) -> (Grid, Vec<Part>) {
    let mut parts = Vec::new();
    let mut body = Grid::new();
    body.row_spacing = 10;
    for (row, element) in bodies.iter().enumerate() {
        let item = match element {
            Body::Plain(message) => {
                let (rect, part) = text_part(arena, (font, lang), message);
                parts.push(part);
                Item::Leaf(rect)
            }
            Body::Item {
                id,
                count,
                description,
                show_decorations,
                width,
                height,
                ..
            } => {
                let icon = arena.add(*width, *height);
                parts.push(Part {
                    rect: icon,
                    lines: None,
                    item: photon_game::item::id_of(id).map(|id| Shown {
                        id,
                        count: *count,
                        decorations: *show_decorations,
                    }),
                });
                match description {
                    Some(message) => {
                        let (text, part) = text_part(arena, (font, lang), message);
                        parts.push(part);
                        let mut pair = Grid::new();
                        pair.column_spacing = 2;
                        pair.add(Item::Leaf(icon), (0, 0), (1, 1), middle());
                        pair.add(Item::Leaf(text), (0, 1), (1, 1), middle());
                        Item::Grid(Box::new(pair))
                    }
                    None => Item::Leaf(icon),
                }
            }
        };
        body.add(item, (row, 0), (1, 1), centred());
    }
    (body, parts)
}

impl DialogScreen {
    /// `SimpleDialogScreen.init`: the screen of a notice or confirmation, in a screen of `size`
    /// units. Any other dialog has no screen yet.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        dialog: &Dialog,
        in_world: bool,
    ) -> Option<Self> {
        let common = dialog.common();
        if matches!(dialog, Dialog::Other(_)) {
            return None;
        }
        let (width, height) = size;
        let mut arena = Arena::default();
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let title_text = runs(&common.title);
        let title = Widget {
            rect: arena.add(font.width_of_runs(&title_text), 9),
            active: false,
            kind: Kind::Text { text: title_text },
        };
        let warning = Widget {
            rect: arena.add(WARNING_SIZE, WARNING_SIZE),
            active: true,
            kind: Kind::Icon {
                sprite: String::new(),
                size: (0, 0),
            },
        };
        let (mut body, parts) = body_layout(&mut arena, (font, lang), &common.body);
        body.arrange(&mut arena);
        let (body_width, body_height) = (body.width, body.height);
        // `ScrollableLayout`: the box is as tall as the body, to what the bars leave.
        let content_room = height - 2 * BAR;
        let box_width = body_width + 2 * RESERVE;
        let box_height = body_height.min(content_room);
        let frame_width = width.max(box_width);
        let frame_y = (BAR + CONTENT_MARGIN).min(height - BAR - box_height);
        let container = Container {
            x: (0.5 * (frame_width - box_width) as f32) as i32,
            y: frame_y,
            width: box_width,
            height: box_height,
        };
        let mut body = Item::Grid(Box::new(body));
        body.set_x(&mut arena, container.x + RESERVE);
        body.set_y(&mut arena, container.y);
        // The header: the title and the warning button, put in the middle of the bar.
        let mut header = Grid::new();
        header.column_spacing = 10;
        header.add(Item::Leaf(title.rect), (0, 0), (1, 1), middle());
        header.add(Item::Leaf(warning.rect), (0, 1), (1, 1), middle());
        header.arrange(&mut arena);
        let header_width = width.max(header.width);
        let header_height = BAR.max(header.height);
        place_in_frame(
            &mut arena,
            Item::Grid(Box::new(header)),
            (0, 0, header_width, header_height),
            (0.5, 0.5),
        );
        let buttons: Vec<(Widget, Holder)> = dialog
            .main_actions()
            .into_iter()
            .map(|action| Self::button(&mut arena, lang, action))
            .collect();
        let leaves: Vec<usize> = buttons.iter().map(|(widget, _)| widget.rect).collect();
        let mut footer = horizontal(&arena, &leaves, 8);
        footer.arrange(&mut arena);
        let footer_width = width.max(footer.width);
        place_in_frame(
            &mut arena,
            Item::Grid(Box::new(footer)),
            (0, height - BAR, footer_width, BAR),
            (0.5, 0.5),
        );
        // `makeSureWarningButtonIsInBounds`.
        let place = &mut arena.0[warning.rect];
        if place.x < 0 || place.y < 0 || place.x > width - 20 || place.y > height - 20 {
            place.x = (width - 40).max(0);
            place.y = 5.min(height);
        }
        Some(Self {
            size,
            in_world,
            can_close: common.can_close_with_escape,
            title,
            warning,
            warning_tooltip: Holder::of(
                lang,
                &Component::translatable("menu.custom_screen_info.tooltip"),
            ),
            parts,
            buttons,
            container,
            content_height: body_height,
            scroll: 0.0,
            dragging: false,
            focus: None,
            input: Cell::new(InputType::Mouse),
            arena,
        })
    }

    /// `DialogControlSet.createActionButton`: a button of the width the dialog asks for, with its
    /// tooltip.
    fn button(arena: &mut Arena, lang: &Lang, action: &ActionButton) -> (Widget, Holder) {
        let widget = Widget {
            rect: arena.add(action.width, 20),
            active: true,
            kind: Kind::Button {
                label: lang.runs(&action.label, &Style::default()),
            },
        };
        let tooltip = action
            .tooltip
            .as_ref()
            .map_or_else(Holder::default, |text| Holder::of(lang, text));
        (widget, tooltip)
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    fn max_scroll(&self) -> i32 {
        (self.content_height - self.container.height).max(0)
    }

    fn place(&self) -> Place {
        Place {
            y: self.container.y,
            width: self.size.0,
            height: self.container.height,
            in_world: self.in_world,
        }
    }

    /// Where a part of the body is now, scrolled.
    fn part_rect(&self, part: &Part) -> Rectangle {
        let rect = self.arena.0[part.rect];
        Rectangle {
            x: rect.x,
            y: rect.y - self.scroll as i32,
            width: rect.width,
            height: rect.height,
        }
    }

    fn in_container(&self, (x, y): (i32, i32)) -> bool {
        let c = self.container;
        x >= c.x && y >= c.y && x < c.x + c.width && y < c.y + c.height
    }

    /// The background, the title, the body in its box, the buttons, the warning button, and the
    /// tooltip of the one the mouse is on (or the keyboard has).
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        item: &mut ItemDrawer<'_>,
        mouse: (i32, i32),
    ) {
        let background = if self.in_world {
            "inworld_menu_background"
        } else {
            "menu_background"
        };
        let (width, height) = self.size;
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        let c = self.container;
        let mark = frame.mark();
        for (index, part) in self.parts.iter().enumerate() {
            let rect = self.part_rect(part);
            if let Some(lines) = &part.lines {
                let middle = rect.x + rect.width / 2;
                for (line, runs) in lines.iter().enumerate() {
                    let x = middle - font.width_of_runs(runs) / 2;
                    let y = rect.y + PADDING + line as i32 * 9;
                    frame.styled_text(atlas, font, runs, x, y, 0xFFFF_FFFF, true);
                }
            }
            // `ItemDisplayWidget`: the item at the widget's corner. The items are not cut by the
            // box, so one it only partly shows is left out.
            if let Some(shown) = &part.item
                && rect.y >= c.y
                && rect.y + 16 <= c.y + c.height
            {
                let stack = photon_world::ClientItemStackState::new(shown.count, shown.id);
                if shown.decorations {
                    frame.item_with_count(
                        atlas,
                        font,
                        item,
                        &stack,
                        shown.count,
                        None,
                        rect.x,
                        rect.y,
                    );
                } else {
                    frame.item(item, &stack, rect.x, rect.y);
                }
            }
            if self.focus == Some(Focus::Part(index)) {
                outline(frame, atlas, rect);
            }
        }
        frame.clip_from(mark, (c.x, c.y, c.x + c.width, c.y + c.height));
        self.place().draw_scrollbar(
            frame,
            atlas,
            c.x + c.width - SCROLLBAR_WIDTH,
            self.scroll,
            self.content_height,
        );
        let millis = frame.millis;
        let mut pending: Option<Pending> = None;
        let mut ask = |tooltip: &Holder, rect: usize, hovered: bool, focused: bool| {
            let rect = self.arena.0[rect];
            let rectangle = Rectangle {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            };
            if let Some(asked) =
                tooltip.refresh(millis, (hovered, focused), self.input.get(), rectangle)
                && (pending.is_none() || focused)
            {
                pending = Some(asked);
            }
        };
        for (index, (widget, tooltip)) in self.buttons.iter().enumerate() {
            let hovered = widget.contains(&self.arena, mouse);
            ask(
                tooltip,
                widget.rect,
                hovered,
                self.focus == Some(Focus::Button(index)),
            );
        }
        for (index, (widget, _)) in self.buttons.iter().enumerate() {
            let highlighted =
                widget.contains(&self.arena, mouse) || self.focus == Some(Focus::Button(index));
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
        let warned = self.warning.contains(&self.arena, mouse);
        let focused = self.focus == Some(Focus::Warning);
        let sprite = if warned || focused {
            "dialog/warning_button_highlighted"
        } else {
            "dialog/warning_button"
        };
        let rect = self.arena.0[self.warning.rect];
        frame.sprite(atlas, sprite, rect.x, rect.y, rect.width, rect.height);
        ask(&self.warning_tooltip, self.warning.rect, warned, focused);
        if let Some(pending) = pending {
            pending.draw(frame, atlas, font, mouse, self.size);
        }
    }

    /// A press of the primary button: the scrollbar taken hold of, a part of the body or a button
    /// (which takes the focus), and for the buttons what they do.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<Press> {
        self.input.set(InputType::Mouse);
        let c = self.container;
        let (x, y) = mouse;
        if self.in_container(mouse) {
            self.dragging = self.max_scroll() > 0 && x >= c.x + c.width - SCROLLBAR_WIDTH;
            let hit = self.parts.iter().position(|part| {
                let rect = self.part_rect(part);
                x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
            });
            if let Some(index) = hit {
                self.focus = Some(Focus::Part(index));
            }
            return None;
        }
        if let Some(index) = self
            .buttons
            .iter()
            .position(|(widget, _)| widget.contains(&self.arena, mouse))
        {
            self.focus = Some(Focus::Button(index));
            return Some(Press::Button(index));
        }
        if self.warning.contains(&self.arena, mouse) {
            self.focus = Some(Focus::Warning);
            return Some(Press::Warning);
        }
        None
    }

    /// The primary button is let go.
    pub fn released(&mut self) {
        self.dragging = false;
    }

    /// The mouse moved with the scrollbar held: `y` is where it is, `dy` how far it went.
    pub fn dragged(&mut self, y: f64, dy: f64) {
        if self.dragging {
            let scroll = self
                .place()
                .dragged_scroll(self.scroll, y, dy, self.content_height);
            self.set_scroll(scroll);
        }
    }

    /// The wheel over the box (`AbstractScrollArea.mouseScrolled`).
    pub fn scrolled(&mut self, mouse: (i32, i32), amount: f64) {
        if self.in_container(mouse) {
            self.set_scroll(self.scroll - amount * SCROLL_RATE);
        }
    }

    fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, f64::from(self.max_scroll()));
    }

    fn order(&self) -> Vec<Focus> {
        let mut order = vec![Focus::Warning];
        order.extend((0..self.parts.len()).map(Focus::Part));
        order.extend((0..self.buttons.len()).map(Focus::Button));
        order
    }

    /// Tab (and Shift+Tab): the focus goes to the next part; at the end it starts over. A part
    /// the box hides is scrolled into view (`ScrollableLayout.Container.setFocused`).
    pub fn tab(&mut self, forward: bool) {
        self.input.set(InputType::KeyboardTab);
        let order = self.order();
        let at = self
            .focus
            .and_then(|focus| order.iter().position(|&other| other == focus));
        let next = match (at, forward) {
            (None, true) => 0,
            (None, false) => order.len() - 1,
            (Some(at), true) => (at + 1) % order.len(),
            (Some(at), false) => (at + order.len() - 1) % order.len(),
        };
        self.focus = Some(order[next]);
        if let Some(Focus::Part(index)) = self.focus {
            let rect = self.part_rect(&self.parts[index]);
            let c = self.container;
            let (top, bottom) = (rect.y - c.y, rect.y + rect.height - (c.y + c.height));
            if top < 0 {
                self.set_scroll(self.scroll + f64::from(top) - SCROLL_RATE);
            } else if bottom > 0 {
                self.set_scroll(self.scroll + f64::from(bottom) + SCROLL_RATE);
            }
        }
    }

    /// Enter or Space: the focused button.
    pub fn pressed(&self) -> Option<Press> {
        match self.focus? {
            Focus::Button(index) => Some(Press::Button(index)),
            Focus::Warning => Some(Press::Warning),
            Focus::Part(_) => None,
        }
    }

    /// Escape, when the dialog lets it close.
    pub fn escape(&self) -> Option<Press> {
        self.can_close.then_some(Press::Close)
    }
}

/// `GuiGraphicsExtractor.outline` in white.
fn outline(frame: &mut GuiFrame, atlas: &GuiAtlas, rect: Rectangle) {
    let Rectangle {
        x,
        y,
        width,
        height,
    } = rect;
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
