//! Groups of entity types the client treats alike, named so that the numbers follow the registry.

/// The numbers of the entity types named, resolved while compiling: a name the registry lacks
/// stops the build.
macro_rules! entity_type_ids {
    ($($name:literal),+ $(,)?) => {
        &[$(photon_data::entity_type::named($name).0),+]
    };
}

/// The number of one entity type, resolved while compiling.
pub(crate) const fn entity_type_id(name: &str) -> i32 {
    photon_data::entity_type::named(name).0
}

pub(crate) const VANILLA_LIVING_ENTITY_TYPE_IDS: &[i32] = entity_type_ids![
    "minecraft:allay",
    "minecraft:armadillo",
    "minecraft:armor_stand",
    "minecraft:axolotl",
    "minecraft:bat",
    "minecraft:bee",
    "minecraft:blaze",
    "minecraft:bogged",
    "minecraft:breeze",
    "minecraft:camel",
    "minecraft:camel_husk",
    "minecraft:cat",
    "minecraft:cave_spider",
    "minecraft:chicken",
    "minecraft:cod",
    "minecraft:copper_golem",
    "minecraft:cow",
    "minecraft:creaking",
    "minecraft:creeper",
    "minecraft:dolphin",
    "minecraft:donkey",
    "minecraft:drowned",
    "minecraft:elder_guardian",
    "minecraft:enderman",
    "minecraft:endermite",
    "minecraft:ender_dragon",
    "minecraft:evoker",
    "minecraft:fox",
    "minecraft:frog",
    "minecraft:ghast",
    "minecraft:happy_ghast",
    "minecraft:giant",
    "minecraft:glow_squid",
    "minecraft:goat",
    "minecraft:guardian",
    "minecraft:hoglin",
    "minecraft:horse",
    "minecraft:husk",
    "minecraft:illusioner",
    "minecraft:iron_golem",
    "minecraft:llama",
    "minecraft:magma_cube",
    "minecraft:mannequin",
    "minecraft:mooshroom",
    "minecraft:mule",
    "minecraft:nautilus",
    "minecraft:ocelot",
    "minecraft:panda",
    "minecraft:parched",
    "minecraft:parrot",
    "minecraft:phantom",
    "minecraft:pig",
    "minecraft:piglin",
    "minecraft:piglin_brute",
    "minecraft:pillager",
    "minecraft:polar_bear",
    "minecraft:pufferfish",
    "minecraft:rabbit",
    "minecraft:ravager",
    "minecraft:salmon",
    "minecraft:sheep",
    "minecraft:shulker",
    "minecraft:silverfish",
    "minecraft:skeleton",
    "minecraft:skeleton_horse",
    "minecraft:slime",
    "minecraft:sniffer",
    "minecraft:snow_golem",
    "minecraft:spider",
    "minecraft:squid",
    "minecraft:stray",
    "minecraft:strider",
    "minecraft:sulfur_cube",
    "minecraft:tadpole",
    "minecraft:trader_llama",
    "minecraft:tropical_fish",
    "minecraft:turtle",
    "minecraft:vex",
    "minecraft:villager",
    "minecraft:vindicator",
    "minecraft:wandering_trader",
    "minecraft:warden",
    "minecraft:witch",
    "minecraft:wither",
    "minecraft:wither_skeleton",
    "minecraft:wolf",
    "minecraft:zoglin",
    "minecraft:zombie",
    "minecraft:zombie_horse",
    "minecraft:zombie_nautilus",
    "minecraft:zombie_villager",
    "minecraft:zombified_piglin",
    "minecraft:player",
];
pub(crate) const VANILLA_ANIMAL_ENTITY_TYPE_IDS: &[i32] = entity_type_ids![
    "minecraft:armadillo",
    "minecraft:axolotl",
    "minecraft:bee",
    "minecraft:camel",
    "minecraft:camel_husk",
    "minecraft:cat",
    "minecraft:chicken",
    "minecraft:cow",
    "minecraft:donkey",
    "minecraft:fox",
    "minecraft:frog",
    "minecraft:happy_ghast",
    "minecraft:goat",
    "minecraft:hoglin",
    "minecraft:horse",
    "minecraft:llama",
    "minecraft:mooshroom",
    "minecraft:mule",
    "minecraft:nautilus",
    "minecraft:ocelot",
    "minecraft:panda",
    "minecraft:parrot",
    "minecraft:pig",
    "minecraft:polar_bear",
    "minecraft:rabbit",
    "minecraft:sheep",
    "minecraft:skeleton_horse",
    "minecraft:sniffer",
    "minecraft:strider",
    "minecraft:trader_llama",
    "minecraft:turtle",
    "minecraft:wolf",
    "minecraft:zombie_horse",
    "minecraft:zombie_nautilus",
];
pub(crate) const VANILLA_CHEST_BOAT_ENTITY_TYPE_IDS: &[i32] = entity_type_ids![
    "minecraft:acacia_chest_boat",
    "minecraft:bamboo_chest_raft",
    "minecraft:birch_chest_boat",
    "minecraft:cherry_chest_boat",
    "minecraft:dark_oak_chest_boat",
    "minecraft:jungle_chest_boat",
    "minecraft:mangrove_chest_boat",
    "minecraft:oak_chest_boat",
    "minecraft:pale_oak_chest_boat",
    "minecraft:spruce_chest_boat",
];
pub(crate) const VANILLA_MINECART_ENTITY_TYPE_IDS: &[i32] = entity_type_ids![
    "minecraft:chest_minecart",
    "minecraft:command_block_minecart",
    "minecraft:furnace_minecart",
    "minecraft:hopper_minecart",
    "minecraft:minecart",
    "minecraft:spawner_minecart",
    "minecraft:tnt_minecart",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_groups_hold_the_types_they_are_named_for() {
        let id = |name| photon_data::entity_type::named(name).0;
        assert!(VANILLA_ANIMAL_ENTITY_TYPE_IDS.contains(&id("minecraft:cow")));
        assert!(!VANILLA_ANIMAL_ENTITY_TYPE_IDS.contains(&id("minecraft:zombie")));
        assert!(VANILLA_LIVING_ENTITY_TYPE_IDS.contains(&id("minecraft:zombie")));
        assert!(VANILLA_CHEST_BOAT_ENTITY_TYPE_IDS.contains(&id("minecraft:oak_chest_boat")));
        assert!(VANILLA_MINECART_ENTITY_TYPE_IDS.contains(&id("minecraft:minecart")));
    }
}
