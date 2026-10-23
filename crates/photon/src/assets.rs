//! The game's assets read from the stack of resource packs.

/// Reads the file of an asset kind: `(name, kind, extension)` as in `textures` and `.png`.
type Reader<'a> = &'a dyn Fn(&photon_core::identifier::Identifier, &str, &str) -> Option<Vec<u8>>;

/// A picture of the item atlas that `atlases/items.json` makes by recolouring another with a
/// palette (the trims on an armor's icon), as PNG bytes.
fn permuted_texture(
    read: Reader<'_>,
    name: &photon_core::identifier::Identifier,
) -> Option<Vec<u8>> {
    let namespace =
        photon_core::identifier::Identifier::parse(&format!("{}:items", name.namespace())).ok()?;
    let text = String::from_utf8(read(&namespace, "atlases", ".json")?).ok()?;
    let atlas = photon_core::json::parse_object(&text)?;
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
        photon_render::texture::png::decode_rgba(&read(&id, "textures", ".png")?).ok()
    };
    let mapped = photon_render::entity::trim::permute(
        &load(texture)?,
        &load(&source.palette_key)?,
        &load(source.permutations.get(suffix)?)?,
    )?;
    Some(photon_render::texture::png_encode::encode_rgba(&mapped))
}

fn texture_of(read: Reader<'_>, name: &photon_core::identifier::Identifier) -> Option<Vec<u8>> {
    read(name, "textures", ".png").or_else(|| permuted_texture(read, name))
}

/// Read models and textures from the stack of resource packs, the topmost pack's file winning
/// (`MultiPackResourceManager`).
pub struct StackAssets {
    stack: std::sync::Arc<photon_assets::ResourceStack>,
}

impl StackAssets {
    pub fn new(stack: std::sync::Arc<photon_assets::ResourceStack>) -> Self {
        Self { stack }
    }

    /// The game's own assets alone, from the folder `root` that holds the namespaces
    /// (`<jar>/assets`), as the stack's bottom pack.
    pub fn vanilla(root: &std::path::Path) -> Self {
        let source = photon_assets::DirectorySource::new(root.parent().unwrap_or(root));
        let pack = photon_assets::Pack::built_in("vanilla", Box::new(source));
        Self::new(std::sync::Arc::new(photon_assets::ResourceStack::new(
            vec![pack],
        )))
    }

    fn id(
        name: &photon_core::identifier::Identifier,
        kind: &str,
        extension: &str,
    ) -> Option<photon_core::identifier::Identifier> {
        photon_core::identifier::Identifier::parse(&format!(
            "{}:{kind}/{}{extension}",
            name.namespace(),
            name.path()
        ))
        .ok()
    }

    fn bytes(
        &self,
        name: &photon_core::identifier::Identifier,
        kind: &str,
        extension: &str,
    ) -> Option<Vec<u8>> {
        self.stack.open(&Self::id(name, kind, extension)?)?.bytes()
    }

    fn text(&self, name: &photon_core::identifier::Identifier, kind: &str) -> Option<String> {
        String::from_utf8(self.bytes(name, kind, ".json")?).ok()
    }

    fn mcmeta(
        &self,
        name: &photon_core::identifier::Identifier,
    ) -> Option<photon_core::json::JsonObject> {
        self.stack
            .open(&Self::id(name, "textures", ".png")?)?
            .metadata()
    }
}

impl photon_render::AssetSource for StackAssets {
    fn model(&self, name: &photon_core::identifier::Identifier) -> Option<String> {
        self.text(name, "models")
    }

    fn item(&self, name: &photon_core::identifier::Identifier) -> Option<String> {
        self.text(name, "items")
    }

    fn texture(&self, name: &photon_core::identifier::Identifier) -> Option<Vec<u8>> {
        texture_of(
            &|name, kind, extension| self.bytes(name, kind, extension),
            name,
        )
    }

    fn block_state(&self, name: &photon_core::identifier::Identifier) -> Option<String> {
        self.text(name, "blockstates")
    }

    fn texture_metadata(
        &self,
        name: &photon_core::identifier::Identifier,
    ) -> Option<photon_render::TextureMetadata> {
        photon_render::TextureMetadata::parse(self.mcmeta(name)?.object("texture")?)
    }

    fn texture_animation(
        &self,
        name: &photon_core::identifier::Identifier,
    ) -> Option<photon_render::AnimationFrames> {
        photon_render::AnimationFrames::parse(self.mcmeta(name)?.object("animation")?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::identifier::Identifier;
    use photon_render::AssetSource;

    fn scratch() -> std::path::PathBuf {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("photon-assets-{}-{n}", std::process::id()));
        std::fs::create_dir_all(dir.join("assets/minecraft/textures/block")).unwrap();
        std::fs::create_dir_all(dir.join("assets/minecraft/models/block")).unwrap();
        dir
    }

    #[test]
    fn the_stack_reads_the_vanilla_folder() {
        let dir = scratch();
        let root = dir.join("assets");
        let block = root.join("minecraft/textures/block");
        std::fs::write(block.join("stone.png"), b"png").unwrap();
        std::fs::write(
            block.join("stone.png.mcmeta"),
            br#"{"animation":{"frametime":3},"texture":{"blur":true}}"#,
        )
        .unwrap();
        std::fs::write(root.join("minecraft/models/block/stone.json"), b"{}").unwrap();
        let stack = StackAssets::vanilla(&root);
        let stone = Identifier::parse("minecraft:block/stone").unwrap();
        let missing = Identifier::parse("minecraft:block/missing").unwrap();
        assert_eq!(stack.texture(&stone).as_deref(), Some(&b"png"[..]));
        assert_eq!(stack.model(&stone).as_deref(), Some("{}"));
        assert!(stack.texture(&missing).is_none());
        assert!(stack.model(&missing).is_none());
        assert!(stack.texture_animation(&stone).is_some());
        assert!(stack.texture_metadata(&stone).is_some());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
