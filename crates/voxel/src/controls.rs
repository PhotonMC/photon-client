//! The player's hands on the client: the keys and the mouse, a script's
//! steps, and the inventory screen they may be typing into.

use std::time::{Duration, Instant};
use voxel_game::attack::AttackKey;
use voxel_game::hotbar::ScrollWheel;
use voxel_game::options::Options;
use voxel_net::connection::PlaySession;
use voxel_render::gui::chat::ChatSettings;
use voxel_render::gui::chat::{ChatDrafts, ChatKey, ChatLog, ChatMethod, ChatScreen, ExitReason};
use voxel_render::gui::chat_options::ChatOptionsScreen;
use voxel_render::gui::confirm::ConfirmAction as LinkAction;
use voxel_render::gui::confirm::{ConfirmAction, ConfirmBackground, ConfirmScreen};
use voxel_render::gui::death::{DeathAction, DeathScreen};
use voxel_render::gui::disconnected::DisconnectedScreen;
use voxel_render::gui::language_select::LanguageSelectScreen;
use voxel_render::gui::mouse_options::{ControlsScreen, MouseSettingsScreen};
use voxel_render::gui::options::{OptionsAction, OptionsInput, OptionsScreen};
use voxel_render::gui::pause::{PauseAction, PauseScreen};
use voxel_render::gui::screen::{KeyMods, KeyResult};
use voxel_render::gui::slider::SliderKey;
use voxel_render::gui::sound_options::SoundOptionsScreen;
use voxel_render::gui::video::VideoSettingsScreen;
use voxel_render::window::WindowSurface;

use crate::screens::{
    OpenScreen, ScreenAssets, WindowClipboard, close_inventory, open_inventory, run_actions,
};
use crate::script::HeldKeys;
use crate::view::CameraType;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

/// What the player is holding and pointing at between events.
pub struct Controls {
    pub keys: HeldKeys,
    /// Attack clicks waiting for the next tick, and `Minecraft.missTime`:
    /// ticks a missed attack keeps the next one waiting.
    pub attack_key: AttackKey,
    /// The use key keeps its clicks and held state the same way.
    pub use_key: AttackKey,
    /// The pick item key (the middle button) and whether a control key is
    /// down (`Minecraft.hasControlDown`), which asks for the item's data too.
    pub pick_key: AttackKey,
    pub ctrl_down: bool,
    scroll_wheel: ScrollWheel,
    /// F5 cycles first person, third person from behind and from the front.
    pub camera_type: CameraType,
    /// Whether the mouse is grabbed to look around.
    pub captured: bool,
    last_cursor: Option<(f64, f64)>,
    /// The open inventory screen, if any.
    pub screen: OpenScreen,
    /// The id of the container the server opened, while its screen is up.
    server_container: Option<i32>,
    /// The creative screen's tab, kept for the next time it opens.
    pub creative_tab: usize,
    /// The chat box, while it is open, what was sent through it and the
    /// lines of chat the player has been told.
    pub chat: Option<ChatScreen>,
    /// The pause menu, while it is open.
    pub pause: Option<PauseScreen>,
    /// The options screen, opened from the pause menu, which waits under it.
    pub options_screen: Option<OptionsScreen>,
    /// The video settings, opened from the options screen, which waits under
    /// it.
    pub video_screen: Option<VideoSettingsScreen>,
    /// The chat settings, opened the same way.
    pub chat_options_screen: Option<ChatOptionsScreen>,
    pub sound_screen: Option<SoundOptionsScreen>,
    /// The controls screen, and the mouse settings over it.
    pub controls_screen: Option<ControlsScreen>,
    /// The language screen, and the language it chose, for the game to load.
    pub language_screen: Option<LanguageSelectScreen>,
    language_request: Option<String>,
    /// When and where the primary button last went down.
    last_press: Option<(Instant, (i32, i32))>,
    pub mouse_screen: Option<MouseSettingsScreen>,
    /// The death screen, from the server's word of a death until the player
    /// has health again.
    pub death: Option<DeathScreen>,
    /// "Title Screen" asks first (`deathScreen.quit.confirm`), over the death
    /// screen, which this takes the place of while it is open.
    pub confirm: Option<ConfirmScreen>,
    /// `ConfirmLinkScreen`, over the chat, and the link it asks about.
    pub link_confirm: Option<(ConfirmScreen, String)>,
    /// The screen a lost connection leaves, which nothing replaces.
    pub disconnected: Option<DisconnectedScreen>,
    /// The values of `options.txt` the client reads.
    pub options: Options,
    chat_history: Vec<String>,
    /// `ChatComponent.latestDraft`.
    chat_drafts: ChatDrafts,
    /// `CommandHistory`: the commands sent, kept in a file.
    command_history: Vec<String>,
    /// `Minecraft.hasControlDown`: a control key is down (the command key on
    /// macOS).
    pub control_held: bool,
    /// Whether a shift key is down, which wheel events carry no mark of.
    shift_held: bool,
    pub chat_log: ChatLog,
    /// Whether the tab list key is held.
    pub tab_held: bool,
    /// The mouse in GUI units.
    pub gui_mouse: (f64, f64),
    /// The button a drag is held with (`MouseHandler.activeButton`).
    active_button: Option<i32>,
}

/// `Util.getPlatform().openUri`: the system's own opener.
fn open_uri(uri: &str) {
    let (program, arguments): (&str, &[&str]) = if cfg!(target_os = "macos") {
        ("open", &[])
    } else if cfg!(target_os = "windows") {
        ("cmd", &["/c", "start", ""])
    } else {
        ("xdg-open", &[])
    };
    let _ = std::process::Command::new(program)
        .args(arguments)
        .arg(uri)
        .spawn();
}

/// `MouseHandler.turnPlayer` for a mouse that moved `(dx, dy)`: scaled by
/// the sensitivity cubed (`* 0.6F + 0.2F` widens floats), either axis
/// reversed by its inversion option.
fn turn_by(options: &Options, (dx, dy): (f64, f64)) -> (f64, f64) {
    let sensitivity = options.sensitivity * f64::from(0.6_f32) + f64::from(0.2_f32);
    let scaled = sensitivity * sensitivity * sensitivity * 8.0;
    let (turn_x, turn_y) = (dx * scaled, dy * scaled);
    (
        if options.invert_mouse_x {
            -turn_x
        } else {
            turn_x
        },
        if options.invert_mouse_y {
            -turn_y
        } else {
            turn_y
        },
    )
}

/// A key a settings screen gives to its widgets: Tab and Shift+Tab, or an
/// arrow, Enter or Space with whether Shift is held.
enum MenuKey {
    Tab(bool),
    Key(SliderKey, bool),
}

fn menu_key(event: &glfw::WindowEvent) -> Option<MenuKey> {
    use glfw::{Action, Key, Modifiers, WindowEvent};
    let WindowEvent::Key(key, _, action, modifiers) = event else {
        return None;
    };
    let shift = modifiers.contains(Modifiers::Shift);
    let held = matches!(action, Action::Press | Action::Repeat);
    match key {
        Key::Tab if held => Some(MenuKey::Tab(!shift)),
        Key::Left if held => Some(MenuKey::Key(SliderKey::Left, shift)),
        Key::Right if held => Some(MenuKey::Key(SliderKey::Right, shift)),
        Key::Up if held => Some(MenuKey::Key(SliderKey::Up, shift)),
        Key::Down if held => Some(MenuKey::Key(SliderKey::Down, shift)),
        Key::Enter | Key::KpEnter | Key::Space if *action == Action::Press => {
            Some(MenuKey::Key(SliderKey::Select, shift))
        }
        _ => None,
    }
}

impl Controls {
    pub fn new(camera_type: CameraType, captured: bool) -> Self {
        let command_history = read_command_history();
        let options = load_options();
        let chat_settings = ChatSettings::from(&options);
        Self {
            keys: HeldKeys::default(),
            attack_key: AttackKey::default(),
            use_key: AttackKey::default(),
            pick_key: AttackKey::default(),
            ctrl_down: false,
            scroll_wheel: ScrollWheel::default(),
            camera_type,
            captured,
            last_cursor: None,
            screen: None,
            server_container: None,
            creative_tab: voxel_game::creative_menu::default_tab(),
            chat: None,
            pause: None,
            options_screen: None,
            video_screen: None,
            chat_options_screen: None,
            sound_screen: None,
            controls_screen: None,
            language_screen: None,
            language_request: None,
            last_press: None,
            mouse_screen: None,
            death: None,
            confirm: None,
            link_confirm: None,
            disconnected: None,
            options,
            chat_history: command_history.clone(),
            chat_drafts: ChatDrafts::default(),
            command_history,
            shift_held: false,
            control_held: false,
            chat_log: ChatLog::with_settings(chat_settings),
            tab_held: false,
            gui_mouse: (0.0, 0.0),
            active_button: None,
        }
    }

    /// `PauseScreen`: the menu opens and the mouse is let go. The game goes
    /// on behind it, as on a server.
    fn open_pause(&mut self, surface: &mut WindowSurface, assets: &ScreenAssets) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.pause = Some(PauseScreen::new(&assets.creative.font, &assets.lang, size));
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// The menu closes and the mouse is grabbed again.
    fn close_pause(&mut self, surface: &mut WindowSurface) {
        self.pause = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// What a pause menu button does: "Back to Game" closes the menu and
    /// "Disconnect" leaves; the screens the others open are not built.
    fn pause_action(
        &mut self,
        action: PauseAction,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        match action {
            PauseAction::Resume => self.close_pause(surface),
            PauseAction::Disconnect => surface.request_close(),
            PauseAction::Options => self.open_options(surface, assets),
            _ => {}
        }
    }

    /// `OptionsScreen`, over the pause menu, in a world.
    fn open_options(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.options_screen = Some(OptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            self.options.fov,
            true,
        ));
    }

    /// `OptionsScreen.onClose` and `removed`: back to the pause menu, and
    /// the options are saved.
    fn close_options(&mut self) {
        self.options_screen = None;
        let _ = self.options.save(&options_path());
    }

    /// What an input to the options screen asks for: a new field of view
    /// takes effect at once; Done leaves; Video opens the video settings;
    /// the screens the others open are not built.
    fn options_input(
        &mut self,
        input: OptionsInput,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        if let Some(fov) = input.fov {
            self.options.fov = fov;
        }
        match input.action {
            Some(OptionsAction::Done) => self.close_options(),
            Some(OptionsAction::Video) => self.open_video(surface, assets),
            Some(OptionsAction::Chat) => self.open_chat_options(surface, assets),
            Some(OptionsAction::Sounds) => self.open_sound_options(surface, assets),
            Some(OptionsAction::Controls) => self.open_controls(surface, assets),
            Some(OptionsAction::Language) => self.open_language(surface, assets),
            _ => {}
        }
    }

    /// `VideoSettingsScreen`, over the options screen.
    fn open_video(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let scale = surface.scale();
        self.video_screen = Some(VideoSettingsScreen::new(
            &assets.creative.font,
            &assets.lang,
            (scale.gui_scaled_width(), scale.gui_scaled_height()),
            true,
            self.options.gui_scale,
            scale.calculate_scale(0, false),
            self.options.attack_indicator,
        ));
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen,
    /// and the options are saved.
    fn close_video(&mut self) {
        self.video_screen = None;
        let _ = self.options.save(&options_path());
    }

    fn gui_size(surface: &WindowSurface) -> (i32, i32) {
        let scale = surface.scale();
        (scale.gui_scaled_width(), scale.gui_scaled_height())
    }

    /// `LanguageSelectScreen`, over the options screen.
    fn open_language(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let Some(source) = assets.languages.as_ref() else {
            return;
        };
        self.language_screen = Some(LanguageSelectScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            source.languages().into_iter().collect(),
            &self.options.language_code,
        ));
    }

    /// The language the player chose, once.
    pub fn take_language_request(&mut self) -> Option<String> {
        self.language_request.take()
    }

    /// The screens that hold their words are made again in the language
    /// now loaded.
    pub fn rebuild_menus(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let size = Self::gui_size(surface);
        if self.pause.is_some() {
            self.pause = Some(PauseScreen::new(&assets.creative.font, &assets.lang, size));
        }
        if self.options_screen.is_some() {
            self.open_options(surface, assets);
        }
    }

    /// An event for the open language screen.
    fn language_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let font = &assets.creative.font;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            let mouse = self.gui_mouse;
            if let Some(screen) = self.language_screen.as_mut() {
                screen.dragged(mouse, mouse.1 - before);
            }
            return;
        }
        let mouse = self.gui_mouse;
        let control = self.control_held;
        let wheel = self.options.mouse_wheel_sensitivity;
        let Some(screen) = self.language_screen.as_mut() else {
            return;
        };
        let mut input = None;
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                input = Some(voxel_render::gui::language_select::LanguageInput {
                    done: true,
                    language: None,
                });
            }
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                input = Some(screen.key_pressed(
                    font,
                    key,
                    (control, shift),
                    &mut WindowClipboard(surface),
                ));
            }
            WindowEvent::Char(character) => screen.char_typed(font, character),
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                // The window's own double click timing is not kept: two
                // presses at the same place within 250 ms.
                let now = Instant::now();
                let double = self.last_press.is_some_and(|(at, place)| {
                    now - at < Duration::from_millis(250)
                        && place == (mouse.0 as i32, mouse.1 as i32)
                });
                self.last_press = Some((now, (mouse.0 as i32, mouse.1 as i32)));
                input = Some(screen.clicked(font, mouse, double));
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => screen.released(),
            WindowEvent::Scroll(_, y) => screen.scrolled(mouse, y * wheel),
            _ => {}
        }
        if let Some(input) = input.filter(|input| input.done) {
            self.language_screen = None;
            if let Some(code) = input.language {
                self.options.language_code.clone_from(&code);
                self.language_request = Some(code);
            }
            let _ = self.options.save(&options_path());
        }
    }

    /// `ControlsScreen`, over the options screen.
    fn open_controls(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.controls_screen = Some(ControlsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
        ));
    }

    /// `MouseSettingsScreen`, over the controls screen.
    fn open_mouse_settings(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.mouse_screen = Some(MouseSettingsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            &self.options,
        ));
    }

    /// The mouse options the screen shows are the options.
    fn take_mouse_choices(&mut self) {
        if let Some(screen) = self.mouse_screen.as_ref() {
            let choices = screen.choices();
            self.options.sensitivity = choices.sensitivity;
            self.options.mouse_wheel_sensitivity = choices.wheel_sensitivity;
            self.options.invert_mouse_x = choices.invert_x;
            self.options.invert_mouse_y = choices.invert_y;
        }
    }

    /// An event for the open mouse settings or the controls screen under
    /// them.
    fn controls_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        let wheel = self.options.mouse_wheel_sensitivity;
        let mut done = false;
        let mut open_mouse = false;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            let (mouse, dy) = (self.gui_mouse, self.gui_mouse.1 - before);
            if let Some(screen) = self.mouse_screen.as_mut() {
                screen.dragged(lang, mouse, dy);
            } else if let Some(screen) = self.controls_screen.as_mut() {
                screen.dragged(lang, mouse, dy);
            }
            self.take_mouse_choices();
            return;
        }
        let mouse = self.gui_mouse;
        if let Some(key) = menu_key(&event) {
            if let Some(screen) = self.mouse_screen.as_mut() {
                match key {
                    MenuKey::Tab(forward) => screen.tab(forward),
                    MenuKey::Key(key, shift) => done = screen.key_pressed(lang, key, shift).done,
                }
            } else if let Some(screen) = self.controls_screen.as_mut() {
                match key {
                    MenuKey::Tab(forward) => screen.tab(forward),
                    MenuKey::Key(key, shift) => {
                        let input = screen.key_pressed(lang, key, shift);
                        done = input.done;
                        open_mouse = input.mouse_settings;
                    }
                }
            }
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                if let Some(screen) = self.mouse_screen.as_mut() {
                    done = screen.clicked(lang, mouse, shift).done;
                } else if let Some(screen) = self.controls_screen.as_mut() {
                    let input = screen.clicked(lang, mouse, shift);
                    done = input.done;
                    open_mouse = input.mouse_settings;
                }
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                if let Some(screen) = self.mouse_screen.as_mut() {
                    screen.released(lang);
                } else if let Some(screen) = self.controls_screen.as_mut() {
                    screen.released(lang);
                }
            }
            WindowEvent::Scroll(_, y) => {
                if let Some(screen) = self.mouse_screen.as_mut() {
                    screen.scrolled(mouse, y * wheel);
                } else if let Some(screen) = self.controls_screen.as_mut() {
                    screen.scrolled(mouse, y * wheel);
                }
            }
            _ => {}
        }
        self.take_mouse_choices();
        if open_mouse {
            self.open_mouse_settings(surface, assets);
        } else if done {
            self.close_controls();
        }
    }

    /// Done or Escape on the mouse settings goes back to the controls
    /// screen, on that to the options; each saves the options.
    fn close_controls(&mut self) {
        if self.mouse_screen.take().is_none() {
            self.controls_screen = None;
        }
        let _ = self.options.save(&options_path());
    }

    /// `SoundOptionsScreen`, over the options screen.
    fn open_sound_options(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        self.sound_screen = Some(SoundOptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            true,
            &self.options,
        ));
    }

    /// `OptionsSubScreen.onClose`: back to the options screen, and the
    /// options are saved.
    fn close_sound_options(&mut self) {
        self.sound_screen = None;
        let _ = self.options.save(&options_path());
    }

    /// The sound options the screen shows are the options.
    fn take_sound_choices(&mut self) {
        let Some(screen) = self.sound_screen.as_ref() else {
            return;
        };
        let choices = screen.choices();
        let options = &mut self.options;
        options.sound_volumes = choices.volumes;
        options.show_subtitles = choices.show_subtitles;
        options.directional_audio = choices.directional_audio;
        options.music_frequency = choices.music_frequency;
        options.music_toast = choices.music_toast;
    }

    /// An event for the open sound settings.
    fn sound_options_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(screen) = self.sound_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            self.take_sound_choices();
            return;
        }
        let Some(screen) = self.sound_screen.as_mut() else {
            return;
        };
        let mut done = false;
        match menu_key(&event) {
            Some(MenuKey::Tab(forward)) => screen.tab(forward),
            Some(MenuKey::Key(key, shift)) => done = screen.key_pressed(lang, key, shift).done,
            None => {}
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                done = screen.clicked(lang, self.gui_mouse, shift).done;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released(lang);
            }
            WindowEvent::Scroll(_, y) => {
                screen.scrolled(self.gui_mouse, y * self.options.mouse_wheel_sensitivity);
            }
            _ => {}
        }
        self.take_sound_choices();
        if done {
            self.close_sound_options();
        }
    }

    /// `ChatOptionsScreen`, over the options screen.
    fn open_chat_options(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let scale = surface.scale();
        self.chat_options_screen = Some(ChatOptionsScreen::new(
            &assets.creative.font,
            &assets.lang,
            (scale.gui_scaled_width(), scale.gui_scaled_height()),
            true,
            &self.options,
        ));
    }

    /// `OptionsSubScreen.onClose` and `removed`: back to the options screen,
    /// and the options are saved.
    fn close_chat_options(&mut self) {
        self.chat_options_screen = None;
        let _ = self.options.save(&options_path());
    }

    /// The chat options the screen shows are the options: the chat wraps
    /// its lines again when they changed (`rescaleChat`).
    fn take_chat_choices(&mut self, assets: &ScreenAssets) {
        let Some(screen) = self.chat_options_screen.as_ref() else {
            return;
        };
        let choices = screen.choices();
        let options = &mut self.options;
        options.chat_visibility = choices.visibility;
        options.save_chat_drafts = choices.save_drafts;
        options.chat_width = choices.settings.width;
        options.chat_scale = choices.settings.scale;
        options.chat_opacity = choices.settings.opacity;
        options.text_background_opacity = choices.settings.background_opacity;
        options.chat_line_spacing = choices.settings.line_spacing;
        options.chat_height_focused = choices.settings.height_focused;
        options.chat_height_unfocused = choices.settings.height_unfocused;
        self.chat_log
            .set_settings(choices.settings, &assets.creative.font);
    }

    /// An event for the open chat settings.
    fn chat_options_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(screen) = self.chat_options_screen.as_mut() {
                screen.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            self.take_chat_choices(assets);
            return;
        }
        let Some(screen) = self.chat_options_screen.as_mut() else {
            return;
        };
        let mut done = false;
        match menu_key(&event) {
            Some(MenuKey::Tab(forward)) => screen.tab(forward),
            Some(MenuKey::Key(key, shift)) => done = screen.key_pressed(lang, key, shift).done,
            None => {}
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                done = screen.clicked(lang, self.gui_mouse, shift).done;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released(lang);
            }
            WindowEvent::Scroll(_, y) => {
                screen.scrolled(self.gui_mouse, y * self.options.mouse_wheel_sensitivity);
            }
            _ => {}
        }
        self.take_chat_choices(assets);
        if done {
            self.close_chat_options();
        }
    }

    /// An event for the open video settings.
    fn video_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let lang = &assets.lang;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            if let Some(video) = self.video_screen.as_mut() {
                video.dragged(lang, self.gui_mouse, self.gui_mouse.1 - before);
            }
            return;
        }
        let Some(video) = self.video_screen.as_mut() else {
            return;
        };
        // `Minecraft.resizeGui`: a new GUI scale is the option's, and the
        // window answers with the size it gives the screen.
        let wheel = self.options.mouse_wheel_sensitivity;
        let options = &mut self.options;
        let mut resize = |scale: i32| {
            options.gui_scale = scale;
            surface.set_gui_scale_option(scale);
            let size = surface.scale();
            (size.gui_scaled_width(), size.gui_scaled_height())
        };
        let mut done = false;
        match menu_key(&event) {
            Some(MenuKey::Tab(forward)) => video.tab(forward),
            Some(MenuKey::Key(key, shift)) => {
                done = video.key_pressed(lang, key, shift, &mut resize).done;
            }
            None => {}
        }
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => done = true,
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                done = video.clicked(lang, self.gui_mouse, shift, &mut resize).done;
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                video.released(lang);
            }
            WindowEvent::Scroll(_, y) => {
                video.scrolled(
                    lang,
                    self.gui_mouse,
                    y * wheel,
                    self.control_held,
                    &mut resize,
                );
            }
            _ => {}
        }
        self.options.attack_indicator = video.attack_indicator();
        if done {
            self.close_video();
        }
    }

    /// An event for the open options screen.
    fn options_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, WindowEvent};
        let lang = &assets.lang;
        let Some(screen) = self.options_screen.as_mut() else {
            return;
        };
        let input = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => OptionsInput {
                action: Some(OptionsAction::Done),
                fov: None,
            },
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                screen.tab(!modifiers.contains(glfw::Modifiers::Shift));
                OptionsInput::default()
            }
            WindowEvent::Key(Key::Left, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(lang, SliderKey::Left)
            }
            WindowEvent::Key(Key::Right, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(lang, SliderKey::Right)
            }
            WindowEvent::Key(Key::Up, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(lang, SliderKey::Up)
            }
            WindowEvent::Key(Key::Down, _, Action::Press | Action::Repeat, _) => {
                screen.key_pressed(lang, SliderKey::Down)
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                screen.key_pressed(lang, SliderKey::Select)
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                match self.options_screen.as_mut() {
                    Some(screen) => screen.dragged(lang, self.gui_mouse.0),
                    None => OptionsInput::default(),
                }
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                screen.clicked(lang, self.gui_mouse)
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Release, _) => {
                screen.released(lang);
                OptionsInput::default()
            }
            _ => OptionsInput::default(),
        };
        self.options_input(input, surface, assets);
    }

    /// `onDisconnect`: a lost connection puts the disconnected screen in the
    /// place of whatever was open.
    pub fn receive_disconnect(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(reason) = session.disconnected() else {
            return;
        };
        if self.disconnected.is_some() {
            return;
        }
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.disconnected = Some(DisconnectedScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            reason,
        ));
        self.chat = None;
        self.link_confirm = None;
        self.pause = None;
        self.options_screen = None;
        self.video_screen = None;
        self.chat_options_screen = None;
        self.sound_screen = None;
        self.controls_screen = None;
        self.language_screen = None;
        self.mouse_screen = None;
        self.death = None;
        self.confirm = None;
        self.screen = None;
        self.captured = false;
        self.last_cursor = None;
        surface.set_cursor_captured(false);
        self.release_keys();
    }

    /// An event for the disconnected screen: it takes no Escape, and its
    /// button leaves, there being no server list to go back to.
    fn disconnected_event(&mut self, event: glfw::WindowEvent, surface: &mut WindowSurface) {
        use glfw::{Action, Key, WindowEvent};
        let Some(screen) = self.disconnected.as_mut() else {
            return;
        };
        match event {
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, _) => screen.tab(),
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _)
                if screen.pressed() =>
            {
                surface.request_close();
            }
            WindowEvent::CursorPos(x, y) => self.move_gui_mouse((x, y), surface),
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _)
                if screen.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32)) =>
            {
                surface.request_close();
            }
            _ => {}
        }
    }

    /// `handlePlayerCombatKill`: a death the server told of opens the death
    /// screen, in the place of whatever was open; the player having health
    /// again closes it.
    pub fn receive_death(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        if let Some(message) = session.take_death() {
            let size = surface.scale();
            let size = (size.gui_scaled_width(), size.gui_scaled_height());
            let score = session
                .world()
                .local_player()
                .map_or(0, voxel_world::ClientPlayerState::score);
            self.death = Some(DeathScreen::new(
                &assets.lang,
                size,
                Some(&message),
                session.hardcore(),
                score,
            ));
            self.confirm = None;
            if let Some(chat) = self.chat.take() {
                // A death takes the box's place: its text is kept as a draft.
                self.leave_chat(&chat, ExitReason::Interrupted);
            }
            self.pause = None;
            self.options_screen = None;
            self.video_screen = None;
            self.chat_options_screen = None;
            self.sound_screen = None;
            self.controls_screen = None;
            self.language_screen = None;
            self.mouse_screen = None;
            self.screen = None;
            self.captured = false;
            self.last_cursor = None;
            surface.set_cursor_captured(false);
            self.release_keys();
        } else if self.death.is_some() && session.status().health > 0.0 {
            self.close_death(surface);
        }
    }

    /// The death screen goes, with the confirmation over it, and the mouse
    /// is grabbed again (`setScreen(null)`).
    fn close_death(&mut self, surface: &mut WindowSurface) {
        self.death = None;
        self.confirm = None;
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// What a death screen button does: "Respawn" asks the server; "Title
    /// Screen" leaves, after a confirmation unless the world is hardcore
    /// (`handleExitToTitleScreen`).
    fn death_action(
        &mut self,
        action: DeathAction,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        match action {
            DeathAction::Respawn => session.respawn()?,
            DeathAction::TitleScreen if session.hardcore() => surface.request_close(),
            DeathAction::TitleScreen => self.open_title_confirm(surface, assets),
        }
        Ok(())
    }

    /// `DeathScreen.TitleConfirmScreen`, its buttons asleep for a second.
    fn open_title_confirm(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        use voxel_game::text::Component;
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let mut confirm = ConfirmScreen::new(
            &assets.creative.font,
            &assets.lang,
            size,
            &Component::translatable("deathScreen.quit.confirm"),
            &Component::empty(),
            &Component::translatable("deathScreen.titleScreen"),
            &Component::translatable("deathScreen.respawn"),
        )
        .with_background(ConfirmBackground::Death);
        confirm.set_delay(20);
        self.confirm = Some(confirm);
    }

    /// What the confirmation's callback does: yes leaves; no respawns and
    /// closes the screens.
    fn confirm_action(
        &mut self,
        action: ConfirmAction,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        match action {
            ConfirmAction::Yes => surface.request_close(),
            ConfirmAction::No => {
                session.respawn()?;
                self.close_death(surface);
            }
            // The death confirmation has no third button.
            ConfirmAction::Copy => {}
        }
        Ok(())
    }

    /// A scripted Escape: the confirmation answers no once awake; else the
    /// pause menu opens, or closes.
    fn scripted_escape(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if let Some(confirm) = &self.confirm {
            if let Some(action) = confirm.escape() {
                self.confirm_action(action, session, surface)?;
            }
        } else if self.language_screen.is_some() {
            self.language_screen = None;
        } else if self.mouse_screen.is_some() || self.controls_screen.is_some() {
            self.close_controls();
        } else if self.sound_screen.is_some() {
            self.close_sound_options();
        } else if self.chat_options_screen.is_some() {
            self.close_chat_options();
        } else if self.video_screen.is_some() {
            self.close_video();
        } else if self.options_screen.is_some() {
            self.close_options();
        } else if self.pause.is_some() {
            self.close_pause(surface);
        } else {
            self.open_pause(surface, assets);
        }
        Ok(())
    }

    /// A scripted press of the primary button on the confirmation, the death
    /// screen or the pause menu, at the mouse.
    fn scripted_press(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        if let Some(confirm) = &self.confirm {
            if let Some(action) = confirm.clicked(mouse) {
                self.confirm_action(action, session, surface)?;
            }
        } else if let Some(screen) = self.options_screen.as_mut() {
            let input = screen.clicked(&assets.lang, self.gui_mouse);
            screen.released(&assets.lang);
            self.options_input(input, surface, assets);
        } else if let Some(action) = self.death.as_mut().and_then(|death| death.clicked(mouse)) {
            self.death_action(action, session, surface, assets)?;
        } else if let Some(action) = self.pause.as_ref().and_then(|pause| pause.clicked(mouse)) {
            self.pause_action(action, surface, assets);
        }
        Ok(())
    }

    /// The mouse in GUI units, from a cursor position in pixels.
    fn move_gui_mouse(&mut self, (x, y): (f64, f64), surface: &WindowSurface) {
        let size = surface.scale();
        let (width, height) = surface.window_size();
        self.gui_mouse = (
            x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
            y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
        );
    }

    /// An event for the open confirmation: Escape answers no once it is
    /// awake.
    fn confirm_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let Some(confirm) = self.confirm.as_mut() else {
            return Ok(());
        };
        let action = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => confirm.escape(),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                confirm.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                confirm.pressed()
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                confirm.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32))
            }
            _ => None,
        };
        match action {
            Some(action) => self.confirm_action(action, session, surface),
            None => Ok(()),
        }
    }

    /// An event for the open death screen: it takes no Escape.
    fn death_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, WindowEvent};
        let Some(death) = self.death.as_mut() else {
            return Ok(());
        };
        let action = match event {
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                death.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                death.pressed()
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                None
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                death.clicked((self.gui_mouse.0 as i32, self.gui_mouse.1 as i32))
            }
            _ => None,
        };
        match action {
            Some(action) => self.death_action(action, session, surface, assets),
            None => Ok(()),
        }
    }

    /// An event for the open pause menu.
    fn pause_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, WindowEvent};
        let Some(pause) = self.pause.as_mut() else {
            return;
        };
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => self.close_pause(surface),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                pause.tab(!modifiers.contains(glfw::Modifiers::Shift));
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                if let Some(action) = pause.pressed() {
                    self.pause_action(action, surface, assets);
                }
            }
            WindowEvent::Key(
                key @ (Key::Up | Key::Down | Key::Left | Key::Right),
                _,
                Action::Press | Action::Repeat,
                _,
            ) => {
                use voxel_render::gui::navigation::Direction;
                pause.arrow(match key {
                    Key::Up => Direction::Up,
                    Key::Down => Direction::Down,
                    Key::Left => Direction::Left,
                    _ => Direction::Right,
                });
            }
            WindowEvent::CursorPos(x, y) => {
                pause.mouse_input();
                let size = surface.scale();
                let (width, height) = surface.window_size();
                self.gui_mouse = (
                    x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
                    y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
                );
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
                if let Some(action) = pause.clicked(mouse) {
                    self.pause_action(action, surface, assets);
                }
            }
            _ => {}
        }
    }

    /// `ChatComponent.openScreen`: the chat box opens on the draft a closed
    /// one left, or empty (a slash for a command), and the mouse is let go.
    /// `keyed` is whether a key opened it.
    fn open_chat(
        &mut self,
        method: ChatMethod,
        keyed: bool,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let (initial, draft) = self.chat_drafts.open(method);
        let mut screen = ChatScreen::new(
            assets.creative.font.clone(),
            size,
            &initial,
            self.chat_history.clone(),
        );
        if draft {
            screen = screen.as_draft();
        }
        self.chat = Some(if keyed { screen } else { screen.without_key() });
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
    }

    /// `ChatScreen.removed`: the box's text is kept or dropped as a draft by
    /// the way it went.
    fn leave_chat(&mut self, chat: &ChatScreen, reason: ExitReason) {
        self.chat_history = chat.history().to_vec();
        self.chat_drafts.removed(
            &chat.value(),
            chat.is_draft(),
            reason,
            self.options.save_chat_drafts,
        );
    }

    /// The chat box closes, keeping what was sent, and the mouse is grabbed.
    fn close_chat(&mut self, surface: &mut WindowSurface, reason: ExitReason) {
        if let Some(chat) = self.chat.take() {
            self.leave_chat(&chat, reason);
        }
        self.chat_log.reset_scroll();
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
    }

    /// `ChatComponent.addRecentChat`'s second half: a command is kept in the
    /// history file (the last 50, no repeat of the last).
    fn remember_command(&mut self, text: &str) {
        if !text.starts_with('/') || self.command_history.last().is_some_and(|last| last == text) {
            return;
        }
        if self.command_history.len() >= COMMAND_HISTORY_LIMIT {
            self.command_history.remove(0);
        }
        self.command_history.push(text.to_owned());
        write_command_history(&self.command_history);
    }

    /// `ClientPacketListener.sendChat` or `sendCommand` for a typed line.
    fn send_chat(session: &mut PlaySession, text: &str) -> Result {
        match text.strip_prefix('/') {
            Some(command) => session.send_chat(command, true)?,
            None => session.send_chat(text, false)?,
        }
        Ok(())
    }

    /// The request the chat box's last edit asked for is sent, and an
    /// answer that came is handed to it.
    fn send_suggestion_request(&mut self, session: &mut PlaySession) -> Result {
        let Some(chat) = self.chat.as_mut() else {
            return Ok(());
        };
        if let Some((id, text)) = chat.take_request() {
            session.request_suggestions(id, &text)?;
        }
        Ok(())
    }

    /// An answer to a suggestion request reaches the open chat box.
    pub fn receive_suggestions(&mut self, session: &mut PlaySession) {
        if let Some(chat) = self.chat.as_mut() {
            chat.set_commands(session.commands().cloned());
            chat.set_selectors(session.can_use_selectors());
            chat.set_names(
                session.online_player_names(),
                session.scoreboard().team_names(),
            );
            chat.set_selected_entities(session.selected_entities());
        }
        if let (Some(chat), Some(answer)) = (self.chat.as_mut(), session.take_suggestions()) {
            chat.suggested(answer.id, answer.start, answer.length, answer.matches);
        }
    }

    /// An event for the open chat box.
    fn chat_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let Some(chat) = self.chat.as_mut() else {
            return Ok(());
        };
        chat.set_commands(session.commands().cloned());
        chat.set_selectors(session.can_use_selectors());
        chat.set_names(
            session.online_player_names(),
            session.scoreboard().team_names(),
        );
        chat.set_selected_entities(session.selected_entities());
        let size = surface.scale();
        chat.resize((size.gui_scaled_width(), size.gui_scaled_height()));
        if let glfw::WindowEvent::Key(glfw::Key::LeftShift | glfw::Key::RightShift, _, action, _) =
            event
        {
            self.shift_held = action != glfw::Action::Release;
        }
        match event {
            glfw::WindowEvent::Key(
                key,
                _,
                glfw::Action::Press | glfw::Action::Repeat,
                modifiers,
            ) => {
                let control = modifiers.contains(glfw::Modifiers::Control)
                    || modifiers.contains(glfw::Modifiers::Super);
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                match chat.key_pressed(key, (control, shift), &mut WindowClipboard(surface)) {
                    ChatKey::Close => self.close_chat(surface, ExitReason::Intentional),
                    ChatKey::Done => self.close_chat(surface, ExitReason::Done),
                    ChatKey::Send(text) => {
                        Self::send_chat(session, &text)?;
                        self.remember_command(&text);
                        self.close_chat(surface, ExitReason::Done);
                    }
                    ChatKey::Page(direction) => {
                        let page = self.chat_log.lines_per_page(true) as i32 - 1;
                        self.chat_log.scroll_by(direction * page, true);
                    }
                    ChatKey::None => {}
                }
                self.send_suggestion_request(session)?;
            }
            glfw::WindowEvent::Char(character) => {
                chat.char_typed(character);
                self.send_suggestion_request(session)?;
            }
            glfw::WindowEvent::Scroll(_, y) => {
                // `ChatScreen.mouseScrolled`: the list of matches first, then
                // the chat by a notch of seven lines.
                let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
                if !chat.mouse_scrolled(y, mouse) {
                    let notch = y.clamp(-1.0, 1.0);
                    let lines = if self.shift_held { notch } else { notch * 7.0 };
                    self.chat_log.scroll_by(lines as i32, true);
                }
            }
            glfw::WindowEvent::CursorPos(x, y) => {
                let (width, height) = surface.window_size();
                self.gui_mouse = (
                    x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
                    y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
                );
            }
            glfw::WindowEvent::MouseButton(
                glfw::MouseButton::Button1,
                glfw::Action::Press,
                modifiers,
            ) => {
                let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
                // `ChatScreen.mouseClicked`: the list of matches first, then
                // the style of the text under the mouse.
                if !chat.mouse_clicked(mouse) {
                    let shift = modifiers.contains(glfw::Modifiers::Shift);
                    let gui = (size.gui_scaled_width(), size.gui_scaled_height());
                    let font = chat.font();
                    if let Some(style) = self.chat_log.clicked_style(&font, gui, mouse, shift) {
                        self.style_clicked(&style, shift, session, surface, assets)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// `ChatScreen.handleComponentClicked`: what a click on text does. With
    /// Shift it types the text's insertion; without, its click event.
    fn style_clicked(
        &mut self,
        style: &voxel_game::text::Style,
        insertions: bool,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use voxel_game::text::ClickEvent;
        if insertions {
            if let (Some(insertion), Some(chat)) = (&style.insertion, self.chat.as_mut()) {
                chat.insert_text(insertion, false);
            }
            return Ok(());
        }
        let Some(event) = style.click.as_deref() else {
            return Ok(());
        };
        match event {
            // `Commands.trimOptionalPrefix`, then the command as typed.
            ClickEvent::RunCommand(command) => {
                session.send_chat(command.strip_prefix('/').unwrap_or(command), true)?;
            }
            ClickEvent::SuggestCommand(command) => {
                if let Some(chat) = self.chat.as_mut() {
                    chat.insert_text(command, true);
                }
            }
            ClickEvent::CopyToClipboard(value) => surface.set_clipboard(value),
            ClickEvent::OpenUrl(url) => {
                // `clickUrlAction`, with the prompt on (the option is not
                // kept): an untrusted link asks first.
                let size = surface.scale();
                let gui = (size.gui_scaled_width(), size.gui_scaled_height());
                let screen =
                    ConfirmScreen::link(&assets.creative.font, &assets.lang, gui, url, false)
                        .with_background(voxel_render::gui::confirm::ConfirmBackground::Menu {
                            in_world: true,
                        });
                self.link_confirm = Some((screen, url.clone()));
            }
            ClickEvent::OpenFile(path) => open_uri(path),
            // Changing a page and the server's own actions have no screen
            // here to act on.
            ClickEvent::ChangePage(_) | ClickEvent::Custom(_) => {}
        }
        Ok(())
    }

    /// An event for the open link confirmation: Yes opens the link, Copy
    /// keeps it on the clipboard, No and Escape go back to the chat.
    fn link_event(&mut self, event: glfw::WindowEvent, surface: &mut WindowSurface) {
        use glfw::{Action, Key, WindowEvent};
        if let WindowEvent::CursorPos(x, y) = event {
            self.move_gui_mouse((x, y), surface);
            return;
        }
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let Some((screen, url)) = self.link_confirm.as_mut() else {
            return;
        };
        let action = match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => screen.escape(),
            WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                screen.tab(!modifiers.contains(glfw::Modifiers::Shift));
                None
            }
            WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
                screen.pressed()
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, _) => {
                screen.clicked(mouse)
            }
            _ => None,
        };
        let Some(action) = action else {
            return;
        };
        match action {
            LinkAction::Yes => open_uri(url),
            LinkAction::Copy => surface.set_clipboard(url),
            LinkAction::No => {}
        }
        self.link_confirm = None;
    }

    /// Let go of every key, as the mouse is freed.
    fn release_keys(&mut self) {
        self.keys = HeldKeys::default();
        self.tab_held = false;
        self.attack_key.release();
        self.use_key.release();
        self.pick_key.release();
    }

    /// `containerTick`: a container the server opens or closes opens or
    /// closes its screen, and each inventory screen hands over to the other
    /// when the game mode changes under it. Neither closes the container.
    pub fn container_tick(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let size = surface.scale();
        let (width, height) = (size.gui_scaled_width(), size.gui_scaled_height());
        // A container with no screen here is closed at once, so the server
        // does not wait on a window the player cannot see.
        if session.open_container_unsupported() {
            session.close_inventory()?;
        }
        match (self.server_container, session.open_container_info()) {
            (None, Some((id, kind, title))) => {
                self.server_container = Some(id);
                self.screen = Some(assets.container_screen(kind, &title, width, height));
                self.release_keys();
                surface.set_cursor_captured(false);
                let (w, h) = surface.window_size();
                surface.set_cursor_position(f64::from(w) / 2.0, f64::from(h) / 2.0);
                return Ok(());
            }
            (Some(open), Some((id, kind, title))) if open != id => {
                self.server_container = Some(id);
                self.screen = Some(assets.container_screen(kind, &title, width, height));
                return Ok(());
            }
            (Some(_), None) => {
                // The server closed it, or the player did.
                self.server_container = None;
                session.clear_recipe_ghost();
                if self.screen.take().is_some() {
                    surface.set_cursor_captured(true);
                }
                return Ok(());
            }
            (Some(_), Some(_)) => return Ok(()),
            (None, None) => {}
        }
        let Some(open) = self.screen.as_mut() else {
            return Ok(());
        };
        let creative = session.game_mode() == voxel_world::ClientPlayerGameMode::Creative;
        let open_tab = open.as_creative_mut().map(|open| open.tab());
        if open_tab.is_some() != creative {
            self.creative_tab = open_tab.unwrap_or(self.creative_tab);
            self.screen = Some(assets.screen(creative, width, height, self.creative_tab));
        }
        Ok(())
    }

    /// A script's step on this tick: the inventory key, a click in the
    /// open screen, the mouse moved or text typed into it, a creative tab
    /// chosen or scrolled, and the attack and use keys. A scripted `attack`
    /// is a click on the tick; `mine` holds the key, and the same goes for
    /// `use` and `hold_use`.
    pub fn script_step(
        &mut self,
        held: HeldKeys,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if held.inventory {
            if self.screen.is_some() {
                close_inventory(session, surface, &mut self.screen, &mut self.creative_tab)?;
                surface.set_cursor_captured(false);
            } else {
                open_inventory(
                    surface,
                    &mut self.screen,
                    session.game_mode() == voxel_world::ClientPlayerGameMode::Creative,
                    self.creative_tab,
                    assets,
                );
            }
        }
        if let Some((slot, button, kind)) = held.click {
            match self.screen.as_mut() {
                Some(screen) => {
                    let action = screen.scripted_click(slot, button, kind);
                    run_actions(session, &mut self.screen, vec![action])?;
                }
                None => session.container_click(slot, button, kind)?,
            }
        }
        if let Some(mouse) = held.mouse {
            self.gui_mouse = mouse;
        }
        if held.escape {
            self.scripted_escape(session, surface, assets)?;
        }
        if held.chat {
            if self.chat.is_some() {
                self.close_chat(surface, ExitReason::Intentional);
            } else {
                self.open_chat(ChatMethod::Message, false, surface, assets);
            }
        }
        if let (Some(text), Some(chat)) = (held.typed, self.chat.as_mut()) {
            for character in text.chars() {
                chat.char_typed(character);
            }
        }
        self.send_suggestion_request(session)?;
        if held.complete
            && let Some(chat) = self.chat.as_mut()
        {
            let mut clipboard = WindowClipboard(surface);
            chat.key_pressed(glfw::Key::Tab, (false, false), &mut clipboard);
        }
        self.send_suggestion_request(session)?;
        if held.enter
            && let Some(chat) = self.chat.as_mut()
        {
            let mut clipboard = WindowClipboard(surface);
            if let ChatKey::Send(text) =
                chat.key_pressed(glfw::Key::Enter, (false, false), &mut clipboard)
            {
                Self::send_chat(session, &text)?;
            }
            self.close_chat(surface, ExitReason::Done);
        }
        if held.press.is_some() {
            self.scripted_press(session, surface, assets)?;
        }
        if let (Some((button, shift)), Some(screen)) = (held.press, self.screen.as_mut()) {
            let menu = session.active_menu().clone();
            let now = std::time::Instant::now();
            let mut actions = screen.mouse_clicked(self.gui_mouse, button, shift, now, &menu);
            actions.extend(screen.mouse_released(self.gui_mouse, button, shift, &menu));
            run_actions(session, &mut self.screen, actions)?;
        }
        if let (Some(text), Some(screen)) = (held.typed, self.screen.as_mut()) {
            for character in text.chars() {
                screen.char_typed(character);
            }
        }
        self.drain_screen_actions(session)?;
        if let Some(screen) = self
            .screen
            .as_mut()
            .and_then(|screen| screen.as_creative_mut())
        {
            if let Some(tab) = held.tab {
                screen.select_tab_index(tab);
            }
            if let Some(scroll) = held.scroll {
                screen.menu_mut().set_scroll(scroll as f32 / 100.0);
            }
        }
        script_key(&mut self.attack_key, held.attack, held.mine);
        script_key(&mut self.use_key, held.use_click, held.hold_use);
        script_key(&mut self.pick_key, held.pick, false);
        Ok(())
    }

    /// One window event. An open screen takes the keys and the mouse.
    pub fn handle(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if let glfw::WindowEvent::Key(key, _, action, _) = event {
            let control_keys: [glfw::Key; 2] = if cfg!(target_os = "macos") {
                [glfw::Key::LeftSuper, glfw::Key::RightSuper]
            } else {
                [glfw::Key::LeftControl, glfw::Key::RightControl]
            };
            if control_keys.contains(&key) {
                self.control_held = action != glfw::Action::Release;
            }
            if matches!(key, glfw::Key::LeftControl | glfw::Key::RightControl) {
                self.ctrl_down = action != glfw::Action::Release;
            }
        }
        if let Some(screen) = self.screen.as_mut() {
            let size = surface.scale();
            screen.resize(size.gui_scaled_width(), size.gui_scaled_height());
        }
        let for_screen = matches!(
            event,
            glfw::WindowEvent::Key(..)
                | glfw::WindowEvent::Char(_)
                | glfw::WindowEvent::MouseButton(..)
                | glfw::WindowEvent::Scroll(..)
                | glfw::WindowEvent::CursorPos(..)
        );
        if self.link_confirm.is_some() && for_screen {
            self.link_event(event, surface);
            return Ok(());
        }
        if self.chat.is_some() && for_screen {
            return self.chat_event(event, session, surface, assets);
        }
        if self.disconnected.is_some() && for_screen {
            self.disconnected_event(event, surface);
            return Ok(());
        }
        if self.confirm.is_some() && for_screen {
            return self.confirm_event(event, session, surface);
        }
        if self.death.is_some() && for_screen {
            return self.death_event(event, session, surface, assets);
        }
        if self.language_screen.is_some() && for_screen {
            self.language_event(event, surface, assets);
            return Ok(());
        }
        if (self.mouse_screen.is_some() || self.controls_screen.is_some()) && for_screen {
            self.controls_event(event, surface, assets);
            return Ok(());
        }
        if self.sound_screen.is_some() && for_screen {
            self.sound_options_event(event, surface, assets);
            return Ok(());
        }
        if self.chat_options_screen.is_some() && for_screen {
            self.chat_options_event(event, surface, assets);
            return Ok(());
        }
        if self.video_screen.is_some() && for_screen {
            self.video_event(event, surface, assets);
            return Ok(());
        }
        if self.options_screen.is_some() && for_screen {
            self.options_event(event, surface, assets);
            return Ok(());
        }
        if self.pause.is_some() && for_screen {
            self.pause_event(event, surface, assets);
            return Ok(());
        }
        if self.screen.is_some() && for_screen {
            let handled = self.screen_event(event, session, surface);
            self.drain_screen_actions(session)?;
            handled
        } else {
            self.world_event(event, session, surface, assets)
        }
    }

    /// The clicks the open screen queued while it took a key or a character.
    fn drain_screen_actions(&mut self, session: &mut PlaySession) -> Result {
        let Some(screen) = self.screen.as_mut() else {
            return Ok(());
        };
        let actions = screen.drain_actions();
        if actions.is_empty() {
            return Ok(());
        }
        run_actions(session, &mut self.screen, actions)
    }

    /// An event for the open screen.
    fn screen_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
    ) -> Result {
        let Some(screen) = self.screen.as_mut() else {
            return Ok(());
        };
        match event {
            glfw::WindowEvent::Key(
                key,
                _,
                glfw::Action::Press | glfw::Action::Repeat,
                modifiers,
            ) => {
                let menu = session.active_menu().clone();
                let mods = KeyMods {
                    control: modifiers.contains(glfw::Modifiers::Control),
                    shift: modifiers.contains(glfw::Modifiers::Shift),
                    command: modifiers.contains(glfw::Modifiers::Super),
                };
                match screen.key_pressed(key, mods, &menu, &mut WindowClipboard(surface)) {
                    KeyResult::Close => {
                        close_inventory(
                            session,
                            surface,
                            &mut self.screen,
                            &mut self.creative_tab,
                        )?;
                        self.captured = true;
                        self.last_cursor = None;
                    }
                    KeyResult::Actions(actions) => {
                        run_actions(session, &mut self.screen, actions)?;
                    }
                }
            }
            glfw::WindowEvent::Key(..) => screen.key_released(),
            glfw::WindowEvent::Char(character) => screen.char_typed(character),
            glfw::WindowEvent::MouseButton(button, action, modifiers) => {
                let button = button as i32;
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                let menu = session.active_menu().clone();
                let actions = if action == glfw::Action::Press {
                    self.active_button = Some(button);
                    let now = std::time::Instant::now();
                    screen.mouse_clicked(self.gui_mouse, button, shift, now, &menu)
                } else {
                    self.active_button = None;
                    screen.mouse_released(self.gui_mouse, button, shift, &menu)
                };
                run_actions(session, &mut self.screen, actions)?;
            }
            glfw::WindowEvent::Scroll(_, y) => screen.mouse_scrolled(y),
            glfw::WindowEvent::CursorPos(x, y) => {
                // `MouseHandler.onMove`: screen coordinates to GUI units.
                let size = surface.scale();
                let (width, height) = surface.window_size();
                self.gui_mouse = (
                    x * f64::from(size.gui_scaled_width()) / f64::from(width.max(1)),
                    y * f64::from(size.gui_scaled_height()) / f64::from(height.max(1)),
                );
                if self.active_button.is_some() {
                    let menu = session.active_menu().clone();
                    screen.mouse_dragged(self.gui_mouse, &menu);
                }
                self.last_cursor = None;
            }
            _ => {}
        }
        Ok(())
    }

    /// An event with no screen in the way: moving, looking, the hotbar, the
    /// attack and use keys, and the keys that open things.
    fn world_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let spectating = session.player().is_some_and(|player| player.spectating());
        match event {
            WindowEvent::Key(Key::E, _, Action::Press, _) if self.captured && !spectating => {
                open_inventory(
                    surface,
                    &mut self.screen,
                    session.game_mode() == voxel_world::ClientPlayerGameMode::Creative,
                    self.creative_tab,
                    assets,
                );
                self.captured = false;
                self.release_keys();
            }
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.open_pause(surface, assets);
            }
            WindowEvent::Key(Key::Tab, _, action, _) => {
                self.tab_held = action != Action::Release && self.captured;
            }
            WindowEvent::Key(Key::T, _, Action::Press, _) if self.captured => {
                self.open_chat(ChatMethod::Message, true, surface, assets);
            }
            WindowEvent::Key(Key::Slash, _, Action::Press, _) if self.captured => {
                self.open_chat(ChatMethod::Command, true, surface, assets);
            }
            WindowEvent::Key(Key::F5, _, Action::Press, _) => {
                self.camera_type = self.camera_type.next();
            }
            WindowEvent::Key(Key::F11, _, Action::Press, _) => surface.toggle_fullscreen(),
            WindowEvent::Scroll(x, y) if self.captured => {
                if !spectating
                    && let Some(slot) = self.scroll_wheel.select(
                        x * self.options.mouse_wheel_sensitivity,
                        y * self.options.mouse_wheel_sensitivity,
                        session.status().selected_slot,
                    )
                {
                    session.select_slot(slot);
                }
            }
            WindowEvent::Key(key, _, Action::Press, _)
                if self.captured && hotbar_key(key).is_some() =>
            {
                if !spectating && let Some(slot) = hotbar_key(key) {
                    session.select_slot(slot);
                }
            }
            WindowEvent::Key(key, _, action, _) if self.captured => {
                self.keys.set(key, action != Action::Release);
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) if self.captured => {
                self.attack_key.press();
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                self.attack_key.release();
            }
            WindowEvent::MouseButton(MouseButton::Button2, Action::Press, _) if self.captured => {
                self.use_key.press();
            }
            WindowEvent::MouseButton(MouseButton::Button2, Action::Release, _) => {
                self.use_key.release();
            }
            WindowEvent::MouseButton(MouseButton::Button3, Action::Press, _) if self.captured => {
                self.pick_key.press();
            }
            WindowEvent::MouseButton(MouseButton::Button3, Action::Release, _) => {
                self.pick_key.release();
            }
            WindowEvent::MouseButton(_, Action::Press, _) if !self.captured => {
                self.captured = true;
                self.last_cursor = None;
                surface.set_cursor_captured(true);
            }
            WindowEvent::CursorPos(x, y) => {
                if self.captured
                    && let Some((last_x, last_y)) = self.last_cursor
                {
                    let (turn_x, turn_y) = turn_by(&self.options, (x - last_x, y - last_y));
                    session.turn_player(turn_x, turn_y);
                }
                self.last_cursor = Some((x, y));
            }
            _ => {}
        }
        Ok(())
    }
}

/// `CommandHistory.MAX_PERSISTED_COMMAND_HISTORY`.
const COMMAND_HISTORY_LIMIT: usize = 50;

/// Where the game keeps its files (`gameDirectory`).
fn game_directory() -> std::path::PathBuf {
    std::env::var_os("VOXEL_GAME_DIR")
        .map_or_else(|| std::path::PathBuf::from(".data/client"), Into::into)
}

/// `Options.optionsFile`.
fn options_path() -> std::path::PathBuf {
    game_directory().join("options.txt")
}

/// `Options.load`: `options.txt` of the game directory, written when it
/// was not there.
fn load_options() -> Options {
    let path = options_path();
    let options = Options::load(&path);
    if !path.exists() {
        let _ = options.save(&path);
    }
    options
}

/// `CommandHistory`'s constructor: the lines of `command_history.txt`.
fn read_command_history() -> Vec<String> {
    std::fs::read_to_string(game_directory().join("command_history.txt"))
        .map(|text| text.lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// `CommandHistory.save`.
fn write_command_history(commands: &[String]) {
    let directory = game_directory();
    let _ = std::fs::create_dir_all(&directory);
    let mut text = commands.join("\n");
    text.push('\n');
    let _ = std::fs::write(directory.join("command_history.txt"), text);
}

/// A scripted key: a click on this tick, and whether it is held.
fn script_key(key: &mut AttackKey, click: bool, hold: bool) {
    if click {
        key.press();
    }
    if !hold {
        key.release();
    } else if !key.is_down() {
        key.press();
        key.take_clicks();
    }
}

/// The number key for a hotbar slot, 1 to 9.
fn hotbar_key(key: glfw::Key) -> Option<i32> {
    use glfw::Key;
    [
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
    ]
    .iter()
    .position(|candidate| *candidate == key)
    .map(|slot| slot as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_camera_turns_by_the_sensitivity_cubed_and_the_inversions() {
        let mut options = Options::default();
        // At the least, (0.2)^3 * 8; at the most, (0.8)^3 * 8, in single
        // precision where the reference widens a float.
        options.sensitivity = 0.0;
        let low = f64::from(0.2_f32).powi(3) * 8.0;
        assert_eq!(turn_by(&options, (10.0, -5.0)), (10.0 * low, -5.0 * low));
        options.sensitivity = 1.0;
        let high = (f64::from(0.6_f32) + f64::from(0.2_f32)).powi(3) * 8.0;
        assert_eq!(turn_by(&options, (1.0, 1.0)), (high, high));
        options.invert_mouse_x = true;
        assert_eq!(turn_by(&options, (1.0, 1.0)), (-high, high));
        options.invert_mouse_y = true;
        assert_eq!(turn_by(&options, (1.0, 1.0)), (-high, -high));
    }
}
