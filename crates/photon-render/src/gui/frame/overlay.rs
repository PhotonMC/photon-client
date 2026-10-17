//! Whole textures over the screen: the HUD's camera overlays and vignettes.

use super::*;

impl GuiFrame {
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
}
