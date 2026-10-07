use super::*;

/// `SoundEvent.STREAM_CODEC`: a registry place plus one, or zero and the event whole.
pub(super) fn read_sound_holder(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundHolder, ConnectionError> {
    use voxel_game::sounds::SoundHolder;
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
    let id = holder - 1;
    if voxel_data::sound_event::name(id).is_none() {
        return Err(ConnectionError::Codec("unknown sound event id"));
    }
    Ok(SoundHolder::Registered(id))
}

/// `FriendlyByteBuf.readEnum(SoundSource.class)`.
pub(super) fn read_sound_source(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundSource, ConnectionError> {
    let ordinal = read_varint(input, "sound source")?;
    voxel_game::sounds::SoundSource::from_ordinal(ordinal)
        .ok_or(ConnectionError::Codec("unknown sound source"))
}

/// `ClientboundSoundPacket`, read from the front of `input` (what follows is the caller's).
pub fn read_sound_packet(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundPacket, ConnectionError> {
    let sound = read_sound_holder(input)?;
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
    Ok(voxel_game::sounds::SoundPacket {
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
) -> Result<voxel_game::sounds::SoundEntityPacket, ConnectionError> {
    Ok(voxel_game::sounds::SoundEntityPacket {
        sound: read_sound_holder(input)?,
        source: read_sound_source(input)?,
        entity_id: read_varint(input, "sound entity id")?,
        volume: read_f32(input, "sound volume")?,
        pitch: read_f32(input, "sound pitch")?,
        seed: read_i64(input, "sound seed")?,
    })
}

/// `ClientboundStopSoundPacket`, read from the front of `input`.
pub fn read_stop_sound_packet(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::StopSoundPacket, ConnectionError> {
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
    Ok(voxel_game::sounds::StopSoundPacket { name, source })
}
