use super::*;
use photon_core::ids::EntityId;

impl PlaySession {
    pub fn world(&self) -> &ClientWorld {
        &self.world
    }

    /// Set which entities the frames leave out as wholly outside the view.
    pub fn set_entities_out_of_view(&mut self, ids: std::collections::BTreeSet<EntityId>) {
        self.world.entities_mut().set_out_of_view(ids);
    }

    /// The sounds the world's events and the player's own blocks made since the last call.
    pub fn take_world_sounds(&mut self) -> Vec<photon_world::local_player::LocalSound> {
        self.world.take_sounds()
    }

    /// The particle packets that arrived since the last call, for the particle engine.
    pub fn take_level_particles(&mut self) -> Vec<photon_world::particles::LevelParticles> {
        // The crumbs of what the local player eats are thrown by its own tick.
        if let Some(player) = self.local.player.as_mut() {
            self.effects.level_particles.extend(player.take_particles());
        }
        std::mem::take(&mut self.effects.level_particles)
    }

    /// The tracking emitters entity events and animations asked for since the last call.
    pub fn take_tracking_emitters(&mut self) -> Vec<photon_world::particles::TrackingRequest> {
        std::mem::take(&mut self.effects.tracking_emitters)
    }

    /// Whether the unsecure-server toast is due (`ClientPacketListener.handleLogin`: the server does
    /// not enforce secure chat and the warning was not shown on this connection), once.
    pub fn take_unsecure_server_warning(&mut self) -> bool {
        std::mem::take(&mut self.effects.unsecure_warning)
    }

    /// The stack a totem's animation is to show, if one was asked for since the last call.
    pub fn take_item_activation(&mut self) -> Option<photon_world::ClientItemStackState> {
        self.effects.item_activation.take()
    }

    /// The entities taken since the last call, each flying to whoever took it.
    pub fn take_item_pickups(&mut self) -> Vec<photon_world::particles::ItemPickup> {
        std::mem::take(&mut self.effects.item_pickups)
    }

    /// The level events since the last call, in the order they came.
    pub fn take_level_events(&mut self) -> Vec<photon_world::LevelEvent> {
        self.world.take_level_events()
    }

    /// `LevelEventHandler.notifyNearbyEntities`: the entities near a jukebox hear its record play or stop.
    pub fn set_record_playing_nearby(&mut self, jukebox: (i32, i32, i32), playing: bool) {
        self.world.set_record_playing_nearby(jukebox, playing);
    }

    /// The blocks the player swung at breaking, with the face aimed at, since the last call.
    pub fn take_breaking_effects(
        &mut self,
    ) -> Vec<((i32, i32, i32), photon_core::direction::Direction)> {
        self.world.take_breaking_effects()
    }

    /// Where an entity is drawn between two ticks.
    pub fn entity_snapshot(
        &self,
        entity_id: EntityId,
        partial_tick: f32,
    ) -> Option<photon_world::EntitySnapshot> {
        self.world.entity_snapshot(entity_id, partial_tick)
    }

    /// How many of the entities tick (those in a loaded chunk).
    pub fn ticking_entity_count(&self) -> usize {
        self.world
            .entities()
            .all()
            .filter(|entity| entity.is_ticking())
            .count()
    }

    /// `ClientLevel.tickRateManager().runsNormally()`: the game's elements tick (the level is not
    /// frozen, or is stepping).
    pub fn runs_normally(&self) -> bool {
        self.level_info.tick_rate.runs_normally()
    }

    /// How many entities the world holds besides the player.
    pub fn entity_count(&self) -> usize {
        self.world.entities().all().count()
    }

    pub fn player(&self) -> Option<&LocalPlayer> {
        self.local.player.as_ref()
    }

    pub fn player_mut(&mut self) -> Option<&mut LocalPlayer> {
        self.local.player.as_mut()
    }

    /// The server's view distance in chunks, from the login packet.
    pub fn view_distance(&self) -> i32 {
        self.level_info.view_distance
    }

    /// The hashed world seed the biome zoom is jittered with.
    pub fn hashed_seed(&self) -> i64 {
        self.level_info.hashed_seed
    }

    /// `Connection.getAverageSentPackets` and `getAverageReceivedPackets`.
    pub fn average_packets(&self) -> (f32, f32) {
        (
            self.link.traffic.average_sent(),
            self.link.traffic.average_received(),
        )
    }

    /// The tick rate manager's frozen state: whether it is stepping forward, and whether frozen.
    pub fn tick_rate_status(&self) -> (bool, bool) {
        let rate = &self.level_info.tick_rate;
        (rate.frozen_ticks_to_run() > 0, rate.is_frozen())
    }

    /// `ClientPacketListener.serverBrand`.
    pub fn server_brand(&self) -> Option<&str> {
        self.state.server_brand.as_deref()
    }

    /// `Player.isReducedDebugInfo`: the server asked for less on the debug screen.
    pub fn reduced_debug_info(&self) -> bool {
        self.local.reduced_debug_info
    }

    /// The current dimension type's registry name.
    pub fn dimension_type(&self) -> Option<&str> {
        self.registry_names("minecraft:dimension_type")
            .get(usize::try_from(self.level_info.dimension_type_id).ok()?)
            .map(String::as_str)
    }

    /// `Player.getCooldowns`: the item groups waiting to be usable again.
    pub fn item_cooldowns(&self) -> &photon_game::item::cooldowns::ItemCooldowns {
        &self.local.item_cooldowns
    }

    /// The entry names of one synchronised registry, in network order.
    pub fn registry_names(&self, registry: &str) -> &[String] {
        self.state
            .registry_entry_names
            .get(registry)
            .map_or(&[], Vec::as_slice)
    }

    /// The key of the level's dimension.
    pub fn dimension(&self) -> &str {
        &self.level_info.dimension
    }

    /// `ClientLevel.getRespawnData`'s position, once the server has said.
    pub fn level_respawn(&self) -> Option<&GlobalBlock> {
        self.level_info.respawn.as_ref()
    }

    /// `Player.getLastDeathLocation` of the local player.
    pub fn last_death(&self) -> Option<&GlobalBlock> {
        self.local.last_death.as_ref()
    }

    /// The level's game time, as the server last set it and the ticks since.
    pub fn game_time(&self) -> i64 {
        self.level_info.game_time
    }

    /// Whether the level is hardcore.
    pub fn hardcore(&self) -> bool {
        self.local.hardcore
    }

    /// The player's game mode, as the server last set it.
    /// `MultiPlayerGameMode.getPreviousPlayerMode`.
    pub fn previous_game_mode(&self) -> Option<ClientPlayerGameMode> {
        self.local.previous_game_mode
    }

    /// Whether the player may run `/gamemode` (`GameModeCommand.PERMISSION_CHECK`: game master).
    pub fn may_change_game_mode(&self) -> bool {
        self.local.permission_level >= 2
    }

    pub fn game_mode(&self) -> ClientPlayerGameMode {
        self.local.game_mode
    }

    /// The sound packets that arrived since the last call, in order (`handleSoundEvent`,
    /// `handleSoundEntityEvent`, `handleStopSoundEvent`): the sound engine's to carry out.
    pub fn take_sounds(&mut self) -> Vec<photon_game::sounds::SoundCommand> {
        std::mem::take(&mut self.effects.sounds)
    }

    /// The player's profile id, as the server logged it in.
    pub fn profile_id(&self) -> Option<photon_core::binary::Uuid> {
        self.state.profile_id
    }

    /// `handleShowDialog`: the dialog is opened over whatever screen is up.
    pub(super) fn on_show_dialog(&mut self, holder: DialogHolder) {
        let dialog = match holder {
            DialogHolder::Direct(dialog) => *dialog,
            DialogHolder::Registry(index) => {
                match self.state.dialog_registry.get(index).cloned().flatten() {
                    Some(dialog) => dialog,
                    // A known pack's entry carries no data, and there is none to look it up in.
                    None => return,
                }
            }
        };
        self.social
            .dialog_events
            .push(DialogEvent::Show(Box::new(dialog)));
    }

    /// `ClientCommonPacketListenerImpl.serverLinks`: the addresses the server offers.
    pub fn server_links(&self) -> &photon_game::server_links::ServerLinks {
        &self.state.server_links
    }

    /// `PauseScreen.getCustomAdditions`: the dialog the pause menu offers a button for. The
    /// `pause_screen_additions` tag of the dialog registry wins (one entry is the dialog, more
    /// are the custom options list); with it empty, the server's links are the dialog.
    pub fn pause_screen_additions(&self) -> Option<photon_game::dialog::Dialog> {
        let added = self
            .state
            .tags
            .iter()
            .flat_map(|packet| &packet.registries)
            .filter(|registry| registry.registry.as_string() == "minecraft:dialog")
            .flat_map(|registry| &registry.tags)
            .filter(|tag| tag.id.as_string() == "minecraft:pause_screen_additions")
            .flat_map(|tag| tag.values.iter().copied())
            .collect::<Vec<_>>();
        let entry = |index: usize| self.state.dialog_registry.get(index).cloned().flatten();
        match added.as_slice() {
            [] if self.state.server_links.is_empty() => None,
            [] => self.dialog_by_name("minecraft:server_links"),
            [only] => entry(usize::try_from(*only).ok()?),
            _ => self.dialog_by_name("minecraft:custom_options"),
        }
    }

    /// The dialog entry a registry name stands for (`ClickEvent.ShowDialog` by name).
    pub fn dialog_by_name(&self, name: &str) -> Option<photon_game::dialog::Dialog> {
        let position = self
            .registry_names("minecraft:dialog")
            .iter()
            .position(|entry| entry == name)?;
        self.state.dialog_registry.get(position).cloned().flatten()
    }

    /// The dialogs the server showed or cleared since the last call.
    pub fn take_dialog_events(&mut self) -> Vec<DialogEvent> {
        std::mem::take(&mut self.social.dialog_events)
    }

    /// `ServerboundCustomClickActionPacket`: a dialog button's custom action.
    pub fn send_custom_click_action(
        &mut self,
        id: &str,
        payload: Option<&photon_core::nbt::FieldValue>,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_CUSTOM_CLICK_ACTION_PACKET_ID, |output| {
            encode_custom_click_action(id, payload, output)
        })
    }

    /// The resource packs the server asked for since the last call: the player is asked about them.
    pub fn take_pack_requests(&mut self) -> Vec<PackRequest> {
        std::mem::take(&mut self.social.pack_requests)
    }

    /// `ServerboundResourcePackPacket`: how the client answers a pack.
    pub fn answer_resource_pack(
        &mut self,
        id: photon_core::binary::Uuid,
        action: PackAction,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_RESOURCE_PACK_PACKET_ID, |output| {
            encode_resource_pack(id, action, output);
            Ok(())
        })
    }

    /// Whether the server took the connection through the configuration phase again: what was
    /// open over the old level is closed (`Minecraft.clearClientLevel`).
    pub fn take_reconfigured(&mut self) -> bool {
        std::mem::take(&mut self.level_info.reconfigured)
    }

    /// Whether the server replaced the level since the last call, so
    /// everything drawn from the old one should go.
    pub fn take_level_reset(&mut self) -> bool {
        std::mem::take(&mut self.level_info.level_reset)
    }

    /// Whether the client got a new level since the last call (`Minecraft.setLevel`): its sounds
    /// stop and what the old one kept goes.
    pub fn take_level_swapped(&mut self) -> bool {
        std::mem::take(&mut self.level_info.level_swapped)
    }

    /// What the HUD shows of the player.
    pub fn status(&self) -> &PlayerStatus {
        &self.local.status
    }

    /// `Level.getRainLevel`: how hard it rains, 0 to 1.
    pub fn rain_level(&self) -> f32 {
        self.level_info.rain_level
    }

    /// `Level.getThunderLevel`: the thunder level scaled by the rain.
    pub fn thunder_level(&self) -> f32 {
        self.level_info.thunder_level * self.level_info.rain_level
    }
}
