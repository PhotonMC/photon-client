use super::*;

/// One Java LivingEntity interpolation target and its three world-tick steps.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityInterpolationObservation {
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_position: (f64, f64, f64),
    pub target_position: (f64, f64, f64),
    pub before_rotation: (f32, f32),
    pub target_rotation: (f32, f32),
    pub positions_after_ticks: Vec<(f64, f64, f64)>,
    pub rotations_after_ticks: Vec<(f32, f32)>,
    pub tick_count: i32,
}

/// One large remote Player position update and its immediate snap state.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPositionSnapObservation {
    pub packet_kind: &'static str,
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_position: (f64, f64, f64),
    pub target_position: (f64, f64, f64),
    pub before_rotation: (f32, f32),
    pub target_rotation: (f32, f32),
    pub applied_position: (f64, f64, f64),
    pub applied_rotation: (f32, f32, f32),
    pub previous_position: (f64, f64, f64),
    pub previous_rotation: (f32, f32, f32),
    pub interpolation_steps: u8,
}

/// Wire-level fields decoded from one clientbound entity-motion packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMotionObservation {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub applied_to_world: bool,
    pub applied_motion: Option<(f64, f64, f64)>,
}

/// One remote Player motion target and its three world-tick interpolation steps.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMotionInterpolationObservation {
    pub entity_id: i32,
    pub entity_type_id: i32,
    pub before_motion: (f64, f64, f64),
    pub target_motion: (f64, f64, f64),
    pub applied_motion: (f64, f64, f64),
    pub motions_after_ticks: Vec<(f64, f64, f64)>,
    pub tick_count: i32,
}

/// Wire-level fields decoded from one clientbound head-rotation packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityHeadRotationObservation {
    pub entity_id: i32,
    pub y_head_rot: f32,
    pub applied_to_world: bool,
    pub applied_head_rotation: Option<f32>,
}

/// `ClientboundMoveMinecartPacket`: the minecart and the steps it took along the track, each a
/// position and a movement (doubles), two turns (a byte each), and a weight (a float).
pub(crate) fn parse_move_minecart_packet(
    input: &[u8],
) -> Result<(i32, Vec<MinecartStep>), ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "minecart entity id")?;
    let count = read_varint(&mut input, "minecart step count")?;
    let mut steps = Vec::new();
    for _ in 0..count {
        let mut vec3 = |name| -> Result<(f64, f64, f64), ConnectionError> {
            Ok((
                read_f64(&mut input, name)?,
                read_f64(&mut input, name)?,
                read_f64(&mut input, name)?,
            ))
        };
        let position = vec3("minecart step position")?;
        let movement = vec3("minecart step movement")?;
        let y_rot = decode_packed_degrees(read_byte(&mut input, "minecart step y rot")?);
        let x_rot = decode_packed_degrees(read_byte(&mut input, "minecart step x rot")?);
        let weight = read_f32(&mut input, "minecart step weight")?;
        steps.push(MinecartStep {
            position,
            movement,
            y_rot,
            x_rot,
            weight,
        });
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("minecart steps trailing payload"));
    }
    Ok((entity_id, steps))
}

pub(crate) fn parse_set_entity_motion_packet(
    input: &[u8],
) -> Result<EntityMotionObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity motion entity id")?;
    let movement = read_lp_vec3(&mut input, "entity motion movement")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity motion trailing payload"));
    }
    Ok(EntityMotionObservation {
        entity_id,
        x: movement.0,
        y: movement.1,
        z: movement.2,
        applied_to_world: false,
        applied_motion: None,
    })
}

pub(crate) fn parse_rotate_head_packet(
    input: &[u8],
) -> Result<EntityHeadRotationObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity head rotation entity id")?;
    let y_head_rot = decode_packed_degrees(read_byte(&mut input, "entity head rotation angle")?);
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "entity head rotation trailing payload",
        ));
    }
    Ok(EntityHeadRotationObservation {
        entity_id,
        y_head_rot,
        applied_to_world: false,
        applied_head_rotation: None,
    })
}

/// Wire-level fields decoded from one clientbound entity-position-sync packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPositionSyncObservation {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
}

/// Wire-level fields decoded from one clientbound entity-teleport packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityTeleportObservation {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: i32,
    pub on_ground: bool,
    pub motion_before: Option<(f64, f64, f64)>,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
    pub applied_motion: Option<(f64, f64, f64)>,
}

/// Wire-level fields decoded from one clientbound relative entity-move packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMoveObservation {
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub y_rot: Option<f32>,
    pub x_rot: Option<f32>,
    pub has_position: bool,
    pub has_rotation: bool,
    pub on_ground: bool,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
}

pub(crate) fn parse_entity_position_sync_packet(
    input: &[u8],
) -> Result<EntityPositionSyncObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity position sync entity id")?;
    let x = read_f64(&mut input, "entity position sync x")?;
    let y = read_f64(&mut input, "entity position sync y")?;
    let z = read_f64(&mut input, "entity position sync z")?;
    let delta_x = read_f64(&mut input, "entity position sync delta x")?;
    let delta_y = read_f64(&mut input, "entity position sync delta y")?;
    let delta_z = read_f64(&mut input, "entity position sync delta z")?;
    let y_rot = read_f32(&mut input, "entity position sync y rotation")?;
    let x_rot = read_f32(&mut input, "entity position sync x rotation")?;
    let on_ground = read_byte(&mut input, "entity position sync on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec(
            "entity position sync trailing payload",
        ));
    }
    Ok(EntityPositionSyncObservation {
        entity_id,
        x,
        y,
        z,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        on_ground,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}

pub(crate) fn parse_entity_teleport_packet(
    input: &[u8],
) -> Result<EntityTeleportObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "entity teleport entity id")?;
    let x = read_f64(&mut input, "entity teleport x")?;
    let y = read_f64(&mut input, "entity teleport y")?;
    let z = read_f64(&mut input, "entity teleport z")?;
    let delta_x = read_f64(&mut input, "entity teleport delta x")?;
    let delta_y = read_f64(&mut input, "entity teleport delta y")?;
    let delta_z = read_f64(&mut input, "entity teleport delta z")?;
    let y_rot = read_f32(&mut input, "entity teleport y rotation")?;
    let x_rot = read_f32(&mut input, "entity teleport x rotation")?;
    let relative_flags = read_i32(&mut input, "entity teleport relative flags")?;
    let on_ground = read_byte(&mut input, "entity teleport on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("entity teleport trailing payload"));
    }
    Ok(EntityTeleportObservation {
        entity_id,
        x,
        y,
        z,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        relative_flags,
        on_ground,
        motion_before: None,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
        applied_motion: None,
    })
}

pub(crate) fn parse_move_entity_pos_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, true, false)
}

pub(crate) fn parse_move_entity_pos_rot_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, true, true)
}

pub(crate) fn parse_move_entity_rot_packet(
    input: &[u8],
) -> Result<EntityMoveObservation, ConnectionError> {
    parse_move_entity_packet(input, false, true)
}

pub(super) fn parse_move_entity_packet(
    input: &[u8],
    has_position: bool,
    has_rotation: bool,
) -> Result<EntityMoveObservation, ConnectionError> {
    let mut input = input;
    let entity_id = read_varint(&mut input, "move entity id")?;
    let (delta_x, delta_y, delta_z) = if has_position {
        (
            read_i16(&mut input, "move entity delta x")?,
            read_i16(&mut input, "move entity delta y")?,
            read_i16(&mut input, "move entity delta z")?,
        )
    } else {
        (0, 0, 0)
    };
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
    let on_ground = read_byte(&mut input, "move entity on ground")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("move entity trailing payload"));
    }
    Ok(EntityMoveObservation {
        entity_id,
        delta_x,
        delta_y,
        delta_z,
        y_rot,
        x_rot,
        has_position,
        has_rotation,
        on_ground,
        applied_to_world: false,
        applied_on_ground: None,
        applied_position: None,
        applied_rotation: None,
    })
}
