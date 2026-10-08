//! The menus before and between games: the title screen and the server list, from which the
//! client joins a server and to which it comes back when the player leaves it.

use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use glfw::{Action, Key, Modifiers, MouseButton, WindowEvent};
use voxel_game::servers::{ServerList, address_with_port};
use voxel_render::gui::screen::servers::{JoinMultiplayerScreen, ServersAction};
use voxel_render::gui::screen::title::{TitleAction, TitleScreen};
use voxel_render::gui::widgets::navigation::Direction;

use crate::gui::Gui;
use crate::play::{self, Client, Exit};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The reference holds a menu with no level to sixty frames a second.
const MENU_FRAME: Duration = Duration::from_micros(16_667);
/// `MouseHandler`: a second press this soon after the first is a double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(250);

/// What the menus chose.
enum Choice {
    Quit,
    Join(SocketAddr),
}

/// The menu open.
enum Screen {
    Title(TitleScreen),
    Servers(JoinMultiplayerScreen),
}

/// `voxel` or `voxel menu [server [width height]]`: the title screen, the server list from its
/// Multiplayer button, and the game a server's Join starts; leaving it comes back here. A
/// scripted run joins `server` at once.
pub fn run_menu(mut args: impl Iterator<Item = String>) -> Result<()> {
    let server: SocketAddr = args
        .next()
        .unwrap_or_else(|| play::DEFAULT_SERVER.to_owned())
        .parse()?;
    let width: i32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(1280);
    let height: i32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(720);
    let mut client = Client::open((width, height))?;
    let mut scripted_join = env::var_os("VOXEL_SCRIPT").is_some();
    loop {
        let choice = if std::mem::take(&mut scripted_join) {
            Choice::Join(server)
        } else {
            Menu::open(&mut client)?.run(&mut client)?
        };
        match choice {
            Choice::Quit => return Ok(()),
            Choice::Join(server) => {
                client.backend.set_gui_only(false);
                let (back, exit) = play::play_in(client, server)?;
                client = back;
                if let Exit::Quit = exit {
                    return Ok(());
                }
            }
        }
    }
}

/// The menus' state between frames.
struct Menu {
    gui: Gui,
    splash: Option<String>,
    game_directory: PathBuf,
    servers: ServerList,
    screen: Screen,
    mouse: (i32, i32),
    /// When and where the last primary press was, for a double click.
    last_click: Option<(Instant, (i32, i32))>,
}

impl Menu {
    /// The title screen, with the language and the GUI scale of the options.
    fn open(client: &mut Client) -> Result<Self> {
        let root =
            env::var("VOXEL_ASSETS").map_err(|_| "VOXEL_ASSETS must name the asset folder")?;
        let root = Path::new(&root);
        let data_root = env::var("VOXEL_DATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| root.with_file_name("data"));
        let options = crate::controls::load_options();
        let registries = voxel_game::registries::Registries::load(&data_root, |_| Vec::new());
        let gui = Gui::load(
            root,
            registries,
            &mut client.backend,
            &options.language_code,
        )?;
        client.surface.set_gui_scale_option(options.gui_scale);
        client.surface.set_cursor_captured(false);
        client.backend.set_gui_only(true);
        let user = env::var("VOXEL_USER").unwrap_or_else(|_| "VoxelClient".to_owned());
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos() as i64);
        let splash = voxel_game::splash::pick(
            &voxel_game::splash::read_splashes(root),
            voxel_game::splash::today(),
            &user,
            &mut voxel_core::random::RandomSource::create(seed),
        );
        let game_directory = crate::controls::game_directory();
        let screen = Screen::Title(TitleScreen::new(
            gui.text_assets().0,
            gui.lang(),
            gui_size(client),
            splash.as_deref(),
        ));
        Ok(Self {
            gui,
            splash,
            servers: ServerList::load(&game_directory),
            game_directory,
            screen,
            mouse: (0, 0),
            last_click: None,
        })
    }

    fn title(&self, size: (i32, i32)) -> Screen {
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        Screen::Title(TitleScreen::new(font, lang, size, self.splash.as_deref()))
    }

    /// `JoinMultiplayerScreen`, the saved servers read again.
    fn server_list(&mut self, size: (i32, i32)) -> Screen {
        self.servers = ServerList::load(&self.game_directory);
        let names = self
            .servers
            .servers
            .iter()
            .map(|server| server.name.clone())
            .collect();
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        Screen::Servers(JoinMultiplayerScreen::new(font, lang, size, names))
    }

    /// Frames until a menu chooses or the window closes. `VOXEL_CAPTURE_PPM`: the tenth frame is
    /// saved there and the client stops.
    fn run(mut self, client: &mut Client) -> Result<Choice> {
        let capture = env::var("VOXEL_CAPTURE_PPM").ok();
        let mut frames = 0;
        while !client.surface.should_close() {
            let frame_start = Instant::now();
            for event in client.surface.poll() {
                if let Some(choice) = self.handle(event, client)? {
                    return Ok(choice);
                }
            }
            play::fit_swapchain(client)?;
            let size = gui_size(client);
            let current = match &self.screen {
                Screen::Title(screen) => screen.size(),
                Screen::Servers(screen) => screen.size(),
            };
            if size != current {
                self.screen = match self.screen {
                    Screen::Title(_) => self.title(size),
                    Screen::Servers(_) => self.server_list(size),
                };
            }
            let mouse = self.mouse;
            let screen = &self.screen;
            self.gui.draw_menu_screen(
                &mut client.backend,
                &client.surface,
                |frame, atlas, font| match screen {
                    Screen::Title(screen) => screen.draw(frame, atlas, font, mouse),
                    Screen::Servers(screen) => screen.draw(frame, atlas, font, mouse),
                },
            )?;
            frames += 1;
            if let Some(path) = capture.as_deref().filter(|_| frames == 10) {
                let captured = client
                    .backend
                    .capture_scene([0.0, 0.0, 0.0, 1.0], IDENTITY)?;
                crate::ppm::write(path, captured.width, captured.height, &captured.rgba)?;
                println!("captured_frame={path}");
                return Ok(Choice::Quit);
            }
            client.backend.draw_scene([0.0, 0.0, 0.0, 1.0], IDENTITY)?;
            if let Some(rest) = MENU_FRAME.checked_sub(frame_start.elapsed()) {
                std::thread::sleep(rest);
            }
        }
        Ok(Choice::Quit)
    }

    /// One window event for the open menu; what it chose, if anything.
    fn handle(&mut self, event: WindowEvent, client: &Client) -> Result<Option<Choice>> {
        let size = gui_size(client);
        if let WindowEvent::CursorPos(x, y) = event {
            let scale = client.surface.scale();
            let (width, height) = client.surface.window_size();
            self.mouse = (
                (x * f64::from(scale.gui_scaled_width()) / f64::from(width.max(1))) as i32,
                (y * f64::from(scale.gui_scaled_height()) / f64::from(height.max(1))) as i32,
            );
        }
        let mouse = self.mouse;
        let action = match &mut self.screen {
            Screen::Title(screen) => {
                match title_event(screen, event, mouse) {
                    Some(TitleAction::Multiplayer) => self.screen = self.server_list(size),
                    Some(TitleAction::Quit) => return Ok(Some(Choice::Quit)),
                    // The other screens are not here yet.
                    _ => {}
                }
                return Ok(None);
            }
            Screen::Servers(screen) => match event {
                WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                    let now = Instant::now();
                    let double = self
                        .last_click
                        .is_some_and(|(at, place)| now - at < DOUBLE_CLICK && place == mouse);
                    self.last_click = Some((now, mouse));
                    screen.clicked(mouse, double)
                }
                WindowEvent::Scroll(_, notches) => {
                    screen.scrolled((f64::from(mouse.0), f64::from(mouse.1)), notches);
                    None
                }
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => screen.key_pressed(false),
                WindowEvent::Key(Key::Enter | Key::KpEnter, _, Action::Press, _) => {
                    screen.key_pressed(true)
                }
                // `JoinMultiplayerScreen.keyPressed`: F5 refreshes.
                WindowEvent::Key(Key::F5, _, Action::Press, _) => Some(ServersAction::Refresh),
                _ => None,
            },
        };
        self.servers_action(action, size)
    }

    /// What a press on the server list does.
    fn servers_action(
        &mut self,
        action: Option<ServersAction>,
        size: (i32, i32),
    ) -> Result<Option<Choice>> {
        match action {
            Some(ServersAction::Join(index)) => {
                let Some(server) = self.servers.servers.get(index) else {
                    return Ok(None);
                };
                match address_with_port(&server.ip).to_socket_addrs() {
                    Ok(mut addresses) => {
                        if let Some(address) = addresses.next() {
                            return Ok(Some(Choice::Join(address)));
                        }
                    }
                    Err(error) => log::warn!("cannot resolve {}: {error}", server.ip),
                }
            }
            Some(ServersAction::Swap(from, to)) => {
                self.servers.swap(from, to);
                self.servers.save(&self.game_directory)?;
            }
            Some(ServersAction::Refresh) => self.screen = self.server_list(size),
            Some(ServersAction::Back) => self.screen = self.title(size),
            // Adding, editing, deleting and joining directly are not here yet.
            _ => {}
        }
        Ok(None)
    }
}

/// An event for the title screen's buttons: what the press chose.
fn title_event(
    screen: &mut TitleScreen,
    event: WindowEvent,
    mouse: (i32, i32),
) -> Option<TitleAction> {
    let buttons = screen.buttons();
    match event {
        WindowEvent::CursorPos(..) => {
            buttons.mouse_input();
            None
        }
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => buttons.clicked(mouse),
        WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
            buttons.tab(!modifiers.contains(Modifiers::Shift));
            None
        }
        WindowEvent::Key(key, _, Action::Press | Action::Repeat, _) => {
            match key {
                Key::Left => buttons.arrow(Direction::Left),
                Key::Right => buttons.arrow(Direction::Right),
                Key::Up => buttons.arrow(Direction::Up),
                Key::Down => buttons.arrow(Direction::Down),
                Key::Enter | Key::KpEnter | Key::Space => return buttons.pressed(),
                _ => {}
            }
            None
        }
        _ => None,
    }
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
