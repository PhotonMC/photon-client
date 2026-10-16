//! The screen of a notice, confirmation or multi-action dialog (`SimpleDialogScreen`,
//! `MultiButtonDialogScreen`): the title with the warning button beside it, the body (and for a
//! button list its grid of buttons) in a scrolling box, and the dialog's own buttons in a row at
//! the bottom, over the menu background.

use crate::gui::ItemSlots;
use crate::gui::{FontRuns, FrameRuns};
use std::cell::Cell;

use super::dialog_body::{
    BAR, PADDING, Part, body_layout, centred, middle, outline, pack_into_columns,
};

use photon_game::dialog::{ActionButton, Dialog};
use photon_game::text::{Component, Lang, Style};

use crate::gui::font::Font;
use crate::gui::layout::{Arena, Grid, Item, horizontal, place_in_frame};

use crate::gui::tooltip::{Holder, Pending};
use crate::gui::widgets::navigation::Rectangle;
use crate::gui::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::gui::widgets::slider::InputType;
use crate::gui::widgets::{Kind, Widget};
use crate::gui::{GuiAtlas, GuiFrame, ItemDrawer};

/// `HeaderAndFooterLayout`'s content margin.
/// `ButtonListDialogScreen.FOOTER_MARGIN`: the footer of a button list with no exit button.
const SHORT_FOOTER: i32 = 5;
const CONTENT_MARGIN: i32 = 30;
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
    /// The dialog's buttons in `main_actions` order: the first `listed` stand in the body (and
    /// scroll with it), the rest in the footer.
    buttons: Vec<(Widget, Holder)>,
    listed: usize,
    container: Container,
    content_height: i32,
    scroll: f64,
    dragging: bool,
    focus: Option<Focus>,
    input: Cell<InputType>,
}

/// The header: the title and the warning button, put in the middle of the bar.
fn place_header(arena: &mut Arena, (title, warning): (&Widget, &Widget), width: i32) {
    let mut header = Grid::new();
    header.column_spacing = 10;
    header.add(Item::Leaf(title.rect), (0, 0), (1, 1), middle());
    header.add(Item::Leaf(warning.rect), (0, 1), (1, 1), middle());
    header.arrange(arena);
    let header_width = width.max(header.width);
    let header_height = BAR.max(header.height);
    place_in_frame(
        arena,
        Item::Grid(Box::new(header)),
        (0, 0, header_width, header_height),
        (0.5, 0.5),
    );
}

/// `ScrollableLayout`: the box is as tall as the body, to what the bars leave, and is put at the
/// top of the content frame.
fn container_for(
    (width, height): (i32, i32),
    footer_height: i32,
    (body_width, body_height): (i32, i32),
) -> Container {
    let box_width = body_width + 2 * RESERVE;
    let box_height = body_height.min(height - BAR - footer_height);
    let frame_width = width.max(box_width);
    Container {
        x: (0.5 * (frame_width - box_width) as f32) as i32,
        y: (BAR + CONTENT_MARGIN).min(height - footer_height - box_height),
        width: box_width,
        height: box_height,
    }
}

impl DialogScreen {
    /// `SimpleDialogScreen.init` and `ButtonListDialogScreen.init`: the screen of a dialog, in a
    /// screen of `size` units. A type without a screen yet gives none.
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
        let buttons: Vec<(Widget, Holder)> = dialog
            .main_actions()
            .into_iter()
            .map(|action| Self::button(&mut arena, lang, action))
            .collect();
        let (listed, columns) = dialog.button_list().unwrap_or((0, 1));
        if listed > 0 {
            let leaves: Vec<usize> = buttons[..listed]
                .iter()
                .map(|(widget, _)| widget.rect)
                .collect();
            let grid = pack_into_columns(&arena, &leaves, columns as usize);
            body.add(
                Item::Grid(Box::new(grid)),
                (common.body.len(), 0),
                (1, 1),
                centred(),
            );
        }
        // A button list with no exit button leaves a short footer.
        let footer_height = if listed > 0 && buttons.len() == listed {
            SHORT_FOOTER
        } else {
            BAR
        };
        body.arrange(&mut arena);
        let (body_width, body_height) = (body.width, body.height);
        let container = container_for((width, height), footer_height, (body_width, body_height));
        let mut body = Item::Grid(Box::new(body));
        body.set_x(&mut arena, container.x + RESERVE);
        body.set_y(&mut arena, container.y);
        place_header(&mut arena, (&title, &warning), width);
        let leaves: Vec<usize> = buttons[listed..]
            .iter()
            .map(|(widget, _)| widget.rect)
            .collect();
        let mut footer = horizontal(&arena, &leaves, 8);
        footer.arrange(&mut arena);
        let footer_width = width.max(footer.width);
        place_in_frame(
            &mut arena,
            Item::Grid(Box::new(footer)),
            (0, height - footer_height, footer_width, footer_height),
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
            listed,
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

    /// Where button `index` is now: the ones in the body scroll with it.
    fn button_rect(&self, index: usize) -> Rectangle {
        let rect = self.arena.0[self.buttons[index].0.rect];
        let shift = if index < self.listed {
            self.scroll as i32
        } else {
            0
        };
        Rectangle {
            x: rect.x,
            y: rect.y - shift,
            width: rect.width,
            height: rect.height,
        }
    }

    /// Whether the mouse is on button `index`; the box cuts the ones in the body.
    fn button_hit(&self, index: usize, mouse: (i32, i32)) -> bool {
        let rect = self.button_rect(index);
        let (x, y) = mouse;
        x >= rect.x
            && x < rect.x + rect.width
            && y >= rect.y
            && y < rect.y + rect.height
            && (index >= self.listed || self.in_container(mouse))
    }

    /// The body's text and items, scrolled, with the focus outline on the part that has it.
    fn draw_parts(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        item: &mut ItemDrawer<'_>,
    ) {
        let c = self.container;
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
        self.draw_parts(frame, (atlas, font), item);
        // The body's buttons are drawn where the scrolling puts them, inside the box.
        let mut shifted = self.arena.clone();
        for (widget, _) in &self.buttons[..self.listed] {
            shifted.0[widget.rect].y -= self.scroll as i32;
        }
        for index in 0..self.listed {
            let highlighted =
                self.button_hit(index, mouse) || self.focus == Some(Focus::Button(index));
            self.buttons[index]
                .0
                .draw(frame, atlas, font, &shifted, highlighted);
        }
        // The scissor stack starts as the screen, so a box wider than it is cut there too.
        frame.clip_from(
            mark,
            (
                c.x.max(0),
                c.y.max(0),
                (c.x + c.width).min(width),
                (c.y + c.height).min(height),
            ),
        );
        self.place().draw_scrollbar(
            frame,
            atlas,
            c.x + c.width - SCROLLBAR_WIDTH,
            self.scroll,
            self.content_height,
        );
        let millis = frame.millis;
        let mut pending: Option<Pending> = None;
        let mut ask = |tooltip: &Holder, rectangle: Rectangle, hovered: bool, focused: bool| {
            if let Some(asked) =
                tooltip.refresh(millis, (hovered, focused), self.input.get(), rectangle)
                && (pending.is_none() || focused)
            {
                pending = Some(asked);
            }
        };
        for (index, (_, tooltip)) in self.buttons.iter().enumerate() {
            let hovered = self.button_hit(index, mouse);
            ask(
                tooltip,
                self.button_rect(index),
                hovered,
                self.focus == Some(Focus::Button(index)),
            );
        }
        for (index, (widget, _)) in self.buttons.iter().enumerate().skip(self.listed) {
            let highlighted =
                self.button_hit(index, mouse) || self.focus == Some(Focus::Button(index));
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
        let rectangle = Rectangle {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        };
        ask(&self.warning_tooltip, rectangle, warned, focused);
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
                return None;
            }
        }
        if let Some(index) = (0..self.buttons.len()).find(|&index| self.button_hit(index, mouse)) {
            self.focus = Some(Focus::Button(index));
            return Some(Press::Button(index));
        }
        if self.in_container(mouse) {
            return None;
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
        let shown = match self.focus {
            Some(Focus::Part(index)) => Some(self.part_rect(&self.parts[index])),
            Some(Focus::Button(index)) if index < self.listed => Some(self.button_rect(index)),
            _ => None,
        };
        if let Some(rect) = shown {
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
