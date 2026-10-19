use super::*;

impl Play {
    /// `Minecraft.tick`'s first multiplayer join shows the social interactions toast once, and
    /// opening the screen with its key hides it.
    pub(super) fn social_interactions_toast(&mut self) {
        if self.controls.social.is_some() && self.controls.pause.is_none() {
            self.gui.hide_social_interactions_toast();
        }
        if self.controls.options.joined_first_server || self.session.player().is_none() {
            return;
        }
        let options = &mut self.controls.options;
        let bound = options.key_binding("key.socialInteractions");
        let lang = &self.gui.lang;
        let key = bound.map_or_else(
            || photon_game::text::Component::literal(""),
            |key| {
                photon_game::options::key_display_name(
                    key,
                    crate::controls::keybinds::system_key_name(key),
                    lang,
                )
            },
        );
        let time = options.notification_display_time;
        options.joined_first_server = true;
        let _ = options.save(&crate::controls::options_path());
        self.gui.add_social_interactions_toast(&key, time);
    }
}
