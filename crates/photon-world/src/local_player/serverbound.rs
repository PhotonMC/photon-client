use super::*;

/// A branch of the reference tick this port does not carry yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsupported(pub String);

impl std::fmt::Display for Unsupported {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "unported local player branch: {}", self.0)
    }
}

impl std::error::Error for Unsupported {}

/// A packet the tick asks the connection to send.
#[derive(Debug, Clone, Copy)]
pub enum ServerboundMovement {
    PlayerInput(Input),
    PlayerCommand {
        entity_id: EntityId,
        action: PlayerCommandAction,
        /// The action's datum: a riding jump's charge, in hundredths.
        data: i32,
    },
    PlayerAbilities {
        flying: bool,
    },
    MovePlayerPos {
        position: Vec3,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerPosRot {
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerRot {
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerStatusOnly {
        on_ground: bool,
        horizontal_collision: bool,
    },
    /// `ServerboundMoveVehiclePacket`: where the vehicle the player is the authority of is.
    MoveVehicle {
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
    },
    /// `ServerboundPaddleBoatPacket`: which paddles of the boat it rides row.
    PaddleBoat {
        left: bool,
        right: bool,
    },
}

impl ServerboundMovement {
    /// The packet type's resource name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::PlayerInput(_) => "minecraft:player_input",
            Self::PlayerCommand { .. } => "minecraft:player_command",
            Self::PlayerAbilities { .. } => "minecraft:player_abilities",
            Self::MovePlayerPos { .. } => "minecraft:move_player_pos",
            Self::MovePlayerPosRot { .. } => "minecraft:move_player_pos_rot",
            Self::MovePlayerRot { .. } => "minecraft:move_player_rot",
            Self::MovePlayerStatusOnly { .. } => "minecraft:move_player_status_only",
            Self::MoveVehicle { .. } => "minecraft:move_vehicle",
            Self::PaddleBoat { .. } => "minecraft:paddle_boat",
        }
    }

    /// The body the packet's codec writes, without the packet id.
    pub fn encode_body(&self) -> Vec<u8> {
        fn move_flags(on_ground: bool, horizontal_collision: bool) -> u8 {
            u8::from(on_ground) | (u8::from(horizontal_collision) << 1)
        }
        fn var_int(output: &mut Vec<u8>, value: i32) {
            let mut value = value as u32;
            loop {
                if value & !0x7f == 0 {
                    output.push(value as u8);
                    return;
                }
                output.push((value as u8 & 0x7f) | 0x80);
                value >>= 7;
            }
        }
        let mut output = Vec::new();
        match *self {
            Self::PlayerInput(input) => output.push(
                u8::from(input.forward)
                    | u8::from(input.backward) << 1
                    | u8::from(input.left) << 2
                    | u8::from(input.right) << 3
                    | u8::from(input.jump) << 4
                    | u8::from(input.shift) << 5
                    | u8::from(input.sprint) << 6,
            ),
            Self::PlayerCommand {
                entity_id,
                action,
                data,
            } => {
                var_int(&mut output, entity_id.0);
                var_int(&mut output, action as i32);
                var_int(&mut output, data);
            }
            Self::PlayerAbilities { flying } => output.push(if flying { 2 } else { 0 }),
            Self::MovePlayerPos {
                position,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerPosRot {
                position,
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerRot {
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerStatusOnly {
                on_ground,
                horizontal_collision,
            } => {
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MoveVehicle {
                position,
                y_rot,
                x_rot,
                on_ground,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(u8::from(on_ground));
            }
            Self::PaddleBoat { left, right } => {
                output.push(u8::from(left));
                output.push(u8::from(right));
            }
        }
        output
    }
}
