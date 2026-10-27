//! The copper blocks' weathering and waxing tables (`WeatheringCopper.PREVIOUS_BY_BLOCK`,
//! `HoneycombItem.WAXABLES` and its inverse), which an axe and a honeycomb read.

use super::chest::connected_direction;
use super::growth::offset;
use super::*;

/// The kinds `HoneycombItem.WAXABLES` zips: unwaxed, then waxed, each of the four ages.
const KINDS: &[&str] = &[
    "copper_block",
    "cut_copper",
    "cut_copper_slab",
    "cut_copper_stairs",
    "chiseled_copper",
    "copper_door",
    "copper_trapdoor",
    "copper_bars",
    "copper_grate",
    "copper_bulb",
    "copper_chest",
    "copper_golem_statue",
    "lightning_rod",
    "copper_lantern",
    "copper_chain",
];

const AGES: [&str; 4] = ["", "exposed_", "weathered_", "oxidized_"];

/// A copper block's kind, how weathered it is and whether it is waxed, from its name.
fn parts(name: &str) -> Option<(&'static str, usize, bool)> {
    let path = name.strip_prefix("minecraft:")?;
    let (waxed, path) = match path.strip_prefix("waxed_") {
        Some(rest) => (true, rest),
        None => (false, path),
    };
    KINDS.iter().find_map(|kind| {
        AGES.iter().enumerate().find_map(|(age, prefix)| {
            let written = if *kind == "copper_block" && age > 0 {
                format!("{prefix}copper")
            } else {
                format!("{prefix}{kind}")
            };
            (written == path).then_some((*kind, age, waxed))
        })
    })
}

/// The name of a kind at an age, waxed or not.
fn name_of(kind: &str, age: usize, waxed: bool) -> String {
    let base = if kind == "copper_block" && age > 0 {
        format!("{}copper", AGES[age])
    } else {
        format!("{}{kind}", AGES[age])
    };
    format!("minecraft:{}{base}", if waxed { "waxed_" } else { "" })
}

/// `Block.withPropertiesOf`: the block named `target`, with the properties `state` has.
fn same_properties(state: BlockStateId, target: &str) -> Option<BlockStateId> {
    super::chest::with_properties_of(registry::default_state_named(target), state)
}

/// `WeatheringCopper.getPrevious`: an unwaxed copper block one age fresher.
pub fn scraped(state: BlockStateId) -> Option<BlockStateId> {
    let (kind, age, waxed) = parts(block(state)?.name)?;
    (age > 0 && !waxed)
        .then(|| same_properties(state, &name_of(kind, age - 1, false)))
        .flatten()
}

/// `HoneycombItem.WAX_OFF_BY_BLOCK`: a waxed copper block as it was before the wax.
pub fn waxed_off(state: BlockStateId) -> Option<BlockStateId> {
    let (kind, age, waxed) = parts(block(state)?.name)?;
    waxed
        .then(|| same_properties(state, &name_of(kind, age, false)))
        .flatten()
}

/// `HoneycombItem.getWaxed`: an unwaxed copper block with its wax.
pub fn waxed(state: BlockStateId) -> Option<BlockStateId> {
    let (kind, age, waxed) = parts(block(state)?.name)?;
    (!waxed)
        .then(|| same_properties(state, &name_of(kind, age, true)))
        .flatten()
}

/// `ChestBlock.getConnectedBlockPos` of a double chest, which the wax also marks.
pub fn other_chest_half(state: BlockStateId, pos: (i32, i32, i32)) -> Option<(i32, i32, i32)> {
    if !block(state)?.name.ends_with("copper_chest") || property(state, "type") == Some("single") {
        return None;
    }
    Some(offset(pos, connected_direction(state)?))
}

#[cfg(test)]
mod tests {
    use super::{scraped, waxed, waxed_off};

    fn named(name: &str) -> photon_core::ids::BlockStateId {
        super::registry::default_state_named(name)
    }

    fn name_of(state: Option<photon_core::ids::BlockStateId>) -> Option<&'static str> {
        state.map(|state| crate::block::state_or_air(state).block().name)
    }

    #[test]
    fn lightning_rods_and_blocks_age_and_wax() {
        let rod = named("minecraft:exposed_lightning_rod");
        assert_eq!(name_of(scraped(rod)), Some("minecraft:lightning_rod"));
        assert_eq!(
            name_of(waxed(rod)),
            Some("minecraft:waxed_exposed_lightning_rod")
        );
        let waxed_rod = named("minecraft:waxed_lightning_rod");
        assert_eq!(
            name_of(waxed_off(waxed_rod)),
            Some("minecraft:lightning_rod")
        );
        assert_eq!(scraped(waxed_rod), None);
    }

    #[test]
    fn other_copper_things_are_left_alone() {
        for name in [
            "minecraft:copper_torch",
            "minecraft:raw_copper_block",
            "minecraft:copper_ore",
            "minecraft:stone",
        ] {
            let state = named(name);
            assert_eq!(scraped(state), None, "{name}");
            assert_eq!(waxed(state), None, "{name}");
            assert_eq!(waxed_off(state), None, "{name}");
        }
        assert_eq!(scraped(named("minecraft:copper_block")), None);
    }
}
