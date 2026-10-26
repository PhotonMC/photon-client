use super::*;

/// `ParticleTypes.STREAM_CODEC.apply(ByteBufCodecs.list())`: the effect particles, each the entity
/// effect's colour or a simple particle (the only ones an effect gives off).
pub(crate) fn read_effect_particles(metadata: &mut &[u8]) -> Option<Vec<EffectParticle>> {
    let count = photon_core::wire::read_varint(metadata).ok()?;
    let count = usize::try_from(count).ok()?;
    if count > metadata.len() {
        return None;
    }
    let mut particles = Vec::with_capacity(count);
    for _ in 0..count {
        particles.push(read_effect_particle(metadata)?);
    }
    Some(particles)
}

/// One particle of an effect: the entity effect's ARGB colour, or a simple particle's nothing.
pub(crate) fn read_effect_particle(metadata: &mut &[u8]) -> Option<EffectParticle> {
    let type_id = photon_core::wire::read_varint(metadata).ok()?;
    let color = if type_id == ENTITY_EFFECT_PARTICLE_TYPE_ID {
        let bytes: [u8; 4] = metadata.get(..4)?.try_into().ok()?;
        *metadata = &metadata[4..];
        Some(u32::from_be_bytes(bytes))
    } else if photon_data::particle_type::is_simple(type_id) {
        None
    } else {
        return None;
    };
    Some(EffectParticle { type_id, color })
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

/// Whether an ageable mob's growth is locked (`AgeableMob.AGE_LOCKED`), when the data holds it.
pub(crate) fn age_locked_from_metadata(metadata: &[u8]) -> Option<bool> {
    metadata_boolean_value(metadata, AGEABLE_AGE_LOCKED_DATA_ID)
}

/// An ageable mob's (or a tadpole's, which keeps the same flag at the same id) age lock in its
/// data, applied to the entity.
pub(crate) fn apply_age_lock_data(entity: &mut crate::entity::ClientEntity, metadata: &[u8]) {
    let tadpole = photon_data::entity_type::name(entity.type_id) == Some("minecraft:tadpole");
    if (entity.is_ageable_mob() || tadpole)
        && let Some(locked) = age_locked_from_metadata(metadata)
    {
        entity.living.age_locked = locked;
    }
}

/// The ageable-mob lock and the goat's own data, applied together.
pub(crate) fn apply_age_lock_and_goat_data(
    entity: &mut crate::entity::ClientEntity,
    metadata: &[u8],
) {
    apply_age_lock_data(entity, metadata);
    super::goat::apply_goat_data(entity, metadata);
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

pub(crate) fn effect_particles_from_metadata(metadata: &[u8]) -> Option<Vec<EffectParticle>> {
    let (serializer_id, mut value_input) =
        find_metadata_field(metadata, ENTITY_EFFECT_PARTICLES_DATA_ID)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_PARTICLES_ID {
        return None;
    }
    read_effect_particles(&mut value_input)
}

pub(crate) fn sleeping_pos_from_metadata(
    metadata: &[u8],
) -> Option<Option<photon_core::binary::BlockPos>> {
    let (serializer_id, value_input) = find_metadata_field(metadata, ENTITY_SLEEPING_POS_DATA_ID)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID {
        return None;
    }
    let (&present, mut rest) = value_input.split_first()?;
    if present == 0 {
        return Some(None);
    }
    let value = photon_core::binary::read_blockpos(&mut rest).ok()?;
    if rest.is_empty() {
        return None;
    }
    Some(Some(value))
}

/// The custom name field's component, as the NBT tag the server sent: `Some(None)` for a field
/// that says there is no name.
pub(crate) fn custom_name_from_metadata(
    metadata: &[u8],
) -> Option<Option<photon_core::nbt::FieldValue>> {
    let (serializer_id, value_input) = find_metadata_field(metadata, ENTITY_CUSTOM_NAME_DATA_ID)?;
    if serializer_id != ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID {
        return None;
    }
    let (&present, mut rest) = value_input.split_first()?;
    if present == 0 {
        return Some(None);
    }
    photon_core::nbt::read_any_nbt(&mut rest).ok()
}

pub(crate) fn custom_name_present_from_metadata(metadata: &[u8]) -> Option<bool> {
    custom_name_from_metadata(metadata).map(|name| name.is_some())
}

/// `AreaEffectCloud`'s data: its radius (`DATA_RADIUS`, 3 until told), whether it still waits to
/// take effect (`DATA_WAITING`), and the particle it gives off (`DATA_PARTICLE`, the plain
/// entity effect swirl until told, and any particle the data names): what a lingering potion's
/// cloud draws itself from.
pub(crate) fn area_effect_cloud_from_metadata(
    metadata: &[u8],
) -> (f32, bool, (i32, crate::particles::ParticleData)) {
    let default = (
        ENTITY_EFFECT_PARTICLE_TYPE_ID,
        crate::particles::ParticleData::Color(u32::MAX),
    );
    let particle = find_metadata_field(metadata, AREA_EFFECT_CLOUD_PARTICLE_DATA_ID)
        .filter(|(serializer, _)| *serializer == ENTITY_DATA_SERIALIZER_PARTICLE_ID)
        .and_then(|(_, mut value)| crate::particles::read_particle_options(&mut value).ok()?)
        .unwrap_or(default);
    (
        metadata_float_value(metadata, AREA_EFFECT_CLOUD_RADIUS_DATA_ID).unwrap_or(3.0),
        metadata_boolean_value(metadata, AREA_EFFECT_CLOUD_WAITING_DATA_ID).unwrap_or(false),
        particle,
    )
}
