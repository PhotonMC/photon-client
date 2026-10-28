use super::*;

/// `SoundEvent.STREAM_CODEC`: a registry place plus one, or zero and the event whole.
pub(super) fn read_sound_holder(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<photon_core::sound_packets::SoundHolder, ConnectionError> {
    use photon_core::sound_packets::SoundHolder;
    let holder = read_varint(input, "sound holder")?;
    if holder == 0 {
        let location = read_identifier_string(input, "sound location")?;
        let fixed_range = if read_bool(input, "sound range present")? {
            Some(read_f32(input, "sound range")?)
        } else {
            None
        };
        return Ok(SoundHolder::Direct {
            location,
            fixed_range,
        });
    }
    let id = registries.sound_events.canonical_or_unknown(holder - 1);
    if photon_data::sound_event::name(id).is_none() {
        return Err(ConnectionError::Codec("unknown sound event id"));
    }
    Ok(SoundHolder::Registered(id))
}

/// `FriendlyByteBuf.readEnum(SoundSource.class)`.
pub(super) fn read_sound_source(
    input: &mut &[u8],
) -> Result<photon_core::sound_packets::SoundSource, ConnectionError> {
    let ordinal = read_varint(input, "sound source")?;
    photon_core::sound_packets::SoundSource::from_ordinal(ordinal)
        .ok_or(ConnectionError::Codec("unknown sound source"))
}

/// `ClientboundSoundPacket`, read from the front of `input` (what follows is the caller's).
pub fn read_sound_packet(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<photon_core::sound_packets::SoundPacket, ConnectionError> {
    let sound = read_sound_holder(input, registries)?;
    let source = read_sound_source(input)?;
    // The packet carries eighths of a block as ints; the reference divides as a float.
    let place = |input: &mut &[u8], name| -> Result<f64, ConnectionError> {
        Ok(f64::from(read_i32(input, name)? as f32 / 8.0))
    };
    let position = (
        place(input, "sound x")?,
        place(input, "sound y")?,
        place(input, "sound z")?,
    );
    Ok(photon_core::sound_packets::SoundPacket {
        sound,
        source,
        position,
        volume: read_f32(input, "sound volume")?,
        pitch: read_f32(input, "sound pitch")?,
        seed: read_i64(input, "sound seed")?,
    })
}

/// `ClientboundSoundEntityPacket`, read from the front of `input`.
pub fn read_sound_entity_packet(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<photon_core::sound_packets::SoundEntityPacket, ConnectionError> {
    Ok(photon_core::sound_packets::SoundEntityPacket {
        sound: read_sound_holder(input, registries)?,
        source: read_sound_source(input)?,
        entity_id: photon_core::ids::EntityId(read_varint(input, "sound entity id")?),
        volume: read_f32(input, "sound volume")?,
        pitch: read_f32(input, "sound pitch")?,
        seed: read_i64(input, "sound seed")?,
    })
}

/// `ClientboundStopSoundPacket`, read from the front of `input`.
pub fn read_stop_sound_packet(
    input: &mut &[u8],
) -> Result<photon_core::sound_packets::StopSoundPacket, ConnectionError> {
    let flags = read_byte(input, "stop sound flags")?;
    let source = if flags & 1 > 0 {
        Some(read_sound_source(input)?)
    } else {
        None
    };
    let name = if flags & 2 > 0 {
        Some(read_identifier_string(input, "stop sound name")?)
    } else {
        None
    };
    Ok(photon_core::sound_packets::StopSoundPacket { name, source })
}

/// A `ClientboundSoundPacket`, `ClientboundSoundEntityPacket` or `ClientboundStopSoundPacket`.
pub struct Sound(pub photon_core::sound_packets::SoundCommand);

impl Sound {
    /// The sound packet with this id, parsed.
    pub fn decode_packet(
        packet_id: i32,
        payload: &[u8],
        registries: &VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        use photon_core::sound_packets::SoundCommand;
        let mut input = payload;
        let command = match packet_id {
            PLAY_CLIENTBOUND_SOUND_PACKET_ID => {
                SoundCommand::At(read_sound_packet(&mut input, registries)?)
            }
            PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID => {
                SoundCommand::FollowingEntity(read_sound_entity_packet(&mut input, registries)?)
            }
            _ => SoundCommand::Stop(read_stop_sound_packet(&mut input)?),
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec("sound packet trailing payload"));
        }
        Ok(Self(command))
    }
}
