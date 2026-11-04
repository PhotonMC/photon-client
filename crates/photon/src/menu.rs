//! The menus before and between games: the title screen and the server list, from which the
//! client joins a server and to which it comes back when the player leaves it.

use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use glfw::{Action, Key, Modifiers, MouseButton, WindowEvent};
use photon_game::server_status::Pinged;
use photon_game::servers::{PackStatus, ServerData, ServerList};
use photon_game::text::Component;
use photon_gui::screen::confirm::{ConfirmAction, ConfirmScreen};
use photon_gui::screen::credits::CreditsScreen;
use photon_gui::screen::home::{HomeAction, HomeScreen};
use photon_gui::screen::loading::LoadingScreen;
use photon_gui::screen::notice::NoticeScreen;
use photon_gui::screen::server_form::{FormKind, FormResult, ServerFormScreen};
use photon_gui::screen::servers::{JoinMultiplayerScreen, ServersAction};
use photon_net::connection::PlaySession;

use self::draw::draw_screen;
use self::input::confirm_event;
use crate::gui::Gui;
use crate::play::{self, Client, Exit, JoinTask, Target};

mod audio;
mod credits;
mod directory;
mod draw;
mod friends;
mod icons;
mod input;
pub(crate) mod panorama;
mod pings;
pub(crate) use pings::resolve_named;
mod servers;
mod settings;
mod signin;
mod startup;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The reference holds a menu with no level to sixty frames a second.
const MENU_FRAME: Duration = Duration::from_micros(16_667);
/// `MouseHandler`: a second press this soon after the first is a double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(250);

/// What the menus chose.
enum Choice {
    Quit,
    /// A joined server, its level loaded.
    Play(Box<PlaySession>),
}

/// The menu open.
enum Screen {
    Home(HomeScreen),
    Servers(JoinMultiplayerScreen),
    /// A form of the server list, with the place of the server it edits.
    Form(ServerFormScreen, Option<usize>),
    /// Whether to delete the saved server at this place.
    Confirm(ConfirmScreen, usize),
    /// A join going on, and the question about the server's resource packs while one is up.
    Connecting(LoadingScreen, JoinTask, Option<ConfirmScreen>),
    /// A join that failed, and whether the way back is the home screen, not the server list.
    Failed(NoticeScreen, bool),
    /// The question of a website sign-in link, with the browser's state.
    Ask(NoticeScreen, String),
    /// The credits of the title screen (`WinScreen` without the poem), and when it last drew.
    Credits(CreditsScreen, Instant),
    /// The options, language or accessibility screens of the title, which the host draws.
    Settings,
}

/// A window size argument, or `default` when it is absent.
fn size_arg(value: Option<String>, default: i32) -> Result<i32> {
    Ok(value
        .map(|text| text.parse())
        .transpose()?
        .unwrap_or(default))
}

/// `photon` or `photon menu [server [width height]]`: the title screen, the server list from its
/// Multiplayer button, and the game a server's Join starts; leaving it comes back here. A
/// scripted run joins `server` at once.
pub fn run_menu(mut args: impl Iterator<Item = String>) -> Result<()> {
    let server: SocketAddr = args
        .next()
        .unwrap_or_else(|| play::DEFAULT_SERVER.to_owned())
        .parse()?;
    let width = size_arg(args.next(), 1280)?;
    let height = size_arg(args.next(), 720)?;
    let mut client = Client::open((width, height))?;
    // The first start shows the loading overlay over the load; a return from a game does not.
    let mut splash = startup::begin(&mut client);
    let mut scripted_join = env::var_os("PHOTON_SCRIPT").is_some();
    // Dev only: the addresses a scripted run joins after each `leave`, in turn.
    let mut hops: std::collections::VecDeque<SocketAddr> = env::var("PHOTON_HOPS")
        .ok()
        .into_iter()
        .flat_map(|list| {
            list.split(',')
                .filter_map(|part| part.parse().ok())
                .collect::<Vec<_>>()
        })
        .collect();
    let mut server = server;
    // Leaving a server comes back to the home screen, which also takes a link handed over meanwhile.
    let mut left_server = false;
    loop {
        let mut menu = Menu::open(&mut client)?;
        menu.loading = splash.take().map(startup::Splash::into_overlay);
        menu.open_requested_screen(&client);
        if std::mem::take(&mut left_server) {
            menu.screen = menu.title(gui_size(&client));
        }
        if std::mem::take(&mut scripted_join) {
            menu.connect_scripted(server, gui_size(&client));
        }
        let choice = menu.run(&mut client)?;
        client.audio = Some(menu.into_audio());
        match choice {
            Choice::Quit => return Ok(()),
            Choice::Play(session) => {
                let (back, exit) = play_game(client, *session)?;
                client = back;
                if let Exit::Quit = exit {
                    return Ok(());
                }
                left_server = env::var_os("PHOTON_SCRIPT").is_none();
                if let Some(next) = hops.pop_front() {
                    server = next;
                    scripted_join = true;
                }
            }
        }
    }
}

/// Play a joined `session`, then drop the packs it pushed.
fn play_game(mut client: Client, session: PlaySession) -> Result<(Client, Exit)> {
    client.backend.set_gui_only(false);
    client.backend.set_panorama(None);
    let (client, exit) = play::play_in(client, session)?;
    crate::packs::shared().cleanup_after_disconnect();
    Ok((client, exit))
}

/// The menus' state between frames.
struct Menu {
    gui: Gui,
    game_directory: PathBuf,
    servers: ServerList,
    screen: Screen,
    mouse: (i32, i32),
    /// When and where the last primary press was, for a double click.
    last_click: Option<(Instant, (i32, i32))>,
    /// The answers of the open list's pings, by the address asked.
    pings: Option<std::sync::mpsc::Receiver<(String, Pinged)>>,
    /// The icon bytes each entry of the open list shows, by index.
    icons_shown: Vec<Option<Vec<u8>>>,
    /// Whether the entries' icons are to be checked again.
    icons_dirty: bool,
    /// The sound engine of the menus, which plays the buttons' clicks.
    sound: crate::sound::SoundSlot,
    /// The menu music (`Musics.MENU`).
    music: crate::music::MenuMusic,
    /// The options as the menu opened, and as the options screens last saved them.
    options: photon_game::options::Options,
    /// The options screens the title opens: the ones of a level, made as outside one.
    host: crate::controls::Controls,
    /// The account the title is looking up, until it is found, and whether friends work now.
    friends_title: friends::TitleFriends,
    /// The servers Photon's backend lists, when one is named.
    directory: directory::Directory,
    /// The launch link the game was started with, while its code is being spent.
    link: Option<crate::launch_link::LinkTask>,
    /// The website sign-in link the game was started with, as far as it has got.
    signin: Option<signin::SignInFlow>,
    /// Whether the join going on began on the home screen, which a failure returns to.
    joined_from_home: bool,
    /// What those screens are made with.
    assets: crate::screens::ScreenAssets,
    /// `Panorama.spin`, in degrees, and when it last advanced.
    spin: f32,
    last_spin: Instant,
    /// The loading overlay of the first start, and the clock it fades by, until it is gone.
    loading: Option<(photon_gui::loading_overlay::LoadingOverlay, Instant)>,
}

impl Menu {
    /// The title screen, with the language and the GUI scale of the options.
    fn open(client: &mut Client) -> Result<Self> {
        let root = photon_core::content_root::assets()
            .map_err(|_| "PHOTON_ASSETS must name the asset folder")?;
        let root = Path::new(&root);
        let data_root = env::var("PHOTON_DATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| root.with_file_name("data"));
        let options = crate::controls::load_options();
        // A game just played leaves its sprites on the GPU and its assets: the menus take them
        // over unless the packs in use changed since.
        let shared = crate::resources::shared(root);
        let gui = match client.gui.take().filter(|gui| gui.reads_from(&shared)) {
            Some(gui) => gui.recycled(menu_versions(), &options.language_code),
            None => Gui::load(
                root,
                menu_registries(&data_root),
                &mut client.backend,
                &options.language_code,
            )?,
        };
        client.surface.set_gui_options(options.gui_scaling());
        client.surface.set_cursor_captured(false);
        client.backend.set_gui_only(true);
        if !client.backend.has_panorama() {
            client
                .backend
                .load_panorama(&photon_render::panorama::layers(&panorama::faces(
                    &root.to_string_lossy(),
                )?))?;
        }
        let (link, signin) = signin::pending_link();
        let game_directory = crate::controls::game_directory();
        let screen = home_screen(gui.text_assets().0, gui.lang(), gui_size(client));
        let (sound, music) = audio::take(client, root, &options);
        let assets = gui.screen_assets();
        Ok(Self {
            host: crate::controls::Controls::for_title(),
            assets,
            gui,
            sound,
            music,
            servers: ServerList::load(&game_directory),
            game_directory,
            screen,
            mouse: (0, 0),
            last_click: None,
            pings: None,
            icons_shown: Vec::new(),
            icons_dirty: false,
            spin: 0.0,
            last_spin: Instant::now(),
            loading: None,
            friends_title: friends::TitleFriends::look_up(),
            directory: directory::Directory::look_up(),
            link,
            signin,
            joined_from_home: false,
            options,
        })
    }

    fn title(&self, size: (i32, i32)) -> Screen {
        let mut screen = HomeScreen::new(
            size,
            &version_line(),
            self.gui.text_assets().0,
            self.gui.lang(),
        );
        if self.friends_title.ready {
            screen.friends_available();
        }
        self.directory.apply(&mut screen, self.gui.lang());
        Screen::Home(screen)
    }

    /// `ConnectScreen.startConnecting`: the join starts on its own thread.
    fn connect(&mut self, target: Target, ip: Option<&str>, about: (&str, &str), size: (i32, i32)) {
        let packs = crate::packs::shared();
        crate::packs::prepare(&packs, ip);
        self.screen = Screen::Connecting(
            LoadingScreen::new(self.gui.lang(), size, about.0, about.1),
            JoinTask::start(target, packs),
            None,
        );
    }

    /// A scripted run's join of `server`, named by its address.
    fn connect_scripted(&mut self, server: SocketAddr, size: (i32, i32)) {
        let shown = server.to_string();
        let name = self.gui.lang().text("photon.loading.server");
        self.connect(Target::Address(server), None, (&name, &shown), size);
    }

    /// Whether this frame counts towards a capture: a join's screens do not, unless the capture is
    /// for them (dev only).
    fn counts_for_capture(&self, joining_too: bool) -> bool {
        let joining = matches!(self.screen, Screen::Connecting(..));
        self.loading.is_none() && (!joining || joining_too)
    }

    /// The connect screen's status, and the session once the join has finished (or the screen
    /// for why it failed).
    fn poll_join(&mut self, size: (i32, i32)) -> Option<Box<PlaySession>> {
        let Screen::Connecting(screen, task, _) = &mut self.screen else {
            return None;
        };
        screen.set_status(self.gui.lang(), task.status.message_key());
        match task.finished()? {
            Ok(session) => Some(Box::new(session)),
            Err(reason) => {
                crate::packs::shared().cleanup_after_disconnect();
                self.join_failed(&reason, size, self.joined_from_home);
                None
            }
        }
    }

    /// `JoinMultiplayerScreen`, the saved servers read again.
    fn server_list(&mut self, size: (i32, i32)) -> Screen {
        self.servers = ServerList::load(&self.game_directory);
        self.pings = Some(pings::ping_all(&self.servers));
        self.gui.atlas.clear_favicons();
        self.icons_shown.clear();
        self.icons_dirty = true;
        let names = self
            .servers
            .servers
            .iter()
            .map(|server| server.name.clone())
            .collect();
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        Screen::Servers(JoinMultiplayerScreen::new(font, lang, size, names))
    }

    /// Frames until a menu chooses or the window closes. `PHOTON_CAPTURE_PPM`: the tenth frame (or
    /// `PHOTON_CAPTURE_AFTER`'s) is saved there and the client stops.
    /// The title and the list follow the window's size; the forms, the question and the join keep
    /// the size they were opened in.
    fn fit_screen(&mut self, size: (i32, i32)) {
        let current = match &self.screen {
            Screen::Home(screen) => Some(screen.size()),
            Screen::Servers(screen) => Some(screen.size()),
            Screen::Credits(screen, _) => Some(screen.size()),
            _ => None,
        };
        if self.refit_credits(size) {
            return;
        }
        if current.is_some_and(|current| current != size) {
            self.screen = match self.screen {
                Screen::Servers(_) => self.server_list(size),
                _ => self.title(size),
            };
        }
    }

    /// A join that failed: the notice screen, back to the home screen for a join that did not
    /// start in the server list.
    fn join_failed(&mut self, reason: &Component, size: (i32, i32), home: bool) {
        let heading = Component::translatable("photon.notice.join_failed");
        self.notice_screen(&heading, reason, size, home);
    }

    /// The notice screen with a way back to the home screen or the server list.
    fn notice_screen(
        &mut self,
        heading: &Component,
        reason: &Component,
        size: (i32, i32),
        home: bool,
    ) {
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        let back = if home {
            "photon.notice.back_home"
        } else {
            "photon.notice.back_servers"
        };
        let back = Component::translatable(back);
        let screen = NoticeScreen::new(font, lang, size, (heading, reason, &back));
        self.screen = Screen::Failed(screen, home);
    }

    fn run(&mut self, client: &mut Client) -> Result<Choice> {
        let capture = env::var("PHOTON_CAPTURE_PPM").ok();
        let capture_at = env::var("PHOTON_CAPTURE_AFTER")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(10);
        let capture_joining = env::var_os("PHOTON_CAPTURE_JOINING").is_some();
        let mut frames = 0;
        while !client.surface.should_close() && !crate::updates::restart_requested() {
            let frame_start = Instant::now();
            crate::updates::poll();
            crate::updates::log_newer_servers();
            for event in client.surface.poll() {
                if let Some(choice) = self.handle(event, client)? {
                    return Ok(choice);
                }
            }
            play::fit_swapchain(client)?;
            let size = gui_size(client);
            self.receive_pings();
            self.frame_audio();
            self.sync_icons(client)?;
            self.host.pack_poll(&self.assets);
            self.sync_pack_icons(client)?;
            self.ask_about_packs(client, size);
            if let Some(session) = self.poll_join(size) {
                return Ok(Choice::Play(session));
            }
            self.credits_ticked(size);
            self.friends_account_polled();
            self.directory_polled();
            self.link_arrived(client);
            self.link_polled(size);
            self.signin_polled(size);
            self.friends_ticked(client);
            self.fit_screen(size);
            self.spin_panorama(client);
            let clear = self.draw_frame(client, size)?;
            self.show_cursor(client);
            self.gui.sync_glyphs(&mut client.backend)?;
            if self.counts_for_capture(capture_joining) {
                frames += 1;
            }
            if let Some(path) = capture.as_deref().filter(|_| frames == capture_at) {
                let captured = client
                    .backend
                    .capture_scene([0.0, 0.0, 0.0, 1.0], IDENTITY)?;
                crate::ppm::write(path, captured.width, captured.height, &captured.rgba)?;
                println!("captured_frame={path}");
                return Ok(Choice::Quit);
            }
            client.backend.draw_scene(clear, IDENTITY)?;
            if let Some(rest) = MENU_FRAME.checked_sub(frame_start.elapsed()) {
                std::thread::sleep(rest);
            }
        }
        Ok(Choice::Quit)
    }

    /// The hand over a hovered button, when the options allow cursor changes.
    fn show_cursor(&self, client: &mut Client) {
        let hand = self.options.play.allow_cursor_changes && client.backend.gui_hand();
        client.surface.set_hand_cursor(hand);
    }

    /// One window event for the open menu; what it chose, if anything.
    fn handle(&mut self, event: WindowEvent, client: &mut Client) -> Result<Option<Choice>> {
        self.track_mouse(&event, &client.surface);
        // The overlay is in the way of the mouse and the keys (`Gui.overlay() != null`).
        if self.loading.is_some() {
            return Ok(None);
        }
        self.handle_open(event, client)
    }

    /// `handle` for the screen that is not under an overlay.
    fn handle_open(&mut self, event: WindowEvent, client: &mut Client) -> Result<Option<Choice>> {
        let size = gui_size(client);
        let mouse = self.mouse;
        let action = match &mut self.screen {
            Screen::Home(_) => return Ok(self.title_input(event, client, size)),
            Screen::Servers(_) => self.servers_event(event),
            Screen::Form(..) => {
                let result = self.form_event(event, &mut client.surface);
                return self.form_result(result, size);
            }
            Screen::Connecting(..) => {
                self.connecting_event(event, size);
                return Ok(None);
            }
            Screen::Credits(..) => return Ok(self.credits_event(event, size)),
            Screen::Settings => {
                self.settings_event(event, client);
                return Ok(None);
            }
            Screen::Failed(..) | Screen::Ask(..) => {
                self.notice_event(event, size);
                return Ok(None);
            }
            Screen::Confirm(screen, index) => {
                let index = *index;
                let answer = confirm_event(screen, event, mouse);
                return self.delete_answer(answer, index, size);
            }
        };
        self.servers_action(action, size)
    }
}

/// The home screen a client starts into.
fn home_screen(
    font: &photon_render::gui::font::Font,
    lang: &photon_game::text::Lang,
    size: (i32, i32),
) -> Screen {
    Screen::Home(HomeScreen::new(size, &version_line(), font, lang))
}

/// The line under the name in the sidebar.
fn version_line() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

fn gui_size(client: &Client) -> (i32, i32) {
    let scale = client.surface.scale();
    (scale.gui_scaled_width(), scale.gui_scaled_height())
}

const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// The numbering the menus use: the newest version's. A game loads its own with the version
/// its connection chose.
fn menu_versions() -> std::sync::Arc<photon_data::version_registries::VersionRegistries> {
    photon_data::version_registries::VersionRegistries::of(photon_core::version::Version::NEWEST)
}

/// What the menus' assets are read with: the data pack's registries before any server named
/// them, and the numbering of the newest version.
fn menu_registries(
    data_root: &Path,
) -> (
    photon_game::registries::Registries,
    std::sync::Arc<photon_data::version_registries::VersionRegistries>,
) {
    (
        photon_game::registries::Registries::load(data_root, |_| Vec::new()),
        menu_versions(),
    )
}
