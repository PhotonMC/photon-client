//! The player's hands on the client: the keys and the mouse, a script's
//! steps, and the inventory screen they may be typing into.

use std::time::{Duration, Instant};

use photon_game::input::attack::AttackKey;

use photon_game::input::hotbar::ScrollWheel;

use photon_game::options::Options;

use photon_net::connection::PlaySession;

use photon_gui::chat::ChatSettings;

use photon_gui::chat::{ChatDrafts, ChatKey, ChatLog, ChatMethod, ChatScreen, ExitReason};

use photon_gui::options::accessibility::AccessibilityScreen;
use photon_gui::options::chat::ChatOptionsScreen;

use photon_gui::options::keybinds::{BindKey, KeyBindsScreen};

use photon_gui::options::language::LanguageSelectScreen;

use photon_gui::options::mouse::{ControlsScreen, MouseSettingsScreen};

use photon_gui::options::sound::SoundOptionsScreen;

use photon_gui::options::video::{VideoSettingsScreen, VideoValues};

use photon_gui::options::{OptionsAction, OptionsInput, OptionsScreen};

use photon_gui::screen::confirm::ConfirmAction as LinkAction;

use photon_gui::screen::confirm::{ConfirmAction, ConfirmBackground, ConfirmScreen};

use photon_gui::screen::credits::CreditsScreen;
use photon_gui::screen::death::{DeathAction, DeathScreen};
use photon_gui::screen::debug_options::DebugOptionsScreen;
use photon_gui::screen::game_mode::GameModeSwitcherScreen;
use photon_gui::screen::social::SocialInteractionsScreen;

use photon_gui::screen::disconnected::DisconnectedScreen;

use photon_gui::screen::pause::{PauseAction, PauseScreen};

use photon_gui::screen::{KeyMods, KeyResult};

use photon_gui::widgets::slider::SliderKey;

use photon_render::window::WindowSurface;

use crate::screens::{
    OpenScreen, ScreenAssets, WindowClipboard, close_inventory, creative_tags, open_inventory,
    run_actions,
};

use crate::script::HeldKeys;

use crate::view::CameraType;

/// `Minecraft.computeChatAbilities`. The launcher and the account allow chat: Photon has neither
/// to ask (the sign-in is #31).
pub(crate) fn chat_abilities(options: &Options) -> photon_game::chat_abilities::ChatAbilities {
    photon_game::chat_abilities::ChatAbilities::compute(options.chat_visibility, (true, true))
}

pub(super) type Result = std::result::Result<(), Box<dyn std::error::Error>>;

/// What the player is holding and pointing at between events.
#[derive(Default)]
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
    pub(super) scroll_wheel: ScrollWheel,
    /// F5 cycles first person, third person from behind and from the front.
    pub camera_type: CameraType,
    /// Whether the mouse is grabbed to look around.
    pub captured: bool,
    pub(super) last_cursor: Option<(f64, f64)>,
    /// The mouse's movement waiting for the frame, and the cinematic camera.
    pub turning: turn::Turning,
    /// The open inventory screen, if any.
    pub screen: OpenScreen,
    /// The id of the container the server opened, while its screen is up.
    pub(super) server_container: Option<i32>,
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
    /// `AccessibilityOptionsScreen`, over the options screen.
    pub accessibility_screen: Option<AccessibilityScreen>,
    /// The skin customization, opened the same way.
    pub skin_screen: Option<photon_gui::options::skin::SkinCustomizationScreen>,
    /// The font settings, opened over the language screen.
    pub font_screen: Option<photon_gui::options::font::FontOptionsScreen>,
    pub sound_screen: Option<SoundOptionsScreen>,
    /// The controls screen, and the mouse settings over it.
    pub controls_screen: Option<ControlsScreen>,
    /// The language screen, and the language it chose, for the game to load.
    pub language_screen: Option<LanguageSelectScreen>,
    pub(super) language_request: Option<String>,
    /// When and where the primary button last went down.
    pub(super) last_press: Option<(Instant, (i32, i32))>,
    pub mouse_screen: Option<MouseSettingsScreen>,
    /// The key binds, over the controls screen.
    pub key_binds_screen: Option<KeyBindsScreen>,
    /// The death screen, from the server's word of a death until the player
    /// has health again.
    pub death: Option<DeathScreen>,
    /// The credits, from the server's `WIN_GAME` until they end or are closed.
    pub credits: Option<CreditsScreen>,
    /// The social interactions screen, while it is open.
    pub social: Option<SocialInteractionsScreen>,
    /// The debug options screen (F3+F6), while it is open.
    pub debug_options: Option<DebugOptionsScreen>,
    /// The statistics screen over the pause menu, while it is open.
    pub stats: Option<photon_gui::screen::stats::StatsScreen>,
    /// The advancements screen, over the game or the pause menu, while it is open.
    pub advancements: Option<advancements::OpenAdvancements>,
    /// `PlayerSocialManager.hiddenPlayers`: whose chat the player hid, for as long as the client runs.
    pub hidden_players: Vec<photon_core::binary::Uuid>,
    /// The game mode switcher (F3+F4), while F3 is held.
    pub game_mode_switcher: Option<GameModeSwitcherScreen>,
    /// The modifier keys held.
    pub mods: HeldModifiers,
    /// F3, the debug modifier, and the debug overlay's entries.
    pub debug: DebugKeys,
    /// The sign the player is writing on, while its screen is up.
    pub sign_edit: Option<SignEdit>,
    /// The command block the player is editing, while its screen is up.
    pub command_block: Option<CommandBlockEdit>,
    /// The structure block the player is editing, while its screen is up.
    pub structure_block: Option<StructureBlockEdit>,
    /// The jigsaw block the player is editing, while its screen is up.
    pub jigsaw_block: Option<JigsawEdit>,
    /// The test block the player is editing, while its screen is up.
    pub test_block: Option<TestBlockEdit>,
    pub test_instance_block: Option<TestInstanceBlockEdit>,
    /// The book and quill being written in, while its screen is up.
    pub book_edit: Option<BookEdit>,
    /// "Title Screen" asks first (`deathScreen.quit.confirm`), over the death
    /// screen, which this takes the place of while it is open.
    pub confirm: Option<ConfirmScreen>,
    /// The chat restrictions screen, over the chat.
    pub restrictions: Option<photon_gui::screen::restrictions::RestrictionsScreen>,
    /// `ConfirmLinkScreen` or a command confirmation, over the chat, and what it asks about.
    pub link_confirm: Option<(ConfirmScreen, ConfirmTarget)>,
    /// The screen a server dialog puts over the game, and the question about it.
    pub dialog: Option<DialogLayer>,
    pub dialog_warning: Option<ConfirmScreen>,
    /// The dialog the pause menu has a button for (`PauseScreen.getCustomAdditions`), as the
    /// session last had it.
    pub pause_additions: Option<photon_game::dialog::Dialog>,
    /// The server resource packs the player is being asked about; the question is `confirm`.
    pub pack_prompt: Option<pack_prompt::PackPrompt>,
    /// The screen of a reconfiguration or a transfer that goes on.
    pub(crate) switch: Option<crate::play::SwitchView>,
    /// The panorama behind the reconfiguring, transfer and disconnected screens (no level).
    pub(crate) backdrop: Option<crate::play::LoadingView>,
    /// The screen a lost connection leaves, which nothing replaces.
    pub disconnected: Option<DisconnectedScreen>,
    /// The player asked to leave the server (the pause menu's Disconnect, or the button of the
    /// disconnected screen).
    pub leave: bool,
    /// The options were saved since the session last looked (`Options.save` then
    /// `broadcastOptions`).
    pub options_saved: bool,
    /// The title screen's host of the options screens: they are made as outside a level.
    pub(crate) title_host: bool,
    /// `Minecraft.lastActiveTime`: the last frame the window was focused (or the option off).
    pub(super) last_active: Option<Instant>,
    /// F3+A asked for every chunk to be built again; the frame takes it.
    pub reload_chunks: bool,
    /// The values of `options.txt` the client reads.
    pub options: Options,
    pub(super) chat_history: Vec<String>,
    /// `ChatComponent.latestDraft`.
    pub(super) chat_drafts: ChatDrafts,
    /// `CommandHistory`: the commands sent, kept in a file.
    pub(super) command_history: Vec<String>,
    pub chat_log: ChatLog,
    /// Whether the tab list key is held.
    pub tab_held: bool,
    /// `Hud.isHidden`: the interface hidden with F1.
    pub hud_hidden: bool,
    /// F2 was pressed: the next frame is written to a file.
    pub screenshot_pending: bool,
    /// The mouse in GUI units.
    pub gui_mouse: (f64, f64),
    /// The button a drag is held with (`MouseHandler.activeButton`).
    pub(super) active_button: Option<i32>,
}

/// `Util.getPlatform().openUri`: the system's own opener.
pub(super) fn open_uri(uri: &str) {
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

/// A key a settings screen gives to its widgets: Tab and Shift+Tab, or an
/// arrow, Enter or Space with whether Shift is held.
pub(super) enum MenuKey {
    Tab(bool),
    Key(SliderKey, bool),
}

pub(super) fn menu_key(event: &glfw::WindowEvent) -> Option<MenuKey> {
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

/// `CommandHistory.MAX_PERSISTED_COMMAND_HISTORY`.
pub(super) const COMMAND_HISTORY_LIMIT: usize = 50;

/// Where the game keeps its files (`gameDirectory`).
pub(crate) fn game_directory() -> std::path::PathBuf {
    std::env::var_os("PHOTON_GAME_DIR")
        .map_or_else(|| std::path::PathBuf::from(".data/client"), Into::into)
}

/// `Options.buildPlayerInformation`: what the server is told of the client. Nothing is filtered, as
/// with the default options; the rest is the options'.
pub(crate) fn client_information(
    options: &Options,
) -> photon_net::configuration::ClientInformation {
    photon_net::configuration::ClientInformation::new(
        options.language_code.encode_utf16().collect(),
        options.render_distance,
        options.chat_visibility,
        options.chat_colors,
        options.model_parts,
        match options.main_hand {
            photon_game::options::MainHand::Left => photon_net::configuration::MainHand::Left,
            photon_game::options::MainHand::Right => photon_net::configuration::MainHand::Right,
        },
        false,
        options.allow_server_listing,
        match options.particles {
            photon_game::options::ParticleStatus::All => {
                photon_net::configuration::ParticleStatus::All
            }
            photon_game::options::ParticleStatus::Decreased => {
                photon_net::configuration::ParticleStatus::Decreased
            }
            photon_game::options::ParticleStatus::Minimal => {
                photon_net::configuration::ParticleStatus::Minimal
            }
        },
    )
}

/// `Options.optionsFile`.
pub(crate) fn options_path() -> std::path::PathBuf {
    game_directory().join("options.txt")
}

/// `Options.load`: `options.txt` of the game directory, written when it
/// was not there.
pub(crate) fn load_options() -> Options {
    let path = options_path();
    let options = Options::load(&path);
    if !path.exists() {
        let _ = options.save(&path);
    }
    options
}

/// `CommandHistory`'s constructor: the lines of `command_history.txt`.
pub(super) fn read_command_history() -> Vec<String> {
    std::fs::read_to_string(game_directory().join("command_history.txt"))
        .map(|text| text.lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// `CommandHistory.save`.
pub(super) fn write_command_history(commands: &[String]) {
    let directory = game_directory();
    let _ = std::fs::create_dir_all(&directory);
    let mut text = commands.join("\n");
    text.push('\n');
    let _ = std::fs::write(directory.join("command_history.txt"), text);
}

/// A scripted key: a click on this tick, and whether it is held.
pub(super) fn script_key(key: &mut AttackKey, click: bool, hold: bool) {
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

mod accessibility;

mod chat;

mod confirm;

mod credits;

mod death;
mod debug_actions;
mod debug_inspect;
mod debug_options;
mod debug_profile;
mod dialogs;
mod game_mode_switcher;

mod events;

pub(crate) mod keybinds;

mod options;
mod pack_prompt;

mod pause;
mod restrictions;
mod unattended;
pub use unattended::ConfirmTarget;

mod scripted;

mod settings;

mod command_block;

mod structure_block;

mod jigsaw;

mod test_block;
mod test_instance_block;

mod sign;

mod font;
mod host;
mod skin;

mod advancements;
#[cfg(test)]
mod debug_tests;
mod screen_keys;
mod social;
mod stats;
mod turn;
pub use command_block::CommandBlockEdit;
pub use dialogs::DialogLayer;
pub use jigsaw::JigsawEdit;
pub use sign::SignEdit;
pub use structure_block::StructureBlockEdit;
pub use test_block::TestBlockEdit;
pub use test_instance_block::TestInstanceBlockEdit;

mod book;
pub use book::BookEdit;

impl Controls {
    /// `setScreen(null)` for the screens a nether portal does not allow (`Screen.isAllowedInPortal`
    /// is `isPauseScreen` unless overridden): the social interactions screen and the game mode
    /// switcher. The pause menu and the settings screens are pause screens and stay open.
    /// Whether any was open.
    pub fn close_menus(&mut self) -> bool {
        let closed = [
            self.social.take().is_some(),
            self.game_mode_switcher.take().is_some(),
        ];
        closed.contains(&true)
    }

    pub fn new(camera_type: CameraType, captured: bool) -> Self {
        let command_history = read_command_history();
        let options = load_options();
        Self {
            camera_type,
            captured,
            creative_tab: photon_game::menus::creative::default_tab(),
            chat_log: ChatLog::with_settings(ChatSettings::from(&options))
                .with_colors(options.chat_colors)
                .with_abilities(chat_abilities(&options)),
            options,
            debug: DebugKeys {
                entries: debug_profile::load_debug_entries(),
                ..DebugKeys::default()
            },
            chat_history: command_history.clone(),
            command_history,
            ..Self::default()
        }
    }
}

/// F3 as the debug modifier and as the overlay's key, and what the overlay shows.
#[derive(Default)]
pub struct DebugKeys {
    /// Whether F3 is held.
    pub(super) down: bool,
    /// `KeyboardHandler.usedDebugKeyAsModifier`: a combination with F3 was used, so letting go of
    /// it does not toggle the overlay.
    pub(super) used: bool,
    pub entries: photon_gui::hud::debug::EntryList,
    /// What the frame loop last said of itself: the frames a second, the limit and the present mode.
    pub frame: photon_gui::hud::debug::FrameStats,
    /// The machine the system entry names, taken from the backend.
    pub system: Option<photon_gui::hud::debug::SystemSpecs>,
    pub memory: crate::system_info::MemorySampler,
    pub gpu: crate::gpu_meter::GpuMeter,
    /// The sound channels in use and the player's mood, as the frame loop last read them.
    pub sound_mood: Option<photon_gui::hud::debug::SoundMood>,
}

impl DebugKeys {
    /// A key pressed with F3 held: the ones `KeyboardHandler.handleDebugKeys` answers (and Escape,
    /// which pauses without the menu) mark F3 as used as a modifier, ported or not. The hitbox,
    /// chunk border and location keys do nothing with reduced debug info.
    pub(super) fn combination(&mut self, key: glfw::Key, reduced: bool) {
        self.used |= self.down && Self::acts(key, reduced);
    }

    /// Whether `handleDebugKeys` answers a key pressed with F3 held, ported or not. Such a key is
    /// not a game key then: it does not open the chat, move or select a slot.
    pub(super) fn acts(key: glfw::Key, reduced: bool) -> bool {
        use glfw::Key;
        match key {
            Key::B | Key::G => !reduced,
            Key::A
            | Key::C
            | Key::D
            | Key::H
            | Key::I
            | Key::N
            | Key::P
            | Key::S
            | Key::T
            | Key::L
            | Key::V
            | Key::F4
            | Key::F6
            | Key::Num1
            | Key::Num2
            | Key::Num3
            | Key::Num4
            | Key::Escape => true,
            _ => false,
        }
    }

    /// F3 pressed or let go of: the overlay toggles on the release unless a combination used F3.
    pub(super) fn key(&mut self, action: glfw::Action) {
        self.down = action != glfw::Action::Release;
        if action == glfw::Action::Release {
            if self.used {
                self.used = false;
            } else {
                self.entries.toggle_overlay();
            }
        }
    }
}

/// The modifier keys that are down.
#[derive(Default)]
pub struct HeldModifiers {
    /// `Minecraft.hasControlDown`: a control key is down (the command key on macOS).
    pub control: bool,
    /// Control itself, which sprinting asks for.
    pub ctrl: bool,
    /// A shift key, which wheel events carry no mark of.
    pub shift: bool,
}
