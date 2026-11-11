use super::*;

/// An explosion for the level, and the push the local player takes.
pub type ExplodeObservation = (
    photon_data::particle_values::Explosion,
    Option<(f64, f64, f64)>,
);

/// The name of a registered sound; a sound sent whole has none.
fn registered_sound_name(holder: photon_core::sound_packets::SoundHolder) -> Option<&'static str> {
    match holder {
        photon_core::sound_packets::SoundHolder::Registered(id) => {
            photon_data::sound_event::name(id)
        }
        photon_core::sound_packets::SoundHolder::Direct { .. } => None,
    }
}

/// The weighted particles of block debris; nothing when one of their options is not read yet.
fn read_block_particles(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<Option<Vec<photon_data::particle_values::ExplosionParticle>>, ConnectionError> {
    const NAME: &str = "explode";
    let count = read_varint(input, NAME)?;
    let mut block_particles = Vec::new();
    for _ in 0..count.max(0) {
        let Some((type_id, data)) = read_particle_options(input, registries)? else {
            return Ok(None);
        };
        let scaling = read_f32(input, NAME)?;
        let speed = read_f32(input, NAME)?;
        let weight = read_varint(input, NAME)?;
        block_particles.push(photon_data::particle_values::ExplosionParticle {
            type_id,
            data,
            scaling,
            speed,
            weight,
        });
    }
    Ok(Some(block_particles))
}

/// `ClientboundExplodePacket`. Nothing for one whose particles' options are not read yet.
pub fn parse_explode_packet(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<Option<ExplodeObservation>, ConnectionError> {
    const NAME: &str = "explode";
    use photon_data::particle_values::Explosion;
    let mut input = input;
    let input = &mut input;
    let vec3 = |input: &mut &[u8]| -> Result<(f64, f64, f64), ConnectionError> {
        Ok((
            read_f64(input, NAME)?,
            read_f64(input, NAME)?,
            read_f64(input, NAME)?,
        ))
    };
    let center = vec3(input)?;
    let radius = read_f32(input, NAME)?;
    let block_count = read_i32(input, NAME)?;
    let knockback = if read_bool(input, NAME)? {
        Some(vec3(input)?)
    } else {
        None
    };
    let Some(particle) = read_particle_options(input, registries)? else {
        return Ok(None);
    };
    let sound = registered_sound_name(read_sound_holder(input, registries)?);
    let Some(block_particles) = read_block_particles(input, registries)? else {
        return Ok(None);
    };
    // 26.3 ends the packet with whether the sound plays; before it the sound always played.
    let play_sound = !photon_core::versions::Rules::of(registries.version())
        .protocol
        .explosion_packet_has_sound_flag
        || read_bool(input, NAME)?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("explode trailing payload"));
    }
    let explosion = Explosion {
        center,
        radius,
        block_count,
        particle,
        sound,
        block_particles,
        play_sound,
    };
    Ok(Some((explosion, knockback)))
}

wrapped_wire_packet!(
    /// `ClientboundExplodePacket`: nothing for a particle the client does not know.
    Explode(Option<ExplodeObservation>) = parse_explode_packet,
    OnError::Ignore("explode")
);
