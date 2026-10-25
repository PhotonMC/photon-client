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

    /// A scripted press of the primary button on the world settings: whether they are open.
    pub(super) fn scripted_world_press(
        &mut self,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> bool {
        if self.world_screen.is_none() {
            return false;
        }
        let press = glfw::WindowEvent::MouseButton(
            glfw::MouseButton::Button1,
            glfw::Action::Press,
            glfw::Modifiers::empty(),
        );
        self.world_options_event(press, surface, assets);
        true
    }

    /// An event for the open world settings. Game Rules has no screen to open yet, so pressing
    /// it does nothing.
    pub(super) fn world_options_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::WindowEvent;
        let lang = &assets.lang;
        // The restrictions it opened are over it.
        if self.restrictions.is_some() {
            return self.restrictions_event(event, surface, assets);
        }
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
        let press = screen_press(screen, &event, (lang, self.gui_mouse));
        match press {
            Some(WorldPress::Done) => self.world_screen = None,
            Some(WorldPress::Restrictions) => self.open_restrictions(surface, assets),
            _ => {}
        }
    }
}

/// What an event does on the screen: the press it made, if any.
fn screen_press(
    screen: &mut WorldOptionsScreen,
    event: &glfw::WindowEvent,
    (lang, mouse): (&photon_game::text::Lang, (f64, f64)),
) -> Option<WorldPress> {
    use glfw::{Action, Key, MouseButton, WindowEvent};
    match (menu_key(event), event) {
        (Some(MenuKey::Tab(forward)), _) => {
            screen.tab(forward);
            None
        }
        (Some(MenuKey::Key(key, shift)), _) => screen.key_pressed(lang, key, shift),
        (None, WindowEvent::Key(Key::Escape, _, Action::Press, _)) => Some(WorldPress::Done),
        (None, WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers)) => {
            screen.clicked(lang, mouse, modifiers.contains(glfw::Modifiers::Shift))
        }
        _ => None,
    }
}
