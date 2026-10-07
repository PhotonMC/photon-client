//! The textures entities are drawn with, packed in one image so that every
//! entity of a frame is one draw: the players' default skins and the mobs'
//! textures. A model's texture coordinates run over its own texture, from
//! zero to one; `uv` moves one to where the texture sits in the image.

use std::collections::HashMap;

use voxel_core::identifier::Identifier;

use crate::DecodedImage;
use crate::texture::atlas::{AtlasSprite, SpriteEntry, Stitcher};

/// Where a texture sits in the packed image: `u0, v0, u1, v1`.
pub type TextureRect = [f32; 4];

/// The packed textures and where each went.
#[derive(Debug, Clone)]
pub struct EntityTextures {
    pub image: DecodedImage,
    rects: HashMap<String, TextureRect>,
}

impl EntityTextures {
    /// Pack the textures, each under its name (a path under `textures/`).
    pub fn build(textures: Vec<(String, DecodedImage)>) -> Option<Self> {
        let mut stitcher = Stitcher::new(4096, 4096, 0, 0);
        let mut by_key: HashMap<String, (&str, &DecodedImage)> = HashMap::new();
        for (name, image) in &textures {
            let key = format!("minecraft:{name}");
            stitcher.register_sprite(SpriteEntry {
                width: image.width as i32,
                height: image.height as i32,
                name: Identifier::parse(&key).ok()?,
            });
            by_key.insert(key, (name.as_str(), image));
        }
        stitcher.stitch().ok()?;
        let width = stitcher.width().max(1) as u32;
        let height = stitcher.height().max(1) as u32;
        let mut atlas = DecodedImage {
            width,
            height,
            rgba: vec![0; (width * height * 4) as usize],
        };
        let mut rects = HashMap::new();
        for placement in stitcher.placements() {
            let (name, image) = by_key[&placement.name.as_string()];
            let left = (placement.x + placement.padding) as u32;
            let top = (placement.y + placement.padding) as u32;
            for row in 0..image.height {
                let from = (row * image.width * 4) as usize;
                let to = (((top + row) * width + left) * 4) as usize;
                atlas.rgba[to..to + image.width as usize * 4]
                    .copy_from_slice(&image.rgba[from..from + image.width as usize * 4]);
            }
            let sprite = AtlasSprite::new(
                &placement,
                image.width as i32,
                image.height as i32,
                width as i32,
                height as i32,
            );
            rects.insert(
                name.to_owned(),
                [sprite.u0, sprite.v0, sprite.u1, sprite.v1],
            );
        }
        Some(Self {
            image: atlas,
            rects,
        })
    }

    /// Where a texture is, if it was packed.
    pub fn rect(&self, name: &str) -> Option<TextureRect> {
        self.rects.get(name).copied()
    }
}

/// A texture coordinate of a texture's own, moved into the packed image.
pub fn uv(rect: TextureRect, [u, v]: [f32; 2]) -> [f32; 2] {
    [
        rect[0] + (rect[2] - rect[0]) * u,
        rect[1] + (rect[3] - rect[1]) * v,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: u32, height: u32, value: u8) -> DecodedImage {
        DecodedImage {
            width,
            height,
            rgba: vec![value; (width * height * 4) as usize],
        }
    }

    #[test]
    fn each_texture_is_found_where_its_rectangle_says() {
        let textures = vec![
            ("entity/a".to_owned(), solid(64, 64, 10)),
            ("entity/b".to_owned(), solid(64, 32, 20)),
            ("entity/c".to_owned(), solid(32, 32, 30)),
        ];
        let packed = EntityTextures::build(textures).unwrap();
        let (width, height) = (packed.image.width as f32, packed.image.height as f32);
        for (name, value, (texture_width, texture_height)) in [
            ("entity/a", 10_u8, (64.0, 64.0)),
            ("entity/b", 20, (64.0, 32.0)),
            ("entity/c", 30, (32.0, 32.0)),
        ] {
            let rect = packed.rect(name).unwrap();
            assert!(((rect[2] - rect[0]) * width - texture_width).abs() < 1e-3);
            assert!(((rect[3] - rect[1]) * height - texture_height).abs() < 1e-3);
            // A point inside the texture's own coordinates lands on its texels.
            let [u, v] = uv(rect, [0.5, 0.5]);
            let (x, y) = ((u * width) as usize, (v * height) as usize);
            assert_eq!(
                packed.image.rgba[(y * width as usize + x) * 4],
                value,
                "{name}"
            );
        }
        assert!(packed.rect("entity/missing").is_none());
    }
}
