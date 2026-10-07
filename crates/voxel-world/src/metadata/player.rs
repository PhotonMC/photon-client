use super::*;

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
