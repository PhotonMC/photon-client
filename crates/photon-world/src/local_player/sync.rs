use super::*;

impl LocalPlayer {
    /// `LocalPlayer.sendChanges` for a player on foot (26.3): the input and the position, sent at
    /// the end of the client tick, after the entities and the block entities have moved the player.
    /// (26.2 sent them from the player's own tick, so there is nothing left to send here.)
    pub fn send_changes(&mut self) -> Vec<ServerboundMovement> {
        if sends_in_own_tick() {
            return Vec::new();
        }
        self.sent.clear();
        self.send_changes_on_foot();
        std::mem::take(&mut self.sent)
    }

    pub(super) fn send_changes_on_foot(&mut self) {
        if self.last_sent_input != self.input.key_presses {
            self.sent
                .push(ServerboundMovement::PlayerInput(self.input.key_presses));
            self.last_sent_input = self.input.key_presses;
        }
        self.send_position();
    }

    /// What `LocalPlayer.sendChanges` (26.2: the end of `LocalPlayer.tick`) tells of a passenger:
    /// the input, its look, and, when it is the vehicle's authority, where the vehicle is.
    pub(super) fn send_changes_riding(&mut self, boat: &crate::VehicleView) {
        if self.last_sent_input != self.input.key_presses {
            self.sent
                .push(ServerboundMovement::PlayerInput(self.input.key_presses));
            self.last_sent_input = self.input.key_presses;
        }
        self.sent.push(ServerboundMovement::MovePlayerRot {
            y_rot: self.y_rot,
            x_rot: self.x_rot,
            on_ground: self.on_ground,
            horizontal_collision: self.horizontal_collision,
        });
        self.send_vehicle_move(boat);
        if boat.controlled && self.sprinting != self.was_sprinting {
            self.sent.push(ServerboundMovement::PlayerCommand {
                entity_id: self.entity_id,
                action: if self.sprinting {
                    PlayerCommandAction::StartSprinting
                } else {
                    PlayerCommandAction::StopSprinting
                },
                data: 0,
            });
            self.was_sprinting = self.sprinting;
        }
    }
}

/// Whether the player tells the server of itself at the end of its own tick (26.2) and not at the
/// end of the client tick (26.3).
pub(super) fn sends_in_own_tick() -> bool {
    photon_core::versions::rules().game.player_sends_in_own_tick
}
