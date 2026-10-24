use super::*;

/// `LevelLightEngine`: a block engine and, where the dimension has a sky, a
/// sky engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelLightEngine {
    pub(super) block: Engine,
    pub(super) sky: Option<Engine>,
    pub(super) min_section_y: i32,
    pub(super) section_count: i32,
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

    pub(super) fn engines(&mut self) -> impl Iterator<Item = &mut Engine> {
        std::iter::once(&mut self.block).chain(self.sky.as_mut())
    }

    pub(super) fn engine(&mut self, layer: LightLayer) -> Option<&mut Engine> {
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

    /// `light_value` along x from `block` for `out.len()` blocks inside one section.
    pub fn light_run(&self, layer: LightLayer, block: BlockKey, out: &mut [u8]) {
        match layer {
            LightLayer::Block => self.block.storage.get_light_run(block, out),
            LightLayer::Sky => match self.sky.as_ref() {
                Some(sky) => sky.storage.get_light_run(block, out),
                None => out.fill(0),
            },
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
