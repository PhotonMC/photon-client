use crate::*;

impl ClientWorld {
    /// Whether an item is in an item tag (`ItemStack.is(TagKey)`).
    pub fn item_in_tag(&self, item: i32, tag: &str) -> bool {
        self.block_state_registry.item_tags().contains(item, tag)
    }

    pub fn new(dimension: DimensionBounds) -> Self {
        Self {
            level: Level::new(dimension),
            weather: Weather::default(),
            block_state_registry: ClientBlockStateRegistry::default(),
            recipe_property_sets: BTreeMap::new(),
            minecart_improvements: false,
            stonecutter: Arc::default(),
            recipe_book: RecipeBook::default(),
            chunks: Chunks::default(),
            light: new_light_engine(dimension, true),
            light_tasks: VecDeque::new(),
            light_dirty: Vec::new(),
            entities: Entities::default(),
            maps: BTreeMap::new(),
            local_player: None,
            sounds: local_player::SoundQueue::default(),
            level_events: Default::default(),
            breaking_effects: Vec::new(),
            world_border: ClientWorldBorder::default(),
            block_animations: block::chest::BlockAnimations::default(),
            local_chunk: None,
            paddle_packets: Vec::new(),
            rider_controls: ride::mount::Controller::default(),
            predictions: prediction::BlockPredictions::new(),
            destroying_blocks: BTreeMap::new(),
        }
    }

    pub fn dimension(&self) -> DimensionBounds {
        self.level.dimension
    }

    pub fn dimension_key(&self) -> &str {
        &self.level.dimension_key
    }

    pub fn dimension_type_id(&self) -> i32 {
        self.level.dimension_type_id
    }

    pub fn seed(&self) -> i64 {
        self.level.seed
    }

    pub fn is_debug(&self) -> bool {
        self.level.is_debug
    }

    pub fn is_flat(&self) -> bool {
        self.level.is_flat
    }

    pub fn sea_level(&self) -> i32 {
        self.level.sea_level
    }

    pub fn dimension_environment(&self) -> &DimensionEnvironment {
        &self.level.dimension_environment
    }

    pub fn block_state_registry(&self) -> &ClientBlockStateRegistry {
        &self.block_state_registry
    }

    /// Apply a clientbound common update-tags packet to the world registry copy.
    pub fn apply_update_tags_packet(
        &mut self,
        packet: &photon_core::tags::ClientboundUpdateTagsPacket,
    ) {
        self.block_state_registry
            .apply_update_tags(&packet.registries);
    }

    pub fn replace_dimension_with_environment(
        &mut self,
        input: DimensionInput<'_>,
        bounds: DimensionBounds,
        environment: DimensionEnvironment,
    ) -> bool {
        let changed = self.level.dimension_key != input.key;
        // A new level has its own light engine.
        if changed
            || self.level.dimension.min_y != bounds.min_y
            || self.level.dimension.height != bounds.height
            || self.level.dimension_environment.has_skylight != environment.has_skylight
        {
            self.light = new_light_engine(bounds, environment.has_skylight);
            self.light_tasks.clear();
            self.light_dirty.clear();
        }
        self.level.dimension = bounds;
        self.level.dimension_key = input.key.to_owned();
        self.level.dimension_type_id = input.dimension_type_id;
        self.level.seed = input.seed;
        self.level.is_debug = input.is_debug;
        self.level.is_flat = input.is_flat;
        self.level.sea_level = input.sea_level;
        self.level.dimension_environment = environment;
        if changed {
            let server_view_radius = self.chunks.cache().map(|cache| cache.server_view_radius);
            self.chunks.reset(server_view_radius);
            self.entities.clear();
            self.maps.clear();
            if let Some(player) = self.local_player.as_mut() {
                player.vehicle_id = None;
                player.passenger_ids.clear();
            }
            self.world_border = ClientWorldBorder::default();
            self.level.game_time = 0;
            // A new level starts with no predictions and no cracks.
            self.predictions = prediction::BlockPredictions::new();
            self.destroying_blocks.clear();
        }
        changed
    }
}
