//! The layout of a confirmation screen: what it is made of and where each part stands.

use crate::chat::split;
use crate::layout::{Arena, Grid, Item, Settings};
use crate::screen::confirm::{ConfirmAction, head_by_width};
use crate::widgets::{Kind, Widget};
use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `PopupScreen`: the content is the default 250 wide less 18 each side; the buttons are six apart.
pub(super) const POPUP_CONTENT: i32 = 214;
pub(super) const POPUP_BUTTON_SPACING: i32 = 6;

/// `setMaxRows(15)`: a message of more lines ends in an ellipsis.
const MAX_ROWS: usize = 15;

/// What a screen is made of, kept to make it again for another size.
#[derive(Debug, Clone)]
pub(super) struct Content {
    pub(super) title: Component,
    pub(super) message: Component,
    /// The buttons, in order, and how wide each is.
    pub(super) buttons: Vec<(Component, ConfirmAction)>,
    pub(super) button_width: i32,
    /// `addAdditionalText`: a line between the message and the buttons.
    pub(super) extra: Option<Component>,
    /// `FriendsListConfirmScreen`: a dark panel behind, and a wider, padded message.
    pub(super) dark_panel: bool,
    /// `PrivacyConfirmLinkScreen`: the title wraps and centres like the message (four rows).
    pub(super) wrapped_title: bool,
    /// `PopupScreen`: a bold title, messages 214 wide and buttons six apart.
    pub(super) popup: bool,
}

/// `MultiLineLabel.create(font, maxWidth, maxRows, message)`: the lines,
/// the last one clipped and ended in an ellipsis when there are too many.
pub(super) fn message_lines(
    font: &Font,
    lang: &Lang,
    message: &Component,
    max_width: i32,
) -> Vec<Vec<Run>> {
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

/// The leaves of a screen's column.
pub(super) struct Rects {
    title: usize,
    message: usize,
    extra: Option<usize>,
    buttons: Vec<usize>,
}

/// The width a wrapped title may take.
pub(super) fn max_width_title(content: &Content, width: i32) -> i32 {
    if content.popup {
        POPUP_CONTENT
    } else {
        width - 50
    }
}

/// The widest of the lines.
pub(super) fn widest_of(font: &Font, lines: &[Vec<Run>]) -> i32 {
    lines
        .iter()
        .map(|line| font.width_of_runs(line))
        .max()
        .unwrap_or(0)
}

/// `setMaxRows(4)` and `setCentered(true)`: the title of a screen whose title wraps.
pub(super) fn wrapped_title(
    font: &Font,
    lang: &Lang,
    title: &Component,
    max_width: i32,
) -> Vec<Vec<Run>> {
    let mut lines = message_lines(font, lang, title, max_width);
    lines.truncate(4);
    lines
}

/// Lines centred on `middle`, the first at `y`, nine apart.
pub(super) fn draw_centred(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    lines: &[Vec<Run>],
    (middle, mut y): (i32, i32),
) {
    for line in lines {
        let x = middle - font.width_of_runs(line) / 2;
        frame.styled_text(atlas, font, line, (x, y), 0xFFFF_FFFF, true);
        y += 9;
    }
}

/// The content of `ConfirmLinkScreen` for `url`.
pub(super) fn link_content(url: &str, trusted: bool) -> Content {
    let translated = Component::translatable;
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
        dark_panel: false,
        wrapped_title: false,
        popup: false,
    }
}

/// A column spaced by 8, its cells centred; the buttons a row spaced by 4, each with 16 above.
pub(super) fn arrange_column(
    arena: &mut Arena,
    rects: &Rects,
    (width, height): (i32, i32),
    (dark, popup): (bool, bool),
) -> Grid {
    let mut row = Grid::new();
    row.column_spacing = if popup { POPUP_BUTTON_SPACING } else { 4 };
    for (index, rect) in rects.buttons.iter().enumerate() {
        row.add(
            Item::Leaf(*rect),
            (0, index),
            (1, 1),
            Settings::default().padding_top(if popup { 0 } else { 16 }),
        );
    }
    let centred = Settings::default().align_horizontally(0.5);
    let message_cell = if dark {
        centred.padding(10, 5, 10, 5)
    } else {
        centred
    };
    let mut column = Grid::new();
    column.row_spacing = if popup { 12 } else { 8 };
    column.add(Item::Leaf(rects.title), (0, 0), (1, 1), centred);
    column.add(Item::Leaf(rects.message), (1, 0), (1, 1), message_cell);
    let mut next = 2;
    if let Some(extra) = rects.extra {
        column.add(Item::Leaf(extra), (next, 0), (1, 1), centred);
        next += 1;
    }
    column.add(Item::Grid(Box::new(row)), (next, 0), (1, 1), centred);
    column.arrange(arena);
    column.align_in_rectangle(arena, (0, 0, width, height), (0.5, 0.5));
    column
}

/// The line between the message and the buttons, and the buttons.
fn extra_and_buttons(
    (font, lang): (&Font, &Lang),
    content: &Content,
    arena: &mut Arena,
) -> (Option<Widget>, Vec<(Widget, ConfirmAction)>) {
    let runs = |component: &Component| lang.runs(component, &Style::default());
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
            let kind = Kind::Button { label: runs(label) };
            (
                Widget {
                    rect,
                    active: true,
                    kind,
                },
                *action,
            )
        })
        .collect();
    (extra, buttons)
}

/// A screen laid out: the widgets and where they stand.
pub(super) struct Laid {
    pub(super) arena: Arena,
    pub(super) title: Widget,
    pub(super) title_lines: Option<Vec<Vec<Run>>>,
    pub(super) message: Vec<Vec<Run>>,
    pub(super) message_rect: usize,
    pub(super) buttons: Vec<(Widget, ConfirmAction)>,
    pub(super) extra: Option<Widget>,
    pub(super) panel: Option<(i32, i32, i32, i32)>,
}

/// `ConfirmScreen.init`: the widgets of `content` in a column centred in a screen of `size`.
pub(super) fn lay_out(font: &Font, lang: &Lang, size: (i32, i32), content: &Content) -> Laid {
    let width = size.0;
    let mut arena = Arena::default();
    let runs = |component: &Component| lang.runs(component, &Style::default());
    let title_text = runs(&content.title);
    let title_lines = content
        .wrapped_title
        .then(|| wrapped_title(font, lang, &content.title, max_width_title(content, width)));
    let title_rect = match &title_lines {
        Some(lines) => arena.add(widest_of(font, lines), lines.len() as i32 * 9),
        None => arena.add(font.width_of_runs(&title_text), 9),
    };
    let max_width = match (content.popup, content.dark_panel) {
        (true, _) => POPUP_CONTENT,
        (_, true) => width - 180,
        _ => width - 50,
    };
    let message = message_lines(font, lang, &content.message, max_width);
    let widest = widest_of(font, &message);
    let message_rect = arena.add(max_width.min(widest), message.len() as i32 * 9);
    let (extra, buttons) = extra_and_buttons((font, lang), content, &mut arena);
    let rects = Rects {
        title: title_rect,
        message: message_rect,
        extra: extra.as_ref().map(|extra| extra.rect),
        buttons: buttons.iter().map(|(widget, _)| widget.rect).collect(),
    };
    let column = arrange_column(
        &mut arena,
        &rects,
        size,
        (content.dark_panel, content.popup),
    );
    let panel = (content.dark_panel || content.popup).then_some((
        column.x,
        column.y,
        column.width,
        column.height,
    ));
    let title = Widget {
        rect: title_rect,
        active: false,
        kind: Kind::Text { text: title_text },
    };
    Laid {
        arena,
        title,
        title_lines,
        message,
        message_rect,
        buttons,
        extra,
        panel,
    }
}
