use super::*;

/// A copper golem's action and weathering (`COPPER_GOLEM_STATE`,
/// `DATA_WEATHER_STATE`: each a varint id), each when the data holds it.
pub(crate) fn copper_golem_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<(Option<i32>, Option<i32>)> {
    if type_id != photon_data::entity_type::ids::COPPER_GOLEM {
        return None;
    }
    let varint = |data_id: u8, serializer: i32| {
        let (serializer_id, mut value_input) = find_metadata_field(metadata, data_id)?;
        if serializer_id != serializer {
            return None;
        }
        let value = photon_core::wire::read_varint(&mut value_input).ok()?;
        (!value_input.is_empty()).then_some(value)
    };
    Some((
        varint(COPPER_GOLEM_STATE_DATA_ID, COPPER_GOLEM_STATE_SERIALIZER_ID),
        varint(
            COPPER_GOLEM_WEATHER_DATA_ID,
            COPPER_GOLEM_WEATHER_SERIALIZER_ID,
        ),
    ))
}

/// A glow squid's dark ticks (`GlowSquid.DATA_DARK_TICKS_REMAINING`), when the
/// data holds them.
pub(crate) fn glow_squid_dark_ticks_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<i32> {
    if type_id != photon_data::entity_type::ids::GLOW_SQUID {
        return None;
    }
    metadata_int_value(metadata, GLOW_SQUID_DARK_TICKS_DATA_ID)
}

/// Whether a ghast is charging a fireball (`Ghast.DATA_IS_CHARGING`), when the
/// data holds it.
pub(crate) fn ghast_charging_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<bool> {
    if type_id != photon_data::entity_type::ids::GHAST {
        return None;
    }
    metadata_boolean_value(metadata, GHAST_CHARGING_DATA_ID)
}

/// Whether a wither skull is the blue one (`WitherSkull.DATA_DANGEROUS`), when the data holds it.
pub(crate) fn wither_skull_dangerous_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<bool> {
    if type_id != photon_data::entity_type::ids::WITHER_SKULL {
        return None;
    }
    metadata_boolean_value(metadata, WITHER_SKULL_DANGEROUS_DATA_ID)
}

/// A bat's flags (`Bat.DATA_ID_FLAGS`: resting), a vex's (`Vex.DATA_FLAGS_ID`:
/// charging) and whether an allay dances (`Allay.DATA_DANCING`), each when the
/// data holds it.
pub(crate) fn flier_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<u8>, Option<u8>, Option<bool>) {
    match photon_data::entity_type::name(type_id) {
        Some("minecraft:bat") => (metadata_byte_value(metadata, BAT_FLAGS_DATA_ID), None, None),
        Some("minecraft:vex") => (None, metadata_byte_value(metadata, VEX_FLAGS_DATA_ID), None),
        Some("minecraft:allay") => (
            None,
            None,
            metadata_boolean_value(metadata, ALLAY_DANCING_DATA_ID),
        ),
        _ => (None, None, None),
    }
}

/// A parrot's variant id (`Parrot.DATA_VARIANT_ID`), when the data holds it.
pub(crate) fn parrot_variant_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<i32> {
    if type_id != photon_data::entity_type::ids::PARROT {
        return None;
    }
    metadata_int_value(metadata, PARROT_VARIANT_DATA_ID)
}

/// A slime's or a magma cube's size (`AbstractCubeMob.ID_SIZE`), when the data holds it.
pub(crate) fn cube_mob_size_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<i32> {
    if !is_cube_mob_type(type_id) {
        return None;
    }
    metadata_int_value(metadata, CUBE_MOB_SIZE_DATA_ID)
}

/// Whether the type is a slime or a magma cube.
pub(crate) fn is_cube_mob_type(type_id: EntityTypeId) -> bool {
    matches!(
        photon_data::entity_type::name(type_id),
        Some("minecraft:slime" | "minecraft:magma_cube" | "minecraft:sulfur_cube")
    )
}

/// A sulfur cube's fuse (`SulfurCube.MAX_FUSE`), when the data holds it.
pub(crate) fn sulfur_cube_fuse_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<i32> {
    if type_id != photon_data::entity_type::ids::SULFUR_CUBE {
        return None;
    }
    metadata_int_value(metadata, SULFUR_CUBE_FUSE_DATA_ID)
}

/// A shulker's attach face (a direction's data value), raw peek and colour, each
/// when the data holds it.
pub(crate) fn shulker_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<i32>, Option<u8>, Option<u8>) {
    if type_id != photon_data::entity_type::ids::SHULKER {
        return (None, None, None);
    }
    let face = find_metadata_field(metadata, SHULKER_ATTACH_FACE_DATA_ID)
        .filter(|(serializer, _)| *serializer == DIRECTION_SERIALIZER_ID)
        .and_then(|(_, mut value)| photon_core::wire::read_varint(&mut value).ok());
    (
        face,
        metadata_byte_value(metadata, SHULKER_PEEK_DATA_ID),
        metadata_byte_value(metadata, SHULKER_COLOR_DATA_ID),
    )
}

/// A phantom's size (`Phantom.ID_SIZE`), when the data holds it.
pub(crate) fn phantom_size_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<i32> {
    if type_id != photon_data::entity_type::ids::PHANTOM {
        return None;
    }
    metadata_int_value(metadata, PHANTOM_SIZE_DATA_ID)
}

/// A warden's client anger level (`Warden.CLIENT_ANGER_LEVEL`), when the data holds it.
pub(crate) fn warden_anger_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<i32> {
    if type_id != photon_data::entity_type::ids::WARDEN {
        return None;
    }
    metadata_int_value(metadata, crate::mobs::warden::CLIENT_ANGER_DATA_ID)
}

/// `WitherBoss.DATA_TARGET_B`, `DATA_TARGET_C` and `DATA_ID_INV`.
pub(crate) fn wither_data_from_metadata(metadata: &[u8]) -> WitherData {
    use crate::mobs::wither::{INVULNERABLE_DATA_ID, TARGET_B_DATA_ID, TARGET_C_DATA_ID};
    WitherData {
        targets: [
            metadata_int_value(metadata, TARGET_B_DATA_ID),
            metadata_int_value(metadata, TARGET_C_DATA_ID),
        ],
        invulnerable_ticks: metadata_int_value(metadata, INVULNERABLE_DATA_ID),
    }
}

/// A sniffer's synced state (`Sniffer.DATA_STATE`), when the data holds it.
pub(crate) fn sniffer_state_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<crate::mobs::sniffer::SnifferMode> {
    /// `EntityDataSerializers.SNIFFER_STATE`'s place in the registry of serializers.
    const SNIFFER_STATE_SERIALIZER_ID: i32 = 35;
    if type_id != photon_data::entity_type::ids::SNIFFER {
        return None;
    }
    let (serializer_id, mut value_input) =
        find_metadata_field(metadata, crate::mobs::sniffer::STATE_DATA_ID)?;
    if serializer_id != SNIFFER_STATE_SERIALIZER_ID {
        return None;
    }
    let id = photon_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(crate::mobs::sniffer::SnifferMode::from_id(id))
}

/// An armadillo's synced state (`Armadillo.ARMADILLO_STATE`), when the data holds it.
pub(crate) fn armadillo_state_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<crate::mobs::armadillo::ArmadilloMode> {
    /// `EntityDataSerializers.ARMADILLO_STATE`'s place in the registry of serializers.
    const ARMADILLO_STATE_SERIALIZER_ID: i32 = 36;
    if type_id != photon_data::entity_type::ids::ARMADILLO {
        return None;
    }
    let (serializer_id, mut value_input) =
        find_metadata_field(metadata, crate::mobs::armadillo::STATE_DATA_ID)?;
    if serializer_id != ARMADILLO_STATE_SERIALIZER_ID {
        return None;
    }
    let id = photon_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(crate::mobs::armadillo::ArmadilloMode::from_id(id))
}

/// `Creaking.CAN_MOVE`, `IS_ACTIVE`, `IS_TEARING_DOWN` and whether `HOME_POS` holds a position.
pub(crate) fn creaking_data_from_metadata(metadata: &[u8]) -> CreakingData {
    use crate::mobs::creaking::{
        CAN_MOVE_DATA_ID, HOME_POS_DATA_ID, IS_ACTIVE_DATA_ID, IS_TEARING_DOWN_DATA_ID,
    };
    let heart_bound =
        find_metadata_field(metadata, HOME_POS_DATA_ID).and_then(|(serializer_id, value_input)| {
            (serializer_id == ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID)
                .then(|| value_input.first().map(|&present| present != 0))
                .flatten()
        });
    CreakingData {
        can_move: metadata_boolean_value(metadata, CAN_MOVE_DATA_ID),
        active: metadata_boolean_value(metadata, IS_ACTIVE_DATA_ID),
        tearing_down: metadata_boolean_value(metadata, IS_TEARING_DOWN_DATA_ID),
        heart_bound,
    }
}

/// Whether a guardian moves and the id of what its beam aims at
/// (`Guardian.DATA_ID_MOVING`, `DATA_ID_ATTACK_TARGET`), each when the data holds it.
pub(crate) fn guardian_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<bool>, Option<i32>) {
    if !matches!(
        photon_data::entity_type::name(type_id),
        Some("minecraft:guardian" | "minecraft:elder_guardian")
    ) {
        return (None, None);
    }
    (
        metadata_boolean_value(metadata, GUARDIAN_MOVING_DATA_ID),
        metadata_int_value(metadata, GUARDIAN_ATTACK_TARGET_DATA_ID),
    )
}

/// What a salmon, a tropical fish or a pufferfish keeps of its looks in one int
/// (`Salmon.DATA_TYPE`, `TropicalFish.DATA_ID_TYPE_VARIANT`, `Pufferfish.PUFF_STATE`),
/// when the data holds it.
pub(crate) fn fish_variant_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<i32> {
    let data_id = match photon_data::entity_type::name(type_id)? {
        "minecraft:salmon" => SALMON_TYPE_DATA_ID,
        "minecraft:tropical_fish" => TROPICAL_FISH_VARIANT_DATA_ID,
        "minecraft:pufferfish" => PUFFERFISH_PUFF_STATE_DATA_ID,
        _ => return None,
    };
    metadata_int_value(metadata, data_id)
}

/// An axolotl's variant and whether it plays dead (`Axolotl.DATA_VARIANT`,
/// `Axolotl.DATA_PLAYING_DEAD`), each when the data holds it.
pub(crate) fn axolotl_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<i32>, Option<bool>) {
    if type_id != photon_data::entity_type::ids::AXOLOTL {
        return (None, None);
    }
    (
        metadata_int_value(metadata, AXOLOTL_VARIANT_DATA_ID),
        metadata_boolean_value(metadata, AXOLOTL_PLAYING_DEAD_DATA_ID),
    )
}

/// Whether a turtle carries an egg and is laying it (`Turtle.HAS_EGG`,
/// `Turtle.LAYING_EGG`), each when the data holds it.
pub(crate) fn turtle_eggs_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>) {
    if type_id != photon_data::entity_type::ids::TURTLE {
        return (None, None);
    }
    (
        metadata_boolean_value(metadata, TURTLE_HAS_EGG_DATA_ID),
        metadata_boolean_value(metadata, TURTLE_LAYING_EGG_DATA_ID),
    )
}

/// A hoglin's immunity to zombification (`Hoglin.DATA_IMMUNE_TO_ZOMBIFICATION`),
/// when the data holds it.
pub(crate) fn hoglin_immune_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<bool> {
    if type_id != photon_data::entity_type::ids::HOGLIN {
        return None;
    }
    metadata_boolean_value(metadata, HOGLIN_IMMUNE_DATA_ID)
}

/// What a panda's data holds, each when it is there.
pub(crate) fn panda_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> crate::mobs::panda::PandaData {
    if type_id != photon_data::entity_type::ids::PANDA {
        return crate::mobs::panda::PandaData::default();
    }
    crate::mobs::panda::PandaData {
        unhappy_counter: metadata_int_value(metadata, PANDA_UNHAPPY_DATA_ID),
        sneeze_counter: metadata_int_value(metadata, PANDA_SNEEZE_DATA_ID),
        eat_counter: metadata_int_value(metadata, PANDA_EAT_DATA_ID),
        main_gene: metadata_byte_value(metadata, PANDA_MAIN_GENE_DATA_ID),
        hidden_gene: metadata_byte_value(metadata, PANDA_HIDDEN_GENE_DATA_ID),
        flags: metadata_byte_value(metadata, PANDA_FLAGS_DATA_ID),
    }
}

/// A polar bear's standing flag (`PolarBear.DATA_STANDING_ID`), when the data holds it.
pub(crate) fn polar_bear_standing_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<bool> {
    if type_id != photon_data::entity_type::ids::POLAR_BEAR {
        return None;
    }
    metadata_boolean_value(metadata, POLAR_BEAR_STANDING_DATA_ID)
}

/// A rabbit's variant (`Rabbit.DATA_TYPE_ID`), when the data holds it.
pub(crate) fn rabbit_type_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<i32> {
    if type_id != photon_data::entity_type::ids::RABBIT {
        return None;
    }
    metadata_int_value(metadata, RABBIT_TYPE_DATA_ID)
}

/// A mooshroom's variant (`MushroomCow.DATA_TYPE`), when the data holds it.
pub(crate) fn mooshroom_type_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<i32> {
    if type_id != photon_data::entity_type::ids::MOOSHROOM {
        return None;
    }
    metadata_int_value(metadata, MOOSHROOM_TYPE_DATA_ID)
}

/// Whether a snow golem wears its pumpkin (`SnowGolem.DATA_PUMPKIN_ID`, bit
/// sixteen), when the data holds it.
pub(crate) fn snow_golem_pumpkin_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<bool> {
    if type_id != photon_data::entity_type::ids::SNOW_GOLEM {
        return None;
    }
    metadata_byte_value(metadata, SNOW_GOLEM_PUMPKIN_DATA_ID).map(|flags| flags & 16 != 0)
}

/// A piglin's baby, dancing and crossbow-charging flags (`Piglin.DATA_BABY_ID`,
/// `DATA_IS_DANCING`, `DATA_IS_CHARGING_CROSSBOW`), when the data holds them.
pub(crate) fn piglin_flags_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>, Option<bool>) {
    if type_id != photon_data::entity_type::ids::PIGLIN {
        return (None, None, None);
    }
    (
        metadata_boolean_value(metadata, PIGLIN_BABY_DATA_ID),
        metadata_boolean_value(metadata, PIGLIN_DANCING_DATA_ID),
        metadata_boolean_value(metadata, PIGLIN_CHARGING_CROSSBOW_DATA_ID),
    )
}

/// A sheep's wool byte (`Sheep.DATA_WOOL_ID`), or the shorn bit alone for a bogged
/// (`Bogged.DATA_SHEARED`), when the data holds it.
pub(crate) fn sheep_wool_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<u8> {
    match photon_data::entity_type::name(type_id)? {
        "minecraft:sheep" => metadata_byte_value(metadata, SHEEP_WOOL_DATA_ID),
        "minecraft:bogged" => metadata_boolean_value(metadata, BOGGED_SHEARED_DATA_ID)
            .map(|shorn| u8::from(shorn) << 4),
        _ => None,
    }
}

/// A creeper's fuse direction and ignited flag (`Creeper.DATA_SWELL_DIR`,
/// `DATA_IS_IGNITED`), when the data holds them.
pub(crate) fn creeper_fuse_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<i32>, Option<bool>) {
    if type_id != photon_data::entity_type::ids::CREEPER {
        return (None, None);
    }
    (
        metadata_int_value(metadata, CREEPER_SWELL_DIR_DATA_ID),
        metadata_boolean_value(metadata, CREEPER_IGNITED_DATA_ID),
    )
}

/// Whether a creeper is charged (`Creeper.DATA_IS_POWERED`), when the data
/// holds it.
pub(crate) fn creeper_powered_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<bool> {
    if type_id != photon_data::entity_type::ids::CREEPER {
        return None;
    }
    metadata_boolean_value(metadata, CREEPER_POWERED_DATA_ID)
}

/// A villager's or zombie villager's type, profession and level
/// (`Villager.DATA_VILLAGER_DATA`), when the data holds them.
pub(crate) fn villager_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> Option<[i32; 3]> {
    let data_id = match photon_data::entity_type::name(type_id)? {
        "minecraft:villager" => VILLAGER_DATA_DATA_ID,
        "minecraft:zombie_villager" => ZOMBIE_VILLAGER_DATA_DATA_ID,
        _ => return None,
    };
    let (serializer, mut value) = find_metadata_field(metadata, data_id)?;
    if serializer != VILLAGER_DATA_SERIALIZER_ID {
        return None;
    }
    let mut read = || photon_core::wire::read_varint(&mut value).ok();
    let data = [read()?, read()?, read()?];
    // The list's terminator must follow.
    (!value.is_empty()).then_some(data)
}

/// A villager's or wandering trader's unhappy counter
/// (`AbstractVillager.DATA_UNHAPPY_COUNTER`), when the data holds it.
pub(crate) fn unhappy_counter_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> Option<i32> {
    matches!(
        photon_data::entity_type::name(type_id)?,
        "minecraft:villager" | "minecraft:wandering_trader"
    )
    .then(|| metadata_int_value(metadata, VILLAGER_UNHAPPY_DATA_ID))?
}

/// A raider's celebrating flag (`Raider.IS_CELEBRATING`) and a spellcaster's
/// casting byte (`SpellcasterIllager.DATA_SPELL_CASTING_ID`, above zero while
/// casting), when the data holds them.
pub(crate) fn illager_flags_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>) {
    let Some(name) = photon_data::entity_type::name(type_id) else {
        return (None, None);
    };
    let celebrating = matches!(
        name,
        "minecraft:vindicator" | "minecraft:pillager" | "minecraft:evoker" | "minecraft:illusioner"
    )
    .then(|| metadata_boolean_value(metadata, RAIDER_CELEBRATING_DATA_ID))
    .flatten();
    let casting = matches!(name, "minecraft:evoker" | "minecraft:illusioner")
        .then(|| metadata_byte_value(metadata, SPELLCASTER_CASTING_DATA_ID).map(|spell| spell > 0))
        .flatten();
    (celebrating, casting)
}

/// The block state an enderman carries (`DATA_CARRY_STATE`, zero for none)
/// and whether it is creepy (`DATA_CREEPY`), when the data holds them.
pub(crate) fn enderman_flags_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<i32>, Option<bool>) {
    if type_id != photon_data::entity_type::ids::ENDERMAN {
        return (None, None);
    }
    let carrying = find_metadata_field(metadata, ENDERMAN_CARRY_DATA_ID)
        .filter(|(serializer, _)| *serializer == OPTIONAL_BLOCK_STATE_SERIALIZER_ID)
        .and_then(|(_, mut value)| photon_core::wire::read_varint(&mut value).ok());
    (
        carrying,
        metadata_boolean_value(metadata, ENDERMAN_CREEPY_DATA_ID),
    )
}

/// A bee's flags (`Bee.DATA_FLAGS_ID`) and the game time its anger ends
/// (`DATA_ANGER_END_TIME`), when the data holds them.
pub(crate) fn bee_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<u8>, Option<i64>) {
    if type_id != photon_data::entity_type::ids::BEE {
        return (None, None);
    }
    let anger = find_metadata_field(metadata, BEE_ANGER_END_DATA_ID)
        .filter(|(serializer, _)| *serializer == LONG_SERIALIZER_ID)
        .and_then(|(_, mut value)| {
            let end = photon_core::wire::read_varlong(&mut value).ok()?;
            (!value.is_empty()).then_some(end)
        });
    (metadata_byte_value(metadata, BEE_FLAGS_DATA_ID), anger)
}

pub(crate) fn wolf_data_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> WolfData {
    if type_id != photon_data::entity_type::ids::WOLF {
        return WolfData::default();
    }
    let anger_end = find_metadata_field(metadata, WOLF_ANGER_END_DATA_ID)
        .filter(|(serializer, _)| *serializer == LONG_SERIALIZER_ID)
        .and_then(|(_, mut value)| {
            let end = photon_core::wire::read_varlong(&mut value).ok()?;
            (!value.is_empty()).then_some(end)
        });
    WolfData {
        flags: metadata_byte_value(metadata, TAMABLE_FLAGS_DATA_ID),
        owner: metadata_owner(metadata),
        interested: metadata_boolean_value(metadata, WOLF_INTERESTED_DATA_ID),
        collar: metadata_int_value(metadata, WOLF_COLLAR_DATA_ID),
        anger_end,
    }
}

/// A nautilus's `TamableAnimal` flags, when the data holds them.
pub(crate) fn tamable_data_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> TamableData {
    if !matches!(
        photon_data::entity_type::name(type_id),
        Some("minecraft:nautilus" | "minecraft:zombie_nautilus" | "minecraft:parrot")
    ) {
        return TamableData::default();
    }
    TamableData {
        flags: metadata_byte_value(metadata, TAMABLE_FLAGS_DATA_ID),
        owner: metadata_owner(metadata),
    }
}

/// What the data of a nautilus or parrot holds of `TamableAnimal`: its flags and owner.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TamableData {
    pub(crate) flags: Option<u8>,
    pub(crate) owner: Option<Option<photon_core::binary::Uuid>>,
}

pub(crate) fn cat_data_from_metadata(type_id: EntityTypeId, metadata: &[u8]) -> CatData {
    if type_id != photon_data::entity_type::ids::CAT {
        return CatData::default();
    }
    CatData {
        flags: metadata_byte_value(metadata, TAMABLE_FLAGS_DATA_ID),
        owner: metadata_owner(metadata),
        lying: metadata_boolean_value(metadata, CAT_LYING_DATA_ID),
        relaxed: metadata_boolean_value(metadata, CAT_RELAX_DATA_ID),
        collar: metadata_int_value(metadata, CAT_COLLAR_DATA_ID),
    }
}

/// A fox's flags and variant (`Fox.DATA_FLAGS_ID`, `DATA_TYPE_ID`), each when
/// the data holds it.
pub(crate) fn fox_data_from_metadata(
    type_id: EntityTypeId,
    metadata: &[u8],
) -> (Option<u8>, Option<i32>) {
    if type_id != photon_data::entity_type::ids::FOX {
        return (None, None);
    }
    (
        metadata_byte_value(metadata, FOX_FLAGS_DATA_ID),
        metadata_int_value(metadata, FOX_VARIANT_DATA_ID),
    )
}

/// What a wolf's data holds (`TamableAnimal.DATA_FLAGS_ID`, `Wolf.DATA_INTERESTED_ID`,
/// `DATA_COLLAR_COLOR`, `DATA_ANGER_END_TIME`), each when it is there.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WolfData {
    pub(crate) flags: Option<u8>,
    pub(crate) owner: Option<Option<photon_core::binary::Uuid>>,
    pub(crate) interested: Option<bool>,
    pub(crate) collar: Option<i32>,
    pub(crate) anger_end: Option<i64>,
}

/// What a cat's data holds (`TamableAnimal.DATA_FLAGS_ID`, `Cat.IS_LYING`,
/// `RELAX_STATE_ONE`, `DATA_COLLAR_COLOR`), each when it is there.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CatData {
    pub(crate) flags: Option<u8>,
    pub(crate) owner: Option<Option<photon_core::binary::Uuid>>,
    pub(crate) lying: Option<bool>,
    pub(crate) relaxed: Option<bool>,
    pub(crate) collar: Option<i32>,
}

/// A wither's synced data, each field when the data holds it.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct WitherData {
    pub(super) targets: [Option<i32>; 2],
    pub(super) invulnerable_ticks: Option<i32>,
}

/// A creaking's synced data, each field when the data holds it.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct CreakingData {
    pub(super) can_move: Option<bool>,
    pub(super) active: Option<bool>,
    pub(super) tearing_down: Option<bool>,
    pub(super) heart_bound: Option<bool>,
}

impl WitherData {
    pub(crate) fn apply(self, state: &mut crate::mobs::wither::WitherState) {
        for (target, new) in state.targets.iter_mut().zip(self.targets) {
            *target = new.unwrap_or(*target);
        }
        state.invulnerable_ticks = self.invulnerable_ticks.unwrap_or(state.invulnerable_ticks);
    }
}

impl CreakingData {
    pub(crate) fn apply(self, state: &mut crate::mobs::creaking::CreakingState) {
        state.can_move = self.can_move.unwrap_or(state.can_move);
        state.active = self.active.unwrap_or(state.active);
        state.tearing_down = self.tearing_down.unwrap_or(state.tearing_down);
        state.heart_bound = self.heart_bound.unwrap_or(state.heart_bound);
    }
}
