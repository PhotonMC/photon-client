//! The friends list's game tick (`Minecraft.tick` runs the friend list update handler).

use super::*;

impl Play {
    /// The account's friends are started once, then followed each tick; the toasts their changes
    /// call for are shown.
    pub(super) fn tick_friends(&mut self) {
        if self.controls.profile_name.is_empty() {
            self.controls.profile_name = self.session.local_name().to_owned();
            self.controls.start_friends(&self.session);
        }
        if self.controls.take_friends_wanted() {
            self.controls
                .toggle_friends(&mut self.surface, &self.screen_assets);
        }
        for title in self.controls.friends_tick(&self.screen_assets.lang) {
            self.gui.add_friend_toast(&title);
        }
    }
}
