//! The client level's light engine: `LevelLightEngine` with its block and
//! sky engines, their section storage and each chunk's sky light sources.
//!
//! The server lights the world and sends the light with each chunk, but it
//! only sends light changes to players whose view ends at that chunk. Inside
//! the view the client relights on its own: every block update that changes
//! how light passes checks that block, and the engine spreads the change.
//!
//! Light is kept per section as nibble layers. The engine writes an
//! "updating" copy and publishes a "visible" copy after each run, which is
//! what the getters read. A layer may be shared: a section's queued data from
//! the server becomes its stored layer as the very same object, and a write
//! during the run shows in both. Layers therefore live in an arena and the
//! maps hold their ids, so sharing and "is it the same layer" behave as in
//! the reference.

use std::collections::{HashMap, HashSet, VecDeque};

use voxel_core::direction::Direction;

/// Section coordinates.
pub type SectionKey = (i32, i32, i32);
/// Block coordinates.
pub type BlockKey = (i32, i32, i32);
type ColumnKey = (i32, i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightLayer {
    Sky,
    Block,
}

/// `DataLayer`: 4096 nibbles, or a single value until something is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayer {
    data: Option<Box<[u8]>>,
    default_value: u8,
}

impl DataLayer {
    pub fn new(default_value: u8) -> Self {
        Self {
            data: None,
            default_value,
        }
    }

    /// A layer from the 2048 bytes the protocol carries.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: Some(bytes.to_vec().into_boxed_slice()),
            default_value: 0,
        }
    }

    fn index(x: i32, y: i32, z: i32) -> usize {
        (y << 8 | z << 4 | x) as usize
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        match &self.data {
            None => self.default_value,
            Some(data) => {
                let index = Self::index(x, y, z);
                (data[index >> 1] >> (4 * (index & 1))) & 15
            }
        }
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, value: u8) {
        let index = Self::index(x, y, z);
        let data = self.data_mut();
        let shift = 4 * (index & 1);
        data[index >> 1] = (data[index >> 1] & !(15 << shift)) | ((value & 15) << shift);
    }

    fn data_mut(&mut self) -> &mut [u8] {
        let default = self.default_value;
        self.data
            .get_or_insert_with(|| vec![default | default << 4; 2048].into_boxed_slice())
    }

    pub fn fill(&mut self, value: u8) {
        self.default_value = value;
        self.data = None;
    }

    pub fn is_definitely_homogenous(&self) -> bool {
        self.data.is_none()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none() && self.default_value == 0
    }

    /// `getData`: the bytes, allocated from the single value if needed.
    pub fn bytes(&mut self) -> &[u8] {
        self.data_mut()
    }
}

type LayerId = u32;

/// Layers by id, counted by how many maps hold them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Arena {
    layers: Vec<Option<(DataLayer, u32)>>,
    free: Vec<LayerId>,
}

impl Arena {
    fn alloc(&mut self, layer: DataLayer) -> LayerId {
        match self.free.pop() {
            Some(id) => {
                self.layers[id as usize] = Some((layer, 0));
                id
            }
            None => {
                self.layers.push(Some((layer, 0)));
                (self.layers.len() - 1) as LayerId
            }
        }
    }

    fn get(&self, id: LayerId) -> &DataLayer {
        &self.layers[id as usize].as_ref().expect("live layer").0
    }

    fn get_mut(&mut self, id: LayerId) -> &mut DataLayer {
        &mut self.layers[id as usize].as_mut().expect("live layer").0
    }

    fn retain(&mut self, id: LayerId) {
        self.layers[id as usize].as_mut().expect("live layer").1 += 1;
    }

    fn release(&mut self, id: LayerId) {
        let slot = self.layers[id as usize].as_mut().expect("live layer");
        slot.1 -= 1;
        if slot.1 == 0 {
            self.layers[id as usize] = None;
            self.free.push(id);
        }
    }

    /// A map let go of the layer; the caller decides whether it is freed.
    fn unhold(&mut self, id: LayerId) {
        self.layers[id as usize].as_mut().expect("live layer").1 -= 1;
    }

    /// Free a layer that no map holds any more.
    fn drop_if_unheld(&mut self, id: LayerId) {
        if self.layers[id as usize]
            .as_ref()
            .is_some_and(|slot| slot.1 == 0)
        {
            self.layers[id as usize] = None;
            self.free.push(id);
        }
    }
}

/// A map of section layers whose entries keep their layers alive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LayerMap {
    map: HashMap<SectionKey, LayerId>,
}

impl LayerMap {
    fn get(&self, key: SectionKey) -> Option<LayerId> {
        self.map.get(&key).copied()
    }

    fn set(&mut self, arena: &mut Arena, key: SectionKey, id: LayerId) {
        arena.retain(id);
        if let Some(old) = self.map.insert(key, id) {
            arena.release(old);
        }
    }

    fn remove(&mut self, arena: &mut Arena, key: SectionKey) -> Option<LayerId> {
        let id = self.map.remove(&key)?;
        // The caller may still want it: it stays alive until the caller
        // stores it again or drops it.
        arena.unhold(id);
        Some(id)
    }
}

/// `SkyDataLayerStorageMap`'s extra state: each column's top section with
/// light data, and the lowest section with any.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SkyTops {
    top_sections: HashMap<ColumnKey, i32>,
    current_lowest_y: i32,
}

impl SkyTops {
    fn new() -> Self {
        Self {
            top_sections: HashMap::new(),
            current_lowest_y: i32::MAX,
        }
    }

    fn top(&self, column: ColumnKey) -> i32 {
        self.top_sections
            .get(&column)
            .copied()
            .unwrap_or(self.current_lowest_y)
    }
}

const HAS_DATA_BIT: u8 = 32;
const NEIGHBOR_COUNT_BITS: u8 = 31;

/// `LayerLightSectionStorage` for one layer.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Storage {
    layer: LightLayer,
    arena: Arena,
    section_states: HashMap<SectionKey, u8>,
    columns_with_sources: HashSet<ColumnKey>,
    updating: LayerMap,
    visible: LayerMap,
    updating_sky: SkyTops,
    visible_sky: SkyTops,
    changed_sections: HashSet<SectionKey>,
    affected_sections: HashSet<SectionKey>,
    queued_sections: LayerMap,
    columns_to_retain: HashSet<ColumnKey>,
    to_remove: Vec<SectionKey>,
    has_inconsistencies: bool,
}

/// What the engine reads of the level.
pub trait LightChunks {
    /// Whether the chunk is loaded; the engine reads an unloaded one as
    /// bedrock.
    fn has_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool;
    /// A block state in a loaded chunk.
    fn block_state(&self, x: i32, y: i32, z: i32) -> i32;
    fn sky_sources(&self, chunk_x: i32, chunk_z: i32) -> Option<&ChunkSkyLightSources>;
}

/// `ChunkSkyLightSources`: per column, the lowest y the sky reaches
/// straight down to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSkyLightSources {
    min_y: i32,
    heights: Box<[i32]>,
}

mod entry {
    use voxel_core::direction::Direction;

    pub const DIRECTIONS_MASK: u64 = 1008;
    const FROM_EMPTY_SHAPE: u64 = 1024;
    const INCREASE_FROM_EMISSION: u64 = 2048;

    fn bit(direction: Direction) -> u64 {
        1 << (direction as u64 + 4)
    }

    fn with_level(entry: u64, level: u8) -> u64 {
        (entry & !15) | u64::from(level & 15)
    }

    pub fn decrease_skip_one_direction(level: u8, skip: Direction) -> u64 {
        with_level(DIRECTIONS_MASK & !bit(skip), level)
    }

    pub fn decrease_all_directions(level: u8) -> u64 {
        with_level(DIRECTIONS_MASK, level)
    }

    pub fn increase_light_from_emission(level: u8, from_empty_shape: bool) -> u64 {
        let mut entry = DIRECTIONS_MASK | INCREASE_FROM_EMISSION;
        if from_empty_shape {
            entry |= FROM_EMPTY_SHAPE;
        }
        with_level(entry, level)
    }

    pub fn increase_skip_one_direction(level: u8, from_empty_shape: bool, skip: Direction) -> u64 {
        let mut entry = DIRECTIONS_MASK & !bit(skip);
        if from_empty_shape {
            entry |= FROM_EMPTY_SHAPE;
        }
        with_level(entry, level)
    }

    pub fn increase_only_one_direction(
        level: u8,
        from_empty_shape: bool,
        direction: Direction,
    ) -> u64 {
        let mut entry = 0;
        if from_empty_shape {
            entry |= FROM_EMPTY_SHAPE;
        }
        with_level(entry | bit(direction), level)
    }

    pub fn increase_sky_source_in_directions(
        down: bool,
        north: bool,
        south: bool,
        west: bool,
        east: bool,
    ) -> u64 {
        let mut entry = with_level(0, 15);
        for (on, direction) in [
            (down, Direction::Down),
            (north, Direction::North),
            (south, Direction::South),
            (west, Direction::West),
            (east, Direction::East),
        ] {
            if on {
                entry |= bit(direction);
            }
        }
        entry
    }

    pub fn from_level(entry: u64) -> u8 {
        (entry & 15) as u8
    }

    pub fn is_from_empty_shape(entry: u64) -> bool {
        entry & FROM_EMPTY_SHAPE != 0
    }

    pub fn is_increase_from_emission(entry: u64) -> bool {
        entry & INCREASE_FROM_EMISSION != 0
    }

    pub fn propagates(entry: u64, direction: Direction) -> bool {
        entry & bit(direction) != 0
    }
}

const PROPAGATION_DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// `LightEngine` for one layer.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Engine {
    storage: Storage,
    /// `blockNodesToCheck`, in the order they were added.
    nodes_to_check: Vec<BlockKey>,
    nodes_seen: HashSet<BlockKey>,
    decrease_queue: VecDeque<(BlockKey, u64)>,
    increase_queue: VecDeque<(BlockKey, u64)>,
    level_min_y: i32,
    section_count: i32,
}

struct Level<'a> {
    chunks: &'a dyn LightChunks,
}

impl Level<'_> {
    fn state(&self, block: BlockKey) -> i32 {
        if self.chunks.has_chunk(block.0 >> 4, block.2 >> 4) {
            self.chunks.block_state(block.0, block.1, block.2)
        } else {
            bedrock()
        }
    }
}

/// `LevelLightEngine`: a block engine and, where the dimension has a sky, a
/// sky engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelLightEngine {
    block: Engine,
    sky: Option<Engine>,
    min_section_y: i32,
    section_count: i32,
}

impl LevelLightEngine {
    pub fn new(level_min_y: i32, section_count: i32, has_sky_light: bool) -> Self {
        let make = |layer| {
            let mut engine = Engine::new(layer, level_min_y);
            engine.section_count = section_count;
            engine
        };
        Self {
            block: make(LightLayer::Block),
            sky: has_sky_light.then(|| make(LightLayer::Sky)),
            min_section_y: level_min_y >> 4,
            section_count,
        }
    }

    /// `lightOnInColumn`: whether a chunk's light has been switched on in
    /// every layer the level has.
    pub fn light_on_in_column(&self, chunk_x: i32, chunk_z: i32) -> bool {
        let section = (chunk_x, 0, chunk_z);
        self.block.storage.light_on_in_section(section)
            && self
                .sky
                .as_ref()
                .is_none_or(|sky| sky.storage.light_on_in_section(section))
    }

    fn engines(&mut self) -> impl Iterator<Item = &mut Engine> {
        std::iter::once(&mut self.block).chain(self.sky.as_mut())
    }

    fn engine(&mut self, layer: LightLayer) -> Option<&mut Engine> {
        match layer {
            LightLayer::Block => Some(&mut self.block),
            LightLayer::Sky => self.sky.as_mut(),
        }
    }

    /// `getMinLightSection`: one below the level.
    pub fn min_light_section(&self) -> i32 {
        self.min_section_y - 1
    }

    /// `getLightSectionCount`: the level's sections and one either side.
    pub fn light_section_count(&self) -> i32 {
        self.section_count + 2
    }

    pub fn check_block(&mut self, block: BlockKey) {
        for engine in self.engines() {
            engine.check_block(block);
        }
    }

    pub fn has_light_work(&self) -> bool {
        self.sky.as_ref().is_some_and(Engine::has_light_work) || self.block.has_light_work()
    }

    /// `runLightUpdates`: the sections whose light changed, to redraw.
    pub fn run_light_updates(&mut self, chunks: &dyn LightChunks) -> Vec<SectionKey> {
        let level = Level { chunks };
        let mut affected = self.block.run(&level);
        if let Some(sky) = self.sky.as_mut() {
            affected.extend(sky.run(&level));
        }
        affected
    }

    pub fn update_section_status(&mut self, section: SectionKey, empty: bool) {
        for engine in self.engines() {
            engine.storage.update_section_status(section, empty);
        }
    }

    pub fn set_light_enabled(&mut self, chunks: &dyn LightChunks, chunk: (i32, i32), enable: bool) {
        let level = Level { chunks };
        for engine in self.engines() {
            engine.set_light_enabled(&level, chunk, enable);
        }
    }

    /// `propagateLightSources`: light a chunk from its own sources, as the
    /// server first lights it.
    pub fn propagate_light_sources(&mut self, chunks: &dyn LightChunks, chunk: (i32, i32)) {
        let level = Level { chunks };
        for engine in self.engines() {
            engine.propagate_light_sources(&level, chunk);
        }
    }

    pub fn queue_section_data(
        &mut self,
        layer: LightLayer,
        section: SectionKey,
        data: Option<DataLayer>,
    ) {
        if let Some(engine) = self.engine(layer) {
            engine.storage.queue_section_data(section, data);
        }
    }

    pub fn retain_data(&mut self, chunk: (i32, i32), retain: bool) {
        for engine in self.engines() {
            engine.storage.retain_data(chunk, retain);
        }
    }

    /// `getLayerListener(layer).getLightValue`.
    pub fn light_value(&self, layer: LightLayer, block: BlockKey) -> u8 {
        match layer {
            LightLayer::Block => self.block.storage.get_light_value(block),
            LightLayer::Sky => self
                .sky
                .as_ref()
                .map_or(0, |sky| sky.storage.get_light_value(block)),
        }
    }

    /// `getDataLayerData`: the section's published layer, or the one queued
    /// for it.
    pub fn data_layer(&self, layer: LightLayer, section: SectionKey) -> Option<&DataLayer> {
        let engine = match layer {
            LightLayer::Block => Some(&self.block),
            LightLayer::Sky => self.sky.as_ref(),
        }?;
        let storage = &engine.storage;
        storage
            .queued_sections
            .get(section)
            .or_else(|| storage.visible.get(section))
            .map(|id| storage.arena.get(id))
    }

    /// The sections with a published layer.
    pub fn visible_sections(&self, layer: LightLayer) -> Vec<SectionKey> {
        let engine = match layer {
            LightLayer::Block => Some(&self.block),
            LightLayer::Sky => self.sky.as_ref(),
        };
        engine.map_or_else(Vec::new, |engine| {
            engine.storage.visible.map.keys().copied().collect()
        })
    }
}

mod engine_sections;
mod engine_sources;

mod storage_section;

mod chunk_sky_light_sources_lowest;

mod shape;
pub use shape::*;
