use crate::*;

impl ClientWorld {
    /// Whether an item is in an item tag (`ItemStack.is(TagKey)`).
    pub fn item_in_tag(&self, item: i32, tag: &str) -> bool {
        self.block_state_registry.item_tags().contains(item, tag)
    }

    pub fn new(dimension: DimensionBounds) -> Self {
        Self {
            dimension,
            dimension_key: "minecraft:overworld".to_owned(),
            dimension_type_id: 0,
            seed: 0,
            is_debug: false,
            is_flat: false,
            sea_level: 63,
            dimension_environment: DimensionEnvironment::for_builtin_dimension(
                "minecraft:overworld",
            ),
            block_state_registry: ClientBlockStateRegistry::default(),
            recipe_property_sets: BTreeMap::new(),
            minecart_improvements: false,
            stonecutter: Arc::default(),
            recipe_book: RecipeBook::default(),
            chunks: BTreeMap::new(),
            chunk_cache: None,
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
            server_simulation_distance: 3,
            local_chunk: None,
            game_time: 0,
            paddle_packets: Vec::new(),
            rider_controls: ride::mount::Controller::default(),
            sky_flash_time: 0,
            rain_level: JavaF32::new(0.0),
            thunder_level: JavaF32::new(0.0),
            predictions: prediction::BlockPredictions::new(),
            destroying_blocks: BTreeMap::new(),
        }
    }

    pub fn dimension(&self) -> DimensionBounds {
        self.dimension
    }

    pub fn dimension_key(&self) -> &str {
        &self.dimension_key
    }

    pub fn dimension_type_id(&self) -> i32 {
        self.dimension_type_id
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }

    pub fn is_debug(&self) -> bool {
        self.is_debug
    }

    pub fn is_flat(&self) -> bool {
        self.is_flat
    }

    pub fn sea_level(&self) -> i32 {
        self.sea_level
    }

    pub fn dimension_environment(&self) -> &DimensionEnvironment {
        &self.dimension_environment
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
        let changed = self.dimension_key != input.key;
        // A new level has its own light engine.
        if changed
            || self.dimension.min_y != bounds.min_y
            || self.dimension.height != bounds.height
            || self.dimension_environment.has_skylight != environment.has_skylight
        {
            self.light = new_light_engine(bounds, environment.has_skylight);
            self.light_tasks.clear();
            self.light_dirty.clear();
        }
        self.dimension = bounds;
        self.dimension_key = input.key.to_owned();
        self.dimension_type_id = input.dimension_type_id;
        self.seed = input.seed;
        self.is_debug = input.is_debug;
        self.is_flat = input.is_flat;
        self.sea_level = input.sea_level;
        self.dimension_environment = environment;
        if changed {
            let server_view_radius = self
                .chunk_cache
                .as_ref()
                .map(|cache| cache.server_view_radius);
            self.chunks.clear();
            self.chunk_cache = server_view_radius.map(ClientChunkCacheState::new);
            self.entities.clear();
            self.maps.clear();
            if let Some(player) = self.local_player.as_mut() {
                player.vehicle_id = None;
                player.passenger_ids.clear();
            }
            self.world_border = ClientWorldBorder::default();
            self.game_time = 0;
            // A new level starts with no predictions and no cracks.
            self.predictions = prediction::BlockPredictions::new();
            self.destroying_blocks.clear();
        }
        changed
    }
}
