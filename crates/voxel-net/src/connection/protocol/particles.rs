use super::*;

/// `ClientboundLevelParticlesPacket`, read from the front of `input`: nothing for a type whose
/// options are not read yet (the packet ends with them).
pub fn read_level_particles_packet(
    input: &mut &[u8],
) -> Result<Option<voxel_world::particles::LevelParticles>, ConnectionError> {
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
    let type_id = read_varint(input, NAME)?;
    if voxel_data::particle_type::name(type_id).is_none() {
        return Err(ConnectionError::Codec("unknown particle type id"));
    }
    use voxel_world::particles::ParticleData;
    let name = voxel_data::particle_type::name(type_id).unwrap_or_default();
    let color = |input: &mut &[u8]| read_i32(input, NAME).map(|value| value as u32);
    let data = if voxel_data::particle_type::carries_block_state(type_id) {
        ParticleData::Block(read_varint(input, NAME)?)
    } else if voxel_data::particle_type::is_simple(type_id) {
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
            _ => {
                *input = &[];
                return Ok(None);
            }
        }
    };
    Ok(Some(voxel_world::particles::LevelParticles {
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
