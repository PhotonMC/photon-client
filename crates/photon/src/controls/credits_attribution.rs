use super::*;
use photon_gui::screen::credits_attribution::{CreditsAction, CreditsAttributionScreen};

/// `CommonLinks.ATTRIBUTION` and `CommonLinks.LICENSES`.
const ATTRIBUTION: &str = "https://aka.ms/MinecraftJavaAttribution";
const LICENSES: &str = "https://aka.ms/MinecraftJavaLicenses";

impl Controls {
    /// `CreditsAndAttributionScreen`, from the title's copyright line or the options.
    pub(super) fn open_credits_attribution(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        self.credits_attribution_screen = Some(CreditsAttributionScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            !self.title_host,
        ));
    }

    /// The copyright line of the title screen.
    pub(crate) fn open_title_credits_attribution(
        &mut self,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        self.open_credits_attribution(surface, assets);
    }

    /// Whether the Credits button was pressed since the last call: the title's menu shows the roll.
    pub(crate) fn take_credits_request(&mut self) -> bool {
        std::mem::take(&mut self.credits_request)
    }

    /// `WinScreen(false, () -> setScreen(this))`: the roll without the poem, over this screen.
    fn open_credits_roll(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        if self.title_host {
            self.credits_request = true;
            return;
        }
        let user = std::env::var("PHOTON_USER").unwrap_or_else(|_| "Photon".to_owned());
        self.credits = Some(CreditsScreen::new(
            &assets.creative.font,
            &*assets.resources,
            Self::gui_size(surface),
            false,
            &user,
        ));
        self.credits_from_options = true;
    }

    /// `ConfirmLinkScreen.confirmLink`: an address to be confirmed over this screen.
    fn confirm_credits_link(&mut self, surface: &WindowSurface, assets: &ScreenAssets, url: &str) {
        let screen = ConfirmScreen::link(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            url,
            true,
        )
        .with_background(ConfirmBackground::Menu {
            in_world: !self.title_host,
        });
        self.link_confirm = Some((screen, ConfirmTarget::Link(url.to_owned())));
    }

    fn credits_attribution_action(
        &mut self,
        action: CreditsAction,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        match action {
            CreditsAction::Credits => self.open_credits_roll(surface, assets),
            CreditsAction::Attribution => self.confirm_credits_link(surface, assets, ATTRIBUTION),
            CreditsAction::Licenses => self.confirm_credits_link(surface, assets, LICENSES),
            CreditsAction::Done => self.credits_attribution_screen = None,
        }
    }

    /// A scripted Escape on the credits and attribution or the skin screen: whether one was open.
    pub(super) fn close_skin_or_credits(&mut self) -> bool {
        if self.credits_attribution_screen.is_some() {
            self.credits_attribution_screen = None;
        } else if self.skin_screen.is_some() {
            self.close_skin();
        } else {
            return false;
        }
        true
    }

    /// An event for the open credits and attribution screen.
    pub(super) fn credits_attribution_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        if let WindowEvent::CursorPos(x, y) = event {
            self.move_gui_mouse((x, y), surface);
            return;
        }
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let Some(screen) = self.credits_attribution_screen.as_mut() else {
            return;
        };
        let action = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => Some(CreditsAction::Done),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                screen.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                screen.pressed()
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                screen.clicked(mouse)
            }
            _ => None,
        };
        if let Some(action) = action {
            self.credits_attribution_action(action, surface, assets);
        }
    }
}
