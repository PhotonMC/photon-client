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
        // A painting's variant is a holder: its registry id plus one, or zero for one sent whole.
        "minecraft:painting" => {
            let (found, mut value) = find_metadata_field(metadata, PAINTING_VARIANT_DATA_ID)?;
            if found != PAINTING_VARIANT_SERIALIZER_ID {
                return None;
            }
            let holder = voxel_core::wire::read_varint(&mut value).ok()?;
            return holder.checked_sub(1).filter(|id| *id >= 0);
        }
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

/// A piglin's baby, dancing and crossbow-charging flags (`Piglin.DATA_BABY_ID`,
/// `DATA_IS_DANCING`, `DATA_IS_CHARGING_CROSSBOW`), when the data holds them.
pub(crate) fn piglin_flags_from_metadata(
    type_id: i32,
    metadata: &[u8],
) -> (Option<bool>, Option<bool>, Option<bool>) {
    if voxel_data::entity_type::name(type_id) != Some("minecraft:piglin") {
        return (None, None, None);
    }
    (
        metadata_boolean_value(metadata, PIGLIN_BABY_DATA_ID),
        metadata_boolean_value(metadata, PIGLIN_DANCING_DATA_ID),
        metadata_boolean_value(metadata, PIGLIN_CHARGING_CROSSBOW_DATA_ID),
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
