//! `BlockStatePredictionHandler` and the block breaking progress a
//! `ClientLevel` keeps.
//!
//! While the client predicts an action, every block it changes remembers the
//! state the server last gave it. Server updates to such a block only replace
//! that remembered state; the block itself keeps the prediction until the
//! server acknowledges the action's sequence number, when the remembered
//! state is put back if it differs. A block that was in the player at the
//! time of the prediction moves the player back to where it stood then,
//! unless a teleport came after the prediction started.

use photon_core::ids::BlockStateId;
use photon_core::ids::EntityId;
use std::collections::BTreeMap;

use photon_core::math::Vec3;

use super::ClientWorld;

type Key = (i32, i32, i32);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BlockPredictions {
    server_verified: BTreeMap<Key, ServerVerified>,
    current_sequence: i32,
    predicting: bool,
    last_teleport_sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ServerVerified {
    sequence: i32,
    state: BlockStateId,
    /// Where the player stood, as `f64` bits.
    player_position: [u64; 3],
}

impl BlockPredictions {
    pub(crate) fn new() -> Self {
        Self {
            last_teleport_sequence: -1,
            ..Self::default()
        }
    }
}

/// A block the server's acknowledgement put back, and where the player stood
/// when it was predicted if the player should be moved back out of it.
#[derive(Debug, Clone, Copy)]
pub struct SyncedBlock {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub state: BlockStateId,
    pub player_position: Option<Vec3>,
}

/// One entity's breaking progress on a block, as `BlockDestructionProgress`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DestroyProgress {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// The crack stage, 0 to 9.
    pub stage: i32,
}

impl ClientWorld {
    /// `startPredicting`: the next sequence number, now predicting.
    pub fn start_predicting(&mut self) -> i32 {
        self.predictions.current_sequence += 1;
        self.predictions.predicting = true;
        self.predictions.current_sequence
    }

    /// The prediction scope closes.
    pub fn stop_predicting(&mut self) {
        self.predictions.predicting = false;
    }

    /// The sequence number of the last prediction.
    pub fn prediction_sequence(&self) -> i32 {
        self.predictions.current_sequence
    }

    /// `onTeleport`: predictions made before now do not move the player back.
    pub fn prediction_teleport(&mut self) {
        self.predictions.last_teleport_sequence = self.predictions.current_sequence;
    }

    /// `ClientLevel.setBlock` as the client's own action changes a block.
    /// While predicting, a change remembers the state it replaced. Returns
    /// whether the block changed.
    pub fn set_block_predicted(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        state: BlockStateId,
        player_position: Vec3,
    ) -> bool {
        let old = self.block_state_at(x, y, z);
        let changed = self.update_block_state(x, y, z, state);
        if changed && self.predictions.predicting {
            // `retainKnownServerState`: a block already remembered only
            // takes the newer sequence number.
            let sequence = self.predictions.current_sequence;
            self.predictions
                .server_verified
                .entry((x, y, z))
                .and_modify(|verified| verified.sequence = sequence)
                .or_insert(ServerVerified {
                    sequence,
                    state: old.unwrap_or(crate::constants::VANILLA_AIR_BLOCK_STATE_ID),
                    player_position: [
                        player_position.x.to_bits(),
                        player_position.y.to_bits(),
                        player_position.z.to_bits(),
                    ],
                });
        }
        changed
    }

    /// `Level.setBlock` from the client's own action: the block changes as a
    /// prediction, then, unless `flags` holds 16, its neighbours reshape to
    /// it. `limit` bounds how far the reshaping reaches, as the reference's
    /// update limit does.
    pub fn set_block_with_updates(
        &mut self,
        (x, y, z): (i32, i32, i32),
        state: BlockStateId,
        flags: u32,
        limit: i32,
        player_position: Vec3,
    ) -> bool {
        if !self.set_block_predicted(x, y, z, state, player_position) {
            return false;
        }
        if flags & 16 == 0 && limit > 0 {
            self.update_neighbour_shapes((x, y, z), flags & !33, limit - 1, player_position);
        }
        true
    }

    /// `updateNeighbourShapes`: each neighbour, in the reference's order,
    /// reshapes to the changed block through `updateOrDestroy`. A neighbour
    /// whose class's rule is not written keeps its state.
    fn update_neighbour_shapes(
        &mut self,
        pos: (i32, i32, i32),
        flags: u32,
        limit: i32,
        player_position: Vec3,
    ) {
        use photon_core::direction::Direction;
        for direction in [
            Direction::West,
            Direction::East,
            Direction::North,
            Direction::South,
            Direction::Down,
            Direction::Up,
        ] {
            let (dx, dy, dz) = direction.step();
            let neighbour = (pos.0 + dx, pos.1 + dy, pos.2 + dz);
            let Some(current) = self.block_state_at(neighbour.0, neighbour.1, neighbour.2) else {
                continue;
            };
            let read = |x: i32, y: i32, z: i32| self.block_state_or_air(x, y, z);
            let updated = match crate::block::rules::update_shape(
                current,
                &read,
                neighbour,
                direction.opposite(),
            ) {
                crate::block::rules::Rule::Known(updated) => updated,
                crate::block::rules::Rule::Unported => continue,
            };
            // A client never destroys a block that reshapes into air.
            if updated != current && !crate::block::rules::state_is_air(updated) {
                self.set_block_with_updates(
                    neighbour,
                    updated,
                    flags & !32,
                    limit,
                    player_position,
                );
            }
        }
    }

    /// `setServerVerifiedBlockState`: a block update from the server. A block
    /// with a prediction pending keeps it and only remembers the state.
    /// Returns whether the block itself changed.
    pub fn apply_server_block_state(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        state: BlockStateId,
    ) -> bool {
        if let Some(verified) = self.predictions.server_verified.get_mut(&(x, y, z)) {
            verified.state = state;
            return false;
        }
        self.update_block_state(x, y, z, state)
    }

    /// `handleBlockChangedAck`: every prediction up to `sequence` ends, and
    /// the remembered states go back in.
    pub fn handle_block_changed_ack(&mut self, sequence: i32) -> Vec<SyncedBlock> {
        let ended: Vec<(Key, ServerVerified)> = self
            .predictions
            .server_verified
            .iter()
            .filter(|(_, verified)| verified.sequence <= sequence)
            .map(|(key, verified)| (*key, *verified))
            .collect();
        let snap = self.predictions.last_teleport_sequence < sequence;
        let mut synced = Vec::new();
        for ((x, y, z), verified) in ended {
            self.predictions.server_verified.remove(&(x, y, z));
            // `syncBlockState`: only a block that differs is set.
            if self.block_state_at(x, y, z) != Some(verified.state)
                && self.update_block_state(x, y, z, verified.state)
            {
                synced.push(SyncedBlock {
                    x,
                    y,
                    z,
                    state: verified.state,
                    player_position: snap.then(|| {
                        let [x, y, z] = verified.player_position.map(f64::from_bits);
                        Vec3::new(x, y, z)
                    }),
                });
            }
        }
        synced
    }

    /// `ClientLevel.destroyBlockProgress`: a stage from 0 to 9 sets an
    /// entity's crack, anything else clears it.
    pub fn destroy_block_progress(
        &mut self,
        entity_id: EntityId,
        x: i32,
        y: i32,
        z: i32,
        stage: i32,
    ) {
        if (0..10).contains(&stage) {
            self.destroying_blocks
                .insert(entity_id, DestroyProgress { x, y, z, stage });
        } else {
            self.destroying_blocks.remove(&entity_id);
        }
    }

    /// Each entity's crack, by entity id.
    pub fn destroying_blocks(&self) -> &BTreeMap<EntityId, DestroyProgress> {
        &self.destroying_blocks
    }
}
