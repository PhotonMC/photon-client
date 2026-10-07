use super::*;

pub(super) fn column(key: SectionKey) -> ColumnKey {
    (key.0, key.2)
}

pub(super) fn section_of(block: BlockKey) -> SectionKey {
    (block.0 >> 4, block.1 >> 4, block.2 >> 4)
}

pub(super) fn local(block: BlockKey) -> (i32, i32, i32) {
    (block.0 & 15, block.1 & 15, block.2 & 15)
}

pub(super) fn repeat_first_layer(layer: &DataLayer) -> DataLayer {
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

/// Section coordinates.
pub type SectionKey = (i32, i32, i32);

/// Block coordinates.
pub type BlockKey = (i32, i32, i32);

pub(super) type ColumnKey = (i32, i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightLayer {
    Sky,
    Block,
}

/// `DataLayer`: 4096 nibbles, or a single value until something is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayer {
    pub(super) data: Option<Box<[u8]>>,
    pub(super) default_value: u8,
}

pub(super) type LayerId = u32;

/// Layers by id, counted by how many maps hold them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Arena {
    pub(super) layers: Vec<Option<(DataLayer, u32)>>,
    pub(super) free: Vec<LayerId>,
}

/// A map of section layers whose entries keep their layers alive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct LayerMap {
    pub(super) map: HashMap<SectionKey, LayerId>,
}

/// `SkyDataLayerStorageMap`'s extra state: each column's top section with
/// light data, and the lowest section with any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SkyTops {
    pub(super) top_sections: HashMap<ColumnKey, i32>,
    pub(super) current_lowest_y: i32,
}

pub(super) const HAS_DATA_BIT: u8 = 32;

pub(super) const NEIGHBOR_COUNT_BITS: u8 = 31;

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

    pub(super) fn index(x: i32, y: i32, z: i32) -> usize {
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

    pub(super) fn data_mut(&mut self) -> &mut [u8] {
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

impl Arena {
    pub(super) fn alloc(&mut self, layer: DataLayer) -> LayerId {
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

    pub(super) fn get(&self, id: LayerId) -> &DataLayer {
        &self.layers[id as usize].as_ref().expect("live layer").0
    }

    pub(super) fn get_mut(&mut self, id: LayerId) -> &mut DataLayer {
        &mut self.layers[id as usize].as_mut().expect("live layer").0
    }

    pub(super) fn retain(&mut self, id: LayerId) {
        self.layers[id as usize].as_mut().expect("live layer").1 += 1;
    }

    pub(super) fn release(&mut self, id: LayerId) {
        let slot = self.layers[id as usize].as_mut().expect("live layer");
        slot.1 -= 1;
        if slot.1 == 0 {
            self.layers[id as usize] = None;
            self.free.push(id);
        }
    }

    /// A map let go of the layer; the caller decides whether it is freed.
    pub(super) fn unhold(&mut self, id: LayerId) {
        self.layers[id as usize].as_mut().expect("live layer").1 -= 1;
    }

    /// Free a layer that no map holds any more.
    pub(super) fn drop_if_unheld(&mut self, id: LayerId) {
        if self.layers[id as usize]
            .as_ref()
            .is_some_and(|slot| slot.1 == 0)
        {
            self.layers[id as usize] = None;
            self.free.push(id);
        }
    }
}

impl LayerMap {
    pub(super) fn get(&self, key: SectionKey) -> Option<LayerId> {
        self.map.get(&key).copied()
    }

    pub(super) fn set(&mut self, arena: &mut Arena, key: SectionKey, id: LayerId) {
        arena.retain(id);
        if let Some(old) = self.map.insert(key, id) {
            arena.release(old);
        }
    }

    pub(super) fn remove(&mut self, arena: &mut Arena, key: SectionKey) -> Option<LayerId> {
        let id = self.map.remove(&key)?;
        // The caller may still want it: it stays alive until the caller
        // stores it again or drops it.
        arena.unhold(id);
        Some(id)
    }
}

impl SkyTops {
    pub(super) fn new() -> Self {
        Self {
            top_sections: HashMap::new(),
            current_lowest_y: i32::MAX,
        }
    }

    pub(super) fn top(&self, column: ColumnKey) -> i32 {
        self.top_sections
            .get(&column)
            .copied()
            .unwrap_or(self.current_lowest_y)
    }
}
