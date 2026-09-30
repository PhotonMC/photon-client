//! One frame of GUI: textured triangles in GUI units and the runs they blend
//! in, and the drawing calls the HUD and the screens build it with
//! (`GuiGraphics`).
//!
//! Everything is in GUI units: the window's pixels divided by the GUI scale,
//! which by default is the largest whole number that keeps the GUI at least
//! 320 by 240.

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
    /// Items in slots, whose run counts in `GuiFrame::items`.
    Items,
    /// The player's model in a screen, whose run counts in
    /// `GuiFrame::items` and which samples the player's skin.
    Entity,
}

/// One frame of GUI: triangles and the runs they blend in.
#[derive(Debug, Default)]
pub struct GuiFrame {
    pub vertices: Vec<GuiVertex>,
    pub batches: Vec<(GuiBlend, u32, u32)>,
    pub items: Vec<crate::gui::items::GuiItemVertex>,
}

impl GuiFrame {
    /// `GuiGraphics.fillGradient`: a rectangle from one colour at the top to
    /// another at the bottom (`0xAARRGGBB`).
    pub fn fill_gradient(
        &mut self,
        atlas: &GuiAtlas,
        (x0, y0, x1, y1): (i32, i32, i32, i32),
        top: u32,
        bottom: u32,
    ) {
        let [u0, v0, u1, v1] = atlas.sprite("white");
        let (u, v) = ((u0 + u1) / 2.0, (v0 + v1) / 2.0);
        let unpack =
            |colour: u32| [16, 8, 0, 24].map(|shift| ((colour >> shift) & 0xFF) as f32 / 255.0);
        let corner = |px: i32, py: i32, colour: u32| GuiVertex {
            position: [px as f32, py as f32],
            uv: [u, v],
            colour: unpack(colour),
        };
        let corners = [
            corner(x0, y0, top),
            corner(x0, y1, bottom),
            corner(x1, y1, bottom),
            corner(x1, y0, top),
        ];
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        match self.batches.last_mut() {
            Some((GuiBlend::Translucent, _, count)) => *count += 6,
            _ => self.batches.push((GuiBlend::Translucent, first, 6)),
        }
    }

    /// `GuiGraphics.fill` in one colour.
    pub fn fill(&mut self, atlas: &GuiAtlas, rect: (i32, i32, i32, i32), colour: u32) {
        self.fill_gradient(atlas, rect, colour, colour);
    }

    /// `GuiGraphics.blitSprite` of a whole sprite.
    pub fn sprite(&mut self, atlas: &GuiAtlas, name: &str, x: i32, y: i32, w: i32, h: i32) {
        self.blit(atlas, GuiBlend::Translucent, name, x, y, w, h);
    }

    /// `GuiGraphics.blit` of a region of a whole texture: `(u, v)` and the
    /// drawn size in texels of a texture `texture_size` big.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_texture(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        texture_size: (i32, i32),
        uv: (i32, i32),
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        self.blit_part(
            atlas,
            GuiBlend::Translucent,
            &format!("texture/{name}"),
            texture_size,
            uv,
            x,
            y,
            w,
            h,
        );
    }

    /// `GuiGraphics.item` and `itemDecorations`: an item in the slot at
    /// `(x, y)`, then its count (or the given text) in the given colour when
    /// it is not one.
    #[allow(clippy::too_many_arguments)]
    pub fn item_with_count(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        draw: &mut ItemDrawer<'_>,
        stack: &voxel_world::ClientItemStackState,
        count: i32,
        text: Option<(&str, u32)>,
        x: i32,
        y: i32,
    ) {
        self.item(draw, stack, x, y);
        let count_text = count.to_string();
        let shown = match text {
            Some(text) => Some(text),
            None if count != 1 => Some((count_text.as_str(), 0xFFFF_FFFF)),
            None => None,
        };
        if let Some((text, colour)) = shown {
            self.text_with_shadow(
                atlas,
                font,
                text,
                x + 19 - 2 - font.width(text),
                y + 6 + 3,
                colour,
            );
        }
    }

    /// `GuiGraphics.entity`: an entity's triangles, in GUI units with depth
    /// towards the viewer, textured from the skin.
    pub fn entity(&mut self, vertices: &[crate::gui::items::GuiItemVertex]) {
        if vertices.is_empty() {
            return;
        }
        let first = self.items.len() as u32;
        self.items.extend_from_slice(vertices);
        self.batches
            .push((GuiBlend::Entity, first, vertices.len() as u32));
    }

    /// `GuiGraphics.item`: one item's triangles into the slot at `(x, y)`.
    pub fn item(
        &mut self,
        draw: &mut ItemDrawer<'_>,
        stack: &voxel_world::ClientItemStackState,
        x: i32,
        y: i32,
    ) {
        let first = self.items.len() as u32;
        draw(stack, x as f32, y as f32, &mut self.items);
        let added = self.items.len() as u32 - first;
        if added == 0 {
            return;
        }
        match self.batches.last_mut() {
            Some((GuiBlend::Items, _, count)) => *count += added,
            _ => self.batches.push((GuiBlend::Items, first, added)),
        }
    }

    /// `GuiGraphics.blitSprite` of a nine-slice sprite: the corners as they
    /// are, the edges and the middle tiled (or stretched, when the sprite
    /// says so) over what is left. `tex` is the sprite's size in units.
    #[allow(clippy::too_many_arguments)]
    pub fn nine_slice(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        tex: (i32, i32),
        border: i32,
        stretch_inner: bool,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        let blend = GuiBlend::Translucent;
        let (inner_w, inner_h) = (tex.0 - 2 * border, tex.1 - 2 * border);
        let (mid_w, mid_h) = (w - 2 * border, h - 2 * border);
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
        let (right, bottom) = (x + w - border, y + h - border);
        // Corners.
        segment((0, 0, border, border), (x, y, border, border));
        segment(
            (tex.0 - border, 0, border, border),
            (right, y, border, border),
        );
        segment(
            (0, tex.1 - border, border, border),
            (x, bottom, border, border),
        );
        segment(
            (tex.0 - border, tex.1 - border, border, border),
            (right, bottom, border, border),
        );
        // Edges.
        segment((border, 0, inner_w, border), (x + border, y, mid_w, border));
        segment(
            (border, tex.1 - border, inner_w, border),
            (x + border, bottom, mid_w, border),
        );
        segment((0, border, border, inner_h), (x, y + border, border, mid_h));
        segment(
            (tex.0 - border, border, border, inner_h),
            (right, y + border, border, mid_h),
        );
        // The middle.
        segment(
            (border, border, inner_w, inner_h),
            (x + border, y + border, mid_w, mid_h),
        );
    }

    /// `TooltipRenderUtil` and `GuiGraphics.tooltip`: a box of text lines
    /// beside the mouse. Lines are runs of styled text, each 10 units tall
    /// with the first followed by two more; a single line is two units
    /// shorter. The box goes where the mouse's tooltip position puts it in a
    /// screen `screen` units big.
    #[allow(clippy::too_many_arguments)]
    pub fn tooltip(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<voxel_game::text::Run>],
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        if lines.is_empty() {
            return;
        }
        let width = lines
            .iter()
            .map(|line| font.width_of_runs(line))
            .max()
            .unwrap_or(0);
        let height = lines.len() as i32 * 10 - if lines.len() == 1 { 2 } else { 0 };
        // `DefaultTooltipPositioner`.
        let (mut x, mut y) = (mouse.0 + 12, mouse.1 - 12);
        if x + width > screen.0 {
            x = (x - 24 - width).max(4);
        }
        if y + height + 3 > screen.1 {
            y = screen.1 - (height + 3);
        }
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
        let mut line_y = y;
        for (index, line) in lines.iter().enumerate() {
            self.styled_text(atlas, font, line, x, line_y, 0xFFFF_FFFF, true);
            line_y += 10 + if index == 0 { 2 } else { 0 };
        }
    }

    /// `GuiGraphics.blitSprite` of a whole sprite at a size.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn blit(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        self.blit_part(atlas, blend, name, (w, h), (0, 0), x, y, w, h);
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
        self.glyphs(atlas, font, text, x, y, colour, false);
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
        self.glyphs(atlas, font, text, x, y, colour, true);
    }

    #[allow(clippy::too_many_arguments)]
    fn glyphs(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        text: &str,
        x: i32,
        y: i32,
        colour: u32,
        shadow: bool,
    ) {
        let runs = [voxel_game::text::Run {
            text: text.to_owned(),
            style: voxel_game::text::Style::default(),
        }];
        self.styled_text(atlas, font, &runs, x, y, colour, shadow);
    }

    /// `GuiGraphics.text` of styled text: each run in its colour, with the
    /// bold glyphs drawn twice a unit apart and the italic ones sheared.
    /// Underlines, strikethrough and obfuscation are not drawn.
    #[allow(clippy::too_many_arguments)]
    pub fn styled_text(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        runs: &[voxel_game::text::Run],
        x: i32,
        y: i32,
        default_colour: u32,
        shadow: bool,
    ) {
        let unpack =
            |colour: u32| [16, 8, 0, 24].map(|shift| ((colour >> shift) & 0xFF) as f32 / 255.0);
        let mut pen = x as f32;
        for run in runs {
            let colour = run
                .style
                .colour
                .map_or(default_colour, |rgb| (default_colour & 0xFF00_0000) | rgb);
            // `ARGB.scaleRGB(colour, 0.25)`.
            let shadow_colour = [16, 8, 0]
                .map(|shift| ((((colour >> shift) & 0xFF) as f32 * 0.25) as u32) << shift)
                .into_iter()
                .fold(colour & 0xFF00_0000, |sum, channel| sum | channel);
            let (shadow_colour, colour) = (unpack(shadow_colour), unpack(colour));
            let bold = run.style.bold == Some(true);
            let italic = run.style.italic == Some(true);
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
                    // `(offset, bold shift, colour)` for each pass: the
                    // shadow's, then the glyph's; a bold glyph twice.
                    let mut passes: Vec<(f32, f32, [f32; 4])> = Vec::new();
                    if shadow {
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
                        let top = y as f32 + up + offset;
                        let bottom = y as f32 + down + offset;
                        let corner = |px: f32, py: f32, uv: [f32; 2]| GuiVertex {
                            position: [px, py],
                            uv,
                            colour,
                        };
                        let corners = [
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
                        ];
                        let first = self.vertices.len() as u32;
                        for index in [0, 1, 2, 0, 2, 3] {
                            self.vertices.push(corners[index]);
                        }
                        match self.batches.last_mut() {
                            Some((GuiBlend::Translucent, _, count)) => *count += 6,
                            _ => self.batches.push((GuiBlend::Translucent, first, 6)),
                        }
                    }
                }
                pen += glyph.advance(bold);
            }
        }
    }

    /// `GuiGraphics.blitSprite` of part of a sprite: the sprite is taken as
    /// `size` units, and the part from `(u, v)` as big as the drawn area.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn blit_part(
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

    /// A region of a sprite, `(u, v, width, height)` of a sprite `size`
    /// units big, stretched to a rectangle `(x, y, w, h)`.
    #[allow(clippy::too_many_arguments)]
    fn blit_region(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        size: (i32, i32),
        (u, v, uw, vh): (i32, i32, i32, i32),
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        if w == 0 || h == 0 || uw == 0 || vh == 0 {
            return;
        }
        let [u0, v0, u1, v1] = atlas.sprite(name);
        let su = |offset: i32| u0 + (u1 - u0) * offset as f32 / size.0 as f32;
        let sv = |offset: i32| v0 + (v1 - v0) * offset as f32 / size.1 as f32;
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
}

/// Draws an item's triangles into the 16 by 16 slot at a GUI position.
pub type ItemDrawer<'a> = dyn FnMut(&voxel_world::ClientItemStackState, f32, f32, &mut Vec<crate::gui::items::GuiItemVertex>)
    + 'a;
