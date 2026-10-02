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

/// The registry id of the variant a pig or a cow of this type wears, if the
/// data holds one (the default is not sent).
pub(crate) fn variant_from_metadata(type_id: i32, metadata: &[u8]) -> Option<i32> {
    let (data_id, serializer) = match voxel_data::entity_type::name(type_id)? {
        "minecraft:pig" => (PIG_VARIANT_DATA_ID, PIG_VARIANT_SERIALIZER_ID),
        "minecraft:cow" => (COW_VARIANT_DATA_ID, COW_VARIANT_SERIALIZER_ID),
        "minecraft:chicken" => (CHICKEN_VARIANT_DATA_ID, CHICKEN_VARIANT_SERIALIZER_ID),
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
