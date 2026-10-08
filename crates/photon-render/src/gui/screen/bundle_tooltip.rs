//! `AbstractContainerScreen.extractTooltip` with a bundle's image: the
//! tooltip's lines with `ClientBundleTooltip` after the first, laid out the
//! way `GuiGraphics.tooltip` lays out mixed components.

use super::ScreenDraw;
use super::container::ContainerLayout;
use crate::gui::frame::Positioner;
use photon_game::menus::container::{BundleView, bundle_view};
use photon_game::text::{Component, Run, Style};
use photon_world::ClientItemStackState as Stack;

/// The image's width, and its slots' size.
const GRID_WIDTH: i32 = 96;
const SLOT: i32 = 24;
const BAR_HEIGHT: i32 = 13;
const WHITE: u32 = 0xFFFF_FFFF;

/// `showTooltipWithItemInHand`: whether a stack's tooltip shows while
/// something is carried (a bundle's does).
pub fn shows_with_item_in_hand(stack: &Stack) -> bool {
    bundle_view(stack).is_some()
}

/// `setTooltipForNextFrame` of a stack: its tooltip lines, with a bundle's
/// image after the first.
pub fn stack_tooltip(
    layout: &ContainerLayout,
    draw: &mut ScreenDraw<'_, '_>,
    stack: &Stack,
    lines: &[Vec<Run>],
    mouse: (f64, f64),
) {
    match bundle_view(stack) {
        Some(view) => tooltip(draw, lines, &view, (layout.width, layout.height), mouse),
        None => layout.tooltip(draw, lines, mouse),
    }
}

/// `AbstractContainerScreen.extractTooltip` for the hovered stack: drawn
/// when nothing is carried or the stack shows anyway; whether it was.
pub fn item_tooltip(
    layout: &ContainerLayout,
    draw: &mut ScreenDraw<'_, '_>,
    stack: &Stack,
    carrying: bool,
    mouse: (f64, f64),
) -> bool {
    if carrying && !shows_with_item_in_hand(stack) {
        return false;
    }
    let lines = (draw.tooltip)(stack, false);
    stack_tooltip(layout, draw, stack, &lines, mouse);
    true
}

/// `GuiGraphics.tooltip` over the lines with the bundle's image put after the
/// first (first of all with no lines): the widest and the heights summed,
/// two less for one component; the text first, then the image.
fn tooltip(
    draw: &mut ScreenDraw<'_, '_>,
    lines: &[Vec<Run>],
    view: &BundleView,
    screen: (i32, i32),
    mouse: (f64, f64),
) {
    let image_at = usize::from(!lines.is_empty());
    let description = description(draw, view);
    let image_height = if view.items.is_empty() {
        description.len() as i32 * 9 + BAR_HEIGHT + 8
    } else {
        rows(view) * SLOT + BAR_HEIGHT + 8
    };
    let count = lines.len() + 1;
    let mut width = GRID_WIDTH;
    let mut height = if count == 1 { -2 } else { 0 };
    for line in lines {
        width = width.max(draw.font.width_of_runs(line));
        height += 10;
    }
    height += image_height;
    let (x, y) =
        Positioner::Default.position(screen, (mouse.0 as i32, mouse.1 as i32), (width, height));
    draw.frame.tooltip_box(draw.atlas, x, y, width, height);
    let mut local_y = y;
    let mut image_y = y;
    for index in 0..count {
        let line_height = if index == image_at {
            image_y = local_y;
            image_height
        } else {
            let line = &lines[index - usize::from(index > image_at)];
            draw.frame
                .styled_text(draw.atlas, draw.font, line, x, local_y, WHITE, true);
            10
        };
        local_y += line_height + if index == 0 { 2 } else { 0 };
    }
    image(draw, view, &description, (x, image_y, width), screen);
}

/// `gridSizeY`: the rows of up to twelve slots.
fn rows(view: &BundleView) -> i32 {
    (view.items.len().min(12) as i32 + 3) / 4
}

/// An empty bundle's description, split to the image's width.
fn description(draw: &ScreenDraw<'_, '_>, view: &BundleView) -> Vec<Vec<Run>> {
    if !view.items.is_empty() {
        return Vec::new();
    }
    let text = Component::translatable("item.minecraft.bundle.empty.description");
    crate::gui::chat::split(
        &draw.lang.runs(&text, &Style::default()),
        GRID_WIDTH,
        draw.font,
    )
}

/// `ClientBundleTooltip.extractImage`: nothing for a weight that is an
/// error; an empty bundle's description over an empty bar; else the slots
/// from the bottom right, the count of what is not shown, the selected
/// stack's name above, and the bar.
fn image(
    draw: &mut ScreenDraw<'_, '_>,
    view: &BundleView,
    description: &[Vec<Run>],
    (x, y, w): (i32, i32, i32),
    screen: (i32, i32),
) {
    let Some(weight) = view.weight else {
        return;
    };
    let left = x + (w - GRID_WIDTH) / 2;
    if view.items.is_empty() {
        for (index, line) in description.iter().enumerate() {
            draw.frame.styled_text(
                draw.atlas,
                draw.font,
                line,
                left,
                y + index as i32 * 9,
                0xFFAA_AAAA,
                true,
            );
        }
        progress_bar(draw, left, y + description.len() as i32 * 9 + 4, (0, 1));
        return;
    }
    let overflowing = view.items.len() > 12;
    let shown = view
        .items
        .len()
        .min(usize::try_from(view.to_show).unwrap_or(0));
    let (x_start, y_start) = (left + GRID_WIDTH, y + rows(view) * SLOT);
    let mut slot_number = 1;
    for row in 1..=rows(view) {
        for column in 1..=4 {
            let (draw_x, draw_y) = (x_start - column * SLOT, y_start - row * SLOT);
            if overflowing && column * row == 1 {
                let hidden: i32 = view.items[shown..].iter().map(Stack::item_count).sum();
                centred_text(
                    draw,
                    &plain(&format!("+{hidden}")),
                    draw_x + 12,
                    draw_y + 10,
                );
            } else if shown >= slot_number {
                let index = shown - slot_number;
                let highlight = index as i32 == view.selected;
                let back = if highlight {
                    "container/bundle/slot_highlight_back"
                } else {
                    "container/bundle/slot_background"
                };
                draw.frame.nine_slice(
                    draw.atlas,
                    back,
                    (24, 24),
                    4,
                    false,
                    draw_x,
                    draw_y,
                    SLOT,
                    SLOT,
                );
                let stack = &view.items[index];
                draw.frame.item_with_count(
                    draw.atlas,
                    draw.font,
                    draw.item,
                    stack,
                    stack.item_count(),
                    None,
                    draw_x + 4,
                    draw_y + 4,
                );
                if highlight {
                    draw.frame.nine_slice(
                        draw.atlas,
                        "container/bundle/slot_highlight_front",
                        (24, 24),
                        4,
                        false,
                        draw_x,
                        draw_y,
                        SLOT,
                        SLOT,
                    );
                }
                slot_number += 1;
            }
        }
    }
    selected_name(draw, view, (x, y, w), screen);
    progress_bar(draw, left, y + rows(view) * SLOT + 4, weight);
}

/// `extractSelectedItemTooltip`: the selected stack's name in a tooltip of
/// its own, centred over the image.
fn selected_name(
    draw: &mut ScreenDraw<'_, '_>,
    view: &BundleView,
    (x, y, w): (i32, i32, i32),
    screen: (i32, i32),
) {
    let Some(stack) = usize::try_from(view.selected)
        .ok()
        .and_then(|index| view.items.get(index))
    else {
        return;
    };
    let Some(name) = (draw.tooltip)(stack, false).into_iter().next() else {
        return;
    };
    let width = draw.font.width_of_runs(&name);
    let centre = x + w / 2 - 12;
    draw.frame.tooltip_with(
        draw.atlas,
        draw.font,
        &[name],
        &Positioner::Default,
        (centre - width / 2, y - 15),
        screen,
    );
}

/// `extractProgressbar`: the fill (full at a whole bundle) as wide as the
/// weight, the border, and "Empty" or "Full" on it.
fn progress_bar(
    draw: &mut ScreenDraw<'_, '_>,
    x: i32,
    y: i32,
    (numerator, denominator): (i64, i64),
) {
    let full = numerator >= denominator;
    // `Mth.mulAndTruncate`.
    let fill = (numerator * 94 / denominator).clamp(0, 94) as i32;
    let sprite = if full {
        "container/bundle/bundle_progressbar_full"
    } else {
        "container/bundle/bundle_progressbar_fill"
    };
    draw.frame.nine_slice(
        draw.atlas,
        sprite,
        (6, 6),
        2,
        false,
        x + 1,
        y,
        fill,
        BAR_HEIGHT,
    );
    draw.frame.nine_slice(
        draw.atlas,
        "container/bundle/bundle_progressbar_border",
        (12, 12),
        2,
        false,
        x,
        y,
        GRID_WIDTH,
        BAR_HEIGHT,
    );
    let key = if numerator == 0 {
        Some("item.minecraft.bundle.empty")
    } else if full {
        Some("item.minecraft.bundle.full")
    } else {
        None
    };
    if let Some(key) = key {
        let runs = draw
            .lang
            .runs(&Component::translatable(key), &Style::default());
        centred_text(draw, &runs, x + 48, y + 3);
    }
}

/// `GuiGraphics.centeredText` in white with its shadow.
fn centred_text(draw: &mut ScreenDraw<'_, '_>, runs: &[Run], x: i32, y: i32) {
    let width = draw.font.width_of_runs(runs);
    draw.frame
        .styled_text(draw.atlas, draw.font, runs, x - width / 2, y, WHITE, true);
}

fn plain(text: &str) -> Vec<Run> {
    vec![Run {
        text: text.to_owned(),
        style: Style::default(),
    }]
}
