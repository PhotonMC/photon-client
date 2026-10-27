use super::*;

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

    /// A pose over everything drawn since `mark`, as the reference's `Matrix3x2f`
    /// `[m00, m01, m10, m11, m20, m21]`: each position `p` becomes
    /// `(m00 p.x + m10 p.y + m20, m01 p.x + m11 p.y + m21)`. A text call keeps `m00` as its scale,
    /// as the reference's draw log does.
    pub fn transform_from(&mut self, mark: FrameMark, m: [f32; 6]) {
        let place = |x: f32, y: f32| (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]);
        if let Some(log) = &mut self.log {
            let from = mark.log.min(log.len());
            for op in &mut log[from..] {
                match op {
                    DrawOp::Text {
                        x,
                        y,
                        scale,
                        bounds,
                        ..
                    } => {
                        (*x, *y) = place(*x, *y);
                        *scale *= m[0];
                        // The text's bounds move with it.
                        let (x0, y0) = place(bounds.0 as f32, bounds.1 as f32);
                        let (x1, y1) = place(bounds.2 as f32, bounds.3 as f32);
                        *bounds = (x0 as i32, y0 as i32, x1 as i32, y1 as i32);
                    }
                    DrawOp::Fill { rect, .. }
                    | DrawOp::Sprite { rect, .. }
                    | DrawOp::Texture { rect, .. } => {
                        let (x0, y0) = place(rect.0, rect.1);
                        let (x1, y1) = place(rect.2, rect.3);
                        *rect = (x0, y0, x1, y1);
                    }
                }
            }
        }
        for vertex in &mut self.vertices[mark.vertices..] {
            let (x, y) = place(vertex.position[0], vertex.position[1]);
            vertex.position = [x, y];
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

    /// `blitSprite` of a nine-slice sprite (`tex` big, with a `border`) laid out `size` big, of which
    /// only the part from `(u, v)` as big as `rect` is drawn, at `rect` (a bar of a box that is only
    /// part full).
    pub fn nine_slice_region(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        ((tex, border), size): (((i32, i32), i32), (i32, i32)),
        (u, v): (i32, i32),
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        let mark = self.mark();
        self.nine_slice(
            atlas,
            name,
            tex,
            border,
            false,
            x - u,
            y - v,
            size.0,
            size.1,
        );
        self.clip_from(mark, (x, y, x + w, y + h));
        if let Some(DrawOp::Sprite { rect, .. }) = self.log.as_mut().and_then(|log| log.last_mut())
        {
            *rect = (x as f32, y as f32, (x + w) as f32, (y + h) as f32);
        }
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
                colour: 0xFFFF_FFFF,
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

    /// `GuiGraphics.blit` of a texture `tile` big repeated over a rectangle, the first row being
    /// `v` texels down: `WinScreen`'s background, which moves by less than a texel.
    pub fn tile_texture_scrolled(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        tile: (i32, i32),
        v: f32,
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        if let Some(log) = &mut self.log {
            let (tw, th) = (tile.0 as f32, tile.1 as f32);
            log.push(DrawOp::Texture {
                name: name.to_owned(),
                rect: (x as f32, y as f32, (x + w) as f32, (y + h) as f32),
                uv: (0.0 / tw, (0.0 + w as f32) / tw, v / th, (v + h as f32) / th),
                colour: 0xFFFF_FFFF,
            });
        }
        // The sampler takes the texel under each pixel's centre, so a fractional scroll is the
        // texel offset rounded: whole rows, with no piece reaching into the next sprite.
        let first = (v + 0.5).floor().rem_euclid(tile.1 as f32);
        let mut top = 0;
        let mut from_v = first;
        while top < h {
            let piece_h = ((tile.1 as f32 - from_v).ceil() as i32).clamp(1, h - top);
            let mut left = 0;
            while left < w {
                let piece_w = tile.0.min(w - left);
                let region = (0.0, from_v, piece_w as f32, piece_h as f32);
                let rect = (x + left, y + top, piece_w, piece_h);
                let texture = format!("texture/{name}");
                self.blit_region_f32(atlas, GuiBlend::Translucent, &texture, tile, region, rect);
                left += piece_w;
            }
            top += piece_h;
            from_v = (from_v + piece_h as f32).rem_euclid(tile.1 as f32);
        }
    }

    /// The log's calls since `from` that the scissor leaves any of.
    fn clip_log_from(&mut self, from: usize, (left, top, right, bottom): (i32, i32, i32, i32)) {
        let Some(log) = &mut self.log else {
            return;
        };
        let mut index = 0;
        log.retain(|op| {
            index += 1;
            if index <= from {
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

    /// `GuiGraphics.enableScissor` over what was drawn since `mark`: each
    /// rectangle of two triangles is cut to `(left, top, right, bottom)`, its
    /// texture with it; what falls outside is flattened to nothing, and the
    /// reference keeps no call for it. Only upright rectangles are cut
    /// exactly, which is all a list has in it.
    pub fn clip_from(&mut self, mark: FrameMark, (left, top, right, bottom): (i32, i32, i32, i32)) {
        // Item triangles are not upright rectangles: the GPU's scissor cuts them.
        if self.items.len() > mark.items {
            self.item_scissors.push((
                mark.items as u32,
                self.items.len() as u32,
                (left, top, right, bottom),
            ));
        }
        self.clip_log_from(mark.log, (left, top, right, bottom));
        let first = mark.vertices;
        let (left, top, right, bottom) = (left as f32, top as f32, right as f32, bottom as f32);
        for quad in self.vertices[first..].as_chunks_mut::<6>().0.iter_mut() {
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
        // A region wider or taller than the texture repeats it, as the reference's sampler does;
        // an atlas cannot, so it is drawn in pieces (one when it fits).
        self.tile_texture_from(atlas, name, texture_size, uv, (x, y, w, h));
    }

    /// `GuiGraphics.blit` of a region `(u, v, width, height)` of a whole texture stretched over
    /// `(x, y, w, h)`.
    pub fn blit_texture_stretched(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        texture_size: (i32, i32),
        region: (i32, i32, i32, i32),
        rect: (i32, i32, i32, i32),
    ) {
        self.blit_stretched_with(
            atlas,
            GuiBlend::Translucent,
            name,
            texture_size,
            region,
            rect,
        );
    }

    /// `itemBar`'s strip: a black one with the bar over it, `width` units of `colour`.
    pub fn item_bar_strip(
        &mut self,
        atlas: &GuiAtlas,
        (x, y): (i32, i32),
        (width, colour): (i32, i32),
    ) {
        let (left, top) = (x + 2, y + 13);
        self.fill(atlas, (left, top, left + 13, top + 2), 0xFF00_0000);
        // `ARGB.opaque`.
        self.fill(
            atlas,
            (left, top, left + width, top + 1),
            colour as u32 | 0xFF00_0000,
        );
    }

    /// `itemCooldown`: a white veil over the item, as tall as what is left of
    /// the cooldown of `group`.
    pub fn item_cooldown_group(&mut self, atlas: &GuiAtlas, (x, y): (i32, i32), group: &str) {
        if self.cooldowns.is_empty() {
            return;
        }
        let cooldown = self
            .cooldowns
            .iter()
            .find(|(name, _)| *name == group)
            .map_or(0.0, |(_, percent)| *percent);
        if cooldown > 0.0 {
            let top = y + photon_core::math::floor_f32(16.0 * (1.0 - cooldown));
            let bottom = top + (16.0 * cooldown).ceil() as i32;
            self.fill(atlas, (x, top, x + 16, bottom), 0x7FFF_FFFF);
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
}

#[cfg(test)]
mod item_batch_tests {
    use super::*;

    fn vertex() -> crate::gui::items::GuiItemVertex {
        crate::gui::items::GuiItemVertex {
            position: [0.0; 3],
            uv: [0.0; 2],
            colour: [1.0; 4],
        }
    }

    #[test]
    fn a_glint_is_drawn_right_after_its_item() {
        let stack = photon_world::ClientItemStackState::new(1, photon_core::ids::ItemId(1));
        let mut frame = GuiFrame::default();
        // Item A has a glint (3 vertices, then 3 of its glint), B and C have none.
        let mut draws = [(3, 3), (3, 0), (3, 0)].into_iter();
        let mut draw = |_: &photon_world::ClientItemStackState,
                        _: (f32, f32),
                        _: crate::gui::ItemHolding,
                        out: &mut Vec<crate::gui::items::GuiItemVertex>| {
            let (item, glint) = draws.next().expect("one draw per item");
            out.extend(std::iter::repeat_n(vertex(), item + glint));
            (item, false)
        };
        for x in 0..3 {
            frame.item(&mut draw, &stack, x * 18, 0);
        }
        assert_eq!(
            frame.batches,
            vec![
                (GuiBlend::Items, 0, 3),
                (GuiBlend::ItemGlint, 3, 3),
                (GuiBlend::Items, 6, 6),
            ]
        );
    }
}

#[cfg(test)]
mod repeat_tests {
    use super::*;

    const SPRITE: [f32; 4] = [0.25, 0.5, 0.375, 0.5625];

    fn inside(frame: &GuiFrame) -> bool {
        frame.vertices.iter().all(|vertex| {
            (SPRITE[0]..=SPRITE[2]).contains(&vertex.uv[0])
                && (SPRITE[1]..=SPRITE[3]).contains(&vertex.uv[1])
        })
    }

    #[test]
    fn a_blit_wider_than_its_texture_never_leaves_the_sprite() {
        let atlas = GuiAtlas::with_sprite("texture/header_separator", SPRITE);
        let mut frame = GuiFrame::default();
        frame.blit_texture(&atlas, "header_separator", (32, 2), (0, 0), 0, 30, 100, 2);
        // 100 wide over a 32 wide texture: three whole copies and a quarter, 6 vertices each.
        assert_eq!(frame.vertices.len(), 4 * 6);
        assert!(inside(&frame));
        let xs: Vec<f32> = frame.vertices.iter().map(|v| v.position[0]).collect();
        assert_eq!(xs.iter().copied().fold(f32::MAX, f32::min), 0.0);
        assert_eq!(xs.iter().copied().fold(f32::MIN, f32::max), 100.0);
    }

    #[test]
    fn a_scrolled_background_repeats_from_its_offset_inside_the_sprite() {
        let atlas = GuiAtlas::with_sprite("texture/menu_list_background", SPRITE);
        let mut frame = GuiFrame::default();
        frame.tile_texture_from(
            &atlas,
            "menu_list_background",
            (32, 32),
            (40, -50),
            (0, 0, 90, 70),
        );
        assert!(!frame.vertices.is_empty());
        assert!(inside(&frame));
    }

    #[test]
    fn a_fractional_scroll_samples_whole_texel_rows_inside_the_sprite() {
        let atlas = GuiAtlas::with_sprite("texture/menu_background", SPRITE);
        let mut frame = GuiFrame::default();
        frame.tile_texture_scrolled(&atlas, "menu_background", (32, 32), 17.4, (0, 0, 70, 90));
        assert!(!frame.vertices.is_empty());
        assert!(inside(&frame));
        let [_, v0, _, v1] = atlas.sprite("texture/menu_background");
        let step = (v1 - v0) / 32.0;
        for vertex in &frame.vertices {
            let texels = (vertex.uv[1] - v0) / step;
            assert!((texels - texels.round()).abs() < 1e-3, "{texels}");
        }
    }
}
