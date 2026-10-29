//! Blocks the code compares by place in the registry ([`super::registry::Block::index`]),
//! resolved while compiling: a name the registry lacks stops the build. Add one when code needs
//! it, not the whole registry.

/// `NAME = name` defines `pub const NAME` as the index of `minecraft:name`. The prefix is joined
/// here so the lint counts a name only where code spells it out.
macro_rules! blocks {
    ($($id:ident = $name:ident),* $(,)?) => {
        $(pub const $id: usize = super::registry::index_named(concat!("minecraft:", stringify!($name)));)*
    };
}

blocks! {
    AIR = air,
    BAMBOO = bamboo,
    BEDROCK = bedrock,
    BIG_DRIPLEAF = big_dripleaf,
    COPPER_GOLEM_STATUE = copper_golem_statue,
    CRIMSON_NYLIUM = crimson_nylium,
    END_PORTAL_FRAME = end_portal_frame,
    FLOWER_POT = flower_pot,
    GRASS_BLOCK = grass_block,
    HAY_BLOCK = hay_block,
    JIGSAW = jigsaw,
    JUKEBOX = jukebox,
    LAVA = lava,
    LECTERN = lectern,
    NETHER_PORTAL = nether_portal,
    NOTE_BLOCK = note_block,
    OPEN_EYEBLOSSOM = open_eyeblossom,
    PALE_MOSS_CARPET = pale_moss_carpet,
    PITCHER_CROP = pitcher_crop,
    POTENT_SULFUR = potent_sulfur,
    REDSTONE_WIRE = redstone_wire,
    STRUCTURE_BLOCK = structure_block,
    TEST_BLOCK = test_block,
    WARPED_NYLIUM = warped_nylium,
    WATER = water,
}
