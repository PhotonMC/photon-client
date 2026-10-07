use super::*;

/// `ClientboundLevelParticlesPacket`, read from the front of `input`: nothing for a type that
/// carries options other than a block state after its id (those are not read yet; the packet
/// ends with them).
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
    let block_state = if voxel_data::particle_type::carries_block_state(type_id) {
        Some(read_varint(input, NAME)?)
    } else if voxel_data::particle_type::is_simple(type_id) {
        None
    } else {
        *input = &[];
        return Ok(None);
    };
    Ok(Some(voxel_world::particles::LevelParticles {
        type_id,
        block_state,
        override_limiter,
        always_show,
        at,
        spread,
        max_speed,
        count,
    }))
}
