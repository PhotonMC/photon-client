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

/// Where a tooltip goes (`ClientTooltipPositioner`).
#[derive(Debug, Clone, Copy)]
pub enum Positioner {
    /// `DefaultTooltipPositioner`: by the mouse.
    Default,
    /// `MenuTooltipPositioner`: by the mouse, off the widget.
    Menu(crate::gui::navigation::Rectangle),
    /// `BelowOrAboveWidgetTooltipPositioner`: under the widget, or over it.
    BelowOrAbove(crate::gui::navigation::Rectangle),
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
        if let Some(log) = &mut self.log {
            log.push(DrawOp::Fill {
                rect: (
                    x0.min(x1) as f32,
                    y0.min(y1) as f32,
                    x0.max(x1) as f32,
                    y0.max(y1) as f32,
                ),
                colour: top,
            });
        }
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

    /// A pose scale and translate over what was drawn since vertex `first`:
    /// each position `p` becomes `translate + scale * p`.
    pub fn scale_from(
        &mut self,
        (first, first_op): (usize, usize),
        scale: f32,
        translate: (f32, f32),
    ) {
        if let Some(log) = &mut self.log {
            let from = first_op.min(log.len());
            let place = |rect: &mut (f32, f32, f32, f32)| {
                *rect = (
                    translate.0 + scale * rect.0,
                    translate.1 + scale * rect.1,
                    translate.0 + scale * rect.2,
                    translate.1 + scale * rect.3,
                );
            };
            for op in &mut log[from..] {
                match op {
                    DrawOp::Text { x, y, scale: s, .. } => {
                        *x = translate.0 + scale * *x;
                        *y = translate.1 + scale * *y;
                        *s *= scale;
                    }
                    DrawOp::Fill { rect, .. }
                    | DrawOp::Sprite { rect, .. }
                    | DrawOp::Texture { rect, .. } => place(rect),
                }
            }
        }
        for vertex in &mut self.vertices[first..] {
            vertex.position = [
                translate.0 + scale * vertex.position[0],
                translate.1 + scale * vertex.position[1],
            ];
        }
    }

    /// Where a frame is now, to apply a pose to what is drawn after.
    pub fn mark(&self) -> FrameMark {
        FrameMark {
            vertices: self.vertices.len(),
            items: self.items.len(),
            log: self.log.as_ref().map_or(0, Vec::len),
        }
    }

    /// A pose over everything drawn since `mark`: scaled by `(sx, sy)` about
    /// the point `pivot` (translate, scale, translate back).
    pub fn scale_about(&mut self, mark: FrameMark, (sx, sy): (f32, f32), pivot: (f32, f32)) {
        for vertex in &mut self.vertices[mark.vertices..] {
            vertex.position[0] = pivot.0 + sx * (vertex.position[0] - pivot.0);
            vertex.position[1] = pivot.1 + sy * (vertex.position[1] - pivot.1);
        }
        for vertex in &mut self.items[mark.items..] {
            vertex.position[0] = pivot.0 + sx * (vertex.position[0] - pivot.0);
            vertex.position[1] = pivot.1 + sy * (vertex.position[1] - pivot.1);
        }
    }

    /// A pose over the items drawn since item vertex `first`: each position
    /// `p` becomes `offset + scale * p`.
    pub fn scale_items_from(&mut self, first: usize, scale: f32, offset: (f32, f32)) {
        for vertex in &mut self.items[first..] {
            vertex.position[0] = offset.0 + scale * vertex.position[0];
            vertex.position[1] = offset.1 + scale * vertex.position[1];
        }
    }

    /// `GuiGraphics.fill` in one colour.
    pub fn fill(&mut self, atlas: &GuiAtlas, rect: (i32, i32, i32, i32), colour: u32) {
        self.fill_gradient(atlas, rect, colour, colour);
    }

    /// `GuiGraphics.blitSprite` of a whole sprite.
    pub fn sprite(&mut self, atlas: &GuiAtlas, name: &str, x: i32, y: i32, w: i32, h: i32) {
        self.sprite_blend(atlas, GuiBlend::Translucent, name, (x, y, w, h));
    }

    /// `blitSprite` of a whole sprite through a pipeline's blend.
    pub fn sprite_blend(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        self.sprite_region_blend(atlas, blend, name, (w, h), (0, 0), (x, y, w, h));
    }

    /// `blitSprite` of the part of a sprite `size` big that starts at `uv`
    /// and is as big as `rect`, drawn at `rect`.
    pub fn sprite_region_blend(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        size: (i32, i32),
        uv: (i32, i32),
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        // The reference draws nothing for a sprite with no width or height.
        if w == 0 || h == 0 {
            return;
        }
        if let Some(log) = &mut self.log {
            log.push(DrawOp::Sprite {
                name: name.to_owned(),
                rect: (x as f32, y as f32, (x + w) as f32, (y + h) as f32),
                colour: 0xFFFF_FFFF,
            });
        }
        self.blit_part(atlas, blend, name, size, uv, x, y, w, h);
    }

    /// `GuiGraphics.blitSprite` of the left `w` by `h` part of a sprite
    /// `size` big (a bar that is only part full).
    pub fn sprite_part(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        size: (i32, i32),
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        if let Some(log) = &mut self.log {
            log.push(DrawOp::Sprite {
                name: name.to_owned(),
                rect: (x as f32, y as f32, (x + w) as f32, (y + h) as f32),
                colour: 0xFFFF_FFFF,
            });
        }
        self.blit_region(
            atlas,
            GuiBlend::Translucent,
            name,
            size,
            (0, 0, w, h),
            (x, y, w, h),
        );
    }

    /// `GuiGraphics.blit` of a whole texture repeated over a rectangle
    /// (`Screen.extractMenuBackgroundTexture`): one call in the reference,
    /// whole tiles here with the last ones cut.
    pub fn tile_texture(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        tile: (i32, i32),
        rect: (i32, i32, i32, i32),
    ) {
        self.tile_texture_from(atlas, name, tile, (0, 0), rect);
    }

    /// `GuiGraphics.blit` of a texture `tile` big repeated over a rectangle,
    /// the first texel being `(u, v)`: the list's background, which moves
    /// with the scroll.
    pub fn tile_texture_from(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        tile: (i32, i32),
        (u, v): (i32, i32),
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        if let Some(log) = &mut self.log {
            let (tw, th) = (tile.0 as f32, tile.1 as f32);
            log.push(DrawOp::Texture {
                name: name.to_owned(),
                rect: (x as f32, y as f32, (x + w) as f32, (y + h) as f32),
                uv: (
                    u as f32 / tw,
                    u as f32 / tw + w as f32 / tw,
                    v as f32 / th,
                    v as f32 / th + h as f32 / th,
                ),
            });
        }
        let mut top = 0;
        while top < h {
            let from_v = (v + top).rem_euclid(tile.1);
            let piece_h = (tile.1 - from_v).min(h - top);
            let mut left = 0;
            while left < w {
                let from_u = (u + left).rem_euclid(tile.0);
                let piece_w = (tile.0 - from_u).min(w - left);
                self.blit_part(
                    atlas,
                    GuiBlend::Translucent,
                    &format!("texture/{name}"),
                    tile,
                    (from_u, from_v),
                    x + left,
                    y + top,
                    piece_w,
                    piece_h,
                );
                left += piece_w;
            }
            top += piece_h;
        }
    }

    /// `GuiGraphics.enableScissor` over what was drawn since `mark`: each
    /// rectangle of two triangles is cut to `(left, top, right, bottom)`, its
    /// texture with it; what falls outside is flattened to nothing, and the
    /// reference keeps no call for it. Only upright rectangles are cut
    /// exactly, which is all a list has in it.
    pub fn clip_from(&mut self, mark: FrameMark, (left, top, right, bottom): (i32, i32, i32, i32)) {
        if let Some(log) = &mut self.log {
            let mut index = 0;
            log.retain(|op| {
                index += 1;
                if index <= mark.log {
                    return true;
                }
                let (x0, y0, x1, y1) = match op {
                    DrawOp::Fill { rect, .. }
                    | DrawOp::Sprite { rect, .. }
                    | DrawOp::Texture { rect, .. } => *rect,
                    DrawOp::Text { bounds, .. } => (
                        bounds.0 as f32,
                        bounds.1 as f32,
                        bounds.2 as f32,
                        bounds.3 as f32,
                    ),
                };
                x0 < right as f32 && x1 > left as f32 && y0 < bottom as f32 && y1 > top as f32
            });
        }
        let first = mark.vertices;
        let (left, top, right, bottom) = (left as f32, top as f32, right as f32, bottom as f32);
        for quad in self.vertices[first..].chunks_exact_mut(6) {
            let xs = quad.iter().map(|vertex| vertex.position[0]);
            let ys = quad.iter().map(|vertex| vertex.position[1]);
            let (x0, x1) = (
                xs.clone().fold(f32::INFINITY, f32::min),
                xs.fold(f32::NEG_INFINITY, f32::max),
            );
            let (y0, y1) = (
                ys.clone().fold(f32::INFINITY, f32::min),
                ys.fold(f32::NEG_INFINITY, f32::max),
            );
            if x0 >= left && x1 <= right && y0 >= top && y1 <= bottom {
                continue;
            }
            if x1 <= left || x0 >= right || y1 <= top || y0 >= bottom {
                let corner = quad[0];
                quad.fill(GuiVertex {
                    position: [left, top],
                    ..corner
                });
                continue;
            }
            let (cx0, cx1) = (x0.max(left), x1.min(right));
            let (cy0, cy1) = (y0.max(top), y1.min(bottom));
            // The texture follows the position across the quad, whose first
            // corner is its top left and third its bottom right.
            let (a, b) = (quad[0], quad[2]);
            let u_at = |x: f32| a.uv[0] + (b.uv[0] - a.uv[0]) * (x - x0) / (x1 - x0);
            let v_at = |y: f32| a.uv[1] + (b.uv[1] - a.uv[1]) * (y - y0) / (y1 - y0);
            for vertex in quad.iter_mut() {
                let x = if vertex.position[0] <= x0 { cx0 } else { cx1 };
                let y = if vertex.position[1] <= y0 { cy0 } else { cy1 };
                vertex.uv = [u_at(x), v_at(y)];
                vertex.position = [x, y];
            }
        }
    }

    /// `GuiGraphics.blit` of a region of a whole texture: `(u, v)` and the
    /// drawn size in texels of a texture `texture_size` big.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
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
        if let Some(log) = &mut self.log {
            let (tw, th) = (texture_size.0 as f32, texture_size.1 as f32);
            log.push(DrawOp::Texture {
                name: name.to_owned(),
                rect: (x as f32, y as f32, (x + w) as f32, (y + h) as f32),
                uv: (
                    uv.0 as f32 / tw,
                    uv.0 as f32 / tw + w as f32 / tw,
                    uv.1 as f32 / th,
                    uv.1 as f32 / th + h as f32 / th,
                ),
            });
        }
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
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
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

    /// `GuiGraphics.blitSprite` of a nine-slice sprite whose border is the
    /// same on every side: see [`Self::nine_slice_edges`].
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
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
        let edges = (border, border, border, border);
        self.nine_slice_edges(atlas, name, tex, edges, stretch_inner, (x, y, w, h));
    }

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
    pub fn tooltip(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<voxel_game::text::Run>],
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        self.tooltip_with(atlas, font, lines, &Positioner::Default, mouse, screen);
    }

    /// A tooltip placed by a positioner (`ClientTooltipPositioner`).
    pub fn tooltip_with(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        lines: &[Vec<voxel_game::text::Run>],
        positioner: &Positioner,
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
        let (x, y) = positioner.position(screen, mouse, (width, height));
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

    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
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

    /// Keeps a text call for a test that asked for the calls.
    fn log_text(
        &mut self,
        (x, y): (i32, i32),
        runs: &[voxel_game::text::Run],
        colour: u32,
        shadow: bool,
        bounds: Option<(i32, i32, i32, i32)>,
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
        });
    }

    /// `GuiGraphics.text` of styled text: each run in its colour, with the
    /// bold glyphs drawn twice a unit apart and the italic ones sheared.
    /// Underlines, strikethrough and obfuscation are not drawn.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
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
        self.log_text(
            (x, y),
            runs,
            default_colour,
            shadow,
            crate::gui::text_hit::bounds(font, runs, (x, y), shadow),
        );
        for corners in glyph_quads(
            atlas,
            font,
            runs,
            (x as f32, y as f32),
            default_colour,
            shadow,
        ) {
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

    /// `GuiGraphics.blitSprite` of part of a sprite as the reference's draw log records it: one
    /// sprite at the rectangle drawn.
    #[expect(
        clippy::too_many_arguments,
        reason = "mirrors the parameters of the reference's `GuiGraphics` call"
    )]
    pub(crate) fn sprite_cut(
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
    pub(crate) fn blit_region(
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

/// The runs as the reference's character sequence yields them: neighbours in
/// one style are one stretch.
fn merged_runs(runs: &[voxel_game::text::Run]) -> Vec<(String, Option<u32>, bool, bool)> {
    let mut merged: Vec<(String, Option<u32>, bool, bool)> = Vec::new();
    let mut last: Option<&voxel_game::text::Style> = None;
    for run in runs {
        match merged.last_mut() {
            Some(previous) if last == Some(&run.style) => previous.0.push_str(&run.text),
            _ => merged.push((
                run.text.clone(),
                run.style.colour,
                run.style.bold == Some(true),
                run.style.italic == Some(true),
            )),
        }
        last = Some(&run.style);
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
    runs: &[voxel_game::text::Run],
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
                // `(offset, bold shift, colour)` for each pass: the shadow's,
                // then the glyph's; a bold glyph twice.
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
