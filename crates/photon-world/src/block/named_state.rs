//! States made from a block's name and properties, and `BlockState.withPropertiesOf`: what a
//! block state provider needs to build the state it offers.

use photon_core::ids::BlockStateId;
use photon_data::block::registry;
use std::collections::BTreeMap;

/// The block of `name` in the state its default state has, with the named properties set over
/// it. `None` for a name the registry lacks or a value a property cannot hold.
pub fn state_named(name: &str, properties: &[(String, String)]) -> Option<BlockStateId> {
    let block = &registry::BLOCKS[registry::block_index(name)?];
    let chosen: BTreeMap<String, String> = properties.iter().cloned().collect();
    block.state_id(&chosen)
}

/// `BlockState.withPropertiesOf`: `state` with every property `source` has set to the value
/// `source` has for it, where `state`'s block has the property and can hold the value.
pub fn with_properties_of(state: BlockStateId, source: BlockStateId) -> BlockStateId {
    let (Some((block, offset)), Some((from, from_offset))) = (
        registry::block_for_state(state),
        registry::block_for_state(source),
    ) else {
        return state;
    };
    let (Some(own), Some(copied)) = (block.properties_at(offset), from.properties_at(from_offset))
    else {
        return state;
    };
    let mut chosen: BTreeMap<String, String> = own
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect();
    for (name, value) in copied {
        if let Some(slot) = chosen.get_mut(name) {
            let previous = std::mem::replace(slot, value.to_owned());
            if block.state_id(&chosen).is_none() {
                *chosen.get_mut(name).expect("the slot is there") = previous;
            }
        }
    }
    block.state_id(&chosen).unwrap_or(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::rules::property;

    #[test]
    fn a_log_keeps_its_axis_through_the_copy() {
        let log = state_named("minecraft:oak_log", &[("axis".into(), "x".into())]).unwrap();
        let stripped = state_named("minecraft:stripped_oak_log", &[]).unwrap();
        assert_eq!(property(stripped, "axis"), Some("y"));
        let copied = with_properties_of(stripped, log);
        assert_eq!(property(copied, "axis"), Some("x"));
        assert_eq!(
            registry::block_for_state(copied).unwrap().0.name,
            "minecraft:stripped_oak_log"
        );
    }

    #[test]
    fn a_property_the_new_block_lacks_is_left_out() {
        let farmland =
            state_named("minecraft:farmland", &[("moisture".into(), "0".into())]).unwrap();
        let grass = state_named("minecraft:grass_block", &[]).unwrap();
        assert_eq!(with_properties_of(farmland, grass), farmland);
        assert!(state_named("minecraft:nothing", &[]).is_none());
        assert!(state_named("minecraft:farmland", &[("moisture".into(), "9".into())]).is_none());
    }
}
