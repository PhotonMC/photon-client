use super::*;

/// An explosion for the level, and the push the local player takes.
pub type ExplodeObservation = (photon_world::particles::Explosion, Option<(f64, f64, f64)>);

/// `ClientboundExplodePacket`. Nothing for one whose particles' options are not read yet.
pub fn parse_explode_packet(input: &[u8]) -> Result<Option<ExplodeObservation>, ConnectionError> {
    const NAME: &str = "explode";
    use photon_world::particles::{Explosion, ExplosionParticle};
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
    let Some(particle) = read_particle_options(input)? else {
        return Ok(None);
    };
    let sound = match read_sound_holder(input)? {
        photon_game::sounds::SoundHolder::Registered(id) => photon_data::sound_event::name(id),
        photon_game::sounds::SoundHolder::Direct { .. } => None,
    };
    let count = read_varint(input, NAME)?;
    let mut block_particles = Vec::new();
    for _ in 0..count.max(0) {
        let Some((type_id, data)) = read_particle_options(input)? else {
            return Ok(None);
        };
        let scaling = read_f32(input, NAME)?;
        let speed = read_f32(input, NAME)?;
        let weight = read_varint(input, NAME)?;
        block_particles.push(ExplosionParticle {
            type_id,
            data,
            scaling,
            speed,
            weight,
        });
    }
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
    };
    Ok(Some((explosion, knockback)))
}

wrapped_packet!(
    /// `ClientboundExplodePacket`: nothing for a particle the client does not know.
    Explode(Option<ExplodeObservation>) = parse_explode_packet,
    OnError::Ignore("explode")
);
