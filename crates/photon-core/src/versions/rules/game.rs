//! What the player's actions do: breaking, using, attacking and the contents of menus.

use crate::version::Version;

rules! {
    /// The rules of the player's actions (see the module).
    GameRules {
        /// A shulker box tooltip line names the count before the item (26.3) or the item before the
        /// count (26.2).
        container_line_count_first: since V26_3,
        /// The client swings for a click that mounts a minecart or opens a grown wandering trader's
        /// trades (26.2); in 26.3 only the server's answer swings.
        mount_and_trade_clicks_swing: before V26_3,
        /// A camel consumes a click it has no use for (26.2); 26.3 lets it go on to the held item.
        camel_consumes_idle_click: before V26_3,
        /// A block being broken sounds its hit every fourth tick of the breaking (26.2); 26.3
        /// sounds it once when the breaking starts.
        hit_sound_repeats: before V26_3,
        /// An attack on an entity in creative waits out the next block break (26.3).
        creative_attack_delays_breaking: since V26_3,
        /// The local player tells the server its input, look and position at the end of its own
        /// tick (26.2), before the block entities move it and the vehicle it rides turns it;
        /// 26.3 sends them at the end of the client tick, after both.
        player_sends_in_own_tick: before V26_3,
        /// The serverbound swing carries a hand and the player action numbers skip the direction
        /// change (before 26.3).
        punch_is_swing: before V26_3,
        /// The brewing stand's ingredient slot takes `PotionBrewing`'s own list (before 26.3).
        brewing_ingredients_are_hard_coded: before V26_3,
        /// Mining fatigue slows breaking by a table of four factors, the last for every level from
        /// the fourth (26.2); 26.3 raises 0.3 to the level.
        mining_fatigue_is_a_table: before V26_3,
        /// A rocket's flight duration line is one translation with the number as its argument
        /// (26.3); before, a translation of the label, a space and the number.
        flight_duration_is_one_translation: since V26_3,
        /// A furnace takes as fuel what carries the `cooking_fuel` component, a stack's own patch
        /// counting (26.3); before, what the level's fuel table lists for the item.
        fuel_is_a_component: since V26_3,
        /// A trade's result can be taken only while the payments satisfy the active offer, in
        /// either order (26.3); before, a result in the slot could always be picked up.
        merchant_result_needs_payment: since V26_3,
        /// An axe, hoe or shovel changes the block it is used on by the `block_transformer`
        /// component of its stack, a patch counting (26.3); before, the item classes carry
        /// the tables themselves.
        tools_transform_by_component: since V26_3,
        /// A lit campfire is doused by what the `douses_campfires` tag lists, through the
        /// block's own `useItemOn` and for any face (26.3); before, only the shovel's
        /// `useOn` did, not on the bottom face.
        campfire_douses_by_tag: since V26_3,
    }
}
