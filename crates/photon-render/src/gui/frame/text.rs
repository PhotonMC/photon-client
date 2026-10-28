use super::*;
use crate::gui::text_effect::{TextEffect, effects};

impl GuiFrame {
    /// `GuiGraphics.blitSprite` of a nine-slice sprite: the corners as they
    /// are, the edges and the middle tiled (or stretched, when the sprite
    /// says so) over what is left. `tex` is the sprite's size in units and
    /// `edges` its border's left, top, right and bottom.
    pub fn nine_slice_edges(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        tex: (i32, i32),
        (left, top, right, bottom): (i32, i32, i32, i32),
        stretch_inner: bool,
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        // Nothing for no width or height; a border is at most half the size.
        if w == 0 || h == 0 {
            return;
        }
        let (left, right) = (left.min(w / 2), right.min(w / 2));
        let (top, bottom) = (top.min(h / 2), bottom.min(h / 2));
        if let Some(log) = &mut self.log {
            // The reference blits a nine-slice sprite as one call.
            log.push(DrawOp::Sprite {
                name: name.to_owned(),
                rect: (x as f32, y as f32, (x + w) as f32, (y + h) as f32),
                colour: 0xFFFF_FFFF,
            });
        }
        let blend = GuiBlend::Translucent;
        let (inner_w, inner_h) = (tex.0 - left - right, tex.1 - top - bottom);
        let (mid_w, mid_h) = (w - left - right, h - top - bottom);
        // One segment: its source origin and size, and where it goes.
        let mut segment = |source: (i32, i32, i32, i32), dest: (i32, i32, i32, i32)| {
            let (sx, sy, sw, sh) = source;
            let (dx, dy, dw, dh) = dest;
            if dw <= 0 || dh <= 0 {
                return;
            }
            if stretch_inner || (sw == dw && sh == dh) {
                self.blit_region(atlas, blend, name, tex, (sx, sy, sw, sh), dest);
                return;
            }
            // Tiled: whole tiles, then the part of one that is left.
            let mut ty = 0;
            while ty < dh {
                let piece_h = sh.min(dh - ty);
                let mut tx = 0;
                while tx < dw {
                    let piece_w = sw.min(dw - tx);
                    self.blit_region(
                        atlas,
                        blend,
                        name,
                        tex,
                        (sx, sy, piece_w, piece_h),
                        (dx + tx, dy + ty, piece_w, piece_h),
                    );
                    tx += sw;
                }
                ty += sh;
            }
        };
        let (far_x, far_y) = (x + w - right, y + h - bottom);
        // Corners.
        segment((0, 0, left, top), (x, y, left, top));
        segment((tex.0 - right, 0, right, top), (far_x, y, right, top));
        segment((0, tex.1 - bottom, left, bottom), (x, far_y, left, bottom));
        segment(
            (tex.0 - right, tex.1 - bottom, right, bottom),
            (far_x, far_y, right, bottom),
        );
        // Edges.
        segment((left, 0, inner_w, top), (x + left, y, mid_w, top));
        segment(
            (left, tex.1 - bottom, inner_w, bottom),
            (x + left, far_y, mid_w, bottom),
        );
        segment((0, top, left, inner_h), (x, y + top, left, mid_h));
        segment(
            (tex.0 - right, top, right, inner_h),
            (far_x, y + top, right, mid_h),
        );
        // The middle.
        segment(
            (left, top, inner_w, inner_h),
            (x + left, y + top, mid_w, mid_h),
        );
    }

    /// `TooltipRenderUtil` and `GuiGraphics.tooltip`: a box of text lines
    /// beside the mouse. Lines are runs of styled text, each 10 units tall
    /// with the first followed by two more; a single line is two units
    /// shorter. The box goes where the mouse's tooltip position puts it in a
    /// screen `screen` units big.
    pub fn tooltip_runs(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<TextRun>],
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        self.tooltip_with_runs(atlas, font, lines, &Positioner::Default, mouse, screen);
    }

    /// A tooltip placed by a positioner (`ClientTooltipPositioner`).
    pub fn tooltip_with_runs(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<TextRun>],
        positioner: &Positioner,
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        if lines.is_empty() {
            return;
        }
        let width = lines
            .iter()
            .map(|line| font.width_of_text_runs(line))
            .max()
            .unwrap_or(0);
        let height = lines.len() as i32 * 10 - if lines.len() == 1 { 2 } else { 0 };
        let (x, y) = positioner.position(screen, mouse, (width, height));
        self.tooltip_box(atlas, x, y, width, height);
        let mut line_y = y;
        for (index, line) in lines.iter().enumerate() {
            self.styled_text_runs(atlas, font, line, (x, line_y), 0xFFFF_FFFF, true);
            line_y += 10 + if index == 0 { 2 } else { 0 };
        }
    }

    /// `TooltipRenderUtil.extractTooltipBackground`: the background and the
    /// frame around a tooltip's contents at `(x, y)`, `width` by `height`.
    pub fn tooltip_box(&mut self, atlas: &GuiAtlas, x: i32, y: i32, width: i32, height: i32) {
        let (x0, y0) = (x - 3 - 9, y - 3 - 9);
        let (padded_w, padded_h) = (width + 6 + 18, height + 6 + 18);
        self.nine_slice(
            atlas,
            "tooltip/background",
            (100, 100),
            9,
            false,
            x0,
            y0,
            padded_w,
            padded_h,
        );
        self.nine_slice(
            atlas,
            "tooltip/frame",
            (100, 100),
            10,
            true,
            x0,
            y0,
            padded_w,
            padded_h,
        );
    }

    /// `GuiGraphics.text` without a shadow: each glyph of the default font
    /// at its place, in one colour (`0xAARRGGBB`).
    pub fn text(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        text: &str,
        x: i32,
        y: i32,
        colour: u32,
    ) {
        self.glyphs(atlas, font, text, (x, y), colour, false);
    }

    /// `GuiGraphics.text` with its shadow: each glyph's copy a unit right and
    /// down in a quarter of its colour, then the glyph.
    pub fn text_with_shadow(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        text: &str,
        x: i32,
        y: i32,
        colour: u32,
    ) {
        self.glyphs(atlas, font, text, (x, y), colour, true);
    }

    pub(super) fn glyphs(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        text: &str,
        at: (i32, i32),
        colour: u32,
        shadow: bool,
    ) {
        let runs = [TextRun::plain(text)];
        self.styled_text_runs(atlas, font, &runs, at, colour, shadow);
    }

    /// Keeps a text call for a test that asked for the calls.
    pub(super) fn log_text(
        &mut self,
        (x, y): (i32, i32),
        runs: &[TextRun],
        colour: u32,
        shadow: bool,
        bounds: Option<(i32, i32, i32, i32)>,
        effects: Vec<TextEffect>,
    ) {
        let Some(log) = &mut self.log else {
            return;
        };
        // The reference draws no call for text with no glyph to draw.
        let Some(bounds) = bounds else {
            return;
        };
        log.push(DrawOp::Text {
            x: x as f32,
            y: y as f32,
            scale: 1.0,
            colour,
            shadow,
            bounds,
            runs: merged_runs(runs),
            effects,
        });
    }

    /// `GuiGraphics.text` of styled text: each run in its colour, with the
    /// bold glyphs drawn twice a unit apart and the italic ones sheared, then
    /// the underlines and strikethrough over them. Obfuscation is not drawn.
    pub fn styled_text_runs(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        runs: &[TextRun],
        (x, y): (i32, i32),
        default_colour: u32,
        shadow: bool,
    ) {
        let effects = effects(font, runs, (x as f32, y as f32), (default_colour, shadow));
        self.log_text(
            (x, y),
            runs,
            default_colour,
            shadow,
            crate::gui::text_hit::bounds(font, runs, (x, y), shadow),
            effects.clone(),
        );
        for corners in glyph_quads(
            atlas,
            font,
            runs,
            (x as f32, y as f32),
            default_colour,
            shadow,
        ) {
            self.push_text_quad(corners);
        }
        for corners in effect_quads(atlas, effects) {
            self.push_text_quad(corners);
        }
    }

    /// A quad of translucent text triangles.
    fn push_text_quad(&mut self, corners: [GuiVertex; 4]) {
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        match self.batches.last_mut() {
            Some((GuiBlend::Translucent, _, count)) => *count += 6,
            _ => self.batches.push((GuiBlend::Translucent, first, 6)),
        }
    }

    /// `GuiGraphics.blitSprite` of a whole sprite in a tint (`0xAARRGGBB`).
    pub fn sprite_tinted(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        (x, y, w, h): (i32, i32, i32, i32),
        colour: u32,
    ) {
        self.sprite(atlas, name, x, y, w, h);
        if let Some(DrawOp::Sprite { colour: logged, .. }) =
            self.log.as_mut().and_then(|log| log.last_mut())
        {
            *logged = colour;
        }
        let tint = [16, 8, 0, 24].map(|shift| ((colour >> shift) & 0xFF) as f32 / 255.0);
        let first = self.vertices.len().saturating_sub(6);
        for vertex in &mut self.vertices[first..] {
            vertex.colour = tint;
        }
    }

    /// `GuiGraphics.blitSprite` of part of a sprite: the sprite is taken as
    /// `size` units, and the part from `(u, v)` as big as the drawn area.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
    pub fn blit_part(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        size: (i32, i32),
        uv: (i32, i32),
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        self.blit_region(atlas, blend, name, size, (uv.0, uv.1, w, h), (x, y, w, h));
    }

    /// `GuiGraphics.blitSprite` of part of a sprite as the reference's draw log records it: one
    /// sprite at the rectangle drawn.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
    pub fn sprite_cut(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        size: (i32, i32),
        uv: (i32, i32),
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        if let Some(log) = &mut self.log {
            log.push(DrawOp::Sprite {
                name: name.to_owned(),
                rect: (x as f32, y as f32, (x + w) as f32, (y + h) as f32),
                colour: 0xFFFF_FFFF,
            });
        }
        self.blit_part(atlas, GuiBlend::Translucent, name, size, uv, x, y, w, h);
    }

    /// A region of a sprite, `(u, v, width, height)` of a sprite `size`
    /// units big, stretched to a rectangle `(x, y, w, h)`.
    pub fn blit_region(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        size: (i32, i32),
        (u, v, uw, vh): (i32, i32, i32, i32),
        rect: (i32, i32, i32, i32),
    ) {
        let region = (u as f32, v as f32, uw as f32, vh as f32);
        self.blit_region_f32(atlas, blend, name, size, region, rect);
    }

    /// `blit_region` with the first texel anywhere in the texture: a scrolling background.
    pub fn blit_region_f32(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        size: (i32, i32),
        (u, v, uw, vh): (f32, f32, f32, f32),
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        if w == 0 || h == 0 || uw == 0.0 || vh == 0.0 {
            return;
        }
        let blend = atlas.blend_of(name, blend);
        let [u0, v0, u1, v1] = atlas.sprite(name);
        let su = |offset: f32| u0 + (u1 - u0) * offset / size.0 as f32;
        let sv = |offset: f32| v0 + (v1 - v0) * offset / size.1 as f32;
        let (left, right) = (su(u), su(u + uw));
        let (top, bottom) = (sv(v), sv(v + vh));
        let corner = |px: i32, py: i32, uv: [f32; 2]| GuiVertex {
            position: [px as f32, py as f32],
            uv,
            colour: [1.0; 4],
        };
        let corners = [
            corner(x, y, [left, top]),
            corner(x, y + h, [left, bottom]),
            corner(x + w, y + h, [right, bottom]),
            corner(x + w, y, [right, top]),
        ];
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        match self.batches.last_mut() {
            Some((last, _, count)) if *last == blend => *count += 6,
            _ => self.batches.push((blend, first, 6)),
        }
    }

    /// `GuiGraphics.fill` through a pipeline's blend: the rectangle's quad moved into a run of
    /// its own blend.
    pub fn fill_blend(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        rect: (i32, i32, i32, i32),
        colour: u32,
    ) {
        self.fill(atlas, rect, colour);
        let first = self.vertices.len() as u32 - 6;
        if let Some((last, start, count)) = self.batches.last_mut() {
            if *count == 6 && *start == first {
                *last = blend;
            } else {
                *count -= 6;
                self.batches.push((blend, first, 6));
            }
        }
        // Neighbours of one blend are one run.
        if let [.., (a, _, count), (b, _, added)] = self.batches.as_mut_slice()
            && *a == *b
        {
            *count += *added;
            self.batches.pop();
        }
    }

    /// `GuiGraphics.textHighlight` over inverted text: what is behind inverted, then the
    /// selection's blue added.
    pub fn text_highlight(&mut self, atlas: &GuiAtlas, rect: (i32, i32, i32, i32)) {
        self.fill_blend(atlas, GuiBlend::Invert, rect, 0xFFFF_FFFF);
        self.fill_blend(atlas, GuiBlend::Highlight, rect, 0xFF00_00FF);
    }
}

/// The quads of text effects (`BakedSheetGlyph.renderEffect`): each rectangle samples the middle
/// of the white sprite, its shadow's copy first, a unit right and down.
pub(crate) fn effect_quads(atlas: &GuiAtlas, effects: Vec<TextEffect>) -> Vec<[GuiVertex; 4]> {
    let [u0, v0, u1, v1] = atlas.sprite("white");
    let uv = [(u0 + u1) / 2.0, (v0 + v1) / 2.0];
    let mut quads = Vec::new();
    for effect in effects {
        let (x0, y0, x1, y1) = effect.rect;
        let copies = [(effect.shadow_colour, 1.0), (effect.colour, 0.0)];
        for (colour, offset) in copies.into_iter().filter(|(colour, _)| *colour != 0) {
            let colour = [16, 8, 0, 24].map(|shift| ((colour >> shift) & 0xFF) as f32 / 255.0);
            let corner = |px: f32, py: f32| GuiVertex {
                position: [px + offset, py + offset],
                uv,
                colour,
            };
            quads.push([
                corner(x0, y0),
                corner(x0, y1),
                corner(x1, y1),
                corner(x1, y0),
            ]);
        }
    }
    quads
}
