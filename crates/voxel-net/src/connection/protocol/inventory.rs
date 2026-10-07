use super::*;

pub(crate) fn inventory_component_spec(
    component_path: &str,
) -> Option<&'static InventoryComponentProbeSpec> {
    let component_path = component_path
        .strip_prefix("minecraft:")
        .unwrap_or(component_path);
    VANILLA_ITEM_COMPONENT_CONFORMANCE
        .iter()
        .find(|spec| spec.component_path == component_path)
}

pub(crate) fn encode_move_player_pos_rot(
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

pub(crate) fn encode_perform_respawn(output: &mut Vec<u8>) -> Result<(), ConnectionError> {
    output.push(0);
    Ok(())
}

pub(crate) fn encode_abort_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 1, position, 0, sequence)
}

pub(crate) fn encode_start_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 0, position, 1, sequence)
}

pub(crate) fn encode_stop_destroy_block(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_player_action(output, 2, position, 1, sequence)
}

pub(crate) fn encode_release_use_item(output: &mut Vec<u8>) -> Result<(), ConnectionError> {
    encode_player_action(output, 5, (0, 0, 0), 0, 0)
}

pub(crate) fn encode_use_item_on(
    output: &mut Vec<u8>,
    position: (i32, i32, i32),
    sequence: i32,
) -> Result<(), ConnectionError> {
    encode_use_item_on_hit(output, position, 1, (0.5, 1.0, 0.5), sequence)
}

pub(crate) fn encode_use_item(
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

pub(crate) fn encode_use_item_on_hit(
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

pub(crate) fn encode_creative_stone_slot(
    output: &mut Vec<u8>,
    slot: i16,
    stone_item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_creative_item_slot(output, slot, 1, stone_item_registry_id)
}

pub(crate) fn encode_creative_item_slot(
    output: &mut Vec<u8>,
    slot: i16,
    count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    output.extend_from_slice(&slot.to_be_bytes());
    wire::write_varint(count, output);
    wire::write_varint(item_registry_id, output);
    wire::write_varint(0, output);
    wire::write_varint(0, output);
    Ok(())
}

/// `ServerboundSetCreativeModeSlotPacket`: a slot and its stack as the
/// network encodes an optional item stack for the server, component patch
/// included, each value behind its length.
pub(crate) fn encode_creative_slot(
    output: &mut Vec<u8>,
    slot: i16,
    stack: Option<&ClientItemStackState>,
) {
    output.extend_from_slice(&slot.to_be_bytes());
    match stack {
        Some(stack) if stack.item_count() > 0 => {
            wire::write_varint(stack.item_count(), output);
            wire::write_varint(stack.item_registry_id(), output);
            // The server reads a client's components behind their lengths. A
            // patch this cannot read is left off rather than sent wrong.
            match voxel_game::item_components::delimit(stack.patch_bytes()) {
                Some(patch) if !stack.patch_bytes().is_empty() => output.extend_from_slice(&patch),
                _ => {
                    wire::write_varint(0, output);
                    wire::write_varint(0, output);
                }
            }
        }
        _ => wire::write_varint(0, output),
    }
}

pub(crate) fn encode_inventory_pickup_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(0, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_secondary_pickup_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(1);
    wire::write_varint(0, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_throw_one_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(4, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_throw_all_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(1);
    wire::write_varint(4, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_drag_empty_cancel_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&(-999_i16).to_be_bytes());
    output.push(0);
    wire::write_varint(5, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_click_with_input(
    output: &mut Vec<u8>,
    state_id: i32,
    slot: i16,
    button: u8,
    input_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&slot.to_be_bytes());
    output.push(button);
    wire::write_varint(input_id, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_pickup_with_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction(output, state_id, None, Some((item_count, item_registry_id)))
}

pub(crate) fn encode_inventory_pickup_with_wrong_slot_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction(
        output,
        state_id,
        Some((item_count, item_registry_id)),
        Some((item_count, item_registry_id)),
    )
}

pub(crate) fn encode_inventory_pickup_with_wrong_carried_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction(output, state_id, None, None)
}

pub(crate) fn encode_inventory_pickup_with_wrong_both_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction(output, state_id, Some((item_count, item_registry_id)), None)
}

pub(crate) fn encode_inventory_pickup_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    changed_slot_item: Option<(i32, i32)>,
    carried_item: Option<(i32, i32)>,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction_with_carried_patch(
        output,
        state_id,
        changed_slot_item,
        carried_item,
        &[],
        &[],
    )
}

pub(crate) fn encode_inventory_pickup_with_component_hash_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    let spec = inventory_component_spec("custom_model_data")
        .expect("custom model data component is in the conformance table");
    let added_components = [(spec.component_type_id, 0x0102_0304)];
    let removed_components = [spec.component_type_id];
    encode_inventory_pickup_prediction_with_carried_patch(
        output,
        state_id,
        None,
        Some((item_count, item_registry_id)),
        &added_components,
        &removed_components,
    )
}

pub(crate) fn encode_inventory_pickup_with_component_spec_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    spec: &'static InventoryComponentProbeSpec,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction_with_carried_patch(
        output,
        state_id,
        None,
        Some((1, spec.item_registry_id)),
        spec.carried_components,
        &[],
    )
}

pub(crate) fn encode_inventory_pickup_with_partial_changed_slot_prediction(
    output: &mut Vec<u8>,
    state_id: i32,
    item_count: i32,
    item_registry_id: i32,
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction_packet(
        output,
        state_id,
        None,
        Some((item_count, item_registry_id)),
        &[],
        &[],
    )
}

pub(crate) fn encode_inventory_pickup_prediction_with_carried_patch(
    output: &mut Vec<u8>,
    state_id: i32,
    changed_slot_item: Option<(i32, i32)>,
    carried_item: Option<(i32, i32)>,
    carried_added_components: &[(i32, i32)],
    carried_removed_components: &[i32],
) -> Result<(), ConnectionError> {
    encode_inventory_pickup_prediction_packet(
        output,
        state_id,
        Some(changed_slot_item),
        carried_item,
        carried_added_components,
        carried_removed_components,
    )
}

pub(crate) fn encode_inventory_pickup_prediction_packet(
    output: &mut Vec<u8>,
    state_id: i32,
    changed_slot_item: Option<Option<(i32, i32)>>,
    carried_item: Option<(i32, i32)>,
    carried_added_components: &[(i32, i32)],
    carried_removed_components: &[i32],
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(0, output);
    if let Some(changed_slot_item) = changed_slot_item {
        wire::write_varint(1, output);
        output.extend_from_slice(&36_i16.to_be_bytes());
        encode_hashed_stack(output, changed_slot_item);
    } else {
        wire::write_varint(0, output);
    }
    encode_hashed_stack_with_components(
        output,
        carried_item,
        carried_added_components,
        carried_removed_components,
    );
    Ok(())
}

pub(crate) fn encode_hashed_stack(output: &mut Vec<u8>, item: Option<(i32, i32)>) {
    encode_hashed_stack_with_components(output, item, &[], &[]);
}

pub(crate) fn encode_hashed_stack_with_components(
    output: &mut Vec<u8>,
    item: Option<(i32, i32)>,
    added_components: &[(i32, i32)],
    removed_components: &[i32],
) {
    let Some((item_count, item_registry_id)) = item else {
        output.push(0);
        return;
    };

    output.push(1);
    wire::write_varint(item_registry_id, output);
    wire::write_varint(item_count, output);
    wire::write_varint(added_components.len() as i32, output);
    for (component_type_id, component_hash) in added_components {
        wire::write_varint(*component_type_id, output);
        output.extend_from_slice(&component_hash.to_be_bytes());
    }
    wire::write_varint(removed_components.len() as i32, output);
    for component_type_id in removed_components {
        wire::write_varint(*component_type_id, output);
    }
}

pub(crate) fn encode_inventory_quick_move_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&36_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(1, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}

pub(crate) fn encode_inventory_swap_state_correction(
    output: &mut Vec<u8>,
    state_id: i32,
) -> Result<(), ConnectionError> {
    wire::write_varint(0, output);
    wire::write_varint(state_id, output);
    output.extend_from_slice(&9_i16.to_be_bytes());
    output.push(0);
    wire::write_varint(2, output);
    wire::write_varint(0, output);
    output.push(0);
    Ok(())
}
