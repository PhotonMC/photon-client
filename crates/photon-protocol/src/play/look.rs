//! Packets that turn the player or move its vehicle.

use super::*;
use photon_core::ids::EntityId;

/// `ClientboundPlayerRotationPacket`: a view the server sets, each angle absolute or relative.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerRotationPacket {
    pub y_rot: f32,
    pub relative_y: bool,
    pub x_rot: f32,
    pub relative_x: bool,
}

impl Decode for PlayerRotationPacket {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let packet = Self {
            y_rot: read_f32(input, "player rotation y")?,
            relative_y: read_bool(input, "player rotation relative y")?,
            x_rot: read_f32(input, "player rotation x")?,
            relative_x: read_bool(input, "player rotation relative x")?,
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec("player rotation trailing payload"));
        }
        Ok(packet)
    }
}

/// The view the player ends up with after a rotation packet (`PositionMoveRotation
/// .calculateAbsolute` with only the rotation changed): a relative yaw adds to the current one,
/// the pitch is kept within a quarter turn.
pub fn resolve_player_rotation(packet: PlayerRotationPacket, current: (f32, f32)) -> (f32, f32) {
    let y_rot = if packet.relative_y {
        current.0 + packet.y_rot
    } else {
        packet.y_rot
    };
    let x_rot = if packet.relative_x {
        current.1 + packet.x_rot
    } else {
        packet.x_rot
    };
    (y_rot, x_rot.clamp(-90.0, 90.0))
}

/// `EntityAnchorArgument.Anchor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnchor {
    Feet,
    Eyes,
}

fn read_anchor(input: &mut &[u8], name: &'static str) -> Result<EntityAnchor, ConnectionError> {
    match read_varint(input, name)? {
        0 => Ok(EntityAnchor::Feet),
        1 => Ok(EntityAnchor::Eyes),
        _ => Err(ConnectionError::Codec(name)),
    }
}

/// `ClientboundPlayerLookAtPacket`: a point to look at, which follows an entity when it names one
/// the level holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerLookAtPacket {
    pub from: EntityAnchor,
    pub at: (f64, f64, f64),
    /// The entity to look at and which of its anchors.
    pub entity: Option<(EntityId, EntityAnchor)>,
}

impl Decode for PlayerLookAtPacket {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let from = read_anchor(input, "look at anchor")?;
        let at = (
            read_f64(input, "look at x")?,
            read_f64(input, "look at y")?,
            read_f64(input, "look at z")?,
        );
        let entity = if read_bool(input, "look at entity flag")? {
            let id = EntityId(read_varint(input, "look at entity")?);
            Some((id, read_anchor(input, "look at entity anchor")?))
        } else {
            None
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec("look at trailing payload"));
        }
        Ok(Self { from, at, entity })
    }
}

/// `ClientboundMoveVehiclePacket`: where the server puts the vehicle the player rides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveVehiclePacket {
    pub position: (f64, f64, f64),
    pub y_rot: f32,
    pub x_rot: f32,
}

impl Decode for MoveVehiclePacket {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let packet = Self {
            position: (
                read_f64(input, "move vehicle x")?,
                read_f64(input, "move vehicle y")?,
                read_f64(input, "move vehicle z")?,
            ),
            y_rot: read_f32(input, "move vehicle y rotation")?,
            x_rot: read_f32(input, "move vehicle x rotation")?,
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec("move vehicle trailing payload"));
        }
        Ok(packet)
    }
}

/// `ClientboundProjectilePowerPacket`: a hurting projectile's acceleration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectilePowerPacket {
    pub entity: EntityId,
    pub acceleration_power: f64,
}

impl Decode for ProjectilePowerPacket {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let packet = Self {
            entity: EntityId(read_varint(input, "projectile power entity")?),
            acceleration_power: read_f64(input, "projectile power")?,
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec("projectile power trailing payload"));
        }
        Ok(packet)
    }
}
