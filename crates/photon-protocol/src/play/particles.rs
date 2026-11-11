use super::*;

/// `ClientboundLevelParticlesPacket`, read from the front of `input` as the session's version
/// lays it out: nothing for a type whose options are not read yet (the rest of the packet then
/// cannot be read, and is left).
pub fn read_level_particles_packet(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<Option<photon_data::particle_values::LevelParticles>, ConnectionError> {
    let axis_speeds = photon_core::versions::Rules::of(registries.version())
        .protocol
        .level_particles_carry_axis_speeds;
    // 26.3 names the particle first; before it the particle came last.
    let first = if axis_speeds {
        Some(read_particle_options(input, registries)?)
    } else {
        None
    };
    if first == Some(None) {
        *input = &[];
        return Ok(None);
    }
    let mut packet = read_particles_body(input, axis_speeds)?;
    let options = match first {
        Some(options) => options,
        None => read_particle_options(input, registries)?,
    };
    let Some((type_id, data)) = options else {
        *input = &[];
        return Ok(None);
    };
    (packet.type_id, packet.data) = (type_id, data);
    Ok(Some(packet))
}

/// What a level particles packet carries besides its particle, with the particle left blank.
fn read_particles_body(
    input: &mut &[u8],
    axis_speeds: bool,
) -> Result<photon_data::particle_values::LevelParticles, ConnectionError> {
    const NAME: &str = "level particles";
    use photon_data::particle_values::{LevelParticles, ParticleData, Randomization};
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
    let (max_speed, count, randomization) = if axis_speeds {
        let speed = (
            read_f32(input, NAME)?,
            read_f32(input, NAME)?,
            read_f32(input, NAME)?,
        );
        let count = read_varint(input, NAME)?;
        (
            speed,
            count,
            Randomization::from_id(read_varint(input, NAME)?),
        )
    } else {
        let speed = read_f32(input, NAME)?;
        let count = read_i32(input, NAME)?;
        ((speed, speed, speed), count, Randomization::Default)
    };
    Ok(LevelParticles {
        type_id: 0,
        data: ParticleData::Plain,
        override_limiter,
        always_show,
        at,
        spread,
        max_speed,
        count,
        randomization,
    })
}

/// `ParticleTypes.STREAM_CODEC`: a type and its options. Nothing for a type whose options are not
/// read yet, after which nothing more of the input can be read.
pub fn read_particle_options(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<Option<(i32, photon_data::particle_values::ParticleData)>, ConnectionError> {
    photon_data::particle_options::read_particle_options(input, registries)
        .map_err(ConnectionError::Codec)
}

/// `ClientboundLevelParticlesPacket`: nothing for a particle the client does not know.
pub struct LevelParticlesPacket(pub Option<photon_data::particle_values::LevelParticles>);

impl Decode for LevelParticlesPacket {
    fn decode(
        payload: &[u8],
        registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let packet = read_level_particles_packet(&mut input, registries)?;
        if !input.is_empty() {
            return Err(ConnectionError::Codec("level particles trailing payload"));
        }
        Ok(Self(packet))
    }
}
