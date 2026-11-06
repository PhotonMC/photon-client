//! The 26.3 layouts of the entity move and position-sync packets, which carry a path of steps.

use super::*;
use photon_core::ids::EntityId;
use photon_data::version_registries::VersionRegistries;

/// One place of a `PositionPath` and the ticks to reach it.
pub type PathStepObservation = ((f64, f64, f64), i32);

/// One step of a stepped `VecDelta`: the ticks to reach it and its offset from the one before.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveStepObservation {
    pub ticks: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
}

/// An offset and the steps it ends (`VecDelta`).
type DeltaAndSteps = ((i16, i16, i16), Vec<MoveStepObservation>);

/// The end of a path and its steps (`PositionPath`).
type EndAndPath = ((f64, f64, f64), Vec<PathStepObservation>);

/// `VecDelta.read`: the packed properties' step count says how many steps follow; none means one
/// plain offset.
fn read_vec_delta(input: &mut &[u8], step_count: usize) -> Result<DeltaAndSteps, ConnectionError> {
    if step_count == 0 {
        let delta = (
            read_i16(input, "move entity delta x")?,
            read_i16(input, "move entity delta y")?,
            read_i16(input, "move entity delta z")?,
        );
        return Ok((delta, Vec::new()));
    }
    // Every step takes at least seven bytes.
    if step_count > input.len() / 7 {
        return Err(ConnectionError::Codec("move entity step count too big"));
    }
    let mut steps = Vec::with_capacity(step_count);
    for _ in 0..step_count {
        steps.push(MoveStepObservation {
            ticks: read_varint(input, "move entity step ticks")?,
            delta_x: read_i16(input, "move entity step x")?,
            delta_y: read_i16(input, "move entity step y")?,
            delta_z: read_i16(input, "move entity step z")?,
        });
    }
    let end = steps
        .last()
        .map_or((0, 0, 0), |step| (step.delta_x, step.delta_y, step.delta_z));
    Ok((end, steps))
}

/// The pos and pos-rot packets: the entity id, the on-ground bit with the step count, the
/// offsets and, for the second, the turn.
fn parse_move_with_position(
    input: &[u8],
    has_rotation: bool,
) -> Result<EntityMoveObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "move entity id")?;
    let properties = read_varint(&mut input, "move entity properties")? as u32;
    let (delta, steps) = read_vec_delta(&mut input, (properties >> 1) as usize)?;
    let (y_rot, x_rot) = if has_rotation {
        (
            Some(decode_packed_degrees(read_byte(
                &mut input,
                "move entity y rotation",
            )?)),
            Some(decode_packed_degrees(read_byte(
                &mut input,
                "move entity x rotation",
            )?)),
        )
    } else {
        (None, None)
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("move entity trailing payload"));
    }
    Ok(EntityMoveObservation {
        entity_id: EntityId(entity_id),
        delta_x: delta.0,
        delta_y: delta.1,
        delta_z: delta.2,
        y_rot,
        x_rot,
        has_position: true,
        has_rotation,
        on_ground: properties & 1 != 0,
        steps,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}

/// The rotation-only packet: the entity id, the on-ground byte, then the turn.
fn parse_move_rotation(input: &[u8]) -> Result<EntityMoveObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "move entity id")?;
    let on_ground = read_byte(&mut input, "move entity on ground")? != 0;
    let y_rot = decode_packed_degrees(read_byte(&mut input, "move entity y rotation")?);
    let x_rot = decode_packed_degrees(read_byte(&mut input, "move entity x rotation")?);
    if !input.is_empty() {
        return Err(ConnectionError::Codec("move entity trailing payload"));
    }
    Ok(EntityMoveObservation {
        entity_id: EntityId(entity_id),
        delta_x: 0,
        delta_y: 0,
        delta_z: 0,
        y_rot: Some(y_rot),
        x_rot: Some(x_rot),
        has_position: false,
        has_rotation: true,
        on_ground,
        steps: Vec::new(),
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}

pub fn parse_move_entity_pos_packet_for(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<EntityMoveObservation, ConnectionError> {
    if registries.has_movement_paths() {
        parse_move_with_position(input, false)
    } else {
        parse_move_entity_pos_packet(input)
    }
}

pub fn parse_move_entity_pos_rot_packet_for(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<EntityMoveObservation, ConnectionError> {
    if registries.has_movement_paths() {
        parse_move_with_position(input, true)
    } else {
        parse_move_entity_pos_rot_packet(input)
    }
}

pub fn parse_move_entity_rot_packet_for(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<EntityMoveObservation, ConnectionError> {
    if registries.has_movement_paths() {
        parse_move_rotation(input)
    } else {
        parse_move_entity_rot_packet(input)
    }
}

/// `PositionPath.STREAM_CODEC`: a type id, then one place or a list of places with their ticks.
fn read_position_path(input: &mut &[u8]) -> Result<EndAndPath, ConnectionError> {
    let read_place = |input: &mut &[u8]| -> Result<(f64, f64, f64), ConnectionError> {
        Ok((
            read_f64(input, "entity position sync x")?,
            read_f64(input, "entity position sync y")?,
            read_f64(input, "entity position sync z")?,
        ))
    };
    // The id mapper falls back to the first kind (linear) for an unknown id.
    if read_varint(input, "entity position sync path type")? != 1 {
        return Ok((read_place(input)?, Vec::new()));
    }
    let count = read_varint(input, "entity position sync step count")?;
    // Every step takes at least 25 bytes.
    if count <= 0 || count as usize > input.len() / 25 {
        return Err(ConnectionError::Codec(
            "entity position sync step count out of range",
        ));
    }
    let mut steps = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let place = read_place(input)?;
        steps.push((
            place,
            read_varint(input, "entity position sync step ticks")?,
        ));
    }
    let end = steps.last().map_or((0.0, 0.0, 0.0), |(place, _)| *place);
    Ok((end, steps))
}

pub fn parse_entity_position_sync_packet_for(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<EntityPositionSyncObservation, ConnectionError> {
    if !registries.has_movement_paths() {
        return parse_entity_position_sync_packet(input);
    }
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity position sync entity id")?;
    let ((x, y, z), steps) = read_position_path(&mut input)?;
    let y_rot = read_f32(&mut input, "entity position sync y rotation")?;
    let x_rot = read_f32(&mut input, "entity position sync x rotation")?;
    let on_ground = read_byte(&mut input, "entity position sync on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "entity position sync trailing payload",
        ));
    }
    Ok(EntityPositionSyncObservation {
        entity_id: EntityId(entity_id),
        x,
        y,
        z,
        delta_x: 0.0,
        delta_y: 0.0,
        delta_z: 0.0,
        y_rot,
        x_rot,
        on_ground,
        steps,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}
