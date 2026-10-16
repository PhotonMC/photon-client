//! What the entity packets do to the world, for the probes and the live
//! session alike: the parsed packet goes in, whether the world took it comes
//! out. The probes add their own observations around these calls.

use photon_core::ids::EntityId;
use photon_world::{
    ClientWorld, EntityInput, EntityMotionInput, EntityMoveInput, EntityPositionInput,
    EntityPositionUpdateMode, EntityRotationInput, EntityTeleportInput, EquipmentSlot,
};

use photon_world::objects::minecart_track::MinecartStep;

use photon_protocol::play::{
    AddEntityObservation, EntityEquipmentObservation, EntityHeadRotationObservation,
    EntityMetadataObservation, EntityMotionObservation, EntityMoveObservation,
    EntityPassengersObservation, EntityPositionSyncObservation, EntityTeleportObservation,
    RemoveEntitiesObservation,
};

/// `ClientPacketListener.handleAddEntity`.
pub fn add(world: &mut ClientWorld, packet: &AddEntityObservation) -> bool {
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
pub fn motion(world: &mut ClientWorld, packet: &EntityMotionObservation) -> bool {
    world.entities_mut().apply_motion(EntityMotionInput {
        entity_id: packet.entity_id,
        x: packet.x,
        y: packet.y,
        z: packet.z,
    })
}

/// `handleMinecartAlongTrack`.
pub fn minecart_steps(
    world: &mut ClientWorld,
    entity_id: EntityId,
    steps: &[MinecartStep],
) -> bool {
    world.apply_minecart_steps(entity_id, steps)
}

/// `handleRotateMob`: only the head turns.
pub fn rotate_head(world: &mut ClientWorld, packet: &EntityHeadRotationObservation) -> bool {
    world.apply_entity_rotation(EntityRotationInput {
        entity_id: packet.entity_id,
        y_rot: None,
        x_rot: None,
        y_head_rot: Some(packet.y_head_rot),
        relative_y_rot: false,
        relative_x_rot: false,
    })
}

/// `handleSetEquipment`: each slot the packet names, in order; a slot the
/// game has no name for is left alone.
pub fn equipment(world: &mut ClientWorld, packet: &EntityEquipmentObservation) -> bool {
    let mut applied = false;
    for (ordinal, stack) in &packet.slots {
        if let Some(slot) = EquipmentSlot::from_ordinal(*ordinal) {
            applied |= world
                .entities_mut()
                .apply_equipment(packet.entity_id, slot, stack.clone());
        }
    }
    applied
}

/// `handleRemoveEntities`: how many were there to remove.
pub fn remove(world: &mut ClientWorld, packet: &RemoveEntitiesObservation) -> usize {
    world.remove_entities(&packet.entity_ids)
}

/// `handleSetEntityPassengersPacket`.
pub fn passengers(world: &mut ClientWorld, packet: &EntityPassengersObservation) -> bool {
    world.apply_entity_passengers(packet.vehicle_id, &packet.passenger_ids)
}

/// `ItemStack`'s id among the entity data serializers, and the place of a
/// dropped item's stack among its data (`ItemEntity.DATA_ITEM`).
const ITEM_STACK_SERIALIZER_ID: i32 = photon_data::entity_type::serializer_named("ITEM_STACK");
const ITEM_ENTITY_STACK_DATA_ID: u8 = 8;

/// `handleSetEntityData`, and for a dropped item the stack it holds.
pub fn metadata(world: &mut ClientWorld, packet: &EntityMetadataObservation) -> bool {
    let applied = world.apply_entity_metadata(packet.entity_id, &packet.metadata);
    // A dropped item, a thrown item or a firework rocket keeps its stack in the same data.
    let is_item = world.entities().at(packet.entity_id).is_some_and(|entity| {
        photon_data::entity_type::id("minecraft:item") == Some(entity.type_id)
            || entity.thrown_item_look().is_some()
            || photon_data::entity_type::id("minecraft:firework_rocket") == Some(entity.type_id)
    });
    // An item frame keeps its item after the direction `HangingEntity` has.
    let is_frame = world
        .entities()
        .at(packet.entity_id)
        .is_some_and(|entity| entity.is_item_frame());
    let is_item_display =
        photon_data::entity_type::id("minecraft:item_display").is_some_and(|id| {
            world
                .entities()
                .at(packet.entity_id)
                .is_some_and(|entity| entity.type_id == id)
        });
    let stack_data_id = if is_frame {
        photon_world::ITEM_FRAME_ITEM_DATA_ID
    } else if is_item_display {
        photon_world::objects::display::ITEM_STACK_DATA_ID
    } else {
        ITEM_ENTITY_STACK_DATA_ID
    };
    if applied
        && (is_item || is_frame || is_item_display)
        && let Some((ITEM_STACK_SERIALIZER_ID, value)) =
            photon_world::entity_metadata_field(&packet.metadata, stack_data_id)
    {
        world.entities_mut().apply_item_stack(
            packet.entity_id,
            photon_protocol::play::parse_entity_item_stack(value),
        );
    }
    applied
}

/// `handleEntityPositionSync`: the entity is put where the server says, and
/// told whether it stands on the ground.
pub fn position_sync(world: &mut ClientWorld, packet: &EntityPositionSyncObservation) -> bool {
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
pub fn teleport(world: &mut ClientWorld, packet: &EntityTeleportObservation) -> bool {
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
pub fn step(world: &mut ClientWorld, packet: &EntityMoveObservation) -> bool {
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
