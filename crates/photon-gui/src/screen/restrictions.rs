//! The screen that says why the chat is limited and what is still allowed (`RestrictionsScreen`):
//! a title, each restriction with the button that does something about it, the four permissions,
//! and Done, over the menu background.

use photon_game::chat_abilities::{ChatAbilities, ChatPermission, RestrictionAction};
use photon_game::dialog::PlainMessage;
use photon_game::text::{Component, Lang, Style, named_colour};

use super::dialog::{Container, RESERVE, SCROLL_RATE, container_for, menu_background};
use super::dialog_body::{BAR, PADDING, Part, centred, outline, text_part};
use crate::layout::{Arena, Grid, Item, place_in_frame};
use crate::widgets::navigation::Rectangle;
use crate::widgets::scroll_area::{Place, SCROLLBAR_WIDTH};
use crate::widgets::{Kind, Widget};
use crate::{FontRuns, FrameRuns};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// The width of the screen's text boxes.
const TEXT_WIDTH: i32 = 250;
const BUTTON_WIDTH: i32 = 200;

/// What the player did on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// A restriction's button.
    Action(RestrictionAction),
    /// Done, or Escape: back to the previous screen.
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Part(usize),
    Button(usize),
}

pub struct RestrictionsScreen {
    arena: Arena,
    size: (i32, i32),
    in_world: bool,
    title: Widget,
    parts: Vec<Part>,
    /// The restrictions' buttons in the body, then Done.
    buttons: Vec<(Widget, Option<RestrictionAction>)>,
    /// The focusable things in the order the keyboard goes through them.
    order: Vec<Focus>,
    container: Container,
    content_height: i32,
    scroll: f64,
    dragging: bool,
    focus: Option<Focus>,
}

impl RestrictionsScreen {
    /// `RestrictionsScreen.init`, in a screen of `size` units.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        abilities: &ChatAbilities,
        in_world: bool,
    ) -> Self {
        let (width, height) = size;
        let mut arena = Arena::default();
        let title = title_widget(&mut arena, (font, lang));
        let Body {
            mut grid,
            parts,
            mut buttons,
            order,
            ..
        } = Body::of(&mut arena, (font, lang), abilities);
        grid.arrange(&mut arena);
        let (body_width, body_height) = (grid.width, grid.height);
        let container = container_for(size, BAR, (body_width, body_height));
        let mut grid = Item::Grid(Box::new(grid));
        grid.set_x(&mut arena, container.x + RESERVE);
        grid.set_y(&mut arena, container.y);
        place_in_frame(
            &mut arena,
            Item::Leaf(title.rect),
            (0, 0, width, BAR),
            (0.5, 0.5),
        );
        let done = done_button(&mut arena, lang);
        place_in_frame(
            &mut arena,
            Item::Leaf(done.0.rect),
            (0, height - BAR, width, BAR),
            (0.5, 0.5),
        );
        buttons.push(done);
        let mut order = order;
        order.push(Focus::Button(buttons.len() - 1));
        Self {
            arena,
            size,
            in_world,
            title,
            parts,
            buttons,
            order,
            container,
            content_height: body_height,
            scroll: 0.0,
            dragging: false,
            focus: None,
        }
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

    fn in_container(&self, (x, y): (i32, i32)) -> bool {
        let c = self.container;
        x >= c.x && y >= c.y && x < c.x + c.width && y < c.y + c.height
    }

    /// Where a rectangle of the body is now, scrolled.
    fn scrolled_rect(&self, rect: usize) -> Rectangle {
        let rect = self.arena.0[rect];
        Rectangle {
            x: rect.x,
            y: rect.y - self.scroll as i32,
            width: rect.width,
            height: rect.height,
        }
    }

    /// Where button `index` is now: Done, the last, stays where the footer puts it.
    fn button_rect(&self, index: usize) -> Rectangle {
        let rect = self.buttons[index].0.rect;
        if index + 1 == self.buttons.len() {
            let place = self.arena.0[rect];
            return Rectangle {
                x: place.x,
                y: place.y,
                width: place.width,
                height: place.height,
            };
        }
        self.scrolled_rect(rect)
    }

    /// Whether the mouse is on button `index`; the box cuts the ones in the body.
    fn button_hit(&self, index: usize, (x, y): (i32, i32)) -> bool {
        let rect = self.button_rect(index);
        let on = x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height;
        on && (index + 1 == self.buttons.len() || self.in_container((x, y)))
    }

    /// The background, the title, the body in its box with its scrollbar, and Done.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        (mouse, blurriness): ((i32, i32), i32),
    ) {
        let (width, height) = self.size;
        let background = menu_background(frame, self.in_world, blurriness);
        frame.tile_texture(atlas, background, (32, 32), (0, 0, width, height));
        self.title.draw(frame, atlas, font, &self.arena, false);
        let c = self.container;
        let mark = frame.mark();
        self.draw_parts(frame, (atlas, font));
        let done = self.buttons.len() - 1;
        let mut shifted = self.arena.clone();
        for (widget, _) in &self.buttons[..done] {
            shifted.0[widget.rect].y -= self.scroll as i32;
        }
        for index in 0..done {
            let highlighted =
                self.button_hit(index, mouse) || self.focus == Some(Focus::Button(index));
            self.buttons[index]
                .0
                .draw(frame, atlas, font, &shifted, highlighted);
        }
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
        let highlighted = self.button_hit(done, mouse) || self.focus == Some(Focus::Button(done));
        self.buttons[done]
            .0
            .draw(frame, atlas, font, &self.arena, highlighted);
    }

    /// `FocusableTextWidget` drawn: a black fill and a white outline while it has the focus, then
    /// its lines, centred.
    fn draw_parts(&self, frame: &mut GuiFrame, (atlas, font): (&GuiAtlas, &Font)) {
        for (index, part) in self.parts.iter().enumerate() {
            let rect = self.scrolled_rect(part.rect);
            if self.focus == Some(Focus::Part(index)) {
                let black = 0xFF00_0000;
                frame.fill(
                    atlas,
                    (
                        rect.x + 1,
                        rect.y,
                        rect.x + rect.width,
                        rect.y + rect.height,
                    ),
                    black,
                );
                outline(frame, atlas, rect);
            }
            for (line, runs) in part.lines.iter().flatten().enumerate() {
                let x = rect.x + rect.width / 2 - font.width_of_runs(runs) / 2;
                let y = rect.y + PADDING + line as i32 * 9;
                frame.styled_text(atlas, font, runs, x, y, 0xFFFF_FFFF, true);
            }
        }
    }

    /// A press of the primary button: the scrollbar taken hold of, a text box or a button (which
    /// takes the focus), and for a button what it does.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<Press> {
        let c = self.container;
        let (x, y) = mouse;
        if self.in_container(mouse) {
            self.dragging = self.max_scroll() > 0 && x >= c.x + c.width - SCROLLBAR_WIDTH;
            let hit = self.parts.iter().position(|part| {
                let rect = self.scrolled_rect(part.rect);
                x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
            });
            if let Some(index) = hit {
                self.focus = Some(Focus::Part(index));
                return None;
            }
        }
        let index = (0..self.buttons.len()).find(|&index| self.button_hit(index, mouse))?;
        self.focus = Some(Focus::Button(index));
        crate::ui_sound::play_down_sound();
        Some(self.press_of(index))
    }

    fn press_of(&self, index: usize) -> Press {
        self.buttons[index].1.map_or(Press::Done, Press::Action)
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

    /// The wheel over the box.
    pub fn scrolled(&mut self, mouse: (i32, i32), amount: f64) {
        if self.in_container(mouse) {
            self.set_scroll(self.scroll - amount * SCROLL_RATE);
        }
    }

    fn set_scroll(&mut self, scroll: f64) {
        self.scroll = scroll.clamp(0.0, f64::from(self.max_scroll()));
    }

    /// Tab (and Shift+Tab): the focus goes to the next thing; at the end it starts over. One the
    /// box hides is scrolled into view.
    pub fn tab(&mut self, forward: bool) {
        let at = self
            .focus
            .and_then(|focus| self.order.iter().position(|&other| other == focus));
        let count = self.order.len();
        let next = match (at, forward) {
            (None, true) => 0,
            (None, false) => count - 1,
            (Some(at), true) => (at + 1) % count,
            (Some(at), false) => (at + count - 1) % count,
        };
        self.focus = Some(self.order[next]);
        let shown = match self.focus {
            Some(Focus::Part(index)) => Some(self.scrolled_rect(self.parts[index].rect)),
            Some(Focus::Button(index)) if index + 1 < self.buttons.len() => {
                Some(self.button_rect(index))
            }
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
            Focus::Button(index) => {
                crate::ui_sound::play_down_sound();
                Some(self.press_of(index))
            }
            Focus::Part(_) => None,
        }
    }

    /// Escape: `Screen.onClose`.
    pub fn escape(&self) -> Press {
        Press::Done
    }
}

/// The four permissions, each on a line: green where allowed, red where not.
fn permissions(abilities: &ChatAbilities) -> Component {
    let mut lines = Component::literal("");
    for (index, permission) in ChatPermission::ALL.into_iter().enumerate() {
        if index > 0 {
            lines = lines.append(Component::literal("\n"));
        }
        let allowed = abilities.has(permission);
        let state = if allowed { "allowed" } else { "denied" };
        let key = format!("restrictions_screen.permission.{}.{state}", permission.id());
        let colour = Style {
            colour: named_colour(if allowed { "green" } else { "red" }),
            ..Style::default()
        };
        lines = lines.append(Component::translatable(key).with_style(&colour));
    }
    lines
}

/// The body's column as it is built: text boxes and buttons one under another, centred.
#[derive(Default)]
struct Body {
    grid: Grid,
    rows: usize,
    parts: Vec<Part>,
    buttons: Vec<(Widget, Option<RestrictionAction>)>,
    order: Vec<Focus>,
}

/// `restrictions_screen.title`, the header's text.
fn title_widget(arena: &mut Arena, (font, lang): (&Font, &Lang)) -> Widget {
    let text = lang.runs(
        &Component::translatable("restrictions_screen.title"),
        &Style::default(),
    );
    Widget {
        rect: arena.add(font.width_of_runs(&text), 9),
        active: false,
        kind: Kind::Text { text },
    }
}

/// The Done button of the footer.
fn done_button(arena: &mut Arena, lang: &Lang) -> (Widget, Option<RestrictionAction>) {
    let mut done = Body::default();
    done.button(arena, lang, Component::translatable("gui.done"), None);
    done.buttons.remove(0)
}

impl Body {
    /// `RestrictionsScreen.init`'s body: each restriction, bold and red, with the button that
    /// does something about it; a gap; the permissions.
    fn of(arena: &mut Arena, (font, lang): (&Font, &Lang), abilities: &ChatAbilities) -> Self {
        let mut body = Self::default();
        for restriction in abilities.restrictions() {
            let style = Style {
                bold: Some(true),
                colour: named_colour("red"),
                ..Style::default()
            };
            let reason = Component::translatable(restriction.display_key()).with_style(&style);
            body.text(arena, (font, lang), reason);
            if let Some(action) = restriction.action() {
                let label = Component::translatable(action.title_key());
                body.button(arena, lang, label, Some(action));
            }
        }
        if abilities.has_any_restrictions() {
            body.spacer(arena);
        }
        body.text(arena, (font, lang), permissions(abilities));
        body
    }

    fn add(&mut self, rect: usize) {
        self.grid
            .add(Item::Leaf(rect), (self.rows, 0), (1, 1), centred());
        self.rows += 1;
    }

    fn text(&mut self, arena: &mut Arena, fonts: (&Font, &Lang), contents: Component) {
        let message = PlainMessage {
            contents,
            width: TEXT_WIDTH,
        };
        let (rect, part) = text_part(arena, fonts, &message);
        self.order.push(Focus::Part(self.parts.len()));
        self.parts.push(part);
        self.add(rect);
    }

    fn button(
        &mut self,
        arena: &mut Arena,
        lang: &Lang,
        label: Component,
        action: Option<RestrictionAction>,
    ) {
        let widget = Widget {
            rect: arena.add(BUTTON_WIDTH, 20),
            active: true,
            kind: Kind::Button {
                label: lang.runs(&label, &Style::default()),
            },
        };
        let rect = widget.rect;
        if action.is_some() {
            self.order.push(Focus::Button(self.buttons.len()));
            self.add(rect);
        }
        self.buttons.push((widget, action));
    }

    fn spacer(&mut self, arena: &mut Arena) {
        let rect = arena.add(10, 10);
        self.add(rect);
    }
}
