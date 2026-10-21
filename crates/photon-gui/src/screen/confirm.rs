//! The confirmation screen (`ConfirmScreen`): a title, a message wrapped and
//! centred under it and two buttons, over the menu background (or the death
//! screen's gradient, for `DeathScreen.TitleConfirmScreen`).

use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Lang, Run, Style};

use crate::chat::split;
use crate::layout::{Arena, Grid, Item, Settings};
use crate::screen::death;
use crate::widgets::{Kind, Widget};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `setMaxRows(15)`: a message of more lines ends in an ellipsis.
const MAX_ROWS: usize = 15;

/// What the screen's callback is told.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Yes,
    No,
    /// `ConfirmLinkScreen`'s "Copy to Clipboard".
    Copy,
}

/// What stands behind the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmBackground {
    /// `extractMenuBackground`: the tiled texture, the in-world one when a
    /// level is loaded.
    Menu { in_world: bool },
    /// `DeathScreen.extractDeathBackground`.
    Death,
}

/// What a screen is made of, kept to make it again for another size.
#[derive(Debug, Clone)]
struct Content {
    title: Component,
    message: Component,
    /// The buttons, in order, and how wide each is.
    buttons: Vec<(Component, ConfirmAction)>,
    button_width: i32,
    /// `addAdditionalText`: a line between the message and the buttons.
    extra: Option<Component>,
}

pub struct ConfirmScreen {
    arena: Arena,
    title: Widget,
    /// The message's lines, and the place of the box they stand in.
    message: Vec<Vec<Run>>,
    message_rect: usize,
    buttons: Vec<(Widget, ConfirmAction)>,
    extra: Option<Widget>,
    size: (i32, i32),
    /// `delayTicker`: the buttons wake, and Escape opens, when it runs out.
    delay: i32,
    focus: Option<usize>,
    background: ConfirmBackground,
    content: Content,
}

/// `Font.substrByWidth`: the start of `runs` that fits in `width`.
fn head_by_width(runs: &[Run], font: &Font, width: i32) -> Vec<Run> {
    let mut left = width as f32;
    let mut head = Vec::new();
    for run in runs {
        let bold = run.style.bold == Some(true);
        let mut kept = String::new();
        for c in run.text.chars() {
            left -= font.glyph(c).map_or(0.0, |glyph| glyph.advance(bold));
            if left < 0.0 {
                if !kept.is_empty() {
                    head.push(Run {
                        text: kept,
                        style: run.style.clone(),
                    });
                }
                return head;
            }
            kept.push(c);
        }
        if !kept.is_empty() {
            head.push(Run {
                text: kept,
                style: run.style.clone(),
            });
        }
    }
    head
}

/// `MultiLineLabel.create(font, maxWidth, maxRows, message)`: the lines,
/// the last one clipped and ended in an ellipsis when there are too many.
fn message_lines(font: &Font, lang: &Lang, message: &Component, max_width: i32) -> Vec<Vec<Run>> {
    let runs = lang.runs(message, &Style::default());
    // `splitLines` of no text is no line.
    let mut lines = if runs.iter().all(|run| run.text.is_empty()) {
        Vec::new()
    } else {
        split(&runs, max_width, font)
    };
    if lines.len() > MAX_ROWS {
        lines.truncate(MAX_ROWS);
        let ellipsis = Component::literal("...");
        let plain = lang.runs(&ellipsis, &Style::default());
        let mut styled = ellipsis;
        styled.style = message.style.clone();
        let last = &lines[MAX_ROWS - 1];
        let keep = font.width_of_runs(last) - font.width_of_runs(&plain);
        let mut clipped = head_by_width(last, font, keep);
        clipped.extend(lang.runs(&styled, &Style::default()));
        lines[MAX_ROWS - 1] = clipped;
    }
    lines
}

impl ConfirmScreen {
    /// `ConfirmScreen.init`, for a title, a message and the two buttons'
    /// labels, in a screen of `width` by `height` units.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        title: &Component,
        message: &Component,
        yes: &Component,
        no: &Component,
    ) -> Self {
        Self::build(
            font,
            lang,
            size,
            Content {
                title: title.clone(),
                message: message.clone(),
                buttons: vec![
                    (yes.clone(), ConfirmAction::Yes),
                    (no.clone(), ConfirmAction::No),
                ],
                button_width: 150,
                extra: None,
            },
        )
    }

    /// `ClientCommonPacketListenerImpl.PackConfirmScreen`: the question about the server's
    /// resource packs, in the wording of a pack that is required or not, with the server's own
    /// prompt under it when it sent one.
    pub fn server_pack(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        required: bool,
        prompt: Option<&Component>,
    ) -> Self {
        let translated = Component::translatable;
        let (title, header, yes, no) = if required {
            (
                translated("multiplayer.requiredTexturePrompt.line1"),
                translated("multiplayer.requiredTexturePrompt.line2").with_style(&Style {
                    colour: photon_game::text::named_colour("yellow"),
                    bold: Some(true),
                    ..Style::default()
                }),
                translated("gui.proceed"),
                translated("menu.disconnect"),
            )
        } else {
            (
                translated("multiplayer.texturePrompt.line1"),
                translated("multiplayer.texturePrompt.line2"),
                translated("gui.yes"),
                translated("gui.no"),
            )
        };
        let message = match prompt {
            Some(prompt) => Component::translatable_with(
                "multiplayer.texturePrompt.serverPrompt",
                vec![header, prompt.clone()],
            ),
            None => header,
        };
        Self::new(font, lang, size, &title, &message, &yes, &no)
    }

    /// `new ConfirmLinkScreen(callback, url, trusted)`: the link asked
    /// about, with "Copy to Clipboard" between the answers, and a warning
    /// when the link is not trusted.
    pub fn link(font: &Font, lang: &Lang, size: (i32, i32), url: &str, trusted: bool) -> Self {
        let translated = Component::translatable;
        Self::build(
            font,
            lang,
            size,
            Content {
                title: translated(if trusted {
                    "chat.link.confirmTrusted"
                } else {
                    "chat.link.confirm"
                }),
                message: Component::literal(url),
                buttons: vec![
                    (
                        translated(if trusted { "chat.link.open" } else { "gui.yes" }),
                        ConfirmAction::Yes,
                    ),
                    (translated("chat.copy"), ConfirmAction::Copy),
                    (
                        translated(if trusted { "gui.cancel" } else { "gui.no" }),
                        ConfirmAction::No,
                    ),
                ],
                button_width: 100,
                extra: (!trusted).then(|| {
                    translated("chat.link.warning").with_style(&Style {
                        colour: Some(0xFF_CCCC),
                        ..Style::default()
                    })
                }),
            },
        )
    }

    fn build(font: &Font, lang: &Lang, size: (i32, i32), content: Content) -> Self {
        let (width, height) = size;
        let mut arena = Arena::default();
        let runs = |component: &Component| lang.runs(component, &Style::default());
        let title_text = runs(&content.title);
        let title_rect = arena.add(font.width_of_runs(&title_text), 9);
        let max_width = width - 50;
        let lines = message_lines(font, lang, &content.message, max_width);
        let widest = lines
            .iter()
            .map(|line| font.width_of_runs(line))
            .max()
            .unwrap_or(0);
        let message_rect = arena.add(max_width.min(widest), lines.len() as i32 * 9);
        let extra = content.extra.as_ref().map(|component| {
            let text = runs(component);
            Widget {
                rect: arena.add(font.width_of_runs(&text), 9),
                active: false,
                kind: Kind::Text { text },
            }
        });
        let buttons: Vec<(Widget, ConfirmAction)> = content
            .buttons
            .iter()
            .map(|(label, action)| {
                let rect = arena.add(content.button_width, 20);
                (
                    Widget {
                        rect,
                        active: true,
                        kind: Kind::Button { label: runs(label) },
                    },
                    *action,
                )
            })
            .collect();
        // A column spaced by 8, its cells centred; the buttons a row spaced
        // by 4, each with 16 above.
        let mut row = Grid::new();
        row.column_spacing = 4;
        for (index, (widget, _)) in buttons.iter().enumerate() {
            row.add(
                Item::Leaf(widget.rect),
                (0, index),
                (1, 1),
                Settings::default().padding_top(16),
            );
        }
        let centred = Settings::default().align_horizontally(0.5);
        let mut column = Grid::new();
        column.row_spacing = 8;
        column.add(Item::Leaf(title_rect), (0, 0), (1, 1), centred);
        column.add(Item::Leaf(message_rect), (1, 0), (1, 1), centred);
        let mut next = 2;
        if let Some(extra) = &extra {
            column.add(Item::Leaf(extra.rect), (next, 0), (1, 1), centred);
            next += 1;
        }
        column.add(Item::Grid(Box::new(row)), (next, 0), (1, 1), centred);
        column.arrange(&mut arena);
        column.align_in_rectangle(&mut arena, (0, 0, width, height), (0.5, 0.5));
        Self {
            title: Widget {
                rect: title_rect,
                active: false,
                kind: Kind::Text { text: title_text },
            },
            message: lines,
            message_rect,
            buttons,
            extra,
            arena,
            size,
            delay: 0,
            focus: None,
            background: ConfirmBackground::Menu { in_world: false },
            content,
        }
    }

    #[must_use]
    pub fn with_background(mut self, background: ConfirmBackground) -> Self {
        self.background = background;
        self
    }

    /// `setDelay`: the buttons sleep, and Escape is shut, for `ticks`.
    pub fn set_delay(&mut self, ticks: i32) {
        self.delay = ticks;
        for (widget, _) in &mut self.buttons {
            widget.active = false;
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The screen again for a screen of another size: as `rebuildWidgets`
    /// leaves it, with the delay as it was and the buttons new (and awake).
    #[must_use]
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        let mut screen =
            Self::build(font, lang, size, self.content.clone()).with_background(self.background);
        screen.delay = self.delay;
        screen
    }

    /// `tick`: the buttons wake when the delay runs out.
    pub fn tick(&mut self) {
        self.delay -= 1;
        if self.delay == 0 {
            for (widget, _) in &mut self.buttons {
                widget.active = true;
            }
        }
    }

    /// The background (over a game, with the frame behind it blurred first), the title, the
    /// message, then the buttons.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
        blurriness: i32,
    ) {
        match self.background {
            ConfirmBackground::Menu { in_world } => {
                if in_world {
                    frame.blur_before_this_stratum(blurriness);
                }
                let texture = if in_world {
                    "inworld_menu_background"
                } else {
                    "menu_background"
                };
                let (width, height) = self.size;
                frame.tile_texture(atlas, texture, (32, 32), (0, 0, width, height));
            }
            ConfirmBackground::Death => death::draw_background(frame, atlas, self.size),
        }
        self.title.draw(frame, atlas, font, &self.arena, false);
        if let Some(extra) = &self.extra {
            extra.draw(frame, atlas, font, &self.arena, false);
        }
        let rect = self.arena.0[self.message_rect];
        let middle = rect.x + rect.width / 2;
        let mut y = rect.y;
        for line in &self.message {
            let x = middle - font.width_of_runs(line) / 2;
            frame.styled_text(atlas, font, line, x, y, 0xFFFF_FFFF, true);
            y += 9;
        }
        for (index, (widget, _)) in self.buttons.iter().enumerate() {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == Some(index);
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
    }

    /// A press of the primary button at `mouse`: the button it was on, if
    /// it was awake.
    pub fn clicked(&self, mouse: (i32, i32)) -> Option<ConfirmAction> {
        self.buttons
            .iter()
            .find(|(widget, _)| widget.is_active() && widget.contains(&self.arena, mouse))
            .map(|(_, action)| *action)
    }

    /// Tab (and Shift+Tab): the focus moves to the next awake button.
    pub fn tab(&mut self, forward: bool) {
        let awake: Vec<usize> = (0..self.buttons.len())
            .filter(|&index| self.buttons[index].0.is_active())
            .collect();
        let (Some(&first), Some(&last)) = (awake.first(), awake.last()) else {
            return;
        };
        let position = self
            .focus
            .and_then(|focus| awake.iter().position(|&index| index == focus));
        self.focus = Some(match (position, forward) {
            (None, true) => first,
            (None, false) => last,
            (Some(at), true) => awake.get(at + 1).copied().unwrap_or(first),
            (Some(at), false) => at.checked_sub(1).map_or(last, |before| awake[before]),
        });
    }

    /// Enter or Space: the focused button, if it is awake.
    pub fn pressed(&self) -> Option<ConfirmAction> {
        let (widget, action) = self.buttons.get(self.focus?)?;
        widget.is_active().then_some(*action)
    }

    /// Escape: "No", once the delay has run out.
    pub fn escape(&self) -> Option<ConfirmAction> {
        (self.delay <= 0).then_some(ConfirmAction::No)
    }
}
