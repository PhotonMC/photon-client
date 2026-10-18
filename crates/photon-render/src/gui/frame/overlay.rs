//! Whole textures over the screen: the HUD's camera overlays and vignettes.

use super::*;

impl GuiFrame {
    /// `blit` through the `VIGNETTE` pipeline with a tint, as `Hud.extractVignette` does.
    pub fn vignette_tinted(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        rect: (i32, i32, i32, i32),
        colour: u32,
    ) {
        self.blit_stretched_with(atlas, GuiBlend::Vignette, name, (1, 1), (0, 0, 1, 1), rect);
        if let Some(DrawOp::Texture { colour: tint, .. }) =
            self.log.as_mut().and_then(|log| log.last_mut())
        {
            *tint = colour;
        }
        let [alpha, red, green, blue] = colour.to_be_bytes().map(|byte| f32::from(byte) / 255.0);
        for vertex in self.vertices.iter_mut().rev().take(6) {
            vertex.colour = [red, green, blue, alpha];
        }
    }

    /// `Hud.extractTextureOverlay`: a whole texture stretched over `rect`, white at `alpha`
    /// (`ARGB.white`).
    pub fn texture_overlay(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        rect: (i32, i32, i32, i32),
        alpha: f32,
    ) {
        // `ARGB.as8BitChannel`.
        let channel = (alpha * 255.0).floor() as i32;
        let colour = (channel as u32) << 24 | 0x00FF_FFFF;
        self.blit_stretched_with(
            atlas,
            GuiBlend::Translucent,
            name,
            (1, 1),
            (0, 0, 1, 1),
            rect,
        );
        if let Some(DrawOp::Texture { colour: tint, .. }) =
            self.log.as_mut().and_then(|log| log.last_mut())
        {
            *tint = colour;
        }
        for vertex in self.vertices.iter_mut().rev().take(6) {
            vertex.colour = [1.0, 1.0, 1.0, channel as f32 / 255.0];
        }
    }

    /// `blit` through the `VIGNETTE` pipeline: the whole texture over `rect`.
    pub fn vignette(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        texture_size: (i32, i32),
        rect: (i32, i32, i32, i32),
    ) {
        let region = (0, 0, texture_size.0, texture_size.1);
        self.blit_stretched_with(atlas, GuiBlend::Vignette, name, texture_size, region, rect);
    }

    pub(super) fn blit_stretched_with(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        texture_size: (i32, i32),
        region: (i32, i32, i32, i32),
        rect: (i32, i32, i32, i32),
    ) {
        if let Some(log) = &mut self.log {
            let (tw, th) = (texture_size.0 as f32, texture_size.1 as f32);
            log.push(DrawOp::Texture {
                name: name.to_owned(),
                rect: (
                    rect.0 as f32,
                    rect.1 as f32,
                    (rect.0 + rect.2) as f32,
                    (rect.1 + rect.3) as f32,
                ),
                uv: (
                    region.0 as f32 / tw,
                    (region.0 + region.2) as f32 / tw,
                    region.1 as f32 / th,
                    (region.1 + region.3) as f32 / th,
                ),
                colour: 0xFFFF_FFFF,
            });
        }
        self.blit_region(
            atlas,
            blend,
            &format!("texture/{name}"),
            texture_size,
            region,
            rect,
        );
    }

    /// `Hud.extractPortalOverlay`'s blit: a sprite of the block atlas, as `[u0, v0, u1, v1]`
    /// there, stretched over `rect` and white at `alpha`.
    pub fn block_sprite_overlay(
        &mut self,
        uv: [f32; 4],
        (x, y, w, h): (i32, i32, i32, i32),
        alpha: f32,
    ) {
        // `ARGB.as8BitChannel`.
        let channel = (alpha * 255.0).floor() as i32;
        if let Some(log) = &mut self.log {
            log.push(DrawOp::Sprite {
                name: "block_atlas".to_owned(),
                rect: (x as f32, y as f32, (x + w) as f32, (y + h) as f32),
                colour: (channel as u32) << 24 | 0x00FF_FFFF,
            });
        }
        let [u0, v0, u1, v1] = uv;
        let corner = |px: i32, py: i32, uv: [f32; 2]| GuiVertex {
            position: [px as f32, py as f32],
            uv,
            colour: [1.0, 1.0, 1.0, channel as f32 / 255.0],
        };
        let corners = [
            corner(x, y, [u0, v0]),
            corner(x, y + h, [u0, v1]),
            corner(x + w, y + h, [u1, v1]),
            corner(x + w, y, [u1, v0]),
        ];
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        match self.batches.last_mut() {
            Some((last, _, count)) if *last == GuiBlend::BlockSprite => *count += 6,
            _ => self.batches.push((GuiBlend::BlockSprite, first, 6)),
        }
    }

    /// `Hud.extractConfusionOverlay`: the nausea texture over the whole screen, grown about its
    /// middle from twice the screen's size to its own as `strength` rises to one, added to what is
    /// behind it in a green tint that grows with the strength.
    pub fn confusion_overlay(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        (width, height): (i32, i32),
        strength: f32,
    ) {
        let (w, h) = (width as f32, height as f32);
        let size = photon_core::math::mth_lerp_f32(strength, 2.0, 1.0);
        // `pushMatrix`, `translate`, `scale`, `translate` of a `Matrix3x2f`, then the corners.
        let (centre_x, centre_y) = (w / 2.0, h / 2.0);
        let m20 = size * -centre_x + (0.0 * -centre_y + centre_x);
        let m21 = 0.0 * -centre_x + (size * -centre_y + centre_y);
        let rect = (m20, m21, size * w + 0.0 * h + m20, 0.0 * w + size * h + m21);
        // `ARGB.colorFromFloat(1, 0.2s, 0.4s, 0.2s)`.
        let channel = |value: f32| (value * 255.0).floor() as u32;
        let colour = 0xFF00_0000
            | channel(0.2 * strength) << 16
            | channel(0.4 * strength) << 8
            | channel(0.2 * strength);
        if let Some(log) = &mut self.log {
            log.push(DrawOp::Texture {
                name: name.to_owned(),
                rect,
                uv: (0.0, 1.0, 0.0, 1.0),
                colour,
            });
        }
        let [u0, v0, u1, v1] = atlas.sprite(&format!("texture/{name}"));
        let tint = colour.to_be_bytes().map(|byte| f32::from(byte) / 255.0);
        let corner = |x: f32, y: f32, uv: [f32; 2]| GuiVertex {
            position: [x, y],
            uv,
            colour: [tint[1], tint[2], tint[3], tint[0]],
        };
        let corners = [
            corner(rect.0, rect.1, [u0, v0]),
            corner(rect.0, rect.3, [u0, v1]),
            corner(rect.2, rect.3, [u1, v1]),
            corner(rect.2, rect.1, [u1, v0]),
        ];
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        match self.batches.last_mut() {
            Some((last, _, count)) if *last == GuiBlend::Highlight => *count += 6,
            _ => self.batches.push((GuiBlend::Highlight, first, 6)),
        }
    }

    /// `Hud.extractSpyglassOverlay`: the scope's texture, as big as a square of the screen's
    /// shorter side scaled by `scale`, in the middle, and black over the rest of the screen.
    pub fn spyglass_overlay(&mut self, atlas: &GuiAtlas, (width, height): (i32, i32), scale: f32) {
        let source = width.min(height) as f32;
        let ratio = (width as f32 / source).min(height as f32 / source) * scale;
        let size = (source * ratio).floor() as i32;
        let (left, top) = ((width - size) / 2, (height - size) / 2);
        let (right, bottom) = (left + size, top + size);
        self.blit_texture(
            atlas,
            "textures/misc/spyglass_scope",
            (size, size),
            (0, 0),
            left,
            top,
            size,
            size,
        );
        let black = 0xFF00_0000;
        self.fill(atlas, (0, bottom, width, height), black);
        self.fill(atlas, (0, 0, width, top), black);
        self.fill(atlas, (0, top, left, bottom), black);
        self.fill(atlas, (right, top, width, bottom), black);
    }
}
