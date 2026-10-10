use super::*;

impl PlaySession {
    pub fn world(&self) -> &ClientWorld {
        &self.trace.world
    }

    /// Set which entities the frames leave out as wholly outside the view.
    pub fn set_entities_out_of_view(&mut self, ids: std::collections::BTreeSet<i32>) {
        self.trace.world.entities_mut().set_out_of_view(ids);
    }

    /// The sounds the world's events and the player's own blocks made since the last call.
    pub fn take_world_sounds(&mut self) -> Vec<photon_world::local_player::LocalSound> {
        self.trace.world.take_sounds()
    }

    /// The particle packets that arrived since the last call, for the particle engine.
    pub fn take_level_particles(&mut self) -> Vec<photon_world::particles::LevelParticles> {
        std::mem::take(&mut self.level_particles)
    }

    /// The entities taken since the last call, each flying to whoever took it.
    pub fn take_item_pickups(&mut self) -> Vec<photon_world::particles::ItemPickup> {
        std::mem::take(&mut self.item_pickups)
    }

    /// The level events since the last call, in the order they came.
    pub fn take_level_events(&mut self) -> Vec<photon_world::LevelEvent> {
        self.trace.world.take_level_events()
    }

    /// The blocks the player swung at breaking, with the face aimed at, since the last call.
    pub fn take_breaking_effects(
        &mut self,
    ) -> Vec<((i32, i32, i32), photon_core::direction::Direction)> {
        self.trace.world.take_breaking_effects()
    }

    /// Where an entity is drawn between two ticks.
    pub fn entity_snapshot(
        &self,
        entity_id: i32,
        partial_tick: f32,
    ) -> Option<photon_world::EntitySnapshot> {
        self.trace.world.entity_snapshot(entity_id, partial_tick)
    }

    /// How many of the entities tick (those in a loaded chunk).
    pub fn ticking_entity_count(&self) -> usize {
        self.trace
            .world
            .entities()
            .all()
            .filter(|entity| entity.is_ticking())
            .count()
    }

    /// How many entities the world holds besides the player.
    pub fn entity_count(&self) -> usize {
        self.trace.world.entities().all().count()
    }

    pub fn player(&self) -> Option<&LocalPlayer> {
        self.player.as_ref()
    }

    pub fn player_mut(&mut self) -> Option<&mut LocalPlayer> {
        self.player.as_mut()
    }

    /// The server's view distance in chunks, from the login packet.
    pub fn view_distance(&self) -> i32 {
        self.view_distance
    }

    /// The hashed world seed the biome zoom is jittered with.
    pub fn hashed_seed(&self) -> i64 {
        self.hashed_seed
    }

    /// The current dimension type's registry name.
    pub fn dimension_type(&self) -> Option<&str> {
        self.registry_names("minecraft:dimension_type")
            .get(usize::try_from(self.dimension_type_id).ok()?)
            .map(String::as_str)
    }

    /// `Player.getCooldowns`: the item groups waiting to be usable again.
    pub fn item_cooldowns(&self) -> &photon_game::item::cooldowns::ItemCooldowns {
        &self.item_cooldowns
    }

    /// The entry names of one synchronised registry, in network order.
    pub fn registry_names(&self, registry: &str) -> &[String] {
        self.trace
            .registry_entry_names
            .get(registry)
            .map_or(&[], Vec::as_slice)
    }

    /// The level's game time, as the server last set it and the ticks since.
    pub fn game_time(&self) -> i64 {
        self.game_time
    }

    /// Whether the level is hardcore.
    pub fn hardcore(&self) -> bool {
        self.hardcore
    }

    /// The player's game mode, as the server last set it.
    /// `MultiPlayerGameMode.getPreviousPlayerMode`.
    pub fn previous_game_mode(&self) -> Option<ClientPlayerGameMode> {
        self.previous_game_mode
    }

    /// Whether the player may run `/gamemode` (`GameModeCommand.PERMISSION_CHECK`: game master).
    pub fn may_change_game_mode(&self) -> bool {
        self.permission_level >= 2
    }

    pub fn game_mode(&self) -> ClientPlayerGameMode {
        self.game_mode
    }

    /// The sound packets that arrived since the last call, in order (`handleSoundEvent`,
    /// `handleSoundEntityEvent`, `handleStopSoundEvent`): the sound engine's to carry out.
    pub fn take_sounds(&mut self) -> Vec<photon_game::sounds::SoundCommand> {
        std::mem::take(&mut self.sounds)
    }

    /// The player's profile id, as the server logged it in.
    pub fn profile_id(&self) -> Option<photon_core::binary::Uuid> {
        self.trace.profile_id
    }

    /// Whether the server replaced the level since the last call, so
    /// everything drawn from the old one should go.
    /// Whether the server took the connection through the configuration phase again: what was
    /// open over the old level is closed (`Minecraft.clearClientLevel`).
    pub fn take_reconfigured(&mut self) -> bool {
        std::mem::take(&mut self.reconfigured)
    }

    pub fn take_level_reset(&mut self) -> bool {
        std::mem::take(&mut self.level_reset)
    }

    /// What the HUD shows of the player.
    pub fn status(&self) -> &PlayerStatus {
        &self.status
    }

    /// `Level.getRainLevel`: how hard it rains, 0 to 1.
    pub fn rain_level(&self) -> f32 {
        self.rain_level
    }

    /// `Level.getThunderLevel`: the thunder level scaled by the rain.
    pub fn thunder_level(&self) -> f32 {
        self.thunder_level * self.rain_level
    }
}
