use super::*;

mod chat;
mod connection;
mod containers;
mod entity;
mod player;
mod recipes;
mod world;

impl PlaySession {
    /// A packet from the reader thread: the heavy ones arrive parsed.
    pub(super) fn handle_incoming(&mut self, packet: Incoming) -> Result<(), ConnectionError> {
        match packet {
            Incoming::Raw(body) => self.handle(&body),
            Incoming::Chunk(chunk) => {
                self.apply_chunk(&chunk);
                Ok(())
            }
            Incoming::Light(update) => {
                self.apply_light_update(&update);
                Ok(())
            }
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one arm per packet id, read like the reference's handler list"
    )]
    pub(super) fn handle(&mut self, body: &[u8]) -> Result<(), ConnectionError> {
        let (packet_id, payload) = split_packet(body)?;
        match packet_id {
            PLAY_CLIENTBOUND_LOGIN_PACKET_ID => self.on_login(payload)?,
            PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID => self.on_add_entity(payload)?,
            PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID => self.on_set_entity_motion(payload)?,
            PLAY_CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID => self.on_map_item_data(payload)?,
            PLAY_CLIENTBOUND_MOVE_MINECART_PACKET_ID => self.on_move_minecart(payload)?,
            PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID => self.on_set_entity_link(payload)?,
            PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID => self.on_rotate_head(payload)?,
            PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID => self.on_remove_entities(payload)?,
            PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID => self.on_set_passengers(payload)?,
            PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID => {
                self.on_entity_position_sync(payload)?
            }
            PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID => self.on_teleport_entity(payload)?,
            PLAY_CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID => self.on_take_item_entity(payload)?,
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID => self.on_move_entity_pos(payload)?,
            PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID => {
                self.on_move_entity_pos_rot(payload)?
            }
            PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID => self.on_move_entity_rot(payload)?,
            PLAY_CLIENTBOUND_ANIMATE_PACKET_ID => self.on_animate(payload)?,
            PLAY_CLIENTBOUND_ENTITY_EVENT_PACKET_ID => self.on_entity_event(payload)?,
            PLAY_CLIENTBOUND_SET_SIMULATION_DISTANCE_PACKET_ID => {
                self.on_set_simulation_distance(payload)?
            }
            PLAY_CLIENTBOUND_DAMAGE_EVENT_PACKET_ID => self.on_damage_event(payload)?,
            PLAY_CLIENTBOUND_HURT_ANIMATION_PACKET_ID => self.on_hurt_animation(payload)?,
            PLAY_CLIENTBOUND_RESPAWN_PACKET_ID => self.on_respawn(payload)?,
            PLAY_CLIENTBOUND_CHUNK_PACKET_ID => self.on_chunk(payload)?,
            PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID => self.on_light_update(payload)?,
            PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID => self.on_forget_level_chunk(payload)?,
            PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID => self.on_block_update(payload)?,
            PLAY_CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID => {
                self.on_section_blocks_update(payload)?
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID => {
                self.on_set_chunk_cache_radius(payload)?
            }
            PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID => {
                self.on_set_chunk_cache_center(payload)?
            }
            PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID => {
                self.on_chunk_batch_finished(payload)?
            }
            PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID => self.on_keep_alive(payload)?,
            PLAY_CLIENTBOUND_PING_PACKET_ID => self.on_ping(payload)?,
            PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID => self.on_set_entity_data(payload)?,
            PLAY_CLIENTBOUND_SET_EQUIPMENT_PACKET_ID => self.on_set_equipment(payload)?,
            PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID => self.on_update_attributes(payload)?,
            PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID => self.on_update_mob_effect(payload)?,
            PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID => self.on_remove_mob_effect(payload)?,
            PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID => self.on_set_health(payload)?,
            PLAY_CLIENTBOUND_LEVEL_EVENT_PACKET_ID => self.on_level_event(payload)?,
            PLAY_CLIENTBOUND_LEVEL_PARTICLES_PACKET_ID => self.on_level_particles(payload)?,
            PLAY_CLIENTBOUND_SOUND_PACKET_ID
            | PLAY_CLIENTBOUND_SOUND_ENTITY_PACKET_ID
            | PLAY_CLIENTBOUND_STOP_SOUND_PACKET_ID => self.on_sound(payload, packet_id)?,
            PLAY_CLIENTBOUND_PLAYER_COMBAT_KILL_PACKET_ID => self.on_player_combat_kill(payload)?,
            PLAY_CLIENTBOUND_SET_EXPERIENCE_PACKET_ID => self.on_set_experience(payload)?,
            PLAY_CLIENTBOUND_SET_HELD_SLOT_PACKET_ID => self.on_set_held_slot(payload)?,
            PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID => self.on_game_event(payload)?,
            PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID => self.on_player_abilities(payload)?,
            PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID => self.on_player_position(payload)?,
            PLAY_CLIENTBOUND_SET_TIME_PACKET_ID => self.on_set_time(payload)?,
            PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID => self.on_block_changed_ack(payload)?,
            PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID => self.on_block_destruction(payload)?,
            PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID => self.on_open_screen(payload)?,
            PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID => self.on_container_close(payload)?,
            PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID => {
                self.on_container_set_content(payload)?
            }
            PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID => self.on_block_entity_data(payload)?,
            PLAY_CLIENTBOUND_BLOCK_EVENT_PACKET_ID => self.on_block_event(payload)?,
            PLAY_CLIENTBOUND_INITIALIZE_BORDER_PACKET_ID => self.on_initialize_border(payload)?,
            PLAY_CLIENTBOUND_SET_BORDER_CENTER_PACKET_ID => self.on_set_border_center(payload)?,
            PLAY_CLIENTBOUND_SET_BORDER_LERP_SIZE_PACKET_ID => {
                self.on_set_border_lerp_size(payload)?
            }
            PLAY_CLIENTBOUND_SET_BORDER_SIZE_PACKET_ID => self.on_set_border_size(payload)?,
            PLAY_CLIENTBOUND_SET_BORDER_WARNING_DELAY_PACKET_ID => {
                self.on_set_border_warning_delay(payload)?
            }
            PLAY_CLIENTBOUND_SET_BORDER_WARNING_DISTANCE_PACKET_ID => {
                self.on_set_border_warning_distance(payload)?
            }
            PLAY_CLIENTBOUND_MERCHANT_OFFERS_PACKET_ID => self.on_merchant_offers(payload)?,
            PLAY_CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID => self.on_container_set_data(payload)?,
            PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID => self.on_system_chat(payload)?,
            PLAY_CLIENTBOUND_DISGUISED_CHAT_PACKET_ID => self.on_disguised_chat(payload)?,
            PLAY_CLIENTBOUND_PLAYER_CHAT_PACKET_ID => self.on_player_chat(payload)?,
            PLAY_CLIENTBOUND_DELETE_CHAT_PACKET_ID => self.on_delete_chat(payload)?,
            PLAY_CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID => {
                self.on_command_suggestions(payload)?
            }
            PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID
            | PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID
            | PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID
            | PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID
            | PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID => self.on_hud(payload, packet_id)?,
            PLAY_CLIENTBOUND_WAYPOINT_PACKET_ID => self.on_waypoint(payload)?,
            PLAY_CLIENTBOUND_COMMANDS_PACKET_ID => self.on_commands(payload)?,
            PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID
            | PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID
            | PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID => {
                self.on_scoreboard(payload, packet_id)?
            }
            PLAY_CLIENTBOUND_PLAYER_INFO_UPDATE_PACKET_ID => self.on_player_info_update(payload)?,
            PLAY_CLIENTBOUND_PLAYER_INFO_REMOVE_PACKET_ID => self.on_player_info_remove(payload)?,
            PLAY_CLIENTBOUND_RECIPE_BOOK_ADD_PACKET_ID => self.on_recipe_book_add(payload)?,
            PLAY_CLIENTBOUND_RECIPE_BOOK_REMOVE_PACKET_ID => self.on_recipe_book_remove(payload)?,
            PLAY_CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID => {
                self.on_recipe_book_settings(payload)?
            }
            PLAY_CLIENTBOUND_PLACE_GHOST_RECIPE_PACKET_ID => self.on_place_ghost_recipe(payload)?,
            PLAY_CLIENTBOUND_UPDATE_RECIPES_PACKET_ID => self.on_update_recipes(payload)?,
            PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID => self.on_container_set_slot(payload)?,
            PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID => self.on_set_cursor_item(payload)?,
            PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID => {
                self.on_set_player_inventory(payload)?
            }
            PLAY_CLIENTBOUND_UPDATE_TAGS_PACKET_ID => self.on_update_tags(payload)?,
            PLAY_CLIENTBOUND_DISCONNECT_PACKET_ID => self.on_disconnect(payload)?,
            _ => {}
        }
        self.maybe_finish_loading()
    }
}
