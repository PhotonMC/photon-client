//! `ParticleTypes.STREAM_CODEC`: a particle type and its options as the wire and the entity data
//! hold them.

use photon_core::binary::BlockPos;
use photon_core::ids::{BlockStateId, EntityId, ItemId};
use photon_core::wire::read_varint;

use crate::particle_values::{ParticleData, PositionSource};

/// The next `N` bytes of `input`.
fn take<const N: usize>(input: &mut &[u8]) -> Result<[u8; N], &'static str> {
    let (bytes, rest) = input
        .split_first_chunk::<N>()
        .ok_or("particle options end early")?;
    *input = rest;
    Ok(*bytes)
}

fn int(input: &mut &[u8]) -> Result<i32, &'static str> {
    take(input).map(i32::from_be_bytes)
}

fn float(input: &mut &[u8]) -> Result<f32, &'static str> {
    take(input).map(f32::from_be_bytes)
}

fn double(input: &mut &[u8]) -> Result<f64, &'static str> {
    take(input).map(f64::from_be_bytes)
}

fn varint(input: &mut &[u8]) -> Result<i32, &'static str> {
    read_varint(input).map_err(|_| "particle options end early")
}

/// A type and its options, read from the front of `input`. `None` for a type whose options are
/// not read yet, after which nothing more of the input can be read.
pub fn read_particle_options(
    input: &mut &[u8],
) -> Result<Option<(i32, ParticleData)>, &'static str> {
    let type_id = varint(input)?;
    let Some(name) = crate::particle_type::name(type_id) else {
        return Err("unknown particle type id");
    };
    let data = if crate::particle_type::carries_block_state(type_id) {
        Some(ParticleData::Block(BlockStateId(varint(input)?)))
    } else if crate::particle_type::is_simple(type_id) {
        Some(ParticleData::Plain)
    } else {
        read_named_options(name, input)?
    };
    Ok(data.map(|data| (type_id, data)))
}

/// The options of the types that tint: a colour, and a power or a size with it.
fn read_named_options(name: &str, input: &mut &[u8]) -> Result<Option<ParticleData>, &'static str> {
    let color = |input: &mut &[u8]| int(input).map(|value| value as u32);
    Ok(Some(match name {
        "entity_effect" | "tinted_leaves" | "flash" => ParticleData::Color(color(input)?),
        "effect" | "instant_effect" => ParticleData::Spell {
            color: color(input)?,
            power: float(input)?,
        },
        // `ScalableParticleOptionsBase`: the size is held between 0.01 and 4.
        "dust" => ParticleData::Dust {
            color: color(input)?,
            scale: float(input)?.clamp(0.01, 4.0),
        },
        "dust_color_transition" => ParticleData::DustTransition {
            from: color(input)?,
            to: color(input)?,
            scale: float(input)?.clamp(0.01, 4.0),
        },
        _ => return read_other_options(name, input),
    }))
}

/// The options of the rest.
fn read_other_options(name: &str, input: &mut &[u8]) -> Result<Option<ParticleData>, &'static str> {
    Ok(Some(match name {
        "sculk_charge" => ParticleData::Roll(float(input)?),
        "dragon_breath" => ParticleData::Power(float(input)?),
        "shriek" => ParticleData::Shriek {
            delay: varint(input)?,
        },
        "trail" => ParticleData::Trail {
            target: (double(input)?, double(input)?, double(input)?),
            color: int(input)? as u32,
            duration: varint(input)?,
        },
        "vibration" => read_vibration(input)?,
        "item" => read_item(input)?,
        _ => return Ok(None),
    }))
}

/// `VibrationParticleOption`: a `PositionSource.STREAM_CODEC` (a registry id,
/// `PositionSourceType.BLOCK` = 0 and `.ENTITY` = 1 in registration order, then the type's own
/// fields) and the ticks to arrive.
fn read_vibration(input: &mut &[u8]) -> Result<ParticleData, &'static str> {
    let destination = match varint(input)? {
        0 => PositionSource::Block(BlockPos::unpack(i64::from_be_bytes(take(input)?) as u64)),
        1 => PositionSource::Entity {
            id: EntityId(varint(input)?),
            y_offset: float(input)?,
        },
        _ => return Err("unknown position source type"),
    };
    Ok(ParticleData::Vibration {
        destination,
        arrival_in_ticks: varint(input)?,
    })
}

/// `ItemParticleOption`, an `ItemStackTemplate.STREAM_CODEC`: the item, its count and its
/// components.
fn read_item(input: &mut &[u8]) -> Result<ParticleData, &'static str> {
    let item = ItemId(varint(input)?);
    varint(input)?;
    crate::item_components::read_patch(input).map_err(|_| "particle item components")?;
    Ok(ParticleData::Item(item))
}
