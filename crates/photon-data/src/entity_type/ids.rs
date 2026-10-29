//! Entity types the code compares by id, resolved while compiling like [`super::named`]: a name
//! the registry lacks stops the build. Add one when code needs it, not the whole registry.

use photon_core::ids::EntityTypeId;

/// `NAME = name` defines `pub const NAME` as the id of `minecraft:name`. The prefix is joined
/// here so the lint counts a name only where code spells it out.
macro_rules! entity_types {
    ($($id:ident = $name:ident),* $(,)?) => {
        $(pub const $id: EntityTypeId = super::named(concat!("minecraft:", stringify!($name)));)*
    };
}

entity_types! {
    ARMADILLO = armadillo,
    AXOLOTL = axolotl,
    BEE = bee,
    CAT = cat,
    COPPER_GOLEM = copper_golem,
    CREEPER = creeper,
    ENDERMAN = enderman,
    FOX = fox,
    GHAST = ghast,
    GLOW_SQUID = glow_squid,
    HOGLIN = hoglin,
    MOOSHROOM = mooshroom,
    PANDA = panda,
    PARROT = parrot,
    PHANTOM = phantom,
    PIGLIN = piglin,
    POLAR_BEAR = polar_bear,
    RABBIT = rabbit,
    SHULKER = shulker,
    SNIFFER = sniffer,
    SNOW_GOLEM = snow_golem,
    SULFUR_CUBE = sulfur_cube,
    TURTLE = turtle,
    WARDEN = warden,
    WITHER_SKULL = wither_skull,
    WOLF = wolf,
}
