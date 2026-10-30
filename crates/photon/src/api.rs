//! What Photon hands to the native API ([ADR 0006](../../../docs/adr/0006-mod-platform.md)):
//! the registries over the canonical ids of the game's own tables.

use photon_api::{Registries, Registry};

use photon_data::block::registry::{BLOCKS, block_index};
use photon_data::item::registry::{ITEMS, index};

/// The registries, by name and by canonical index.
pub(crate) fn registries() -> Registries {
    Registries {
        items: Registry::new(
            || ITEMS.len(),
            |at| ITEMS.get(at).map(|item| item.name),
            index,
        ),
        blocks: Registry::new(
            || BLOCKS.len(),
            |at| BLOCKS.get(at).map(|block| block.name),
            block_index,
        ),
        entity_types: Registry::new(
            || photon_data::entity_type::names().count(),
            |at| photon_data::entity_type::names().nth(at),
            |name| photon_data::entity_type::id(name).and_then(|id| usize::try_from(id.0).ok()),
        ),
        sound_events: Registry::new(
            || photon_data::sound_event::names().count(),
            |at| photon_data::sound_event::names().nth(at),
            |name| photon_data::sound_event::id(name).and_then(|id| usize::try_from(id).ok()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_registry_names_what_it_finds_and_finds_what_it_names() {
        let registries = registries();
        for (registry, known) in [
            (registries.items, "minecraft:stone"),
            (registries.blocks, "minecraft:stone"),
            (registries.entity_types, "minecraft:pig"),
            (registries.sound_events, "minecraft:entity.pig.ambient"),
        ] {
            let index = registry.index_of(known).expect(known);
            assert_eq!(registry.name_at(index), Some(known));
            assert!(registry.len() > index);
            assert!(!registry.contains("minecraft:no_such_entry"));
        }
    }
}
