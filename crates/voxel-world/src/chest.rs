//! The chests and shulker boxes the client draws (`ChestBlockEntity`, `EnderChestBlockEntity`,
//! `ShulkerBoxBlockEntity`): which blocks they are, the partner half of a double chest, and the
//! lid that opens and shuts as the server's block events say.

use std::collections::HashMap;
use std::sync::OnceLock;

use voxel_core::direction::Direction;
use voxel_data::block::classes::BlockClass;

use super::ClientWorld;

/// `ChestLidController`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ChestLid {
    should_be_open: bool,
    openness: f32,
    old_openness: f32,
}

impl ChestLid {
    /// `tickLid`.
    fn tick(&mut self) {
        self.old_openness = self.openness;
        if !self.should_be_open && self.openness > 0.0 {
            self.openness = (self.openness - 0.1).max(0.0);
        } else if self.should_be_open && self.openness < 1.0 {
            self.openness = (self.openness + 0.1).min(1.0);
        }
    }

    /// `getOpenness`: `Mth.lerp(a, old, now)`.
    fn openness(&self, partial: f32) -> f32 {
        self.old_openness + partial * (self.openness - self.old_openness)
    }
}

/// `ShulkerBoxBlockEntity`'s lid: how far the box is open, which a block event starts and the
/// tick carries on at a tenth a tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ShulkerLid {
    status: ShulkerStatus,
    progress: f32,
    progress_old: f32,
}

/// `ShulkerBoxBlockEntity.AnimationStatus`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ShulkerStatus {
    #[default]
    Closed,
    Opening,
    Opened,
    Closing,
}

impl ShulkerLid {
    /// `updateAnimation` (without the neighbour updates and the pushing of entities).
    fn tick(&mut self) {
        self.progress_old = self.progress;
        match self.status {
            ShulkerStatus::Closed => self.progress = 0.0,
            ShulkerStatus::Opening => {
                self.progress += 0.1;
                if self.progress >= 1.0 {
                    self.status = ShulkerStatus::Opened;
                    self.progress = 1.0;
                }
            }
            ShulkerStatus::Opened => self.progress = 1.0,
            ShulkerStatus::Closing => {
                self.progress -= 0.1;
                if self.progress <= 0.0 {
                    self.status = ShulkerStatus::Closed;
                    self.progress = 0.0;
                }
            }
        }
    }

    /// `triggerEvent` for event 1: a count of one opens the box, none shuts it.
    fn trigger(&mut self, count: u8) {
        if count == 0 {
            self.status = ShulkerStatus::Closing;
        }
        if count == 1 {
            self.status = ShulkerStatus::Opening;
        }
    }

    /// `getProgress`: `Mth.lerp(a, old, now)`.
    fn progress(&self, partial: f32) -> f32 {
        self.progress_old + partial * (self.progress - self.progress_old)
    }
}

/// A table of which blocks are of some classes, by the block's registry id.
fn is_of_classes(
    state_id: i32,
    classes: &[BlockClass],
    table: &'static OnceLock<Vec<bool>>,
) -> bool {
    let blocks = table.get_or_init(|| {
        voxel_data::block::registry::BLOCKS
            .iter()
            .map(|block| classes.iter().any(|class| block.class.is(*class)))
            .collect()
    });
    let block = crate::block::state_or_air(i64::from(state_id)).block;
    blocks.get(usize::from(block)).copied().unwrap_or(false)
}

/// Whether a block state is a chest's (the blocks `ChestBlock` and `EnderChestBlock` make).
pub(crate) fn is_chest_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(
        state_id,
        &[BlockClass::ChestBlock, BlockClass::EnderChestBlock],
        &BLOCKS,
    )
}

/// Whether a block state is a shulker box's.
pub(crate) fn is_shulker_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::ShulkerBoxBlock], &BLOCKS)
}

/// Whether a block state has a lid the client draws.
pub(crate) fn is_lidded_state(state_id: i32) -> bool {
    is_chest_state(state_id) || is_shulker_state(state_id)
}

/// The chest blocks of a chunk's sections.
pub(crate) fn chests_of(
    chunk_x: i32,
    chunk_z: i32,
    sections: &std::collections::BTreeMap<i32, crate::chunk::ClientSection>,
) -> std::collections::BTreeSet<(i32, i32, i32)> {
    let mut found = std::collections::BTreeSet::new();
    for (&section_y, section) in sections {
        if section.has_only_air() {
            continue;
        }
        for index in 0..4096 {
            if section.block_states.get(index).is_some_and(is_lidded_state) {
                found.insert((
                    chunk_x * 16 + (index % 16) as i32,
                    section_y * 16 + (index / 256) as i32,
                    chunk_z * 16 + (index / 16 % 16) as i32,
                ));
            }
        }
    }
    found
}

/// What a chest block is, for drawing it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChestView {
    /// The block's name (`minecraft:chest`, `minecraft:ender_chest`, a copper one…).
    pub name: &'static str,
    pub facing: Direction,
    /// `ChestType`: 0 single, 1 left, 2 right.
    pub kind: u8,
    /// The other half of a double chest.
    pub partner: Option<(i32, i32, i32)>,
}

/// What a shulker box block is, for drawing it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerBoxView {
    /// The dye of the box (`red`), none for the undyed one.
    pub color: Option<&'static str>,
    pub facing: Direction,
}

/// The lids the server has opened, by block.
#[derive(Debug, Clone, Default)]
pub(crate) struct ChestLids {
    pub(crate) chests: HashMap<(i32, i32, i32), ChestLid>,
    pub(crate) shulkers: HashMap<(i32, i32, i32), ShulkerLid>,
}

impl ChestLids {
    pub(crate) fn tick(&mut self) {
        for lid in self.chests.values_mut() {
            lid.tick();
        }
        for lid in self.shulkers.values_mut() {
            lid.tick();
        }
    }

    pub(crate) fn remove(&mut self, pos: &(i32, i32, i32)) {
        self.chests.remove(pos);
        self.shulkers.remove(pos);
    }
}

impl PartialEq for ChestLids {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for ChestLids {}

impl ClientWorld {
    /// The chest at a block, if it is one the client draws as a block entity.
    pub fn chest_view(&self, x: i32, y: i32, z: i32) -> Option<ChestView> {
        let state = self.block_state_at(x, y, z)?;
        let block = crate::block::state_or_air(i64::from(state)).block();
        let name = block.name;
        if !is_chest_state(state) {
            return None;
        }
        let property = |key| crate::local_player::property(Some(state), key);
        let facing = Direction::from_name(property("facing")?)?;
        let kind = match property("type") {
            Some("left") => 1,
            Some("right") => 2,
            _ => 0,
        };
        let partner = (kind != 0).then(|| {
            // `ChestBlock.getConnectedDirection`: the left half's partner is clockwise of the
            // facing, the right half's counter-clockwise.
            let toward = if kind == 1 {
                facing.clockwise_y()
            } else {
                facing.counter_clockwise_y()
            };
            let (dx, _, dz) = toward.expect("a chest faces sideways").step();
            (x + dx, y, z + dz)
        });
        // The partner has to be the same chest block, the other half, and face the same way.
        let partner = partner.filter(|&(px, py, pz)| {
            self.block_state_at(px, py, pz).is_some_and(|other| {
                let other_block = crate::block::state_or_air(i64::from(other)).block();
                let other_property = |key| crate::local_player::property(Some(other), key);
                other_block.name == name
                    && other_property("facing") == property("facing")
                    && matches!(
                        (kind, other_property("type")),
                        (1, Some("right")) | (2, Some("left"))
                    )
            })
        });
        Some(ChestView {
            name,
            facing,
            kind,
            partner,
        })
    }

    /// How far a chest's lid is open (`ChestBlock.opennessCombiner`): the more open of a double
    /// chest's halves.
    pub fn chest_openness(
        &self,
        pos: (i32, i32, i32),
        partner: Option<(i32, i32, i32)>,
        partial: f32,
    ) -> f32 {
        let own = |pos: (i32, i32, i32)| {
            self.chest_lids
                .chests
                .get(&pos)
                .map_or(0.0, |lid| lid.openness(partial))
        };
        match partner {
            Some(other) => own(pos).max(own(other)),
            None => own(pos),
        }
    }

    /// The shulker box at a block.
    pub fn shulker_box_view(&self, x: i32, y: i32, z: i32) -> Option<ShulkerBoxView> {
        let state = self.block_state_at(x, y, z)?;
        if !is_shulker_state(state) {
            return None;
        }
        let name = crate::block::state_or_air(i64::from(state)).block().name;
        // `minecraft:shulker_box` is undyed, the rest are `minecraft:<dye>_shulker_box`.
        let color = name
            .strip_prefix("minecraft:")
            .and_then(|name| name.strip_suffix("_shulker_box"));
        // `ShulkerBoxRenderer.extractRenderState`: a box without a facing faces up.
        let facing = crate::local_player::property(Some(state), "facing")
            .and_then(Direction::from_name)
            .unwrap_or(Direction::Up);
        Some(ShulkerBoxView { color, facing })
    }

    /// How far a shulker box is open (`getProgress`).
    pub fn shulker_box_progress(&self, pos: (i32, i32, i32), partial: f32) -> f32 {
        self.chest_lids
            .shulkers
            .get(&pos)
            .map_or(0.0, |lid| lid.progress(partial))
    }

    /// `ClientLevel.blockEvent` on a chest or a shulker box: event 1 opens the lid for a positive
    /// count, shuts it for none (a box opens for one only).
    pub fn apply_chest_block_event(&mut self, pos: (i32, i32, i32), event: u8, count: u8) -> bool {
        if event != 1 {
            return false;
        }
        if self.chest_view(pos.0, pos.1, pos.2).is_some() {
            self.chest_lids
                .chests
                .entry(pos)
                .or_default()
                .should_be_open = count > 0;
            return true;
        }
        if self.shulker_box_view(pos.0, pos.1, pos.2).is_some() {
            self.chest_lids
                .shulkers
                .entry(pos)
                .or_default()
                .trigger(count);
            return true;
        }
        false
    }

    /// The chests, double ones once each half, around a block column within a distance in chunks.
    pub fn chest_positions_near(&self, chunk: (i32, i32), radius: i32) -> Vec<(i32, i32, i32)> {
        let mut found = Vec::new();
        for chunk_x in chunk.0 - radius..=chunk.0 + radius {
            for chunk_z in chunk.1 - radius..=chunk.1 + radius {
                let Some(loaded) = self.chunk_for_query(chunk_x, chunk_z) else {
                    continue;
                };
                found.extend(loaded.chests.iter().copied());
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::{ChestLid, ShulkerLid};

    #[test]
    fn shulker_lid_opens_then_stays_open_and_closes() {
        let mut lid = ShulkerLid::default();
        lid.trigger(1);
        for _ in 0..9 {
            lid.tick();
        }
        assert!(lid.progress < 1.0);
        lid.tick();
        lid.tick();
        assert_eq!(lid.progress, 1.0);
        // Another opener changes nothing; the last one leaving closes it.
        lid.trigger(2);
        lid.tick();
        assert_eq!(lid.progress, 1.0);
        lid.trigger(0);
        for _ in 0..11 {
            lid.tick();
        }
        assert_eq!(lid.progress, 0.0);
    }

    #[test]
    fn lid_opens_a_tenth_a_tick_and_closes_the_same_way() {
        let mut lid = ChestLid {
            should_be_open: true,
            ..ChestLid::default()
        };
        for _ in 0..4 {
            lid.tick();
        }
        assert!((lid.openness - 0.4).abs() < 1e-6);
        assert!((lid.openness(0.5) - 0.35).abs() < 1e-6);
        lid.should_be_open = false;
        lid.tick();
        assert!((lid.openness - 0.3).abs() < 1e-6);
        for _ in 0..10 {
            lid.tick();
        }
        assert_eq!(lid.openness, 0.0);
    }
}
