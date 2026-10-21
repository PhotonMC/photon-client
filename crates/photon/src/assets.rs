//! The game's assets read from a folder.

/// Read models and textures from a folder laid out like the game's assets.
pub struct FolderAssets {
    pub root: std::path::PathBuf,
}

impl FolderAssets {
    /// The layout the game's asset tree uses: namespace, kind, then the name.
    pub fn path(
        &self,
        name: &photon_core::identifier::Identifier,
        kind: &str,
        extension: &str,
    ) -> std::path::PathBuf {
        self.root.join(name.namespace()).join(kind).join(format!(
            "{}{extension}",
            name.path().replace('/', std::path::MAIN_SEPARATOR_STR)
        ))
    }
}

impl FolderAssets {
    /// A picture of the item atlas that `atlases/items.json` makes by recolouring another with a
    /// palette (the trims on an armor's icon), as PNG bytes.
    fn permuted_texture(&self, name: &photon_core::identifier::Identifier) -> Option<Vec<u8>> {
        let atlas = photon_core::json::read_object(
            &self.root.join(name.namespace()).join("atlases/items.json"),
        )?;
        let source = photon_render::entity::trim::PalettedPermutations::parse(&atlas)?;
        let (texture, suffix) = source.textures.iter().find_map(|texture| {
            let suffix = name
                .path()
                .strip_prefix(texture.as_str())?
                .strip_prefix('_')?;
            Some((texture, suffix))
        })?;
        let load = |path: &str| {
            let id =
                photon_core::identifier::Identifier::parse(&format!("{}:{path}", name.namespace()))
                    .ok()?;
            let bytes = std::fs::read(self.path(&id, "textures", ".png")).ok()?;
            photon_render::texture::png::decode_rgba(&bytes).ok()
        };
        let mapped = photon_render::entity::trim::permute(
            &load(texture)?,
            &load(&source.palette_key)?,
            &load(source.permutations.get(suffix)?)?,
        )?;
        Some(photon_render::texture::png_encode::encode_rgba(&mapped))
    }
}

impl photon_render::AssetSource for FolderAssets {
    fn model(&self, name: &photon_core::identifier::Identifier) -> Option<String> {
        std::fs::read_to_string(self.path(name, "models", ".json")).ok()
    }

    fn item(&self, name: &photon_core::identifier::Identifier) -> Option<String> {
        std::fs::read_to_string(self.path(name, "items", ".json")).ok()
    }

    fn texture(&self, name: &photon_core::identifier::Identifier) -> Option<Vec<u8>> {
        std::fs::read(self.path(name, "textures", ".png"))
            .ok()
            .or_else(|| self.permuted_texture(name))
    }

    fn block_state(&self, name: &photon_core::identifier::Identifier) -> Option<String> {
        // A blockstate file sits in its own directory, named after the block
        // rather than after a model.
        std::fs::read_to_string(self.path(name, "blockstates", ".json")).ok()
    }

    fn texture_metadata(
        &self,
        name: &photon_core::identifier::Identifier,
    ) -> Option<photon_render::TextureMetadata> {
        // A sprite's metadata file sits next to its texture with a second
        // extension, and the mip strategy is one section inside it.
        let text = std::fs::read_to_string(self.path(name, "textures", ".png.mcmeta")).ok()?;
        let object = photon_core::json::parse_object(&text)?;
        photon_render::TextureMetadata::parse(object.object("texture")?)
    }

    fn texture_animation(
        &self,
        name: &photon_core::identifier::Identifier,
    ) -> Option<photon_render::AnimationFrames> {
        let text = std::fs::read_to_string(self.path(name, "textures", ".png.mcmeta")).ok()?;
        let object = photon_core::json::parse_object(&text)?;
        photon_render::AnimationFrames::parse(object.object("animation")?)
    }
}
