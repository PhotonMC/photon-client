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
    ALLAY = allay,
    ARMADILLO = armadillo,
    ARMOR_STAND = armor_stand,
    AREA_EFFECT_CLOUD = area_effect_cloud,
    BLOCK_DISPLAY = block_display,
    ITEM_DISPLAY = item_display,
    OMINOUS_ITEM_SPAWNER = ominous_item_spawner,
    TEXT_DISPLAY = text_display,
    AXOLOTL = axolotl,
    BEE = bee,
    BLAZE = blaze,
    BREEZE = breeze,
    CAT = cat,
    COPPER_GOLEM = copper_golem,
    CREEPER = creeper,
    ELDER_GUARDIAN = elder_guardian,
    ENDER_DRAGON = ender_dragon,
    ENDERMAN = enderman,
    EVOKER_FANGS = evoker_fangs,
    FOX = fox,
    GHAST = ghast,
    GLOW_ITEM_FRAME = glow_item_frame,
    ITEM_FRAME = item_frame,
    PAINTING = painting,
    PIG = pig,
    GLOW_SQUID = glow_squid,
    GUARDIAN = guardian,
    HOGLIN = hoglin,
    INTERACTION = interaction,
    LEASH_KNOT = leash_knot,
    MARKER = marker,
    ILLUSIONER = illusioner,
    MOOSHROOM = mooshroom,
    OCELOT = ocelot,
    PANDA = panda,
    PARROT = parrot,
    PHANTOM = phantom,
    PIGLIN = piglin,
    POLAR_BEAR = polar_bear,
    RABBIT = rabbit,
    SHULKER = shulker,
    SHULKER_BULLET = shulker_bullet,
    SNIFFER = sniffer,
    SNOW_GOLEM = snow_golem,
    SULFUR_CUBE = sulfur_cube,
    TURTLE = turtle,
    WARDEN = warden,
    WITHER_SKULL = wither_skull,
    WOLF = wolf,
    ZOMBIE_VILLAGER = zombie_villager,
    RAVAGER = ravager,
    PLAYER = player,
    MANNEQUIN = mannequin,
    GOAT = goat,
    FROG = frog,
    DOLPHIN = dolphin,
    CAMEL_HUSK = camel_husk,
    CAMEL = camel,
}

/// `Attributes.STEP_HEIGHT`, the name the registry gives it.
pub const STEP_HEIGHT_ATTRIBUTE: &str = concat!("minecraft:", "step_height");
