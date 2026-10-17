//! One frame of GUI: textured triangles in GUI units and the runs they blend
//! in, and the drawing calls the HUD and the screens build it with
//! (`GuiGraphics`).
//!
//! Everything is in GUI units: the window's pixels divided by the GUI scale,
//! which by default is the largest whole number that keeps the GUI at least
//! 320 by 240.

use super::TextRun;
use super::atlas::GuiAtlas;

/// One GUI vertex, in GUI units.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GuiVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub colour: [f32; 4],
}

/// How a run of GUI triangles blends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiBlend {
    /// Source alpha over the destination.
    Translucent,
    /// The crosshair's inversion of what is behind it.
    Invert,
    /// A text selection, added to what is behind it.
    Highlight,
    /// The credits' vignette: what is behind it darkened by its colour.
    Vignette,
    /// Items in slots, whose run counts in `GuiFrame::items`.
    Items,
    /// The player's model in a screen, whose run counts in
    /// `GuiFrame::items` and which samples the player's skin.
    Entity,
}

/// A drawing call as the behaviour tests compare it with the reference's
/// recorded GUI state.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawOp {
    Fill {
        rect: (f32, f32, f32, f32),
        colour: u32,
    },
    Sprite {
        name: String,
        rect: (f32, f32, f32, f32),
        /// The tint, `0xAARRGGBB` (white for a sprite drawn as it is).
        colour: u32,
    },
    /// A part of a whole texture, as fractions of it.
    Texture {
        name: String,
        rect: (f32, f32, f32, f32),
        uv: (f32, f32, f32, f32),
        /// The tint, as ARGB.
        colour: u32,
    },
    Text {
        /// Where the text starts, after any pose a caller applied, and the
        /// scale of the pose.
        x: f32,
        y: f32,
        scale: f32,
        colour: u32,
        shadow: bool,
        /// The box its glyphs fill, which decides whether a scissor leaves
        /// any of it.
        bounds: (i32, i32, i32, i32),
        /// Each run: its text, its colour if it has one, bold, italic.
        runs: Vec<(String, Option<u32>, bool, bool)>,
    },
}

/// `ScreenRectangle`: where a widget is, and how big.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Where a tooltip goes (`ClientTooltipPositioner`).
#[derive(Debug, Clone, Copy)]
pub enum Positioner {
    /// `DefaultTooltipPositioner`: by the mouse.
    Default,
    /// `MenuTooltipPositioner`: by the mouse, off the widget.
    Menu(ScreenRect),
    /// `BelowOrAboveWidgetTooltipPositioner`: under the widget, or over it.
    BelowOrAbove(ScreenRect),
}

impl Positioner {
    /// `positionTooltip`: the top left of the tooltip's text for a tooltip
    /// `size` big and the mouse at `mouse`, in a screen of `screen`.
    pub fn position(
        &self,
        (screen_width, screen_height): (i32, i32),
        (mouse_x, mouse_y): (i32, i32),
        (width, height): (i32, i32),
    ) -> (i32, i32) {
        match self {
            Self::Default => {
                let (mut x, mut y) = (mouse_x + 12, mouse_y - 12);
                if x + width > screen_width {
                    x = (x - 24 - width).max(4);
                }
                if y + height + 3 > screen_height {
                    y = screen_height - (height + 3);
                }
                (x, y)
            }
            Self::Menu(widget) => {
                let (top, bottom) = (widget.y, widget.y + widget.height);
                // `getOffset`: how far from the widget, by how far the mouse is.
                let offset = |mouse_y: i32, widget_y: i32, widget_height: i32| {
                    let distance = (mouse_y - widget_y).abs().min(widget_height);
                    let delta = distance as f32 / widget_height as f32;
                    let start = (widget_height - 3) as f32;
                    ((start + delta * (5.0 - start)) + 0.5).floor() as i32
                };
                let mut x = mouse_x + 12;
                if x + width > screen_width - 5 {
                    x = (mouse_x - 12 - width).max(9);
                }
                let mut y = mouse_y + 3;
                let padded = height + 3 + 3;
                let lowest = bottom + 3 + offset(0, 0, widget.height);
                if lowest + padded <= screen_height - 5 {
                    y += offset(y, top, widget.height);
                } else {
                    y -= padded + offset(y, bottom, widget.height);
                }
                (x, y)
            }
            Self::BelowOrAbove(widget) => {
                let (right, bottom) = (widget.x + widget.width, widget.y + widget.height);
                let mut x = widget.x + 3;
                let mut y = bottom + 3 + 1;
                if y + height + 3 > screen_height {
                    y = widget.y - height - 3 - 1;
                }
                if x + width > screen_width {
                    x = (right - width - 3).max(4);
                }
                (x, y)
            }
        }
    }
}

/// A place in a [`GuiFrame`]: how much of it was drawn.
#[derive(Debug, Clone, Copy)]
pub struct FrameMark {
    vertices: usize,
    items: usize,
    log: usize,
}

/// One frame of GUI: triangles and the runs they blend in.
#[derive(Debug, Default)]
pub struct GuiFrame {
    /// The calls made, when a test asked to keep them.
    pub log: Option<Vec<DrawOp>>,
    pub vertices: Vec<GuiVertex>,
    pub batches: Vec<(GuiBlend, u32, u32)>,
    pub items: Vec<crate::gui::items::GuiItemVertex>,
    /// The clock a label that does not fit in its button scrolls by
    /// (`Util.getMillis`).
    pub millis: u64,
    /// The player's item cooldowns at this frame: each group and what is left
    /// of it (`ItemCooldowns.getCooldownPercent`).
    pub cooldowns: Vec<(String, f32)>,
}

/// Draws an item's triangles into the 16 by 16 slot at a GUI position.
pub type ItemDrawer<'a> = dyn FnMut(&photon_world::ClientItemStackState, f32, f32, &mut Vec<crate::gui::items::GuiItemVertex>)
    + 'a;

/// The runs as the reference's character sequence yields them: neighbours in
/// one style are one stretch.
fn merged_runs(runs: &[TextRun]) -> Vec<(String, Option<u32>, bool, bool)> {
    let mut merged: Vec<(String, Option<u32>, bool, bool)> = Vec::new();
    for run in runs {
        match merged.last_mut() {
            Some(previous) if run.joins_previous => previous.0.push_str(&run.text),
            _ => merged.push((run.text.to_string(), run.colour, run.bold, run.italic)),
        }
    }
    merged
}

/// Each glyph of styled text as a quad of four corners (top left, bottom
/// left, bottom right, top right), as `BakedSheetGlyph.render` places them:
/// the shadow's copies first when it has one, bold glyphs twice a unit apart
/// and italic ones sheared. `x` and `y` are where the line starts.
pub(crate) fn glyph_quads(
    atlas: &GuiAtlas,
    font: &crate::gui::font::Font,
    runs: &[TextRun],
    (x, y): (f32, f32),
    default_colour: u32,
    shadow: bool,
) -> Vec<[GuiVertex; 4]> {
    let unpack =
        |colour: u32| [16, 8, 0, 24].map(|shift| ((colour >> shift) & 0xFF) as f32 / 255.0);
    let mut quads = Vec::new();
    let mut pen = x;
    for run in runs {
        let colour = run
            .colour
            .map_or(default_colour, |rgb| (default_colour & 0xFF00_0000) | rgb);
        // `ARGB.scaleRGB(colour, 0.25)`.
        let shadow_colour = [16, 8, 0]
            .map(|shift| ((((colour >> shift) & 0xFF) as f32 * 0.25) as u32) << shift)
            .into_iter()
            .fold(colour & 0xFF00_0000, |sum, channel| sum | channel);
        // A style that names its shadow colour has that one; a clear one has none.
        let shadow_colour = run.shadow.unwrap_or(shadow_colour);
        let shadow_drawn = shadow && shadow_colour >> 24 != 0;
        let (shadow_colour, colour) = (unpack(shadow_colour), unpack(colour));
        let bold = run.bold;
        let italic = run.italic;
        for codepoint in run.text.chars() {
            let Some(glyph) = font.glyph(codepoint) else {
                continue;
            };
            if let crate::gui::font::Glyph::Bitmap(bitmap) = glyph {
                let (name, image) = &font.images[bitmap.image];
                let [u0, v0, u1, v1] = atlas.sprite(name);
                let [cx, cy, cw, ch] = bitmap.cell;
                let su = |pixel: u32| u0 + (u1 - u0) * pixel as f32 / image.width as f32;
                let sv = |pixel: u32| v0 + (v1 - v0) * pixel as f32 / image.height as f32;
                // `GlyphBitmap`: the cell from its left edge, its top at
                // seven units less the ascent.
                let up = 7.0 - bitmap.ascent as f32;
                let down = up + ch as f32 * bitmap.scale;
                let (shear_top, shear_bottom) = if italic {
                    (1.0 - 0.25 * up, 1.0 - 0.25 * down)
                } else {
                    (0.0, 0.0)
                };
                let thickness = if bold { 0.1 } else { 0.0 };
                // `(offset, bold shift, colour)` for each pass: the shadow's,
                // then the glyph's; a bold glyph twice.
                let mut passes: Vec<(f32, f32, [f32; 4])> = Vec::new();
                if shadow_drawn {
                    passes.push((1.0, 0.0, shadow_colour));
                    if bold {
                        passes.push((1.0, 1.0, shadow_colour));
                    }
                }
                passes.push((0.0, 0.0, colour));
                if bold {
                    passes.push((0.0, 1.0, colour));
                }
                for (offset, shift, colour) in passes {
                    let left = pen + offset + shift;
                    let right = left + cw as f32 * bitmap.scale;
                    let top = y + up + offset;
                    let bottom = y + down + offset;
                    let corner = |px: f32, py: f32, uv: [f32; 2]| GuiVertex {
                        position: [px, py],
                        uv,
                        colour,
                    };
                    quads.push([
                        corner(
                            left + shear_top - thickness,
                            top - thickness,
                            [su(cx), sv(cy)],
                        ),
                        corner(
                            left + shear_bottom - thickness,
                            bottom + thickness,
                            [su(cx), sv(cy + ch)],
                        ),
                        corner(
                            right + shear_bottom + thickness,
                            bottom + thickness,
                            [su(cx + cw), sv(cy + ch)],
                        ),
                        corner(
                            right + shear_top + thickness,
                            top - thickness,
                            [su(cx + cw), sv(cy)],
                        ),
                    ]);
                }
            }
            pen += glyph.advance(bold);
        }
    }
    quads
}

mod overlay;
mod text;
mod texture;

#[cfg(test)]
mod tests {
    use super::*;

    /// A rectangle `(x0, y0, x1, y1)` as the frame builds it, its texture
    /// running from `(0, 0)` to `(1, 1)`.
    fn quad(frame: &mut GuiFrame, (x0, y0, x1, y1): (f32, f32, f32, f32)) {
        let corner = |x: f32, y: f32, u: f32, v: f32| GuiVertex {
            position: [x, y],
            uv: [u, v],
            colour: [1.0; 4],
        };
        let corners = [
            corner(x0, y0, 0.0, 0.0),
            corner(x0, y1, 0.0, 1.0),
            corner(x1, y1, 1.0, 1.0),
            corner(x1, y0, 1.0, 0.0),
        ];
        for index in [0, 1, 2, 0, 2, 3] {
            frame.vertices.push(corners[index]);
        }
    }

    #[test]
    fn a_scissor_cuts_a_rectangle_and_its_texture_and_drops_what_is_outside() {
        let mut frame = GuiFrame {
            log: Some(Vec::new()),
            ..GuiFrame::default()
        };
        let mark = frame.mark();
        let sprite = |(x0, y0, x1, y1): (i32, i32, i32, i32)| DrawOp::Sprite {
            name: "a".to_owned(),
            rect: (x0 as f32, y0 as f32, x1 as f32, y1 as f32),
            colour: 0xFFFF_FFFF,
        };
        // Half in, wholly below, wholly above.
        quad(&mut frame, (0.0, 0.0, 10.0, 20.0));
        quad(&mut frame, (0.0, 30.0, 10.0, 40.0));
        quad(&mut frame, (0.0, 2.0, 10.0, 8.0));
        frame.log.as_mut().unwrap().extend([
            sprite((0, 0, 10, 20)),
            sprite((0, 30, 10, 40)),
            sprite((0, 2, 10, 8)),
        ]);
        frame.clip_from(mark, (0, 10, 10, 25));
        // The log keeps what the scissor leaves any of, as the reference.
        assert_eq!(frame.log.as_ref().unwrap(), &[sprite((0, 0, 10, 20))]);
        // The first rectangle keeps the part from 10 down, its texture the
        // lower half; the second has none left.
        let first = &frame.vertices[0..6];
        assert!(first.iter().all(|vertex| vertex.position[1] >= 10.0));
        assert!(first.iter().any(|vertex| (vertex.uv[1] - 0.5).abs() < 1e-6));
        let second = &frame.vertices[6..12];
        assert!(
            second
                .iter()
                .all(|vertex| vertex.position == second[0].position)
        );
    }
}
