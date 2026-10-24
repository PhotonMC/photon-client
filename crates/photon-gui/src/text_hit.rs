//! Which style of a line of text a point is over (`ActiveTextCollector.
//! findElementUnderCursor`): each glyph has an active area, the glyph's box
//! across its advance, and the last glyph whose area holds the point gives
//! its style.

use photon_game::text::{Run, Style};
use photon_render::gui::font::{Font, Glyph};

/// `extraThickness`: what a bold glyph is drawn wider by, each side.
const BOLD_THICKNESS: f32 = 0.1;

/// Where a glyph drawn with its pen at `(pen, y)` can be hit; none for one that is not drawn yet.
fn hit_box(
    glyph: &Glyph,
    (pen, y): (f32, f32),
    (bold, italic): (bool, bool),
) -> Option<(f32, f32, f32, f32)> {
    Some(match glyph {
        // `EmptyArea`: the advance wide and a line tall.
        Glyph::Space(_) => (pen, y, pen + glyph.advance(bold), y + 9.0),
        // A unifont glyph is not drawn yet, so it has no box to hit.
        Glyph::Unihex(_) => return None,
        Glyph::Bitmap(bitmap) => {
            let up = 7.0 - bitmap.ascent as f32;
            let down = up + bitmap.cell[3] as f32 * bitmap.scale;
            let shear = if italic {
                (1.0 - 0.25 * up).min(1.0 - 0.25 * down)
            } else {
                0.0
            };
            let thickness = if bold { BOLD_THICKNESS } else { 0.0 };
            (
                pen + shear - thickness,
                y + up - thickness,
                pen + glyph.advance(bold),
                // The shadow is a unit down.
                y + down + 1.0 + thickness,
            )
        }
    })
}

/// The style at `point` of `runs` drawn with their top left at `origin` (and
/// a shadow, as chat text has).
pub fn style_under<'a>(
    font: &Font,
    runs: &'a [Run],
    (x, y): (i32, i32),
    (point_x, point_y): (f32, f32),
) -> Option<&'a Style> {
    let mut found = None;
    let mut pen = x as f32;
    let y = y as f32;
    for run in runs {
        let bold = run.style.bold == Some(true);
        let italic = run.style.italic == Some(true);
        for codepoint in run.text.chars() {
            let Some(glyph) = font.glyph(codepoint) else {
                continue;
            };
            let hit = hit_box(&glyph, (pen, y), (bold, italic)).is_some_and(
                |(left, top, right, bottom)| {
                    point_x >= left && point_x < right && point_y >= top && point_y < bottom
                },
            );
            if hit {
                found = Some(&run.style);
            }
            pen += glyph.advance(bold);
        }
    }
    found
}
