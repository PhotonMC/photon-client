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

fn column(key: SectionKey) -> ColumnKey {
    (key.0, key.2)
}

fn section_of(block: BlockKey) -> SectionKey {
    (block.0 >> 4, block.1 >> 4, block.2 >> 4)
}

fn local(block: BlockKey) -> (i32, i32, i32) {
    (block.0 & 15, block.1 & 15, block.2 & 15)
}

impl Storage {
    fn new(layer: LightLayer) -> Self {
        Self {
            layer,
            arena: Arena::default(),
            section_states: HashMap::new(),
            columns_with_sources: HashSet::new(),
            updating: LayerMap::default(),
            visible: LayerMap::default(),
            updating_sky: SkyTops::new(),
            visible_sky: SkyTops::new(),
            changed_sections: HashSet::new(),
            affected_sections: HashSet::new(),
            queued_sections: LayerMap::default(),
            columns_to_retain: HashSet::new(),
            to_remove: Vec::new(),
            has_inconsistencies: false,
        }
    }

    fn storing_light_for_section(&self, key: SectionKey) -> bool {
        self.updating.get(key).is_some()
    }

    fn visible_layer(&self, key: SectionKey) -> Option<&DataLayer> {
        self.visible.get(key).map(|id| self.arena.get(id))
    }

    /// `getDataLayerToWrite`: the updating layer, copied on its first
    /// change of the run.
    fn data_layer_to_write(&mut self, key: SectionKey) -> Option<LayerId> {
        let id = self.updating.get(key)?;
        if self.changed_sections.insert(key) {
            let copy = self.arena.alloc(self.arena.get(id).clone());
            self.updating.set(&mut self.arena, key, copy);
            return Some(copy);
        }
        Some(id)
    }

    fn get_stored_level(&self, block: BlockKey) -> u8 {
        let id = self
            .updating
            .get(section_of(block))
            .expect("stored light read where no section is stored");
        let (x, y, z) = local(block);
        self.arena.get(id).get(x, y, z)
    }

    fn set_stored_level(&mut self, block: BlockKey, level: u8) {
        let key = section_of(block);
        let id = if self.changed_sections.insert(key) {
            // `copyDataLayer`.
            let old = self.updating.get(key).expect("stored section");
            let copy = self.arena.alloc(self.arena.get(old).clone());
            self.updating.set(&mut self.arena, key, copy);
            copy
        } else {
            self.updating.get(key).expect("stored section")
        };
        let (x, y, z) = local(block);
        self.arena.get_mut(id).set(x, y, z, level);
        // `SectionPos.aroundAndAtBlockPos`.
        let (bx, by, bz) = block;
        for sx in (bx - 1) >> 4..=(bx + 1) >> 4 {
            for sy in (by - 1) >> 4..=(by + 1) >> 4 {
                for sz in (bz - 1) >> 4..=(bz + 1) >> 4 {
                    self.affected_sections.insert((sx, sy, sz));
                }
            }
        }
    }

    fn mark_section_and_neighbors_as_affected(&mut self, key: SectionKey) {
        for dz in -1..=1 {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    self.affected_sections
                        .insert((key.0 + dx, key.1 + dy, key.2 + dz));
                }
            }
        }
    }

    /// `createDataLayer`: the queued layer itself if there is one.
    fn create_data_layer(&mut self, key: SectionKey) -> LayerId {
        if let Some(queued) = self.queued_sections.get(key) {
            return queued;
        }
        match self.layer {
            LightLayer::Block => self.arena.alloc(DataLayer::new(0)),
            LightLayer::Sky => {
                let top = self.updating_sky.top(column(key));
                if top != self.updating_sky.current_lowest_y && key.1 < top {
                    let mut above = (key.0, key.1 + 1, key.2);
                    let above_id = loop {
                        if let Some(id) = self.updating.get(above) {
                            break id;
                        }
                        above.1 += 1;
                    };
                    let layer = repeat_first_layer(self.arena.get(above_id));
                    self.arena.alloc(layer)
                } else if self.light_on_in_section(key) {
                    self.arena.alloc(DataLayer::new(15))
                } else {
                    self.arena.alloc(DataLayer::new(0))
                }
            }
        }
    }

    fn mark_new_inconsistencies(&mut self) {
        if !self.has_inconsistencies {
            return;
        }
        self.has_inconsistencies = false;
        let to_remove = std::mem::take(&mut self.to_remove);
        for &node in &to_remove {
            let queued = self.queued_sections.remove(&mut self.arena, node);
            let stored = self.updating.remove(&mut self.arena, node);
            if self.columns_to_retain.contains(&column(node)) {
                if let Some(queued) = queued {
                    self.queued_sections.set(&mut self.arena, node, queued);
                } else if let Some(stored) = stored {
                    self.queued_sections.set(&mut self.arena, node, stored);
                }
            }
            for id in [queued, stored].into_iter().flatten() {
                self.arena.drop_if_unheld(id);
            }
        }
        for &node in &to_remove {
            self.on_node_removed(node);
            self.changed_sections.insert(node);
        }
        let queued: Vec<(SectionKey, LayerId)> = self
            .queued_sections
            .map
            .iter()
            .map(|(key, id)| (*key, *id))
            .collect();
        for (key, id) in queued {
            if self.storing_light_for_section(key) {
                if self.updating.get(key) != Some(id) {
                    self.updating.set(&mut self.arena, key, id);
                    self.changed_sections.insert(key);
                }
                if let Some(id) = self.queued_sections.remove(&mut self.arena, key) {
                    self.arena.drop_if_unheld(id);
                }
            }
        }
    }

    fn on_node_added(&mut self, key: SectionKey) {
        if self.layer != LightLayer::Sky {
            return;
        }
        let y = key.1;
        if self.updating_sky.current_lowest_y > y {
            self.updating_sky.current_lowest_y = y;
        }
        let old_top = self.updating_sky.top(column(key));
        if old_top < y + 1 {
            self.updating_sky.top_sections.insert(column(key), y + 1);
        }
    }

    fn on_node_removed(&mut self, key: SectionKey) {
        if self.layer != LightLayer::Sky {
            return;
        }
        let col = column(key);
        let mut y = key.1;
        if self.updating_sky.top(col) == y + 1 {
            let mut top = key;
            while !self.storing_light_for_section(top) && self.has_light_data_at_or_below(y) {
                y -= 1;
                top.1 -= 1;
            }
            if self.storing_light_for_section(top) {
                self.updating_sky.top_sections.insert(col, y + 1);
            } else {
                self.updating_sky.top_sections.remove(&col);
            }
        }
    }

    fn set_light_enabled(&mut self, col: ColumnKey, enable: bool) {
        if enable {
            self.columns_with_sources.insert(col);
        } else {
            self.columns_with_sources.remove(&col);
        }
    }

    fn light_on_in_section(&self, key: SectionKey) -> bool {
        self.columns_with_sources.contains(&column(key))
    }

    fn retain_data(&mut self, col: ColumnKey, retain: bool) {
        if retain {
            self.columns_to_retain.insert(col);
        } else {
            self.columns_to_retain.remove(&col);
        }
    }

    fn queue_section_data(&mut self, key: SectionKey, data: Option<DataLayer>) {
        match data {
            Some(data) => {
                let id = self.arena.alloc(data);
                self.queued_sections.set(&mut self.arena, key, id);
                self.has_inconsistencies = true;
            }
            None => {
                if let Some(id) = self.queued_sections.remove(&mut self.arena, key) {
                    self.arena.drop_if_unheld(id);
                }
            }
        }
    }

    fn update_section_status(&mut self, key: SectionKey, empty: bool) {
        let state = self.section_states.get(&key).copied().unwrap_or(0);
        let new_state = if empty {
            state & !HAS_DATA_BIT
        } else {
            state | HAS_DATA_BIT
        };
        if state == new_state {
            return;
        }
        self.put_section_state(key, new_state);
        let increment: i32 = if empty { -1 } else { 1 };
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let neighbor = (key.0 + dx, key.1 + dy, key.2 + dz);
                    let neighbor_state = self.section_states.get(&neighbor).copied().unwrap_or(0);
                    let count = i32::from(neighbor_state & NEIGHBOR_COUNT_BITS) + increment;
                    assert!(
                        (0..=26).contains(&count),
                        "Neighbor count was not within range [0; 26]"
                    );
                    self.put_section_state(
                        neighbor,
                        (neighbor_state & !NEIGHBOR_COUNT_BITS) | count as u8,
                    );
                }
            }
        }
    }

    fn put_section_state(&mut self, key: SectionKey, state: u8) {
        if state != 0 {
            if self.section_states.insert(key, state).unwrap_or(0) == 0 {
                self.initialize_section(key);
            }
        } else if self.section_states.remove(&key).unwrap_or(0) != 0 {
            self.to_remove.push(key);
            self.has_inconsistencies = true;
        }
    }

    fn initialize_section(&mut self, key: SectionKey) {
        if let Some(position) = self.to_remove.iter().position(|node| *node == key) {
            self.to_remove.remove(position);
            return;
        }
        let id = self.create_data_layer(key);
        self.updating.set(&mut self.arena, key, id);
        self.changed_sections.insert(key);
        self.on_node_added(key);
        self.mark_section_and_neighbors_as_affected(key);
        self.has_inconsistencies = true;
    }

    /// `swapSectionMap`: publish the updating layers; the sections whose
    /// light changed come back to be redrawn.
    fn swap_section_map(&mut self) -> Vec<SectionKey> {
        if !self.changed_sections.is_empty() {
            let old: Vec<LayerId> = self.visible.map.values().copied().collect();
            self.visible.map = self.updating.map.clone();
            for id in self.visible.map.values() {
                self.arena.retain(*id);
            }
            for id in old {
                self.arena.release(id);
            }
            self.visible_sky = self.updating_sky.clone();
            self.changed_sections.clear();
        }
        self.affected_sections.drain().collect()
    }

    fn has_light_data_at_or_below(&self, section_y: i32) -> bool {
        section_y >= self.updating_sky.current_lowest_y
    }

    fn is_above_data(&self, key: SectionKey) -> bool {
        let top = self.updating_sky.top(column(key));
        top == self.updating_sky.current_lowest_y || key.1 >= top
    }

    /// The level a getter reads, from the published layers.
    fn get_light_value(&self, block: BlockKey) -> u8 {
        let key = section_of(block);
        let (x, y, z) = local(block);
        match self.layer {
            LightLayer::Block => self
                .visible_layer(key)
                .map_or(0, |layer| layer.get(x, y, z)),
            LightLayer::Sky => {
                let top = self.visible_sky.top(column(key));
                if top != self.visible_sky.current_lowest_y && key.1 < top {
                    let mut section = key;
                    let mut local_y = y;
                    loop {
                        if let Some(layer) = self.visible_layer(section) {
                            return layer.get(x, local_y, z);
                        }
                        // The flat index: the bottom row of the layer above.
                        local_y = 0;
                        section.1 += 1;
                        if section.1 >= top {
                            return 15;
                        }
                    }
                }
                15
            }
        }
    }
}

fn repeat_first_layer(layer: &DataLayer) -> DataLayer {
    if layer.is_definitely_homogenous() {
        return layer.clone();
    }
    let input = layer.data.as_ref().expect("not homogenous");
    let mut output = vec![0_u8; 2048];
    for i in 0..16 {
        output[i * 128..(i + 1) * 128].copy_from_slice(&input[..128]);
    }
    DataLayer::from_bytes(&output)
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

fn light_traits(state: i32) -> &'static voxel_data::block::light::LightTraits {
    let (index, offset) =
        voxel_data::block::registry::block_index_for_state(state as u32).unwrap_or((0, 0));
    let ids = voxel_data::block::light::BLOCKS[index];
    let id = ids[if ids.len() == 1 { 0 } else { offset as usize }];
    &voxel_data::block::light::TRAITS[usize::from(id)]
}

fn emission(state: i32) -> u8 {
    light_traits(state).emission
}

fn dampening(state: i32) -> u8 {
    light_traits(state).dampening
}

/// `LightEngine.isEmptyShape`: light ignores the shape of a block that
/// cannot occlude or does not use its shape for light.
fn is_empty_shape(state: i32) -> bool {
    let traits = light_traits(state);
    !traits.can_occlude || !traits.use_shape_for_light_occlusion
}

/// `LightEngine.getOcclusionShape`, as a face shape kind.
fn occlusion_shape(state: i32, direction: Direction) -> u8 {
    if is_empty_shape(state) {
        0
    } else {
        crate::face_culling::occlusion_kind(state as u32, direction).unwrap_or(0)
    }
}

/// `Shapes.faceShapeOccludes`.
fn face_shape_occludes(shape: u8, occluder: u8) -> bool {
    voxel_data::block::face_occlusion::COVERS[usize::from(shape)].as_bytes()[usize::from(occluder)]
        == b'1'
}

/// `LightEngine.hasDifferentLightProperties`.
pub fn has_different_light_properties(old: i32, new: i32) -> bool {
    if old == new {
        return false;
    }
    let (a, b) = (light_traits(old), light_traits(new));
    b.dampening != a.dampening
        || b.emission != a.emission
        || b.use_shape_for_light_occlusion
        || a.use_shape_for_light_occlusion
}

/// `ChunkSkyLightSources.isEdgeOccluded`.
fn is_edge_occluded(top: i32, bottom: i32) -> bool {
    dampening(bottom) != 0
        || face_shape_occludes(
            occlusion_shape(top, Direction::Down),
            occlusion_shape(bottom, Direction::Up),
        )
}

fn bedrock() -> i32 {
    static STATE: std::sync::OnceLock<i32> = std::sync::OnceLock::new();
    *STATE.get_or_init(|| {
        voxel_data::block::registry::block("minecraft:bedrock")
            .map_or(0, |block| block.default_state_id() as i32)
    })
}

/// `ChunkSkyLightSources`: per column, the lowest y the sky reaches
/// straight down to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSkyLightSources {
    min_y: i32,
    heights: Box<[i32]>,
}

impl ChunkSkyLightSources {
    /// Every column open to below the world, as a new chunk starts.
    pub fn new(level_min_y: i32) -> Self {
        let min_y = level_min_y - 1;
        Self {
            min_y,
            heights: vec![min_y; 256].into_boxed_slice(),
        }
    }

    fn index(x: i32, z: i32) -> usize {
        (x + z * 16) as usize
    }

    /// `fillFrom`: from the chunk's sections, the highest first. `section`
    /// answers a section's block states (y, z, x order, 4096 of them) by
    /// index, or nothing if it holds only air.
    pub fn fill_from(&mut self, section_count: usize, section: impl Fn(usize) -> Option<Vec<i32>>) {
        let sections: Vec<Option<Vec<i32>>> = (0..section_count).map(&section).collect();
        let Some(highest) = sections.iter().rposition(Option::is_some) else {
            self.heights.fill(self.min_y);
            return;
        };
        for z in 0..16 {
            for x in 0..16 {
                let found = self.find_lowest_source_y(&sections, highest, x, z);
                self.heights[Self::index(x, z)] = found.max(self.min_y);
            }
        }
    }

    fn find_lowest_source_y(
        &self,
        sections: &[Option<Vec<i32>>],
        top: usize,
        x: i32,
        z: i32,
    ) -> i32 {
        let level_min_y = self.min_y + 1;
        let section_min = |index: usize| level_min_y + index as i32 * 16;
        let mut top_y = section_min(top) + 16;
        let mut top_state = 0;
        for index in (0..=top).rev() {
            match &sections[index] {
                None => {
                    top_state = 0;
                    top_y = section_min(index);
                }
                Some(states) => {
                    for y in (0..16).rev() {
                        let bottom_state = states[(y * 256 + z * 16 + x) as usize];
                        if is_edge_occluded(top_state, bottom_state) {
                            return top_y;
                        }
                        top_state = bottom_state;
                        top_y -= 1;
                    }
                }
            }
        }
        self.min_y
    }

    /// `update` after the block at local `x`, `z` and height `y` changed.
    /// `state` answers the chunk's block at a local position and height.
    pub fn update(&mut self, state: impl Fn(i32, i32, i32) -> i32, x: i32, y: i32, z: i32) -> bool {
        let index = Self::index(x, z);
        let current = self.heights[index];
        if y + 1 < current {
            return false;
        }
        let top = state(x, y + 1, z);
        let middle = state(x, y, z);
        if self.update_edge(
            &state,
            index,
            current,
            (x, y + 1, z),
            top,
            (x, y, z),
            middle,
        ) {
            return true;
        }
        let bottom = state(x, y - 1, z);
        self.update_edge(
            &state,
            index,
            current,
            (x, y, z),
            middle,
            (x, y - 1, z),
            bottom,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn update_edge(
        &mut self,
        state: &impl Fn(i32, i32, i32) -> i32,
        index: usize,
        old_top_edge_y: i32,
        top_pos: (i32, i32, i32),
        top_state: i32,
        bottom_pos: (i32, i32, i32),
        bottom_state: i32,
    ) -> bool {
        let checked = top_pos.1;
        if is_edge_occluded(top_state, bottom_state) {
            if checked > old_top_edge_y {
                self.heights[index] = checked;
                return true;
            }
        } else if checked == old_top_edge_y {
            self.heights[index] = self.find_lowest_source_below(state, bottom_pos, bottom_state);
            return true;
        }
        false
    }

    fn find_lowest_source_below(
        &self,
        state: &impl Fn(i32, i32, i32) -> i32,
        start: (i32, i32, i32),
        start_state: i32,
    ) -> i32 {
        let (x, mut top_y, z) = start;
        let mut top_state = start_state;
        while top_y - 1 >= self.min_y {
            let bottom_state = state(x, top_y - 1, z);
            if is_edge_occluded(top_state, bottom_state) {
                return top_y;
            }
            top_state = bottom_state;
            top_y -= 1;
        }
        self.min_y
    }

    /// `getLowestSourceY`: below the world reads as negative infinity.
    pub fn lowest_source_y(&self, x: i32, z: i32) -> i32 {
        self.extend(self.heights[Self::index(x, z)])
    }

    fn highest_lowest_source_y(&self) -> i32 {
        self.extend(self.heights.iter().copied().max().unwrap_or(self.min_y))
    }

    fn extend(&self, value: i32) -> i32 {
        if value == self.min_y { i32::MIN } else { value }
    }
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

fn offset(block: BlockKey, direction: Direction) -> BlockKey {
    let (dx, dy, dz) = direction.step();
    (block.0 + dx, block.1 + dy, block.2 + dz)
}

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

impl Engine {
    fn new(layer: LightLayer, level_min_y: i32) -> Self {
        Self {
            storage: Storage::new(layer),
            nodes_to_check: Vec::new(),
            nodes_seen: HashSet::new(),
            decrease_queue: VecDeque::new(),
            increase_queue: VecDeque::new(),
            level_min_y,
            section_count: 0,
        }
    }

    fn opacity(state: i32) -> u8 {
        dampening(state).max(1)
    }

    fn shape_occludes(from: i32, to: i32, direction: Direction) -> bool {
        face_shape_occludes(
            occlusion_shape(from, direction),
            occlusion_shape(to, direction.opposite()),
        )
    }

    fn check_block(&mut self, block: BlockKey) {
        if self.nodes_seen.insert(block) {
            self.nodes_to_check.push(block);
        }
    }

    fn run(&mut self, level: &Level<'_>) -> Vec<SectionKey> {
        let nodes = std::mem::take(&mut self.nodes_to_check);
        self.nodes_seen.clear();
        for node in nodes {
            self.check_node(level, node);
        }
        while let Some((node, data)) = self.decrease_queue.pop_front() {
            self.propagate_decrease(level, node, data);
        }
        while let Some((from, data)) = self.increase_queue.pop_front() {
            let mut from_level = self.storage.get_stored_level(from);
            let target = entry::from_level(data);
            if entry::is_increase_from_emission(data) && from_level < target {
                self.storage.set_stored_level(from, target);
                from_level = target;
            }
            if from_level == target {
                self.propagate_increase(level, from, data, from_level);
            }
        }
        self.storage.mark_new_inconsistencies();
        self.storage.swap_section_map()
    }

    fn has_light_work(&self) -> bool {
        self.storage.has_inconsistencies
            || !self.nodes_to_check.is_empty()
            || !self.decrease_queue.is_empty()
            || !self.increase_queue.is_empty()
    }

    fn block_emission(&self, block: BlockKey, state: i32) -> u8 {
        let emission = emission(state);
        if emission > 0 && self.storage.light_on_in_section(section_of(block)) {
            emission
        } else {
            0
        }
    }

    fn check_node(&mut self, level: &Level<'_>, block: BlockKey) {
        match self.storage.layer {
            LightLayer::Block => {
                if !self.storage.storing_light_for_section(section_of(block)) {
                    return;
                }
                let state = level.state(block);
                let emission = self.block_emission(block, state);
                let old = self.storage.get_stored_level(block);
                if emission < old {
                    self.storage.set_stored_level(block, 0);
                    self.decrease_queue
                        .push_back((block, entry::decrease_all_directions(old)));
                } else {
                    self.decrease_queue
                        .push_back((block, entry::decrease_all_directions(1)));
                }
                if emission > 0 {
                    self.increase_queue.push_back((
                        block,
                        entry::increase_light_from_emission(emission, is_empty_shape(state)),
                    ));
                }
            }
            LightLayer::Sky => self.check_sky_node(level, block),
        }
    }

    fn propagate_increase(&mut self, level: &Level<'_>, from: BlockKey, data: u64, from_level: u8) {
        let sky = self.storage.layer == LightLayer::Sky;
        let empty_below = if sky {
            self.count_empty_sections_below_if_at_border(from)
        } else {
            0
        };
        let mut from_state = None;
        for direction in PROPAGATION_DIRECTIONS {
            if !entry::propagates(data, direction) {
                continue;
            }
            let to = offset(from, direction);
            if !self.storage.storing_light_for_section(section_of(to)) {
                continue;
            }
            let to_level = self.storage.get_stored_level(to);
            if i32::from(from_level) - 1 <= i32::from(to_level) {
                continue;
            }
            let to_state = level.state(to);
            let new_level = i32::from(from_level) - i32::from(Self::opacity(to_state));
            if new_level <= i32::from(to_level) {
                continue;
            }
            let new_level = new_level as u8;
            let from_state = *from_state.get_or_insert_with(|| {
                if entry::is_from_empty_shape(data) {
                    0
                } else {
                    level.state(from)
                }
            });
            if Self::shape_occludes(from_state, to_state, direction) {
                continue;
            }
            self.storage.set_stored_level(to, new_level);
            if new_level > 1 {
                self.increase_queue.push_back((
                    to,
                    entry::increase_skip_one_direction(
                        new_level,
                        is_empty_shape(to_state),
                        direction.opposite(),
                    ),
                ));
            }
            if sky {
                self.propagate_from_empty_sections(to, direction, new_level, true, empty_below);
            }
        }
    }

    fn propagate_decrease(&mut self, level: &Level<'_>, from: BlockKey, data: u64) {
        let sky = self.storage.layer == LightLayer::Sky;
        let empty_below = if sky {
            self.count_empty_sections_below_if_at_border(from)
        } else {
            0
        };
        let old_from_level = entry::from_level(data);
        for direction in PROPAGATION_DIRECTIONS {
            if !entry::propagates(data, direction) {
                continue;
            }
            let to = offset(from, direction);
            if !self.storage.storing_light_for_section(section_of(to)) {
                continue;
            }
            let to_level = self.storage.get_stored_level(to);
            if to_level == 0 {
                continue;
            }
            if i32::from(to_level) <= i32::from(old_from_level) - 1 {
                if sky {
                    self.storage.set_stored_level(to, 0);
                    self.decrease_queue.push_back((
                        to,
                        entry::decrease_skip_one_direction(to_level, direction.opposite()),
                    ));
                    self.propagate_from_empty_sections(to, direction, to_level, false, empty_below);
                } else {
                    let to_state = level.state(to);
                    let to_emission = self.block_emission(to, to_state);
                    self.storage.set_stored_level(to, 0);
                    if to_emission < to_level {
                        self.decrease_queue.push_back((
                            to,
                            entry::decrease_skip_one_direction(to_level, direction.opposite()),
                        ));
                    }
                    if to_emission > 0 {
                        self.increase_queue.push_back((
                            to,
                            entry::increase_light_from_emission(
                                to_emission,
                                is_empty_shape(to_state),
                            ),
                        ));
                    }
                }
            } else {
                self.increase_queue.push_back((
                    to,
                    entry::increase_only_one_direction(to_level, false, direction.opposite()),
                ));
            }
        }
    }

    // Sky light.

    fn chunk_sources<'a>(
        level: &'a Level<'_>,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&'a ChunkSkyLightSources> {
        level.chunks.sky_sources(chunk_x, chunk_z)
    }

    fn lowest_source_y(level: &Level<'_>, x: i32, z: i32, default: i32) -> i32 {
        Self::chunk_sources(level, x >> 4, z >> 4)
            .map_or(default, |sources| sources.lowest_source_y(x & 15, z & 15))
    }

    fn check_sky_node(&mut self, level: &Level<'_>, block: BlockKey) {
        let (x, y, z) = block;
        let key = section_of(block);
        let lowest = if self.storage.light_on_in_section(key) {
            Self::lowest_source_y(level, x, z, i32::MAX)
        } else {
            i32::MAX
        };
        if lowest != i32::MAX {
            self.update_sources_in_column(level, x, z, lowest);
        }
        if !self.storage.storing_light_for_section(key) {
            return;
        }
        if y >= lowest {
            self.decrease_queue
                .push_back((block, entry::decrease_skip_one_direction(15, Direction::Up)));
            self.increase_queue.push_back((
                block,
                entry::increase_skip_one_direction(15, false, Direction::Up),
            ));
        } else {
            let old = self.storage.get_stored_level(block);
            if old > 0 {
                self.storage.set_stored_level(block, 0);
                self.decrease_queue
                    .push_back((block, entry::decrease_all_directions(old)));
            } else {
                self.decrease_queue
                    .push_back((block, entry::decrease_all_directions(1)));
            }
        }
    }

    fn update_sources_in_column(&mut self, level: &Level<'_>, x: i32, z: i32, lowest: i32) {
        // `sectionToBlockCoord`, wrapping as the reference's int shift does.
        let world_bottom_y = self.storage.updating_sky.current_lowest_y.wrapping_shl(4);
        self.remove_sources_below(x, z, lowest, world_bottom_y);
        self.add_sources_above(level, x, z, lowest, world_bottom_y);
    }

    fn remove_sources_below(&mut self, x: i32, z: i32, lowest: i32, world_bottom_y: i32) {
        if lowest <= world_bottom_y {
            return;
        }
        let start_y = lowest - 1;
        let mut section_y = start_y >> 4;
        while self.storage.has_light_data_at_or_below(section_y) {
            if self
                .storage
                .storing_light_for_section((x >> 4, section_y, z >> 4))
            {
                let bottom = section_y * 16;
                let top = bottom + 15;
                let mut y = top.min(start_y);
                while y >= bottom {
                    let block = (x, y, z);
                    if self.storage.get_stored_level(block) != 15 {
                        return;
                    }
                    self.storage.set_stored_level(block, 0);
                    let data = if y == lowest - 1 {
                        entry::decrease_all_directions(15)
                    } else {
                        entry::decrease_skip_one_direction(15, Direction::Up)
                    };
                    self.decrease_queue.push_back((block, data));
                    y -= 1;
                }
            }
            section_y -= 1;
        }
    }

    fn add_sources_above(
        &mut self,
        level: &Level<'_>,
        x: i32,
        z: i32,
        lowest: i32,
        world_bottom_y: i32,
    ) {
        let neighbor_lowest = Self::lowest_source_y(level, x - 1, z, i32::MIN)
            .max(Self::lowest_source_y(level, x + 1, z, i32::MIN))
            .max(
                Self::lowest_source_y(level, x, z - 1, i32::MIN).max(Self::lowest_source_y(
                    level,
                    x,
                    z + 1,
                    i32::MIN,
                )),
            );
        let start_y = lowest.max(world_bottom_y);
        let mut key = (x >> 4, start_y >> 4, z >> 4);
        while !self.storage.is_above_data(key) {
            if self.storage.storing_light_for_section(key) {
                let bottom = key.1 * 16;
                let top = bottom + 15;
                for y in bottom.max(start_y)..=top {
                    let block = (x, y, z);
                    if self.storage.get_stored_level(block) == 15 {
                        return;
                    }
                    self.storage.set_stored_level(block, 15);
                    if y < neighbor_lowest || y == lowest {
                        self.increase_queue.push_back((
                            block,
                            entry::increase_skip_one_direction(15, false, Direction::Up),
                        ));
                    }
                }
            }
            key.1 += 1;
        }
    }

    fn count_empty_sections_below_if_at_border(&self, block: BlockKey) -> i32 {
        let (x, y, z) = block;
        if y & 15 != 0 {
            return 0;
        }
        let (lx, lz) = (x & 15, z & 15);
        if lx != 0 && lx != 15 && lz != 0 && lz != 15 {
            return 0;
        }
        let (sx, sy, sz) = section_of(block);
        let mut empty = 0;
        while !self
            .storage
            .storing_light_for_section((sx, sy - empty - 1, sz))
            && self.storage.has_light_data_at_or_below(sy - empty - 1)
        {
            empty += 1;
        }
        empty
    }

    fn propagate_from_empty_sections(
        &mut self,
        to: BlockKey,
        direction: Direction,
        to_level: u8,
        increase: bool,
        empty_below: i32,
    ) {
        if empty_below == 0 {
            return;
        }
        let (x, y, z) = to;
        let crossed = match direction {
            Direction::North => z & 15 == 15,
            Direction::South => z & 15 == 0,
            Direction::West => x & 15 == 15,
            Direction::East => x & 15 == 0,
            _ => false,
        };
        if !crossed {
            return;
        }
        let (sx, sz) = (x >> 4, z >> 4);
        let mut section_y = (y >> 4) - 1;
        let bottom = section_y - empty_below + 1;
        while section_y >= bottom {
            if self.storage.storing_light_for_section((sx, section_y, sz)) {
                let min_y = section_y * 16;
                for local_y in (0..16).rev() {
                    let block = (x, min_y + local_y, z);
                    if increase {
                        self.storage.set_stored_level(block, to_level);
                        if to_level > 1 {
                            self.increase_queue.push_back((
                                block,
                                entry::increase_skip_one_direction(
                                    to_level,
                                    true,
                                    direction.opposite(),
                                ),
                            ));
                        }
                    } else {
                        self.storage.set_stored_level(block, 0);
                        self.decrease_queue.push_back((
                            block,
                            entry::decrease_skip_one_direction(to_level, direction.opposite()),
                        ));
                    }
                }
            }
            section_y -= 1;
        }
    }

    fn set_light_enabled(&mut self, level: &Level<'_>, chunk: ColumnKey, enable: bool) {
        self.storage.set_light_enabled(chunk, enable);
        if self.storage.layer != LightLayer::Sky || !enable {
            return;
        }
        let empty = ChunkSkyLightSources::new(self.level_min_y);
        let sources = Self::chunk_sources(level, chunk.0, chunk.1).unwrap_or(&empty);
        let highest_non_source_y = sources.highest_lowest_source_y().wrapping_sub(1);
        let lowest_fully_source_section_y = (highest_non_source_y >> 4) + 1;
        let top = self.storage.updating_sky.top(chunk);
        let bottom = self
            .storage
            .updating_sky
            .current_lowest_y
            .max(lowest_fully_source_section_y);
        let mut section_y = top - 1;
        while section_y >= bottom {
            if let Some(id) = self
                .storage
                .data_layer_to_write((chunk.0, section_y, chunk.1))
                && self.storage.arena.get(id).is_empty()
            {
                self.storage.arena.get_mut(id).fill(15);
            }
            section_y -= 1;
        }
    }

    fn propagate_light_sources(&mut self, level: &Level<'_>, chunk: ColumnKey) {
        match self.storage.layer {
            LightLayer::Block => {
                self.set_light_enabled(level, chunk, true);
                if !level.chunks.has_chunk(chunk.0, chunk.1) {
                    return;
                }
                // `findBlockLightSources`: every emitting state in the
                // chunk's non-empty sections.
                let min_section = self.level_min_y >> 4;
                let section_count = self.section_count;
                for index in 0..section_count {
                    let base_y = (min_section + index) * 16;
                    for y in 0..16 {
                        for z in 0..16 {
                            for x in 0..16 {
                                let block = (chunk.0 * 16 + x, base_y + y, chunk.1 * 16 + z);
                                let state = level.state(block);
                                let emission = emission(state);
                                if emission != 0 {
                                    self.increase_queue.push_back((
                                        block,
                                        entry::increase_light_from_emission(
                                            emission,
                                            is_empty_shape(state),
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            LightLayer::Sky => self.propagate_sky_sources(level, chunk),
        }
    }

    fn propagate_sky_sources(&mut self, level: &Level<'_>, chunk: ColumnKey) {
        self.storage.set_light_enabled(chunk, true);
        let empty = ChunkSkyLightSources::new(self.level_min_y);
        let get = |cx: i32, cz: i32| Self::chunk_sources(level, cx, cz).unwrap_or(&empty);
        let sources = get(chunk.0, chunk.1);
        let north = get(chunk.0, chunk.1 - 1);
        let south = get(chunk.0, chunk.1 + 1);
        let west = get(chunk.0 - 1, chunk.1);
        let east = get(chunk.0 + 1, chunk.1);
        let top = self.storage.updating_sky.top(chunk);
        let bottom = self.storage.updating_sky.current_lowest_y;
        let (min_x, min_z) = (chunk.0 * 16, chunk.1 * 16);
        let mut section_y = top - 1;
        while section_y >= bottom {
            let key = (chunk.0, section_y, chunk.1);
            if let Some(id) = self.storage.data_layer_to_write(key) {
                let section_min_y = section_y * 16;
                let section_max_y = section_min_y + 15;
                let mut sources_below = false;
                for z in 0..16 {
                    for x in 0..16 {
                        let lowest = sources.lowest_source_y(x, z);
                        if lowest > section_max_y {
                            continue;
                        }
                        let north_y = if z == 0 {
                            north.lowest_source_y(x, 15)
                        } else {
                            sources.lowest_source_y(x, z - 1)
                        };
                        let south_y = if z == 15 {
                            south.lowest_source_y(x, 0)
                        } else {
                            sources.lowest_source_y(x, z + 1)
                        };
                        let west_y = if x == 0 {
                            west.lowest_source_y(15, z)
                        } else {
                            sources.lowest_source_y(x - 1, z)
                        };
                        let east_y = if x == 15 {
                            east.lowest_source_y(0, z)
                        } else {
                            sources.lowest_source_y(x + 1, z)
                        };
                        let neighbor = north_y.max(south_y).max(west_y.max(east_y));
                        let mut y = section_max_y;
                        while y >= section_min_y.max(lowest) {
                            self.storage.arena.get_mut(id).set(x, y & 15, z, 15);
                            if y == lowest || y < neighbor {
                                self.increase_queue.push_back((
                                    (min_x + x, y, min_z + z),
                                    entry::increase_sky_source_in_directions(
                                        y == lowest,
                                        y < north_y,
                                        y < south_y,
                                        y < west_y,
                                        y < east_y,
                                    ),
                                ));
                            }
                            y -= 1;
                        }
                        if lowest < section_min_y {
                            sources_below = true;
                        }
                    }
                }
                if !sources_below {
                    break;
                }
            }
            section_y -= 1;
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
