//! `LoomMenu.slotsChanged`'s list of patterns.

use photon_world::block::tags::OrderedTags;

use super::ContainerMenu;

/// With a banner and a dye in, the patterns that need no item or those the pattern item in the
/// loom gives, in the order of the tag the server listed; and the banner's layers. Nothing without
/// a banner and a dye.
pub fn loom_patterns(tags: &OrderedTags, menu: &ContainerMenu) -> (Vec<i32>, usize) {
    let stack = |slot: usize| menu.slots.get(slot).and_then(Option::as_ref);
    let (Some(banner), Some(_dye)) = (stack(0), stack(1)) else {
        return (Vec::new(), 0);
    };
    let layers = crate::item::values::Values::new(banner.item_registry_id(), banner.patch_bytes())
        .banner_layers()
        .len();
    let tag = match stack(2) {
        None => "minecraft:no_item_required".to_owned(),
        Some(pattern) => {
            // `provides_banner_patterns` names `pattern_item/<name>`.
            let name = usize::try_from(pattern.item_registry_id().0)
                .ok()
                .and_then(|id| photon_data::item::registry::ITEMS.get(id))
                .map_or("", |item| item.name);
            let short = name
                .trim_start_matches("minecraft:")
                .trim_end_matches("_banner_pattern");
            format!("minecraft:pattern_item/{short}")
        }
    };
    (tags.members(&tag).collect(), layers)
}
