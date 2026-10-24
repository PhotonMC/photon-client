//! The loom's patterns as the vanilla data pack offers them, through the client's own selection.

use photon_game::menus::container::ContainerMenu;
use photon_world::block::tags::OrderedTags;

fn data_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.data/minecraft/assets/jar/data")
}

/// The entries a banner pattern tag lists, in the file's order.
fn tag_values(file: &std::path::Path) -> Vec<String> {
    let parsed: serde_json::Value = std::fs::read_to_string(file)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    parsed["values"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|name| name.as_str().map(str::to_owned))
        .collect()
}

/// Every banner pattern tag of the data pack as registry ids, as a tag packet lists them.
fn pack_tags(names: &[String]) -> OrderedTags {
    let root = data_root().join("minecraft/tags/banner_pattern");
    let mut files = vec![(
        "no_item_required".to_owned(),
        root.join("no_item_required.json"),
    )];
    for entry in std::fs::read_dir(root.join("pattern_item"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let file = entry.file_name().to_string_lossy().into_owned();
        if let Some(short) = file.strip_suffix(".json") {
            files.push((format!("pattern_item/{short}"), entry.path()));
        }
    }
    OrderedTags::new(files.into_iter().map(|(tag, file)| {
        let ids = tag_values(&file)
            .iter()
            .filter_map(|name| names.iter().position(|known| known == name))
            .map(|id| id as i32)
            .collect();
        (format!("minecraft:{tag}"), ids)
    }))
}

/// Gives the loom menu the patterns the client would list for its slots, and returns the
/// registry's names for the screen.
pub(super) fn offer(menu: &mut ContainerMenu) -> std::sync::Arc<Vec<(String, String)>> {
    if menu.kind != photon_game::menus::container::MenuKind::Loom {
        return std::sync::Arc::default();
    }
    let registry = photon_game::registries::Registries::vanilla(&data_root()).banner_patterns;
    let names: Vec<String> = registry.iter().map(|entry| entry.name.clone()).collect();
    (menu.patterns, menu.banner_layers) =
        photon_game::menus::container::loom_patterns(&pack_tags(&names), menu);
    std::sync::Arc::new(
        registry
            .into_iter()
            .map(|entry| (entry.asset_id, entry.translation_key))
            .collect(),
    )
}
