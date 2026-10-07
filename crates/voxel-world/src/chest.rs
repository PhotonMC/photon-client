//! The block entities the client draws (`ChestBlockEntity`, `EnderChestBlockEntity`,
//! `ShulkerBoxBlockEntity`, `BellBlockEntity`, `SkullBlockEntity`): which blocks they are, the partner half of a double
//! chest, and the lid or the clapper that moves as the server's block events say.

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

/// The dyes, in `DyeColor.getId` order.
pub const DYES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

/// `BellBlockEntity`: the shake a ring starts and the tick wears away.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct BellShake {
    ticks: i32,
    shaking: bool,
    direction: Option<Direction>,
}

impl BellShake {
    /// `tick`, without the raiders' resonance.
    fn tick(&mut self) {
        if self.shaking {
            self.ticks += 1;
        }
        if self.ticks >= 50 {
            self.shaking = false;
            self.ticks = 0;
        }
    }

    /// `triggerEvent` for event 1: the direction of the blow, in `Direction.from3DDataValue`.
    fn ring(&mut self, direction: u8) {
        self.direction = Some(Direction::from_3d_data_value(i32::from(direction)));
        self.ticks = 0;
        self.shaking = true;
    }
}

/// `SkullBlockEntity`'s animation: counts the ticks a powered dragon or piglin head has been on.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct SkullAnimation {
    ticks: i32,
    animating: bool,
}

impl SkullAnimation {
    /// `SkullBlockEntity.animation`.
    fn tick(&mut self, powered: bool) {
        self.animating = powered;
        if powered {
            self.ticks += 1;
        }
    }

    /// `getAnimation`.
    fn animation(&self, partial: f32) -> f32 {
        if self.animating {
            self.ticks as f32 + partial
        } else {
            self.ticks as f32
        }
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

/// Whether a block state is a bell's.
pub(crate) fn is_bell_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::BellBlock], &BLOCKS)
}

/// Whether a block state is a skull's or a head's.
pub(crate) fn is_skull_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::AbstractSkullBlock], &BLOCKS)
}

/// Whether a block state is a banner's.
pub(crate) fn is_banner_state(state_id: i32) -> bool {
    static BLOCKS: OnceLock<Vec<bool>> = OnceLock::new();
    is_of_classes(state_id, &[BlockClass::AbstractBannerBlock], &BLOCKS)
}

/// `BannerBlockEntity.loadAdditional`: the layers of a `patterns` list, a layer without a pattern
/// id or a colour left out.
fn banner_patterns(tag: &[u8]) -> Vec<(String, String)> {
    use voxel_core::nbt::FieldValue;
    let mut input = tag;
    let Ok(Some(fields)) = voxel_core::nbt::read_nbt(&mut input) else {
        return Vec::new();
    };
    let Some(FieldValue::List(layers)) = fields
        .iter()
        .find(|field| field.key == "patterns")
        .map(|field| &field.value)
    else {
        return Vec::new();
    };
    let text = |fields: &[voxel_core::nbt::Field], key: &str| {
        fields.iter().find_map(|field| match &field.value {
            FieldValue::Str(value) if field.key == key => Some(value.clone()),
            _ => None,
        })
    };
    layers
        .iter()
        .filter_map(|layer| {
            let FieldValue::Compound(layer) = layer else {
                return None;
            };
            Some((text(layer, "pattern")?, text(layer, "color")?))
        })
        .collect()
}

/// Whether a block state has a block entity the client draws.
pub(crate) fn is_drawn_state(state_id: i32) -> bool {
    is_chest_state(state_id)
        || is_shulker_state(state_id)
        || is_bell_state(state_id)
        || is_skull_state(state_id)
        || is_banner_state(state_id)
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
            if section.block_states.get(index).is_some_and(is_drawn_state) {
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

/// What a banner block is, for drawing it.
#[derive(Debug, Clone, PartialEq)]
pub struct BannerView {
    /// The dye of the banner's cloth (`red`).
    pub color: &'static str,
    /// The wall a wall banner hangs on's facing, none for a standing one.
    pub wall: Option<Direction>,
    /// `BannerBlock.ROTATION` of a standing one.
    pub rotation: i32,
    /// `BannerBlockEntity.getPatterns`: each layer's pattern (a registry id) and dye, bottom first.
    pub patterns: Vec<(String, String)>,
}

/// What a skull or head block is, for drawing it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkullBlockView {
    /// The block's name (`minecraft:zombie_head`, `minecraft:dragon_wall_head`…).
    pub name: &'static str,
    /// The wall a wall skull hangs on's facing, none for a standing one.
    pub wall: Option<Direction>,
    /// `SkullBlock.ROTATION` of a standing one.
    pub rotation: i32,
    /// `SkullBlockEntity.getAnimation`.
    pub animation: f32,
}

/// The lids the server has opened, by block.
#[derive(Debug, Clone, Default)]
pub(crate) struct BlockAnimations {
    pub(crate) chests: HashMap<(i32, i32, i32), ChestLid>,
    pub(crate) shulkers: HashMap<(i32, i32, i32), ShulkerLid>,
    pub(crate) bells: HashMap<(i32, i32, i32), BellShake>,
    pub(crate) skulls: HashMap<(i32, i32, i32), SkullAnimation>,
}

impl BlockAnimations {
    pub(crate) fn tick(&mut self) {
        for lid in self.chests.values_mut() {
            lid.tick();
        }
        for lid in self.shulkers.values_mut() {
            lid.tick();
        }
        for bell in self.bells.values_mut() {
            bell.tick();
        }
    }

    pub(crate) fn remove(&mut self, pos: &(i32, i32, i32)) {
        self.chests.remove(pos);
        self.shulkers.remove(pos);
        self.bells.remove(pos);
        self.skulls.remove(pos);
    }
}

impl PartialEq for BlockAnimations {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for BlockAnimations {}

#[cfg(test)]
mod tests {
    use voxel_core::direction::Direction;

    use super::{BellShake, ChestLid, ShulkerLid, SkullAnimation};

    #[test]
    fn skull_counts_only_while_powered() {
        let mut skull = SkullAnimation::default();
        skull.tick(true);
        skull.tick(true);
        assert_eq!(skull.animation(0.5), 2.5);
        skull.tick(false);
        assert_eq!(skull.animation(0.5), 2.0);
    }

    #[test]
    fn banner_layers_come_from_the_patterns_list() {
        use voxel_core::nbt::{Field, FieldValue};
        let layer = |pattern: &str, color: &str| {
            FieldValue::Compound(vec![
                Field {
                    key: "pattern".to_owned(),
                    value: FieldValue::Str(pattern.to_owned()),
                },
                Field {
                    key: "color".to_owned(),
                    value: FieldValue::Str(color.to_owned()),
                },
            ])
        };
        let mut tag = Vec::new();
        voxel_core::nbt::write_nbt(
            Some(&[Field {
                key: "patterns".to_owned(),
                value: FieldValue::List(vec![
                    layer("minecraft:cross", "red"),
                    FieldValue::Compound(Vec::new()),
                ]),
            }]),
            &mut tag,
        );
        assert_eq!(
            super::banner_patterns(&tag),
            [("minecraft:cross".to_owned(), "red".to_owned())]
        );
        assert!(super::banner_patterns(&[]).is_empty());
    }

    #[test]
    fn bell_swings_for_fifty_ticks_from_the_blow() {
        let mut bell = BellShake::default();
        bell.tick();
        assert!(!bell.shaking && bell.ticks == 0);
        bell.ring(2);
        assert_eq!(bell.direction, Some(Direction::North));
        for _ in 0..49 {
            bell.tick();
        }
        assert!(bell.shaking && bell.ticks == 49);
        bell.tick();
        assert!(!bell.shaking && bell.ticks == 0);
    }

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

mod client_world_chest;
