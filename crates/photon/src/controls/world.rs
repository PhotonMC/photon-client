use super::*;
use photon_gui::options::world::{WorldFacts, WorldOptionsScreen, WorldPress};

impl Controls {
    /// `WorldOptionsScreen`, over the options screen.
    pub(super) fn open_world(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.world_screen = Some(WorldOptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            !self.title_host,
            self.world_facts,
        ));
    }

    /// What the server last said about the world: the difficulty and its lock, a hardcore level,
    /// and whether the player may use gamemaster commands. The open screen follows it
    /// (`HasDifficultyReaction`, `HasGamemasterPermissionReaction`): losing the permission with no
    /// integrated server sends the player back to the options screen.
    pub fn sync_world_facts(&mut self, session: &PlaySession) {
        let (difficulty, locked) = session.difficulty();
        self.world_facts = WorldFacts {
            difficulty,
            locked,
            hardcore: session.hardcore(),
            gamemaster: session.permission_level() >= 2,
        };
        if self
            .world_screen
            .as_ref()
            .is_some_and(|screen| !self.world_facts.gamemaster && screen.facts().gamemaster)
        {
            self.world_screen = None;
        }
    }

    /// An event for the open world settings. Game Rules and Restrictions have no screen to open
    /// yet, so pressing them does nothing.
    pub(super) fn world_options_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        if let WindowEvent::CursorPos(x, y) = event {
            self.move_gui_mouse((x, y), surface);
            return;
        }
        let facts = self.world_facts;
        let Some(screen) = self.world_screen.as_mut() else {
            return;
        };
        if screen.facts() != facts {
            screen.rebuild(&assets.creative.font, lang, Self::gui_size(surface), facts);
        }
        let mut press = None;
        match menu_key(&event) {
            Some(MenuKey::Tab(forward)) => screen.tab(forward),
            Some(MenuKey::Key(key, shift)) => press = screen.key_pressed(lang, key, shift),
            None => {}
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => press = Some(WorldPress::Done),
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                press = screen.clicked(lang, self.gui_mouse, shift);
            }
            _ => {}
        }
        if press == Some(WorldPress::Done) {
            self.world_screen = None;
        }
    }
}
