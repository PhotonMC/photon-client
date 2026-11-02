//! The debug charts F3 shows (`AbstractDebugChart`): a bar for each sample over a dark box, with
//! a border, the smallest, mean and largest sample above it and what the kind of chart adds.

use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};
use photon_render::sky::srgb_lerp;

use super::debug::fixed;

mod kinds;
pub use kinds::Kind;

/// The samples a chart keeps (`LocalSampleLogger.CAPACITY`).
pub const CAPACITY: i32 = 240;
/// `AbstractDebugChart.CHART_HEIGHT`.
const HEIGHT: i32 = 60;
const BACKGROUND: u32 = 0x9050_5050;
const LABEL: u32 = 0xFFE0_E0E0;
const BORDER: u32 = 0xFFFF_FFFF;
/// The dimensions a sample has (`TpsDebugDimensions`); the other charts use the first.
pub const DIMENSIONS: usize = 4;

/// `AbstractDebugChart.getWidth`: the room a chart takes in a window `max_width` wide.
pub fn width(max_width: i32) -> i32 {
    (CAPACITY + 2).min(max_width)
}

/// `AbstractDebugChart.getFullHeight`.
pub const FULL_HEIGHT: i32 = HEIGHT + 9;

/// `AbstractDebugChart.extractRenderState` for the samples held, oldest first, in a window
/// `gui_height` high: the box at `left`, `width` wide, on the bottom edge.
pub fn draw(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    kind: Kind,
    samples: &[[i64; DIMENSIONS]],
    (left, width, gui_height): (i32, i32, i32),
) {
    let bottom = gui_height;
    frame.fill(
        atlas,
        (left, bottom - HEIGHT, left + width, bottom),
        BACKGROUND,
    );
    let start = (CAPACITY - (width - 2)).max(0);
    let count = samples.len() as i32 - start;
    let (mut sum, mut min, mut max) = (0_i64, i64::from(i32::MAX), i64::from(i32::MIN));
    for index in 0..count.max(0) {
        let row = &samples[(start + index) as usize];
        let value = kind.aggregate(row);
        min = min.min(value);
        max = max.max(value);
        sum += value;
        kind.bars(frame, atlas, row, (left + index + 1, bottom));
    }
    border(frame, atlas, (left, width, bottom));
    if count > 0 {
        labels(
            frame,
            (atlas, font),
            kind,
            (min, sum as f64 / f64::from(count), max),
            (left, width, bottom),
        );
    }
    kind.extras(frame, (atlas, font), (left, width, bottom));
}

/// The four lines round the box: `horizontalLine` takes its end in, `verticalLine` leaves its
/// start out.
fn border(frame: &mut GuiFrame, atlas: &GuiAtlas, (left, width, bottom): (i32, i32, i32)) {
    let top = bottom - HEIGHT;
    let right = left + width - 1;
    frame.fill(atlas, (left, top, right + 1, top + 1), BORDER);
    frame.fill(atlas, (left, bottom - 1, right + 1, bottom), BORDER);
    frame.fill(atlas, (left, top + 1, left + 1, bottom), BORDER);
    frame.fill(atlas, (right, top + 1, right + 1, bottom), BORDER);
}

/// The smallest, mean and largest sample over the box's top edge.
fn labels(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    kind: Kind,
    (min, mean, max): (i64, f64, i64),
    (left, width, bottom): (i32, i32, i32),
) {
    let y = bottom - HEIGHT - 9;
    let low = format!("{} min", kind.display(min as f64));
    let mid = format!("{} avg", kind.display(mean));
    let high = format!("{} max", kind.display(max as f64));
    frame.text_with_shadow(atlas, font, &low, left + 2, y, LABEL);
    let centre = left + width / 2 - font.width(&mid) / 2;
    frame.text_with_shadow(atlas, font, &mid, centre, y, LABEL);
    frame.text_with_shadow(
        atlas,
        font,
        &high,
        left + width - font.width(&high) - 2,
        y,
        LABEL,
    );
}

/// `extractStringWithShade`: the text on a box, with no shadow.
fn shaded(frame: &mut GuiFrame, (atlas, font): (&GuiAtlas, &Font), text: &str, (x, y): (i32, i32)) {
    frame.fill(atlas, (x, y, x + font.width(text) + 1, y + 9), BACKGROUND);
    frame.text(atlas, font, text, x + 1, y + 1, LABEL);
}

/// `horizontalLine`: both ends are in.
fn horizontal(frame: &mut GuiFrame, atlas: &GuiAtlas, (x0, x1, y): (i32, i32, i32), colour: u32) {
    frame.fill(atlas, (x0.min(x1), y, x0.max(x1) + 1, y + 1), colour);
}

/// `Math.round`: halves go up.
fn java_round(value: f64) -> i64 {
    (value + 0.5).floor() as i64
}

/// `getSampleColor` with three stops: the sample is held between the ends and blended in the half
/// it falls in.
fn ramp(sample: f64, (min, mid, max): (f64, f64, f64), colours: (u32, u32, u32)) -> u32 {
    let sample = sample.clamp(min, max);
    if sample < mid {
        srgb_lerp(((sample - min) / (mid - min)) as f32, colours.0, colours.1)
    } else {
        srgb_lerp(((sample - mid) / (max - mid)) as f32, colours.1, colours.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_is_capacity_and_border_or_the_window() {
        assert_eq!(width(1000), 242);
        assert_eq!(width(100), 100);
    }

    #[test]
    fn ping_ramps_from_green_through_yellow_to_red() {
        assert_eq!(Kind::Ping.colour(0), 0xFF00_FF00);
        assert_eq!(Kind::Ping.colour(250), 0xFFFF_FF00);
        assert_eq!(Kind::Ping.colour(900), 0xFFFF_0000);
        assert_eq!(Kind::Ping.bar_height(250.0), 30);
    }
}
