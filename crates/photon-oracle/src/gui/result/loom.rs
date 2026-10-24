//! The loom's patterns as the vanilla data pack offers them: the tag of the pattern slot's item.

use photon_game::menus::container::ContainerMenu;

fn data_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.data/minecraft/assets/jar/data")
}

/// The entries a banner pattern tag lists, in the file's order.
fn tag_values(tag: &str) -> Vec<String> {
    let path = data_root().join(format!(
        "minecraft/tags/banner_pattern/{}.json",
        tag.trim_start_matches("minecraft:")
    ));
    let file: serde_json::Value = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    file["values"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|name| name.as_str().map(str::to_owned))
        .collect()
}

/// What the loom offers for the menu's slots: the registry's names and the offered ids.
pub(super) fn offer(menu: &mut ContainerMenu) -> std::sync::Arc<Vec<(String, String)>> {
    if menu.kind != photon_game::menus::container::MenuKind::Loom {
        return std::sync::Arc::default();
    }
    let item = |slot: usize| menu.slots.get(slot).and_then(Option::as_ref);
    let tag = match item(2) {
        None => "no_item_required".to_owned(),
        Some(pattern) => {
            let name =
                photon_data::item::registry::ITEMS[pattern.item_registry_id().0 as usize].name;
            let short = name
                .trim_start_matches("minecraft:")
                .trim_end_matches("_banner_pattern");
            format!("pattern_item/{short}")
        }
    };
    let offered = tag_values(&tag);
    let registry = photon_game::registries::Registries::vanilla(&data_root()).banner_patterns;
    // The ids in the tag's order, as the server lists them.
    let ids: Vec<i32> = offered
        .iter()
        .filter_map(|name| registry.iter().position(|entry| entry.name == *name))
        .map(|id| id as i32)
        .collect();
    menu.patterns = ids;
    menu.banner_layers = item(0)
        .map(|banner| {
            photon_game::item::values::Values::new(banner.item_registry_id(), banner.patch_bytes())
                .banner_layers()
                .len()
        })
        .unwrap_or(0);
    std::sync::Arc::new(
        registry
            .into_iter()
            .map(|entry| (entry.asset_id, entry.translation_key))
            .collect(),
    )
}
