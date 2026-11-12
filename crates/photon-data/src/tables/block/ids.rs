//! The places of the blocks the client names in code, each resolved from its name while compiling.
//! The block table itself is written by `cargo xtask gen` and re-exports these.

use super::blocks::BLOCKS;

/// A block's place in the registry, resolved while compiling: a name the registry lacks stops
/// the build.
pub const fn index_of(name: &str) -> usize {
    let mut index = 0;
    while index < BLOCKS.len() {
        if crate::name::same(BLOCKS[index].name, name) {
            return index;
        }
        index += 1;
    }
    panic!("the block registry has no such name");
}

/// The places of the blocks the client names in code, each resolved from its name.
pub mod id {
    use super::index_of;

    pub const BAMBOO_SAPLING: usize = index_of("minecraft:bamboo_sapling");
    pub const BEACON: usize = index_of("minecraft:beacon");
    pub const CAMPFIRE: usize = index_of("minecraft:campfire");
    pub const COBWEB: usize = index_of("minecraft:cobweb");
    pub const CONDUIT: usize = index_of("minecraft:conduit");
    pub const CRYING_OBSIDIAN: usize = index_of("minecraft:crying_obsidian");
    pub const DARK_PRISMARINE: usize = index_of("minecraft:dark_prismarine");
    pub const ENCHANTING_TABLE: usize = index_of("minecraft:enchanting_table");
    pub const END_GATEWAY: usize = index_of("minecraft:end_gateway");
    pub const HONEY_BLOCK: usize = index_of("minecraft:honey_block");
    pub const LAVA: usize = index_of("minecraft:lava");
    pub const MANGROVE_ROOTS: usize = index_of("minecraft:mangrove_roots");
    pub const MOVING_PISTON: usize = index_of("minecraft:moving_piston");
    pub const MUD: usize = index_of("minecraft:mud");
    pub const OBSIDIAN: usize = index_of("minecraft:obsidian");
    pub const PALE_MOSS_BLOCK: usize = index_of("minecraft:pale_moss_block");
    pub const PALE_OAK_LEAVES: usize = index_of("minecraft:pale_oak_leaves");
    pub const PISTON: usize = index_of("minecraft:piston");
    pub const PISTON_HEAD: usize = index_of("minecraft:piston_head");
    pub const POTENT_SULFUR: usize = index_of("minecraft:potent_sulfur");
    pub const PRISMARINE: usize = index_of("minecraft:prismarine");
    pub const PRISMARINE_BRICKS: usize = index_of("minecraft:prismarine_bricks");
    pub const RED_SAND: usize = index_of("minecraft:red_sand");
    pub const REINFORCED_DEEPSLATE: usize = index_of("minecraft:reinforced_deepslate");
    pub const RESPAWN_ANCHOR: usize = index_of("minecraft:respawn_anchor");
    pub const SEA_LANTERN: usize = index_of("minecraft:sea_lantern");
    pub const SLIME_BLOCK: usize = index_of("minecraft:slime_block");
    pub const SOUL_CAMPFIRE: usize = index_of("minecraft:soul_campfire");
    pub const SPAWNER: usize = index_of("minecraft:spawner");
    pub const STICKY_PISTON: usize = index_of("minecraft:sticky_piston");
    pub const TRIAL_SPAWNER: usize = index_of("minecraft:trial_spawner");
    pub const VAULT: usize = index_of("minecraft:vault");
    pub const WATER: usize = index_of("minecraft:water");
}
