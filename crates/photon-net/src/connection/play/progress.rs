use super::*;

impl PlaySession {
    /// `StatsScreen`'s request: ask the server for the statistics.
    pub fn request_stats(&mut self) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            encode_request_stats,
        )
    }

    /// `ClientAdvancements`.
    pub fn advancements(&self) -> &photon_game::advancements::ClientAdvancements {
        &self.local.advancements
    }

    /// `ClientAdvancements`, for the screen that listens to it.
    pub fn advancements_mut(&mut self) -> &mut photon_game::advancements::ClientAdvancements {
        &mut self.local.advancements
    }

    /// `ServerboundSeenAdvancementsPacket`: the tab that was opened, or (`None`) the screen closed.
    pub fn send_seen_advancements(&mut self, tab: Option<&str>) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SEEN_ADVANCEMENTS_PACKET_ID, |output| {
            encode_seen_advancements(output, tab)
        })
    }

    /// The toasts the packets asked for since this was last asked, in the order they came.
    pub fn take_pending_toasts(&mut self) -> Vec<photon_game::toast::PendingToast> {
        std::mem::take(&mut self.local.pending_toasts)
    }

    /// The breaks the tutorial has not been told of, with the tag test of each block done:
    /// whether it is a log, and how far the break is.
    pub fn take_destroyed_blocks(&mut self, tag: &str) -> Vec<(bool, f32)> {
        let registry = self.world.block_state_registry();
        self.local
            .game_mode_state
            .take_destroyed()
            .into_iter()
            .map(|(state, percent)| (registry.is_state_in_tag(state, tag), percent))
            .collect()
    }

    /// `ItemStack.is(tag)` for an item of the registry.
    pub fn item_in_tag(&self, item: i32, tag: &str) -> bool {
        self.world.item_in_tag(item, tag)
    }

    /// `Inventory.contains(tag)`: an item of the armor, main, hotbar or offhand slots is in the
    /// tag (the crafting grid is not part of the inventory).
    pub fn inventory_has_item_in_tag(&self, tag: &str) -> bool {
        let Some((slots, _, _)) = self.world.local_player_menu() else {
            return false;
        };
        slots
            .iter()
            .skip(5)
            .flatten()
            .any(|stack| self.world.item_in_tag(stack.item_registry_id().0, tag))
    }

    /// `StatsCounter` says a block of the tag was mined (`Stats.BLOCK_MINED`).
    pub fn mined_block_in_tag(&self, tag: &str) -> bool {
        let Some(stat_type) = photon_data::stat::type_id("minecraft:mined") else {
            return false;
        };
        self.world
            .block_state_registry()
            .tags()
            .members(tag)
            .any(|block| {
                self.stat(StatKey {
                    stat_type,
                    value: block as i32,
                }) > 0
            })
    }

    /// `StatsCounter` says an item of the tag was crafted (`Stats.ITEM_CRAFTED`).
    pub fn crafted_item_in_tag(&self, tag: &str) -> bool {
        let Some(stat_type) = photon_data::stat::type_id("minecraft:crafted") else {
            return false;
        };
        self.world
            .block_state_registry()
            .item_tags()
            .members(tag)
            .any(|item| {
                self.stat(StatKey {
                    stat_type,
                    value: item,
                }) > 0
            })
    }

    /// The items the server put in slots since this was last asked.
    pub fn take_received_items(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.local.received_items)
    }

    /// `LocalPlayer.getStats`: what the server said of the statistics.
    pub fn stats(&self) -> &std::collections::HashMap<StatKey, i32> {
        &self.local.stats
    }

    /// `StatsCounter.getValue` of the statistic with this key: none counted is zero.
    pub fn stat(&self, key: StatKey) -> i32 {
        self.local.stats.get(&key).copied().unwrap_or(0)
    }

    /// Whether statistics arrived since this was last asked (`StatsScreen.onStatsUpdated`'s cue).
    pub fn take_stats_updated(&mut self) -> bool {
        std::mem::take(&mut self.local.stats_updated)
    }
}
