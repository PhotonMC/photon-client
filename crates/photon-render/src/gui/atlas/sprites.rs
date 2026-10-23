//! Which sprites the GUI atlas holds: the `gui/sprites` directory source of
//! `atlases/gui.json`, as full ids (`namespace:path under gui/sprites`).

use super::{DecodedImage, Resources, SPRITE_FOLDERS, png_names};
use photon_core::identifier::Identifier;

const ROOT: &str = "textures/gui/sprites";

/// Every sprite, sorted. The `minecraft` namespace is read from the folders the screens draw
/// from; the files of any other namespace (a resource pack's) are all taken.
pub(super) fn sprite_ids(resources: &dyn Resources) -> Vec<String> {
    let mut ids = Vec::new();
    for folder in SPRITE_FOLDERS {
        let root = format!("{ROOT}/{folder}");
        for name in png_names(resources, &root, true) {
            ids.push(format!("minecraft:{folder}/{name}"));
        }
    }
    for id in resources.list(ROOT, ".png") {
        let below = id
            .path()
            .strip_prefix(ROOT)
            .and_then(|rest| rest.strip_prefix('/'));
        if let Some(name) = below.and_then(|rest| rest.strip_suffix(".png"))
            && id.namespace() != "minecraft"
        {
            ids.push(format!("{}:{name}", id.namespace()));
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// The picture of the sprite `id`.
pub(super) fn read_sprite(resources: &dyn Resources, id: &str) -> Option<DecodedImage> {
    let (namespace, name) = id.split_once(':')?;
    let file = Identifier::parse(&format!("{namespace}:{ROOT}/{name}.png")).ok()?;
    crate::texture::png::decode_rgba(&resources.read(&file)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Files(Vec<&'static str>);

    impl Resources for Files {
        fn read(&self, _: &Identifier) -> Option<Vec<u8>> {
            None
        }
        fn read_stack(&self, _: &Identifier) -> Vec<Vec<u8>> {
            Vec::new()
        }
        fn list(&self, directory: &str, extension: &str) -> Vec<Identifier> {
            self.0
                .iter()
                .filter_map(|id| Identifier::parse(id).ok())
                .filter(|id| id.path().starts_with(&format!("{directory}/")))
                .filter(|id| id.path().ends_with(extension))
                .collect()
        }
        fn namespaces(&self) -> Vec<String> {
            Vec::new()
        }
    }

    #[test]
    fn a_pack_namespace_adds_all_its_sprites() {
        let files = Files(vec![
            "minecraft:textures/gui/sprites/hud/heart/full.png",
            "minecraft:textures/gui/sprites/unlisted/x.png",
            "foo:textures/gui/sprites/hud/locator_bar_dot/x.png",
            "foo:textures/gui/sprites/other/y.png",
        ]);
        assert_eq!(
            sprite_ids(&files),
            [
                "foo:hud/locator_bar_dot/x",
                "foo:other/y",
                "minecraft:hud/heart/full"
            ]
        );
    }
}
