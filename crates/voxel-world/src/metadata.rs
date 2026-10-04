//! `ClientWorld` metadata layer split out of the former single-file module.

use super::*;

pub(crate) fn read_effect_particle_type_ids(metadata: &mut &[u8]) -> Option<Vec<i32>> {
    let count = voxel_core::wire::read_varint(metadata).ok()?;
    let count = usize::try_from(count).ok()?;
    if count > metadata.len() {
        return None;
    }
    let mut type_ids = Vec::with_capacity(count);
    for _ in 0..count {
        let type_id = voxel_core::wire::read_varint(metadata).ok()?;
        if type_id != ENTITY_EFFECT_PARTICLE_TYPE_ID {
            return None;
        }
        *metadata = metadata.get(4..)?;
        type_ids.push(type_id);
    }
    Some(type_ids)
}

/// The entity data serializers by id, in the order `EntityDataSerializers`
/// registers them, and the bytes each one's value takes. A value this port
/// cannot walk leaves the fields after it unread: a particle with data of its
/// own, a resolvable profile and a painting variant sent whole.
pub(crate) fn skip_metadata_value(metadata: &mut &[u8], serializer_id: i32) -> Option<()> {
    let varint = |metadata: &mut &[u8]| voxel_core::wire::read_varint(metadata).ok().map(drop);
    let bytes = |metadata: &mut &[u8], count: usize| {
        *metadata = metadata.get(count..)?;
        Some(())
    };
    match serializer_id {
        // Byte, boolean.
        ENTITY_DATA_SERIALIZER_BYTE_ID | ENTITY_DATA_SERIALIZER_BOOLEAN_ID => bytes(metadata, 1)?,
        // Int, block state, optional block state, optional unsigned int,
        // direction, pose, humanoid arm, and the enums and registry holders:
        // each a var int.
        ENTITY_DATA_SERIALIZER_INT_ID
        | ENTITY_DATA_SERIALIZER_POSE_ID
        | 12
        | 14
        | 15
        | 19
        | 21..=32
        | 35..=38
        | 42 => varint(metadata)?,
        2 => {
            voxel_core::wire::read_varlong(metadata).ok()?;
        }
        ENTITY_DATA_SERIALIZER_FLOAT_ID => bytes(metadata, 4)?,
        4 => {
            let length = usize::try_from(voxel_core::wire::read_varint(metadata).ok()?).ok()?;
            bytes(metadata, length)?;
        }
        5 => {
            voxel_core::nbt::read_any_nbt(metadata).ok()?.as_ref()?;
        }
        ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                let component = voxel_core::nbt::read_any_nbt(metadata).ok()?;
                component.as_ref()?;
            }
        }
        // Item stack (`ItemStack.OPTIONAL_STREAM_CODEC`).
        7 => {
            if voxel_core::wire::read_varint(metadata).ok()? > 0 {
                varint(metadata)?;
                crate::item_components::read_patch(metadata).ok()?;
            }
        }
        // Rotations, vector3.
        9 | 39 => bytes(metadata, 12)?,
        // Block position.
        10 => bytes(metadata, 8)?,
        ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                voxel_core::binary::read_blockpos(metadata).ok()?;
            }
        }
        // Optional living entity reference: a flag and a UUID.
        13 => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                bytes(metadata, 16)?;
            }
        }
        // One particle, which only the entity effect's colour is known for.
        16 => {
            if voxel_core::wire::read_varint(metadata).ok()? != ENTITY_EFFECT_PARTICLE_TYPE_ID {
                return None;
            }
            bytes(metadata, 4)?;
        }
        ENTITY_DATA_SERIALIZER_PARTICLES_ID => {
            read_effect_particle_type_ids(metadata)?;
        }
        // Villager data: type, profession, level.
        18 => {
            varint(metadata)?;
            varint(metadata)?;
            varint(metadata)?;
        }
        // Optional global position: a flag, a dimension, a block position.
        33 => {
            let (&present, rest) = metadata.split_first()?;
            *metadata = rest;
            if present != 0 {
                let length = usize::try_from(voxel_core::wire::read_varint(metadata).ok()?).ok()?;
                bytes(metadata, length)?;
                bytes(metadata, 8)?;
            }
        }
        // Painting variant: a registry holder, or the variant itself.
        34 => {
            if voxel_core::wire::read_varint(metadata).ok()? == 0 {
                return None;
            }
        }
        // Quaternion.
        40 => bytes(metadata, 16)?,
        _ => return None,
    }
    Some(())
}

/// The registry id of the variant a pig, cow, frog or other of this type wears, if the
/// data holds one (the default is not sent).
pub(crate) fn variant_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    let (data_id, serializer) = match voxel_data::entity_type::name(type_id)? {
        "minecraft:pig" => (PIG_VARIANT_DATA_ID, PIG_VARIANT_SERIALIZER_ID),
        "minecraft:cow" => (COW_VARIANT_DATA_ID, COW_VARIANT_SERIALIZER_ID),
        "minecraft:chicken" => (CHICKEN_VARIANT_DATA_ID, CHICKEN_VARIANT_SERIALIZER_ID),
        "minecraft:wolf" => (WOLF_VARIANT_DATA_ID, WOLF_VARIANT_SERIALIZER_ID),
        "minecraft:cat" => (CAT_VARIANT_DATA_ID, CAT_VARIANT_SERIALIZER_ID),
        "minecraft:frog" => (FROG_VARIANT_DATA_ID, FROG_VARIANT_SERIALIZER_ID),
        "minecraft:zombie_nautilus" => (
            ZOMBIE_NAUTILUS_VARIANT_DATA_ID,
            ZOMBIE_NAUTILUS_VARIANT_SERIALIZER_ID,
        ),
        _ => return None,
    };
    let (found, mut value) = find_metadata_field(metadata, data_id)?;
    if found != serializer {
        return None;
    }
    voxel_core::wire::read_varint(&mut value).ok()
}

/// A piglin's baby and dancing flags (`Piglin.DATA_BABY_ID`,
/// `DATA_IS_DANCING`), when the data holds them.
pub(crate) fn piglin_flags_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:piglin") {
        return (None, None);
    }
    (
        metadata_boolean_value(metadata, PIGLIN_BABY_DATA_ID),
        metadata_boolean_value(metadata, PIGLIN_DANCING_DATA_ID),
    )
}

/// Whether an avatar's main arm is the left (`Avatar.DATA_PLAYER_MAIN_HAND`),
/// when the data holds it.
pub(crate) fn avatar_main_arm_is_left(metadata: &[u8]) -> Option<bool> {
    let (serializer_id, mut value_input) = find_metadata_field(metadata, AVATAR_MAIN_HAND_DATA_ID)?;
    if serializer_id != HUMANOID_ARM_SERIALIZER_ID {
        return None;
    }
    let id = voxel_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(id == 0)
}

/// A sheep's wool byte (`Sheep.DATA_WOOL_ID`), when the data holds it.
pub(crate) fn sheep_wool_from_metadata(type_id: i32, metadata: &[u8]) -> Option<u8> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:sheep" {
        return None;
    }
    metadata_byte_value(metadata, SHEEP_WOOL_DATA_ID)
}

/// A creeper's fuse direction and ignited flag (`Creeper.DATA_SWELL_DIR`,
/// `DATA_IS_IGNITED`), when the data holds them.
pub(crate) fn creeper_fuse_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<i32>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:creeper") {
        return (None, None);
    }
    (
        metadata_int_value(metadata, CREEPER_SWELL_DIR_DATA_ID),
        metadata_boolean_value(metadata, CREEPER_IGNITED_DATA_ID),
    )
}

/// Whether a creeper is charged (`Creeper.DATA_IS_POWERED`), when the data
/// holds it.
pub(crate) fn creeper_powered_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:creeper" {
        return None;
    }
    metadata_boolean_value(metadata, CREEPER_POWERED_DATA_ID)
}

/// A villager's or zombie villager's type, profession and level
/// (`Villager.DATA_VILLAGER_DATA`), when the data holds them.
pub(crate) fn villager_data_from_metadata(type_id: i32, metadata: &[u8]) -> Option<[i32; 3]> {
    let data_id = match voxel_data::entity_type::name(type_id)? {
        "minecraft:villager" => VILLAGER_DATA_DATA_ID,
        "minecraft:zombie_villager" => ZOMBIE_VILLAGER_DATA_DATA_ID,
        _ => return None,
    };
    let (serializer, mut value) = find_metadata_field(metadata, data_id)?;
    if serializer != VILLAGER_DATA_SERIALIZER_ID {
        return None;
    }
    let mut read = || voxel_core::wire::read_varint(&mut value).ok();
    let data = [read()?, read()?, read()?];
    // The list's terminator must follow.
    (!value.is_empty()).then_some(data)
}

/// A villager's or wandering trader's unhappy counter
/// (`AbstractVillager.DATA_UNHAPPY_COUNTER`), when the data holds it.
pub(crate) fn unhappy_counter_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    matches!(
        voxel_data::entity_type::name(type_id)?,
        "minecraft:villager" | "minecraft:wandering_trader"
    )
    .then(|| metadata_int_value(metadata, VILLAGER_UNHAPPY_DATA_ID))?
}

/// A raider's celebrating flag (`Raider.IS_CELEBRATING`) and a spellcaster's
/// casting byte (`SpellcasterIllager.DATA_SPELL_CASTING_ID`, above zero while
/// casting), when the data holds them.
pub(crate) fn illager_flags_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>) {
    let Some(name) = voxel_data::entity_type::name(type_id) else {
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
    type_id: i32,
    metadata: &[u8],
) -> (Option<i32>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:enderman") {
        return (None, None);
    }
    let carrying = find_metadata_field(metadata, ENDERMAN_CARRY_DATA_ID)
        .filter(|(serializer, _)| *serializer == OPTIONAL_BLOCK_STATE_SERIALIZER_ID)
        .and_then(|(_, mut value)| voxel_core::wire::read_varint(&mut value).ok());
    (
        carrying,
        metadata_boolean_value(metadata, ENDERMAN_CREEPY_DATA_ID),
    )
}

/// Whether a goat has its left and its right horn (`Goat.DATA_HAS_LEFT_HORN`,
/// `DATA_HAS_RIGHT_HORN`), when the data holds them.
pub(crate) fn goat_horns_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:goat") {
        return (None, None);
    }
    (
        metadata_boolean_value(metadata, GOAT_LEFT_HORN_DATA_ID),
        metadata_boolean_value(metadata, GOAT_RIGHT_HORN_DATA_ID),
    )
}

/// An abstract horse's flags (`AbstractHorse.DATA_ID_FLAGS`), a horse's
/// variant data (`Horse.DATA_ID_TYPE_VARIANT`) and a chested horse's chest
/// (`AbstractChestedHorse.DATA_ID_CHEST`), when the data holds them.
pub(crate) fn horse_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<u8>, Option<i32>, Option<bool>) {
    let Some(name) = voxel_data::entity_type::name(type_id) else {
        return (None, None, None);
    };
    let horselike = matches!(
        name,
        "minecraft:horse"
            | "minecraft:donkey"
            | "minecraft:mule"
            | "minecraft:skeleton_horse"
            | "minecraft:zombie_horse"
            | "minecraft:llama"
            | "minecraft:trader_llama"
            | "minecraft:camel"
            | "minecraft:camel_husk"
    );
    if !horselike {
        return (None, None, None);
    }
    let variant = match name {
        "minecraft:horse" => metadata_int_value(metadata, HORSE_VARIANT_DATA_ID),
        "minecraft:llama" | "minecraft:trader_llama" => {
            metadata_int_value(metadata, LLAMA_VARIANT_DATA_ID)
        }
        _ => None,
    };
    let chest = matches!(
        name,
        "minecraft:donkey" | "minecraft:mule" | "minecraft:llama" | "minecraft:trader_llama"
    )
    .then(|| metadata_boolean_value(metadata, HORSE_CHEST_DATA_ID))
    .flatten();
    (
        metadata_byte_value(metadata, HORSE_FLAGS_DATA_ID),
        variant,
        chest,
    )
}

/// A camel's dash flag (`Camel.DASH`) and the tick its pose last changed
/// (`LAST_POSE_CHANGE_TICK`), when the data holds them.
pub(crate) fn camel_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<i64>) {
    if !matches!(
        voxel_data::entity_type::name(type_id),
        Some("minecraft:camel" | "minecraft:camel_husk")
    ) {
        return (None, None);
    }
    let last_pose = find_metadata_field(metadata, CAMEL_LAST_POSE_CHANGE_DATA_ID)
        .filter(|(serializer, _)| *serializer == LONG_SERIALIZER_ID)
        .and_then(|(_, mut value)| {
            let tick = voxel_core::wire::read_varlong(&mut value).ok()?;
            (!value.is_empty()).then_some(tick)
        });
    (
        metadata_boolean_value(metadata, CAMEL_DASH_DATA_ID),
        last_pose,
    )
}

/// A bee's flags (`Bee.DATA_FLAGS_ID`) and the game time its anger ends
/// (`DATA_ANGER_END_TIME`), when the data holds them.
pub(crate) fn bee_data_from_metadata(type_id: i32, metadata: &[u8]) -> (Option<u8>, Option<i64>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:bee") {
        return (None, None);
    }
    let anger = find_metadata_field(metadata, BEE_ANGER_END_DATA_ID)
        .filter(|(serializer, _)| *serializer == LONG_SERIALIZER_ID)
        .and_then(|(_, mut value)| {
            let end = voxel_core::wire::read_varlong(&mut value).ok()?;
            (!value.is_empty()).then_some(end)
        });
    (metadata_byte_value(metadata, BEE_FLAGS_DATA_ID), anger)
}

/// What a wolf's data holds (`TamableAnimal.DATA_FLAGS_ID`, `Wolf.DATA_INTERESTED_ID`,
/// `DATA_COLLAR_COLOR`, `DATA_ANGER_END_TIME`), each when it is there.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WolfData {
    pub(crate) flags: Option<u8>,
    pub(crate) interested: Option<bool>,
    pub(crate) collar: Option<i32>,
    pub(crate) anger_end: Option<i64>,
}

pub(crate) fn wolf_data_from_metadata(type_id: i32, metadata: &[u8]) -> WolfData {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:wolf") {
        return WolfData::default();
    }
    let anger_end = find_metadata_field(metadata, WOLF_ANGER_END_DATA_ID)
        .filter(|(serializer, _)| *serializer == LONG_SERIALIZER_ID)
        .and_then(|(_, mut value)| {
            let end = voxel_core::wire::read_varlong(&mut value).ok()?;
            (!value.is_empty()).then_some(end)
        });
    WolfData {
        flags: metadata_byte_value(metadata, TAMABLE_FLAGS_DATA_ID),
        interested: metadata_boolean_value(metadata, WOLF_INTERESTED_DATA_ID),
        collar: metadata_int_value(metadata, WOLF_COLLAR_DATA_ID),
        anger_end,
    }
}

/// What a cat's data holds (`TamableAnimal.DATA_FLAGS_ID`, `Cat.IS_LYING`,
/// `RELAX_STATE_ONE`, `DATA_COLLAR_COLOR`), each when it is there.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CatData {
    pub(crate) flags: Option<u8>,
    pub(crate) lying: Option<bool>,
    pub(crate) relaxed: Option<bool>,
    pub(crate) collar: Option<i32>,
}

pub(crate) fn cat_data_from_metadata(type_id: i32, metadata: &[u8]) -> CatData {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:cat") {
        return CatData::default();
    }
    CatData {
        flags: metadata_byte_value(metadata, TAMABLE_FLAGS_DATA_ID),
        lying: metadata_boolean_value(metadata, CAT_LYING_DATA_ID),
        relaxed: metadata_boolean_value(metadata, CAT_RELAX_DATA_ID),
        collar: metadata_int_value(metadata, CAT_COLLAR_DATA_ID),
    }
}

/// A fox's flags and variant (`Fox.DATA_FLAGS_ID`, `DATA_TYPE_ID`), each when
/// the data holds it.
pub(crate) fn fox_data_from_metadata(type_id: i32, metadata: &[u8]) -> (Option<u8>, Option<i32>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:fox") {
        return (None, None);
    }
    (
        metadata_byte_value(metadata, FOX_FLAGS_DATA_ID),
        metadata_int_value(metadata, FOX_VARIANT_DATA_ID),
    )
}

/// A strider's suffocating flag (`Strider.DATA_SUFFOCATING`), when the data
/// holds it.
pub(crate) fn strider_suffocating_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:strider" {
        return None;
    }
    metadata_boolean_value(metadata, STRIDER_SUFFOCATING_DATA_ID)
}

/// A copper golem's action and weathering (`COPPER_GOLEM_STATE`,
/// `DATA_WEATHER_STATE`: each a varint id), each when the data holds it.
pub(crate) fn copper_golem_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> Option<(Option<i32>, Option<i32>)> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:copper_golem" {
        return None;
    }
    let varint = |data_id: u8, serializer: i32| {
        let (serializer_id, mut value_input) = find_metadata_field(metadata, data_id)?;
        if serializer_id != serializer {
            return None;
        }
        let value = voxel_core::wire::read_varint(&mut value_input).ok()?;
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
pub(crate) fn glow_squid_dark_ticks_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:glow_squid" {
        return None;
    }
    metadata_int_value(metadata, GLOW_SQUID_DARK_TICKS_DATA_ID)
}

/// Whether a ghast is charging a fireball (`Ghast.DATA_IS_CHARGING`), when the
/// data holds it.
pub(crate) fn ghast_charging_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:ghast" {
        return None;
    }
    metadata_boolean_value(metadata, GHAST_CHARGING_DATA_ID)
}

/// A bat's flags (`Bat.DATA_ID_FLAGS`: resting), a vex's (`Vex.DATA_FLAGS_ID`:
/// charging) and whether an allay dances (`Allay.DATA_DANCING`), each when the
/// data holds it.
pub(crate) fn flier_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<u8>, Option<u8>, Option<bool>) {
    match voxel_data::entity_type::name(type_id) {
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

/// A phantom's size (`Phantom.ID_SIZE`), when the data holds it.
pub(crate) fn phantom_size_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:phantom" {
        return None;
    }
    metadata_int_value(metadata, PHANTOM_SIZE_DATA_ID)
}

/// Whether a guardian moves and the id of what its beam aims at
/// (`Guardian.DATA_ID_MOVING`, `DATA_ID_ATTACK_TARGET`), each when the data holds it.
pub(crate) fn guardian_data_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<i32>) {
    if !matches!(
        voxel_data::entity_type::name(type_id),
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
pub(crate) fn fish_variant_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    let data_id = match voxel_data::entity_type::name(type_id)? {
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
    type_id: i32,
    metadata: &[u8],
) -> (Option<i32>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:axolotl") {
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
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:turtle") {
        return (None, None);
    }
    (
        metadata_boolean_value(metadata, TURTLE_HAS_EGG_DATA_ID),
        metadata_boolean_value(metadata, TURTLE_LAYING_EGG_DATA_ID),
    )
}

/// A hoglin's immunity to zombification (`Hoglin.DATA_IMMUNE_TO_ZOMBIFICATION`),
/// when the data holds it.
pub(crate) fn hoglin_immune_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:hoglin" {
        return None;
    }
    metadata_boolean_value(metadata, HOGLIN_IMMUNE_DATA_ID)
}

/// What a panda's data holds, each when it is there.
pub(crate) fn panda_data_from_metadata(type_id: i32, metadata: &[u8]) -> crate::panda::PandaData {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:panda") {
        return crate::panda::PandaData::default();
    }
    crate::panda::PandaData {
        unhappy_counter: metadata_int_value(metadata, PANDA_UNHAPPY_DATA_ID),
        sneeze_counter: metadata_int_value(metadata, PANDA_SNEEZE_DATA_ID),
        eat_counter: metadata_int_value(metadata, PANDA_EAT_DATA_ID),
        main_gene: metadata_byte_value(metadata, PANDA_MAIN_GENE_DATA_ID),
        hidden_gene: metadata_byte_value(metadata, PANDA_HIDDEN_GENE_DATA_ID),
        flags: metadata_byte_value(metadata, PANDA_FLAGS_DATA_ID),
    }
}

/// A polar bear's standing flag (`PolarBear.DATA_STANDING_ID`), when the data holds it.
pub(crate) fn polar_bear_standing_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:polar_bear" {
        return None;
    }
    metadata_boolean_value(metadata, POLAR_BEAR_STANDING_DATA_ID)
}

/// A rabbit's variant (`Rabbit.DATA_TYPE_ID`), when the data holds it.
pub(crate) fn rabbit_type_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:rabbit" {
        return None;
    }
    metadata_int_value(metadata, RABBIT_TYPE_DATA_ID)
}

/// A mooshroom's variant (`MushroomCow.DATA_TYPE`), when the data holds it.
pub(crate) fn mooshroom_type_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:mooshroom" {
        return None;
    }
    metadata_int_value(metadata, MOOSHROOM_TYPE_DATA_ID)
}

/// Whether a snow golem wears its pumpkin (`SnowGolem.DATA_PUMPKIN_ID`, bit
/// sixteen), when the data holds it.
pub(crate) fn snow_golem_pumpkin_from_metadata(type_id: i32, metadata: &[u8]) -> Option<bool> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:snow_golem" {
        return None;
    }
    metadata_byte_value(metadata, SNOW_GOLEM_PUMPKIN_DATA_ID).map(|flags| flags & 16 != 0)
}

/// An experience orb's value (`ExperienceOrb.DATA_VALUE`).
pub(crate) fn orb_value_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    if voxel_data::entity_type::name(type_id)? != "minecraft:experience_orb" {
        return None;
    }
    metadata_int_value(metadata, EXPERIENCE_ORB_VALUE_DATA_ID)
}

/// Position the cursor at one entity-metadata field value.
///
/// The reference writes `data_id`, serializer id and value per field and ends
/// the list with `255`. This walks that layout once and hands the caller the
/// serializer id the reference recorded plus the bytes starting at the value,
/// so each typed reader below keeps its own trailing-byte rule.
pub fn find_metadata_field(metadata: &[u8], target_data_id: u8) -> Option<(i32, &[u8])> {
    if metadata.last() != Some(&255) {
        return None;
    }
    let mut metadata = metadata;
    loop {
        let (&data_id, rest) = metadata.split_first()?;
        if data_id == 255 {
            return None;
        }
        metadata = rest;
        let serializer_id = voxel_core::wire::read_varint(&mut metadata).ok()?;
        if data_id == target_data_id {
            return Some((serializer_id, metadata));
        }
        skip_metadata_value(&mut metadata, serializer_id)?;
    }
}

pub(crate) fn metadata_byte_value(metadata: &[u8], target_data_id: u8) -> Option<u8> {
    let (serializer_id, value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_BYTE_ID {
        return None;
    }
    let (&value, rest) = value_input.split_first()?;
    if rest.is_empty() {
        return None;
    }
    Some(value)
}

pub(crate) fn metadata_boolean_value(metadata: &[u8], target_data_id: u8) -> Option<bool> {
    let (serializer_id, value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_BOOLEAN_ID {
        return None;
    }
    let (&value, rest) = value_input.split_first()?;
    if rest.is_empty() {
        return None;
    }
    Some(value != 0)
}

pub(crate) fn metadata_int_value(metadata: &[u8], target_data_id: u8) -> Option<i32> {
    let (serializer_id, mut value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_INT_ID {
        return None;
    }
    let value = voxel_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(value)
}

pub(crate) fn metadata_float_value(metadata: &[u8], target_data_id: u8) -> Option<f32> {
    let (serializer_id, value_input) = find_metadata_field(metadata, target_data_id)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_FLOAT_ID {
        return None;
    }
    let value = u32::from_be_bytes(value_input.get(..4)?.try_into().ok()?);
    if value_input.get(4..)?.is_empty() {
        return None;
    }
    Some(f32::from_bits(value))
}

pub(crate) fn metadata_pose_value(metadata: &[u8]) -> Option<ClientEntityPose> {
    let (serializer_id, mut value_input) = find_metadata_field(metadata, ENTITY_POSE_DATA_ID)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_POSE_ID {
        return None;
    }
    let value = voxel_core::wire::read_varint(&mut value_input).ok()?;
    if value_input.is_empty() {
        return None;
    }
    Some(ClientEntityPose::from_wire_id(value))
}

/// `Avatar.DATA_PLAYER_MODE_CUSTOMISATION`: which skin layers show.
pub(crate) const PLAYER_MODE_CUSTOMISATION_DATA_ID: u8 = 16;
/// `Player.DATA_PLAYER_ABSORPTION_ID`.
pub(crate) const PLAYER_ABSORPTION_DATA_ID: u8 = 17;
/// `Player.DATA_SCORE_ID`: what the death screen shows.
pub(crate) const PLAYER_SCORE_DATA_ID: u8 = 18;

pub(crate) fn player_absorption_from_metadata(metadata: &[u8]) -> Option<f32> {
    metadata_float_value(metadata, PLAYER_ABSORPTION_DATA_ID)
}

pub(crate) fn player_model_customisation_from_metadata(metadata: &[u8]) -> Option<u8> {
    metadata_byte_value(metadata, PLAYER_MODE_CUSTOMISATION_DATA_ID)
}

pub(crate) fn player_score_from_metadata(metadata: &[u8]) -> Option<i32> {
    metadata_int_value(metadata, PLAYER_SCORE_DATA_ID)
}

pub(crate) fn shared_flags_from_metadata(metadata: &[u8]) -> Option<u8> {
    metadata_byte_value(metadata, ENTITY_SHARED_FLAGS_DATA_ID)
}

pub(crate) fn living_flags_from_metadata(metadata: &[u8]) -> Option<u8> {
    metadata_byte_value(metadata, ENTITY_LIVING_FLAGS_DATA_ID)
}

pub(crate) fn mob_flags_from_metadata(metadata: &[u8]) -> Option<u8> {
    metadata_byte_value(metadata, ENTITY_MOB_FLAGS_DATA_ID)
}

pub(crate) fn air_supply_from_metadata(metadata: &[u8]) -> Option<i32> {
    metadata_int_value(metadata, ENTITY_AIR_SUPPLY_DATA_ID)
}

pub(crate) fn health_from_metadata(metadata: &[u8]) -> Option<f32> {
    metadata_float_value(metadata, ENTITY_HEALTH_DATA_ID)
}

pub(crate) fn living_count_from_metadata(metadata: &[u8], target_data_id: u8) -> Option<i32> {
    metadata_int_value(metadata, target_data_id)
}

pub(crate) fn arrow_count_from_metadata(metadata: &[u8]) -> Option<i32> {
    living_count_from_metadata(metadata, ENTITY_ARROW_COUNT_DATA_ID)
}

pub(crate) fn stinger_count_from_metadata(metadata: &[u8]) -> Option<i32> {
    living_count_from_metadata(metadata, ENTITY_STINGER_COUNT_DATA_ID)
}

pub(crate) fn effect_ambience_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, ENTITY_EFFECT_AMBIENCE_DATA_ID)
}

pub(crate) fn zombie_baby_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, VANILLA_ZOMBIE_BABY_DATA_ID)
}

pub(crate) fn ticks_frozen_from_metadata(metadata: &[u8]) -> Option<i32> {
    metadata_int_value(metadata, ENTITY_TICKS_FROZEN_DATA_ID)
}

pub(crate) fn no_gravity_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, ENTITY_NO_GRAVITY_DATA_ID)
}

pub(crate) fn silent_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, ENTITY_SILENT_DATA_ID)
}

pub(crate) fn custom_name_visible_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID)
}

pub(crate) fn effect_particle_type_ids_from_metadata(metadata: &[u8]) -> Option<Vec<i32>> {
    let (serializer_id, mut value_input) =
        find_metadata_field(metadata, ENTITY_EFFECT_PARTICLES_DATA_ID)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_PARTICLES_ID {
        return None;
    }
    read_effect_particle_type_ids(&mut value_input)
}

pub(crate) fn sleeping_pos_from_metadata(
    metadata: &[u8],
) -> Option<Option<voxel_core::binary::BlockPos>> {
    let (serializer_id, value_input) = find_metadata_field(metadata, ENTITY_SLEEPING_POS_DATA_ID)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID {
        return None;
    }
    let (&present, mut rest) = value_input.split_first()?;
    if present == 0 {
        return Some(None);
    }
    let value = voxel_core::binary::read_blockpos(&mut rest).ok()?;
    if rest.is_empty() {
        return None;
    }
    Some(Some(value))
}

pub(crate) fn custom_name_present_from_metadata(metadata: &[u8]) -> Option<bool> {
    let (serializer_id, value_input) = find_metadata_field(metadata, ENTITY_CUSTOM_NAME_DATA_ID)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID {
        return None;
    }
    let (&present, mut rest) = value_input.split_first()?;
    if present == 0 {
        return Some(false);
    }
    let component = voxel_core::nbt::read_any_nbt(&mut rest).ok()?;
    Some(component.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Entity data as the wire writes it: fields of `(data id, serializer id,
    /// value bytes)`, ended by 255.
    fn data(fields: &[(u8, i32, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (id, serializer, value) in fields {
            out.push(*id);
            voxel_core::wire::write_varint(*serializer, &mut out);
            out.extend_from_slice(value);
        }
        out.push(255);
        out
    }

    #[test]
    fn a_wolfs_flags_collar_and_anger_are_read_and_its_variant_is_a_registry_id() {
        let wolf = voxel_data::entity_type::id("minecraft:wolf").unwrap();
        let pig = voxel_data::entity_type::id("minecraft:pig").unwrap();
        let metadata = data(&[
            (TAMABLE_FLAGS_DATA_ID, 0, &[5]),
            (WOLF_INTERESTED_DATA_ID, 8, &[1]),
            (WOLF_COLLAR_DATA_ID, 1, &[3]),
            (WOLF_ANGER_END_DATA_ID, 2, &[0x90, 0x4e]),
            (WOLF_VARIANT_DATA_ID, WOLF_VARIANT_SERIALIZER_ID, &[4]),
        ]);
        let read = wolf_data_from_metadata(wolf, &metadata);
        assert_eq!(read.flags, Some(5));
        assert_eq!(read.interested, Some(true));
        assert_eq!(read.collar, Some(3));
        assert_eq!(read.anger_end, Some(10_000));
        assert_eq!(variant_from_metadata(wolf, &metadata), Some(4));
        assert_eq!(wolf_data_from_metadata(pig, &metadata), WolfData::default());
    }

    #[test]
    fn a_cats_flags_lying_relax_and_collar_are_read_and_its_variant_is_a_registry_id() {
        let cat = voxel_data::entity_type::id("minecraft:cat").unwrap();
        let wolf = voxel_data::entity_type::id("minecraft:wolf").unwrap();
        let metadata = data(&[
            (TAMABLE_FLAGS_DATA_ID, 0, &[5]),
            (CAT_VARIANT_DATA_ID, CAT_VARIANT_SERIALIZER_ID, &[7]),
            (CAT_LYING_DATA_ID, 8, &[1]),
            (CAT_RELAX_DATA_ID, 8, &[0]),
            (CAT_COLLAR_DATA_ID, 1, &[3]),
        ]);
        let read = cat_data_from_metadata(cat, &metadata);
        assert_eq!(read.flags, Some(5));
        assert_eq!(read.lying, Some(true));
        assert_eq!(read.relaxed, Some(false));
        assert_eq!(read.collar, Some(3));
        assert_eq!(variant_from_metadata(cat, &metadata), Some(7));
        assert_eq!(cat_data_from_metadata(wolf, &metadata), CatData::default());
    }

    #[test]
    fn a_villagers_data_and_unhappy_counter_are_read_from_their_fields() {
        let villager = voxel_data::entity_type::id("minecraft:villager").expect("a villager");
        let cow = voxel_data::entity_type::id("minecraft:cow").expect("a cow");
        let metadata = data(&[
            (VILLAGER_UNHAPPY_DATA_ID, 1, &[40]),
            (
                VILLAGER_DATA_DATA_ID,
                VILLAGER_DATA_SERIALIZER_ID,
                &[4, 5, 3],
            ),
        ]);
        assert_eq!(
            villager_data_from_metadata(villager, &metadata),
            Some([4, 5, 3])
        );
        assert_eq!(unhappy_counter_from_metadata(villager, &metadata), Some(40));
        assert_eq!(villager_data_from_metadata(cow, &metadata), None);
        assert_eq!(unhappy_counter_from_metadata(cow, &metadata), None);
    }

    #[test]
    fn an_endermans_block_and_a_zombie_villagers_data_are_read_from_their_fields() {
        let id = |name| voxel_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (
                ENDERMAN_CARRY_DATA_ID,
                OPTIONAL_BLOCK_STATE_SERIALIZER_ID,
                &[0x81, 0x01],
            ),
            (
                ENDERMAN_CREEPY_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
            (
                ZOMBIE_VILLAGER_DATA_DATA_ID,
                VILLAGER_DATA_SERIALIZER_ID,
                &[1, 2, 3],
            ),
        ]);
        assert_eq!(
            enderman_flags_from_metadata(id("minecraft:enderman"), &metadata),
            (Some(129), Some(true))
        );
        let empty = data(&[(
            ENDERMAN_CARRY_DATA_ID,
            OPTIONAL_BLOCK_STATE_SERIALIZER_ID,
            &[0],
        )]);
        assert_eq!(
            enderman_flags_from_metadata(id("minecraft:enderman"), &empty),
            (Some(0), None)
        );
        let pumpkin = data(&[(
            SNOW_GOLEM_PUMPKIN_DATA_ID,
            ENTITY_DATA_SERIALIZER_BYTE_ID,
            &[16],
        )]);
        let bare = data(&[(
            SNOW_GOLEM_PUMPKIN_DATA_ID,
            ENTITY_DATA_SERIALIZER_BYTE_ID,
            &[0],
        )]);
        let golem = id("minecraft:snow_golem");
        assert_eq!(
            snow_golem_pumpkin_from_metadata(golem, &pumpkin),
            Some(true)
        );
        assert_eq!(snow_golem_pumpkin_from_metadata(golem, &bare), Some(false));
        assert_eq!(
            snow_golem_pumpkin_from_metadata(id("minecraft:cow"), &pumpkin),
            None
        );
        assert_eq!(
            villager_data_from_metadata(id("minecraft:zombie_villager"), &metadata),
            Some([1, 2, 3])
        );
    }

    #[test]
    fn a_bees_flags_and_anger_are_read_from_their_fields() {
        let id = |name| voxel_data::entity_type::id(name).expect("a type");
        let mut anger = Vec::new();
        voxel_core::wire::write_varlong(1_234_567_890_123, &mut anger);
        let metadata = data(&[
            (BEE_FLAGS_DATA_ID, ENTITY_DATA_SERIALIZER_BYTE_ID, &[12]),
            (BEE_ANGER_END_DATA_ID, LONG_SERIALIZER_ID, &anger),
        ]);
        assert_eq!(
            bee_data_from_metadata(id("minecraft:bee"), &metadata),
            (Some(12), Some(1_234_567_890_123))
        );
        assert_eq!(
            bee_data_from_metadata(id("minecraft:cow"), &metadata),
            (None, None)
        );
    }

    #[test]
    fn a_goats_horns_and_a_mooshrooms_variant_are_read_from_their_fields() {
        let id = |name| voxel_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (MOOSHROOM_TYPE_DATA_ID, ENTITY_DATA_SERIALIZER_INT_ID, &[1]),
            (
                GOAT_LEFT_HORN_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[0],
            ),
            (
                GOAT_RIGHT_HORN_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
        ]);
        assert_eq!(
            goat_horns_from_metadata(id("minecraft:goat"), &metadata),
            (Some(false), Some(true))
        );
        assert_eq!(
            mooshroom_type_from_metadata(id("minecraft:mooshroom"), &metadata),
            Some(1)
        );
        assert_eq!(
            mooshroom_type_from_metadata(id("minecraft:cow"), &metadata),
            None
        );
    }

    #[test]
    fn an_illagers_celebrating_and_casting_are_read_from_their_fields() {
        let id = |name| voxel_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (
                RAIDER_CELEBRATING_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
            (
                SPELLCASTER_CASTING_DATA_ID,
                ENTITY_DATA_SERIALIZER_BYTE_ID,
                &[2],
            ),
        ]);
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:evoker"), &metadata),
            (Some(true), Some(true))
        );
        // A vindicator does not cast, and a cow is no raider.
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:vindicator"), &metadata),
            (Some(true), None)
        );
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:cow"), &metadata),
            (None, None)
        );
    }

    #[test]
    fn the_serializer_ids_are_the_names_this_walk_assumes() {
        let name = |id| voxel_data::entity_type::data_serializer(id).expect("a serializer");
        assert_eq!(name(ENTITY_DATA_SERIALIZER_PARTICLES_ID), "PARTICLES");
        assert_eq!(name(PIG_VARIANT_SERIALIZER_ID), "PIG_VARIANT");
        assert_eq!(name(COW_VARIANT_SERIALIZER_ID), "COW_VARIANT");
        // Registry holders and enums are one var int each.
        for id in 21..=32 {
            assert!(name(id).ends_with("_VARIANT"), "{id}");
        }
        for id in 35..=38 {
            assert!(name(id).contains("STATE"), "{id}");
        }
        assert_eq!(name(42), "HUMANOID_ARM");
        assert_eq!(name(7), "ITEM_STACK");
        assert_eq!(name(41), "RESOLVABLE_PROFILE");
    }

    #[test]
    fn every_serializer_the_walk_knows_is_stepped_over() {
        // The value of each serializer, so the field after it is still found.
        let values: [(i32, &[u8]); 41] = [
            (0, &[7]),
            (1, &[0x81, 0x01]),
            (2, &[0xFF, 0xFF, 0x03]),
            (3, &[0, 0, 0x80, 0x3F]),
            (4, &[2, b'h', b'i']),
            (5, &[8, 0, 2, b'h', b'i']),
            (6, &[0]),
            (7, &[0]),
            (8, &[1]),
            (9, &[0; 12]),
            (10, &[0; 8]),
            (11, &[0]),
            (12, &[3]),
            (
                13,
                &[1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            ),
            (14, &[0x81, 0x01]),
            (15, &[0]),
            (17, &[0]),
            (18, &[1, 2, 3]),
            (19, &[0]),
            (20, &[5]),
            (21, &[1]),
            (22, &[1]),
            (23, &[1]),
            (24, &[1]),
            (25, &[1]),
            (26, &[1]),
            (27, &[1]),
            (28, &[1]),
            (29, &[1]),
            (30, &[1]),
            (31, &[1]),
            (32, &[1]),
            (33, &[1, 3, b'a', b'b', b'c', 0, 0, 0, 0, 0, 0, 0, 0]),
            (34, &[4]),
            (35, &[1]),
            (36, &[1]),
            (37, &[1]),
            (38, &[1]),
            (39, &[0; 12]),
            (40, &[0; 16]),
            (42, &[1]),
        ];
        for (serializer, value) in values {
            let metadata = data(&[(1, serializer, value), (2, 8, &[1])]);
            assert_eq!(
                metadata_boolean_value(&metadata, 2),
                Some(true),
                "serializer {serializer}"
            );
        }
        // A particle with data of its own, a profile and a painting sent
        // whole are not walked, and the fields after them are not read.
        for serializer in [16, 41] {
            let metadata = data(&[(1, serializer, &[1, 1]), (2, 8, &[1])]);
            assert_eq!(metadata_boolean_value(&metadata, 2), None, "{serializer}");
        }
        let direct_painting = data(&[(1, 34, &[0, 1, 1]), (2, 8, &[1])]);
        assert_eq!(metadata_boolean_value(&direct_painting, 2), None);
    }

    #[test]
    fn a_pigs_and_a_cows_variant_is_read_after_the_fields_before_it() {
        let pig = voxel_data::entity_type::id("minecraft:pig").expect("pig");
        let cow = voxel_data::entity_type::id("minecraft:cow").expect("cow");
        let zombie = voxel_data::entity_type::id("minecraft:zombie").expect("zombie");
        let metadata = data(&[
            (6, 20, &[0]),
            (PIG_VARIANT_DATA_ID, PIG_VARIANT_SERIALIZER_ID, &[2]),
        ]);
        assert_eq!(variant_from_metadata(pig, &metadata), Some(2));
        // A cow's data is told apart by the entity's type, not the id alone.
        assert_eq!(variant_from_metadata(cow, &metadata), None);
        assert_eq!(variant_from_metadata(zombie, &metadata), None);
        let metadata = data(&[(COW_VARIANT_DATA_ID, COW_VARIANT_SERIALIZER_ID, &[1])]);
        assert_eq!(variant_from_metadata(cow, &metadata), Some(1));
        // The variant not sent: the default.
        assert_eq!(variant_from_metadata(pig, &data(&[(6, 20, &[0])])), None);
    }
}
