use super::*;

/// `ClientboundLevelParticlesPacket`, read from the front of `input`: nothing for a type whose
/// options are not read yet (the packet ends with them).
pub fn read_level_particles_packet(
    input: &mut &[u8],
) -> Result<Option<photon_world::particles::LevelParticles>, ConnectionError> {
    const NAME: &str = "level particles";
    let override_limiter = read_bool(input, NAME)?;
    let always_show = read_bool(input, NAME)?;
    let at = (
        read_f64(input, NAME)?,
        read_f64(input, NAME)?,
        read_f64(input, NAME)?,
    );
    let spread = (
        read_f32(input, NAME)?,
        read_f32(input, NAME)?,
        read_f32(input, NAME)?,
    );
    let max_speed = read_f32(input, NAME)?;
    let count = read_i32(input, NAME)?;
    let Some((type_id, data)) = read_particle_options(input)? else {
        *input = &[];
        return Ok(None);
    };
    Ok(Some(photon_world::particles::LevelParticles {
        type_id,
        data,
        override_limiter,
        always_show,
        at,
        spread,
        max_speed,
        count,
    }))
}

/// `ParticleTypes.STREAM_CODEC`: a type and its options. Nothing for a type whose options are not
/// read yet, after which nothing more of the input can be read.
pub(crate) fn read_particle_options(
    input: &mut &[u8],
) -> Result<Option<(i32, photon_world::particles::ParticleData)>, ConnectionError> {
    const NAME: &str = "particle options";
    use photon_world::particles::ParticleData;
    let type_id = read_varint(input, NAME)?;
    let Some(name) = photon_data::particle_type::name(type_id) else {
        return Err(ConnectionError::Codec("unknown particle type id"));
    };
    let color = |input: &mut &[u8]| read_i32(input, NAME).map(|value| value as u32);
    let data = if photon_data::particle_type::carries_block_state(type_id) {
        ParticleData::Block(read_varint(input, NAME)?)
    } else if photon_data::particle_type::is_simple(type_id) {
        ParticleData::Plain
    } else {
        match name {
            "entity_effect" | "tinted_leaves" | "flash" => ParticleData::Color(color(input)?),
            "effect" | "instant_effect" => ParticleData::Spell {
                color: color(input)?,
                power: read_f32(input, NAME)?,
            },
            // `ScalableParticleOptionsBase`: the size is held between 0.01 and 4.
            "dust" => ParticleData::Dust {
                color: color(input)?,
                scale: read_f32(input, NAME)?.clamp(0.01, 4.0),
            },
            "sculk_charge" => ParticleData::Roll(read_f32(input, NAME)?),
            "dust_color_transition" => ParticleData::DustTransition {
                from: color(input)?,
                to: color(input)?,
                scale: read_f32(input, NAME)?.clamp(0.01, 4.0),
            },
            "vibration" => {
                // `PositionSource.STREAM_CODEC`: a registry id (`PositionSourceType.BLOCK` = 0,
                // `.ENTITY` = 1, their registration order) then the type's own fields.
                let destination = match read_varint(input, NAME)? {
                    0 => photon_world::particles::PositionSource::Block(
                        photon_core::binary::BlockPos::unpack(read_i64(input, NAME)? as u64),
                    ),
                    1 => photon_world::particles::PositionSource::Entity {
                        id: read_varint(input, NAME)?,
                        y_offset: read_f32(input, NAME)?,
                    },
                    _ => return Err(ConnectionError::Codec("unknown position source type")),
                };
                ParticleData::Vibration {
                    destination,
                    arrival_in_ticks: read_varint(input, NAME)?,
                }
            }
            "shriek" => ParticleData::Shriek {
                delay: read_varint(input, NAME)?,
            },
            "trail" => ParticleData::Trail {
                target: (
                    read_f64(input, NAME)?,
                    read_f64(input, NAME)?,
                    read_f64(input, NAME)?,
                ),
                color: color(input)?,
                duration: read_varint(input, NAME)?,
            },
            // `ItemStackTemplate.STREAM_CODEC`: the item, its count and its components.
            "item" => {
                let item = read_varint(input, NAME)?;
                read_varint(input, NAME)?;
                photon_game::item_components::read_patch(input)
                    .map_err(|_| ConnectionError::Codec(NAME))?;
                ParticleData::Item(item)
            }
            _ => return Ok(None),
        }
    };
    Ok(Some((type_id, data)))
}
