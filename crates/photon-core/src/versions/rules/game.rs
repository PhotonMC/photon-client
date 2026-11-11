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
        /// The serverbound swing carries a hand and the player action numbers skip the direction
        /// change (before 26.3).
        punch_is_swing: before V26_3,
        /// The brewing stand's ingredient slot takes `PotionBrewing`'s own list (before 26.3).
        brewing_ingredients_are_hard_coded: before V26_3,
    }
}
