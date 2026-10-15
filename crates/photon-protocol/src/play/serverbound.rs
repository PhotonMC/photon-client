use super::*;

/// `ServerboundSelectTradePacket`: the offer's place in the list.
pub fn encode_select_trade(trade: i32) -> Vec<u8> {
    let mut output = Vec::new();
    photon_core::wire::write_varint(trade, &mut output);
    output
}

/// `ServerboundContainerSlotStateChangedPacket`: the slot, the container and the new state.
pub fn encode_container_slot_state(slot: i32, container_id: i32, enabled: bool) -> Vec<u8> {
    let mut output = Vec::new();
    photon_core::wire::write_varint(slot, &mut output);
    photon_core::wire::write_varint(container_id, &mut output);
    output.push(u8::from(enabled));
    output
}

/// `ServerboundSetBeaconPacket`: each effect as a present flag and, when present, its registry id.
pub fn encode_set_beacon(primary: Option<i32>, secondary: Option<i32>) -> Vec<u8> {
    let mut output = Vec::new();
    for effect in [primary, secondary] {
        output.push(u8::from(effect.is_some()));
        if let Some(id) = effect {
            photon_core::wire::write_varint(id, &mut output);
        }
    }
    output
}

pub fn encode_move_player_pos_rot(
    output: &mut Vec<u8>,
    position: (f64, f64, f64, f32, f32),
    flags: u8,
) -> Result<(), ConnectionError> {
    output.extend_from_slice(&position.0.to_be_bytes());
    output.extend_from_slice(&position.1.to_be_bytes());
    output.extend_from_slice(&position.2.to_be_bytes());
    output.extend_from_slice(&position.3.to_be_bytes());
    output.extend_from_slice(&position.4.to_be_bytes());
    output.push(flags);
    Ok(())
}

pub fn encode_perform_respawn(output: &mut Vec<u8>) -> Result<(), ConnectionError> {
    output.push(0);
    Ok(())
}

pub fn encode_abort_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 1, position, 0, sequence)
}

pub fn encode_start_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 0, position, 1, sequence)
}

pub fn encode_stop_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 2, position, 1, sequence)
}

pub fn encode_release_use_item(output: &mut Vec<u8>) -> Result<(), ConnectionError> {
    encode_player_action(output, 5, (0, 0, 0), 0, 0)
}

pub fn encode_use_item_on(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_use_item_on_hit(output, position, 1, (0.5, 1.0, 0.5), sequence)
}

pub fn encode_use_item(
    output: &mut Vec<u8>,
    sequence: i32,
    y_rot: f32,
    x_rot: f32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(sequence, output);
    output.extend_from_slice(&y_rot.to_be_bytes());
    output.extend_from_slice(&x_rot.to_be_bytes());
    Ok(())
}

pub fn encode_use_item_on_hit(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    direction: u8,
    click: (f32, f32, f32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    output.extend_from_slice(&pack_block_position(position).to_be_bytes());
    wire::write_varint(i32::from(direction), output);
    output.extend_from_slice(&click.0.to_be_bytes());
    output.extend_from_slice(&click.1.to_be_bytes());
    output.extend_from_slice(&click.2.to_be_bytes());
    output.push(0);
    output.push(0);
    wire::write_varint(sequence, output);
    Ok(())
}

pub fn encode_set_carried_item(output: &mut Vec<u8>, slot: i16) -> Result<(), ConnectionError> {
    output.extend_from_slice(&slot.to_be_bytes());
    Ok(())
}

pub fn encode_player_action(
    output: &mut Vec<u8>,
    action: i32,
    position: (i32, i32, i32),
    direction: u8,
    sequence: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(action, output);
    output.extend_from_slice(&pack_block_position(position).to_be_bytes());
    output.push(direction);
    wire::write_varint(sequence, output);
    Ok(())
}

pub fn pack_block_position(position: (i32, i32, i32)) -> i64 {
    const HORIZONTAL_MASK: i64 = (1_i64 << 26) - 1;
    const Y_MASK: i64 = (1_i64 << 12) - 1;

    ((i64::from(position.0) & HORIZONTAL_MASK) << 38)
        | ((i64::from(position.2) & HORIZONTAL_MASK) << 12)
        | (i64::from(position.1) & Y_MASK)
}

pub fn support_block_position(position: (f64, f64, f64, f32, f32)) -> (i32, i32, i32) {
    let block = photon_core::binary::BlockPos::containing(position.0, position.1 - 1.0, position.2);
    (block.x, block.y, block.z)
}
