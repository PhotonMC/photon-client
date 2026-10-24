//! Textures an atlas does not list.

use super::*;

/// A source that hides some textures, so that they are read as missing and the atlas draws the
/// missing texture for them.
pub struct Hiding<'a, S: AssetSource> {
    pub inner: &'a S,
    /// The sprites to hide, as `namespace:path`.
    pub hidden: std::collections::HashSet<String>,
}

impl<S: AssetSource> AssetSource for Hiding<'_, S> {
    fn model(&self, name: &Identifier) -> Option<String> {
        self.inner.model(name)
    }
    fn texture(&self, name: &Identifier) -> Option<Vec<u8>> {
        if self.hidden.contains(&name.as_string()) {
            return None;
        }
        self.inner.texture(name)
    }
    fn block_state(&self, name: &Identifier) -> Option<String> {
        self.inner.block_state(name)
    }
    fn item(&self, name: &Identifier) -> Option<String> {
        self.inner.item(name)
    }
    fn sprite_listed(&self, name: &Identifier) -> bool {
        self.inner.sprite_listed(name)
    }
    fn texture_metadata(&self, name: &Identifier) -> Option<TextureMetadata> {
        self.inner.texture_metadata(name)
    }
    fn texture_animation(&self, name: &Identifier) -> Option<AnimationFrames> {
        if self.hidden.contains(&name.as_string()) {
            return None;
        }
        self.inner.texture_animation(name)
    }
}

/// The sprites the faces of `models` are drawn with.
pub fn face_sprites(models: &[&ResolvedModel]) -> Vec<Identifier> {
    models
        .iter()
        .flat_map(|model| {
            model.elements.iter().flat_map(|element| {
                element
                    .faces
                    .iter()
                    .filter_map(|(_, face)| model.textures.material(&face.texture))
                    .map(|material| material.sprite.clone())
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One red texel for every texture.
    struct Red;

    impl AssetSource for Red {
        fn model(&self, _: &Identifier) -> Option<String> {
            None
        }
        fn texture(&self, _: &Identifier) -> Option<Vec<u8>> {
            let image = DecodedImage {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 255],
            };
            Some(crate::texture::png_encode::encode_rgba(&image))
        }
    }

    #[test]
    fn a_hidden_texture_is_drawn_as_the_missing_texture() {
        let (shown, hidden) = (
            Identifier::parse("minecraft:a").unwrap(),
            Identifier::parse("minecraft:b").unwrap(),
        );
        let source = Hiding {
            inner: &Red,
            hidden: ["minecraft:b".to_owned()].into(),
        };
        let atlas = build_atlas_with(&source, &[], &[shown, hidden], (128, 0), 0).unwrap();
        let size = [atlas.width as f32, atlas.height as f32];
        let pixel = |name: &str| {
            let [u0, v0, ..] = atlas.sprites[name];
            let (x, y) = ((u0 * size[0]) as usize, (v0 * size[1]) as usize);
            let at = (y * atlas.width as usize + x) * 4;
            atlas.levels[0].rgba[at..at + 4].to_vec()
        };
        assert_eq!(pixel("minecraft:a"), vec![255, 0, 0, 255]);
        // The missing texture's first texel is in its black half.
        assert_eq!(pixel("minecraft:b"), vec![0, 0, 0, 255]);
    }
}
