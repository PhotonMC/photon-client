//! The game's assets read from a folder.

/// Read models and textures from a folder laid out like the game's assets.
pub struct FolderAssets {
    pub root: std::path::PathBuf,
}

impl FolderAssets {
    /// The layout the game's asset tree uses: namespace, kind, then the name.
    pub fn path(
        &self,
        name: &voxel_core::identifier::Identifier,
        kind: &str,
        extension: &str,
    ) -> std::path::PathBuf {
        self.root.join(name.namespace()).join(kind).join(format!(
            "{}{extension}",
            name.path().replace('/', std::path::MAIN_SEPARATOR_STR)
        ))
    }
}

impl voxel_render::AssetSource for FolderAssets {
    fn model(&self, name: &voxel_core::identifier::Identifier) -> Option<String> {
        std::fs::read_to_string(self.path(name, "models", ".json")).ok()
    }

    fn item(&self, name: &voxel_core::identifier::Identifier) -> Option<String> {
        std::fs::read_to_string(self.path(name, "items", ".json")).ok()
    }

    fn texture(&self, name: &voxel_core::identifier::Identifier) -> Option<Vec<u8>> {
        std::fs::read(self.path(name, "textures", ".png")).ok()
    }

    fn block_state(&self, name: &voxel_core::identifier::Identifier) -> Option<String> {
        // A blockstate file sits in its own directory, named after the block
        // rather than after a model.
        std::fs::read_to_string(self.path(name, "blockstates", ".json")).ok()
    }

    fn texture_metadata(
        &self,
        name: &voxel_core::identifier::Identifier,
    ) -> Option<voxel_render::TextureMetadata> {
        // A sprite's metadata file sits next to its texture with a second
        // extension, and the mip strategy is one section inside it.
        let text = std::fs::read_to_string(self.path(name, "textures", ".png.mcmeta")).ok()?;
        let object = voxel_core::json::parse_object(&text)?;
        voxel_render::TextureMetadata::parse(object.object("texture")?)
    }

    fn texture_animation(
        &self,
        name: &voxel_core::identifier::Identifier,
    ) -> Option<voxel_render::AnimationFrames> {
        let text = std::fs::read_to_string(self.path(name, "textures", ".png.mcmeta")).ok()?;
        let object = voxel_core::json::parse_object(&text)?;
        voxel_render::AnimationFrames::parse(object.object("animation")?)
    }
}
