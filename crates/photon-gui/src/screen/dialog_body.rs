//! The body of a dialog screen: its parts (wrapped text, item boxes) and how they are laid out.

use photon_core::ids::ItemId;
use photon_game::dialog::{Body, PlainMessage};
use photon_game::text::{Lang, Run, Style};

use crate::chat::split;
use crate::layout::{Arena, Grid, Item, Settings, horizontal};
use crate::widgets::navigation::Rectangle;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `HeaderAndFooterLayout.DEFAULT_HEADER_AND_FOOTER_HEIGHT`.
pub(super) const BAR: i32 = 33;
/// `FocusableTextWidget.DEFAULT_PADDING`.
pub(super) const PADDING: i32 = 4;

/// The item an item body shows.
pub(super) struct Shown {
    pub(super) id: ItemId,
    pub(super) count: i32,
    pub(super) decorations: bool,
}

/// A focusable part of the body: wrapped text, or an item's box.
pub(super) struct Part {
    pub(super) rect: usize,
    pub(super) lines: Option<Vec<Vec<Run>>>,
    pub(super) item: Option<Shown>,
}

/// `FocusableTextWidget`: `message` wrapped to the width less its padding, as tall as its lines.
pub(super) fn text_part(
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

pub(super) fn centred() -> Settings {
    Settings::default().align_horizontally(0.5)
}

pub(super) fn middle() -> Settings {
    Settings {
        x_align: 0.5,
        y_align: 0.5,
        ..Settings::default()
    }
}

/// `DialogScreen.packControlsIntoColumns`: full rows in a grid spaced by 2, the last short row a
/// centred line of its own across all the columns.
pub(super) fn pack_into_columns(arena: &Arena, leaves: &[usize], columns: usize) -> Grid {
    let mut grid = Grid::new();
    grid.column_spacing = 2;
    grid.row_spacing = 2;
    let last_full_row = leaves.len() / columns;
    let in_full_rows = last_full_row * columns;
    for (index, leaf) in leaves[..in_full_rows].iter().enumerate() {
        grid.add(
            Item::Leaf(*leaf),
            (index / columns, index % columns),
            (1, 1),
            centred(),
        );
    }
    if in_full_rows != leaves.len() {
        let last = horizontal(arena, &leaves[in_full_rows..], 2);
        grid.add(
            Item::Grid(Box::new(last)),
            (last_full_row, 0),
            (1, columns),
            centred(),
        );
    }
    grid
}

/// `DialogBodyHandlers.createBodyElement` for each body, in a column spaced by 10.
pub(super) fn body_layout(
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

/// `GuiGraphicsExtractor.outline` in white.
pub(super) fn outline(frame: &mut GuiFrame, atlas: &GuiAtlas, rect: Rectangle) {
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
