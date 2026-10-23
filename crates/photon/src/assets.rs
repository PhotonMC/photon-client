//! The game's assets read from the stack of resource packs.

use photon_render::entity::trim::PalettedPermutations;

/// Reads the file of an asset kind: `(name, kind, extension)` as in `textures` and `.png`.
type Reader<'a> = &'a dyn Fn(&photon_core::identifier::Identifier, &str, &str) -> Option<Vec<u8>>;

/// The `paletted_permutations` sources of the atlas `id` (`atlases/<name>.json`) in every pack
/// of the stack, the lowest pack's first (`AtlasLoader`).
pub(crate) fn atlas_permutations(
    stack: &photon_assets::ResourceStack,
    id: &photon_core::identifier::Identifier,
) -> Vec<photon_render::entity::trim::PalettedPermutations> {
    stack
        .stack(id)
        .iter()
        .filter_map(|resource| String::from_utf8(resource.bytes()?).ok())
        .filter_map(|text| photon_core::json::parse_object(&text))
        .flat_map(|atlas| photon_render::entity::trim::PalettedPermutations::parse_all(&atlas))
        .collect()
}

/// A picture of the item atlas that `atlases/items.json` makes by recolouring another with a
/// palette (the trims on an armor's icon), as PNG bytes.
fn permuted_texture(
    read: Reader<'_>,
    sources: &dyn Fn(&photon_core::identifier::Identifier) -> Vec<PalettedPermutations>,
    name: &photon_core::identifier::Identifier,
) -> Option<Vec<u8>> {
    let atlas = photon_core::identifier::Identifier::parse(&format!(
        "{}:atlases/items.json",
        name.namespace()
    ))
    .ok()?;
    // A later source replaces the sprite an earlier one made of the same name.
    let (source, texture, suffix) = sources(&atlas).into_iter().rev().find_map(|source| {
        let (texture, suffix) = source.textures.iter().find_map(|texture| {
            let suffix = name
                .path()
                .strip_prefix(texture.as_str())?
                .strip_prefix('_')?;
            Some((texture.clone(), suffix.to_owned()))
        })?;
        Some((source, texture, suffix))
    })?;
    let load = |path: &str| {
        let id =
            photon_core::identifier::Identifier::parse(&format!("{}:{path}", name.namespace()))
                .ok()?;
        photon_render::texture::png::decode_rgba(&read(&id, "textures", ".png")?).ok()
    };
    let mapped = photon_render::entity::trim::permute(
        &load(&texture)?,
        &load(&source.palette_key)?,
        &load(source.permutations.get(&suffix)?)?,
    )?;
    Some(photon_render::texture::png_encode::encode_rgba(&mapped))
}

fn texture_of(
    read: Reader<'_>,
    sources: &dyn Fn(&photon_core::identifier::Identifier) -> Vec<PalettedPermutations>,
    name: &photon_core::identifier::Identifier,
) -> Option<Vec<u8>> {
    read(name, "textures", ".png").or_else(|| permuted_texture(read, sources, name))
}

/// The game's own assets as a pack: the jar's folder, whose `assets` folder is `root`, and the
/// objects of the asset index around it, which the vanilla pack reads before the jar.
struct AssetsFolder {
    jar: photon_assets::DirectorySource,
    indexed: Option<crate::indexed::IndexedAssets>,
}

impl AssetsFolder {
    /// The path inside the folder, for a path inside the pack's `assets` folder.
    fn inside(path: &str) -> Option<&str> {
        match path.strip_prefix("assets") {
            Some("") => Some(""),
            Some(rest) => rest.strip_prefix('/'),
            None => None,
        }
    }
}

impl photon_assets::Source for AssetsFolder {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let inside = Self::inside(path)?;
        let indexed = self.indexed.as_ref().and_then(|index| index.object(inside));
        indexed.or_else(|| self.jar.read(inside))
    }

    fn exists(&self, path: &str) -> bool {
        Self::inside(path).is_some_and(|inside| {
            self.indexed
                .as_ref()
                .is_some_and(|index| index.contains(inside))
                || self.jar.exists(inside)
        })
    }

    fn files_below(&self, directory: &str) -> Vec<String> {
        let Some(inside) = Self::inside(directory) else {
            return Vec::new();
        };
        let mut files = self.jar.files_below(inside);
        if let Some(index) = &self.indexed {
            let prefix = if inside.is_empty() {
                String::new()
            } else {
                format!("{inside}/")
            };
            let indexed = index.paths().filter(|path| path.starts_with(&prefix));
            let known: std::collections::HashSet<String> = files.iter().cloned().collect();
            files.extend(
                indexed
                    .filter(|path| !known.contains(*path))
                    .map(str::to_owned),
            );
        }
        files
            .into_iter()
            .map(|path| format!("assets/{path}"))
            .collect()
    }

    fn folders_in(&self, directory: &str) -> Vec<String> {
        Self::inside(directory).map_or_else(Vec::new, |inside| self.jar.folders_in(inside))
    }
}

/// The game's own assets as the bottom pack.
pub(crate) fn vanilla_pack(root: &std::path::Path) -> photon_assets::Pack {
    let source = AssetsFolder {
        jar: photon_assets::DirectorySource::new(root),
        indexed: crate::indexed::IndexedAssets::open(root),
    };
    photon_assets::Pack::built_in("vanilla", Box::new(source))
}

/// The stack of the game's own assets alone.
pub(crate) fn vanilla_stack(root: &std::path::Path) -> photon_assets::ResourceStack {
    photon_assets::ResourceStack::new(vec![vanilla_pack(root)])
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
        Self::new(std::sync::Arc::new(vanilla_stack(root)))
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
            &|atlas| atlas_permutations(&self.stack, atlas),
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
        std::fs::create_dir_all(dir.join("jar-assets/minecraft/textures/block")).unwrap();
        std::fs::create_dir_all(dir.join("jar-assets/minecraft/models/block")).unwrap();
        dir
    }

    #[test]
    fn the_stack_reads_the_vanilla_folder() {
        let dir = scratch();
        let root = dir.join("jar-assets");
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

    #[test]
    fn every_pack_adds_its_atlas_sources() {
        let atlas = |texture: &str| {
            format!(
                r#"{{"sources":[{{"type":"minecraft:paletted_permutations","textures":["{texture}"],"palette_key":"key","permutations":{{"a":"pa"}}}}]}}"#
            )
        };
        let dir = scratch();
        let packs: Vec<photon_assets::Pack> = ["low", "high"]
            .iter()
            .map(|name| {
                let root = dir.join(name);
                std::fs::create_dir_all(root.join("assets/minecraft/atlases")).unwrap();
                std::fs::write(
                    root.join("assets/minecraft/atlases/items.json"),
                    atlas(&format!("t_{name}")),
                )
                .unwrap();
                photon_assets::Pack::built_in(
                    *name,
                    Box::new(photon_assets::DirectorySource::new(&root)),
                )
            })
            .collect();
        let stack = photon_assets::ResourceStack::new(packs);
        let id = Identifier::parse("minecraft:atlases/items.json").unwrap();
        let sources = atlas_permutations(&stack, &id);
        let textures: Vec<&str> = sources.iter().map(|s| s.textures[0].as_str()).collect();
        assert_eq!(textures, ["t_low", "t_high"]);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
