use super::*;

/// `ClientboundLevelParticlesPacket`, read from the front of `input`: nothing for a type whose
/// options are not read yet (the packet ends with them).
pub fn read_level_particles_packet(
    input: &mut &[u8],
) -> Result<Option<photon_data::particle_values::LevelParticles>, ConnectionError> {
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
    Ok(Some(photon_data::particle_values::LevelParticles {
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
pub fn read_particle_options(
    input: &mut &[u8],
) -> Result<Option<(i32, photon_data::particle_values::ParticleData)>, ConnectionError> {
    photon_data::particle_options::read_particle_options(input).map_err(ConnectionError::Codec)
}

/// `ClientboundLevelParticlesPacket`: nothing for a particle the client does not know.
pub struct LevelParticlesPacket(pub Option<photon_data::particle_values::LevelParticles>);

impl Decode for LevelParticlesPacket {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let packet = read_level_particles_packet(&mut input)?;
        if !input.is_empty() {
            return Err(ConnectionError::Codec("level particles trailing payload"));
        }
        Ok(Self(packet))
    }
}
