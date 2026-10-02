//! What the entity packets do to the world, for the probes and the live
//! session alike: the parsed packet goes in, whether the world took it comes
//! out. The probes add their own observations around these calls.

use voxel_world::{
    ClientWorld, EntityInput, EntityMotionInput, EntityMoveInput, EntityPositionInput,
    EntityPositionUpdateMode, EntityRotationInput, EntityTeleportInput,
};

use super::protocol::{
    AddEntityObservation, EntityHeadRotationObservation, EntityMetadataObservation,
    EntityMotionObservation, EntityMoveObservation, EntityPassengersObservation,
    EntityPositionSyncObservation, EntityTeleportObservation, RemoveEntitiesObservation,
};

/// `ClientPacketListener.handleAddEntity`.
pub(crate) fn add(world: &mut ClientWorld, packet: &AddEntityObservation) -> bool {
    world.apply_entity(EntityInput {
        entity_id: packet.entity_id,
        uuid: packet.uuid,
        type_id: packet.entity_type_id,
        x: packet.x,
        y: packet.y,
        z: packet.z,
        movement: packet.movement,
        rotation: (packet.y_rot, packet.x_rot, packet.y_head_rot),
        data: packet.data,
    })
}

/// `handleSetEntityMotion`.
pub(crate) fn motion(world: &mut ClientWorld, packet: &EntityMotionObservation) -> bool {
    world.apply_entity_motion(EntityMotionInput {
        entity_id: packet.entity_id,
        x: packet.x,
        y: packet.y,
        z: packet.z,
    })
}

/// `handleRotateMob`: only the head turns.
pub(crate) fn rotate_head(world: &mut ClientWorld, packet: &EntityHeadRotationObservation) -> bool {
    world.apply_entity_rotation(EntityRotationInput {
        entity_id: packet.entity_id,
        y_rot: None,
        x_rot: None,
        y_head_rot: Some(packet.y_head_rot),
        relative_y_rot: false,
        relative_x_rot: false,
    })
}

/// `handleRemoveEntities`: how many were there to remove.
pub(crate) fn remove(world: &mut ClientWorld, packet: &RemoveEntitiesObservation) -> usize {
    world.remove_entities(&packet.entity_ids)
}

/// `handleSetEntityPassengersPacket`.
pub(crate) fn passengers(world: &mut ClientWorld, packet: &EntityPassengersObservation) -> bool {
    world.apply_entity_passengers(packet.vehicle_id, &packet.passenger_ids)
}

/// `handleSetEntityData`.
pub(crate) fn metadata(world: &mut ClientWorld, packet: &EntityMetadataObservation) -> bool {
    world.apply_entity_metadata(packet.entity_id, &packet.metadata)
}

/// `handleEntityPositionSync`: the entity is put where the server says, and
/// told whether it stands on the ground.
pub(crate) fn position_sync(
    world: &mut ClientWorld,
    packet: &EntityPositionSyncObservation,
) -> bool {
    let applied = world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: packet.entity_id,
            x: packet.x,
            y: packet.y,
            z: packet.z,
            relative_flags: 0,
        },
        EntityRotationInput {
            entity_id: packet.entity_id,
            y_rot: Some(packet.y_rot),
            x_rot: Some(packet.x_rot),
            y_head_rot: None,
            relative_y_rot: false,
            relative_x_rot: false,
        },
        EntityPositionUpdateMode::PositionSync,
    );
    if applied {
        world.apply_entity_ground_state(packet.entity_id, packet.on_ground);
    }
    applied
}

/// `handleTeleportEntity`.
pub(crate) fn teleport(world: &mut ClientWorld, packet: &EntityTeleportObservation) -> bool {
    let applied = world.apply_entity_teleport(EntityTeleportInput {
        entity_id: packet.entity_id,
        x: packet.x,
        y: packet.y,
        z: packet.z,
        delta_x: packet.delta_x,
        delta_y: packet.delta_y,
        delta_z: packet.delta_z,
        y_rot: packet.y_rot,
        x_rot: packet.x_rot,
        relative_flags: packet.relative_flags,
    });
    if applied {
        world.apply_entity_ground_state(packet.entity_id, packet.on_ground);
    }
    applied
}

/// `handleMoveEntity`: a step, a turn or both.
pub(crate) fn step(world: &mut ClientWorld, packet: &EntityMoveObservation) -> bool {
    let applied = world.apply_entity_move(EntityMoveInput {
        entity_id: packet.entity_id,
        delta_x: packet.delta_x,
        delta_y: packet.delta_y,
        delta_z: packet.delta_z,
        has_position: packet.has_position,
        y_rot: packet.y_rot,
        x_rot: packet.x_rot,
    });
    if applied {
        world.apply_entity_ground_state(packet.entity_id, packet.on_ground);
    }
    applied
}
