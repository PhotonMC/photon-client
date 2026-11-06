use super::*;
use photon_core::ids::EntityId;

/// A packet the game mode sends, in order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outgoing {
    /// `ServerboundPlayerActionPacket`.
    PlayerAction {
        action: PlayerAction,
        x: i32,
        y: i32,
        z: i32,
        face: u8,
        sequence: i32,
    },
    /// `ServerboundPunchPacket`: the main hand attacked or kept mining.
    Punch,
    /// `ServerboundSwingPacket` (before 26.3): a hand swung after a use.
    Swing { hand: u8 },
    /// `ServerboundUseItemOnPacket`: the hit relative to the block.
    UseItemOn {
        hand: u8,
        x: i32,
        y: i32,
        z: i32,
        face: u8,
        cursor: [f32; 3],
        inside: bool,
        sequence: i32,
    },
    /// `ServerboundUseItemPacket`.
    UseItem {
        hand: u8,
        sequence: i32,
        y_rot: f32,
        x_rot: f32,
    },
    /// `ServerboundAttackPacket`.
    Attack { entity_id: EntityId },
    /// `ServerboundPickItemFromBlockPacket`.
    PickItemFromBlock {
        x: i32,
        y: i32,
        z: i32,
        include_data: bool,
    },
    /// `ServerboundPickItemFromEntityPacket`.
    PickItemFromEntity {
        entity_id: EntityId,
        include_data: bool,
    },
    /// `ServerboundSpectatorActionPacket`.
    SpectatorAction { entity_id: Option<EntityId> },
    /// `ServerboundInteractPacket`: the hit relative to the entity, and
    /// whether the secondary use (sneaking) is active.
    Interact {
        entity_id: EntityId,
        hand: u8,
        location: [f64; 3],
        secondary_use: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    StartDestroyBlock = 0,
    /// New in 26.3: the face under the cursor changed while a block is being broken.
    ChangeDestroyDirection = 1,
    AbortDestroyBlock = 2,
    StopDestroyBlock = 3,
    DropAllItems = 4,
    DropItem = 5,
    ReleaseUseItem = 6,
    SwapItemWithOffhand = 7,
    Stab = 8,
}

impl Outgoing {
    /// The packet's type, as the protocol names it.
    pub fn type_name(&self) -> &'static str {
        match self {
            Outgoing::PlayerAction { .. } => "minecraft:player_action",
            Outgoing::Punch | Outgoing::Swing { .. } => "minecraft:punch",
            Outgoing::UseItemOn { .. } => "minecraft:use_item_on",
            Outgoing::UseItem { .. } => "minecraft:use_item",
            Outgoing::Attack { .. } => "minecraft:attack",
            Outgoing::PickItemFromBlock { .. } => "minecraft:pick_item_from_block",
            Outgoing::PickItemFromEntity { .. } => "minecraft:pick_item_from_entity",
            Outgoing::SpectatorAction { .. } => "minecraft:spectator_action",
            Outgoing::Interact { .. } => "minecraft:interact",
        }
    }

    /// The packet body as its codec writes it, without the packet id.
    pub fn encode_body(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match *self {
            Outgoing::PlayerAction {
                action,
                x,
                y,
                z,
                face,
                sequence,
            } => {
                photon_core::wire::write_varint(action_wire(action), &mut out);
                out.extend_from_slice(
                    &photon_core::binary::BlockPos::new(x, y, z)
                        .as_long()
                        .to_be_bytes(),
                );
                out.push(face);
                photon_core::wire::write_varint(sequence, &mut out);
            }
            Outgoing::Punch | Outgoing::Swing { .. } => out.extend_from_slice(&swing_body(self)),
            Outgoing::UseItemOn {
                hand,
                x,
                y,
                z,
                face,
                cursor,
                inside,
                sequence,
            } => {
                photon_core::wire::write_varint(i32::from(hand), &mut out);
                out.extend_from_slice(
                    &photon_core::binary::BlockPos::new(x, y, z)
                        .as_long()
                        .to_be_bytes(),
                );
                photon_core::wire::write_varint(i32::from(face), &mut out);
                for value in cursor {
                    out.extend_from_slice(&value.to_be_bytes());
                }
                out.push(u8::from(inside));
                // Never a world border hit: the pick does not aim at it.
                out.push(0);
                photon_core::wire::write_varint(sequence, &mut out);
            }
            Outgoing::UseItem {
                hand,
                sequence,
                y_rot,
                x_rot,
            } => {
                photon_core::wire::write_varint(i32::from(hand), &mut out);
                photon_core::wire::write_varint(sequence, &mut out);
                out.extend_from_slice(&y_rot.to_be_bytes());
                out.extend_from_slice(&x_rot.to_be_bytes());
            }
            Outgoing::Attack { entity_id: id } => photon_core::wire::write_varint(id.0, &mut out),
            Outgoing::PickItemFromBlock {
                x,
                y,
                z,
                include_data,
            } => {
                out.extend_from_slice(
                    &photon_core::binary::BlockPos::new(x, y, z)
                        .as_long()
                        .to_be_bytes(),
                );
                out.push(u8::from(include_data));
            }
            Outgoing::PickItemFromEntity {
                entity_id,
                include_data,
            } => {
                photon_core::wire::write_varint(entity_id.0, &mut out);
                out.push(u8::from(include_data));
            }
            // `OptionalInt`: the value and one, or nothing as zero.
            Outgoing::SpectatorAction { entity_id } => {
                photon_core::wire::write_varint(entity_id.map_or(0, |id| id.0 + 1), &mut out);
            }
            Outgoing::Interact {
                entity_id,
                hand,
                location,
                secondary_use,
            } => {
                photon_core::wire::write_varint(entity_id.0, &mut out);
                photon_core::wire::write_varint(i32::from(hand), &mut out);
                photon_core::binary::write_lp_vec3(
                    Vec3::new(location[0], location[1], location[2]),
                    &mut out,
                );
                out.push(u8::from(secondary_use));
            }
        }
        out
    }
}

/// The number a player action goes out under: 26.3 inserted the direction change at 1.
fn action_wire(action: PlayerAction) -> i32 {
    let number = action as i32;
    if punch_is_swing() && number > 1 {
        number - 1
    } else {
        number
    }
}

/// Before 26.3 the packet is the swing of a hand.
pub fn punch_is_swing() -> bool {
    photon_core::versions::behaviour::punch_is_swing()
}

/// The body of a punch or a swing: a hand before 26.3, nothing after.
fn swing_body(packet: &Outgoing) -> Vec<u8> {
    match packet {
        Outgoing::Swing { hand } => {
            let mut out = Vec::new();
            photon_core::wire::write_varint(i32::from(*hand), &mut out);
            out
        }
        _ => punch_body(),
    }
}

/// The body of the punch packet: none in 26.3, the main hand (`ServerboundSwingPacket`) before.
pub fn punch_body() -> Vec<u8> {
    let mut out = Vec::new();
    if punch_is_swing() {
        photon_core::wire::write_varint(0, &mut out);
    }
    out
}
