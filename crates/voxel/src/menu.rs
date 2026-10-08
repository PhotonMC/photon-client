//! The menus before and between games: the title screen, from which the client joins a server and
//! to which it comes back when the player leaves it.

use std::env;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use glfw::{Action, Key, Modifiers, MouseButton, WindowEvent};
use voxel_render::gui::screen::title::{TitleAction, TitleScreen};
use voxel_render::gui::widgets::navigation::Direction;

use crate::gui::Gui;
use crate::play::{self, Client, Exit};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The reference holds a menu with no level to sixty frames a second.
const MENU_FRAME: Duration = Duration::from_micros(16_667);

/// What the title screen chose.
enum Choice {
    Quit,
    Join(SocketAddr),
}

/// `voxel` or `voxel menu [server [width height]]`: the title screen; Multiplayer joins the
/// server (until the server list chooses one), and leaving it comes back here.
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
    // A scripted run joins from the first title screen, as a person pressing Multiplayer would.
    let mut scripted_join = env::var_os("VOXEL_SCRIPT").is_some();
    loop {
        let choice = if std::mem::take(&mut scripted_join) {
            Choice::Join(server)
        } else {
            title(&mut client, server)?
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

/// The title screen until a button chooses or the window closes.
fn title(client: &mut Client, server: SocketAddr) -> Result<Choice> {
    let root = env::var("VOXEL_ASSETS").map_err(|_| "VOXEL_ASSETS must name the asset folder")?;
    let root = std::path::Path::new(&root);
    let data_root = env::var("VOXEL_DATA")
        .map(std::path::PathBuf::from)
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
    let (font, lang) = (gui.text_assets().0, gui.lang());
    let new_screen = |size: (i32, i32)| TitleScreen::new(font, lang, size, splash.as_deref());
    let gui_size = |client: &Client| {
        let scale = client.surface.scale();
        (scale.gui_scaled_width(), scale.gui_scaled_height())
    };
    let mut screen = new_screen(gui_size(client));
    let mut mouse = (0, 0);
    // `VOXEL_CAPTURE_PPM`: the tenth frame is saved there and the client stops.
    let capture = env::var("VOXEL_CAPTURE_PPM").ok();
    let mut frames = 0;
    while !client.surface.should_close() {
        let frame_start = Instant::now();
        let mut chosen = None;
        for event in client.surface.poll() {
            let buttons = screen.buttons();
            match event {
                WindowEvent::CursorPos(x, y) => {
                    let scale = client.surface.scale();
                    let (width, height) = client.surface.window_size();
                    mouse = (
                        (x * f64::from(scale.gui_scaled_width()) / f64::from(width.max(1))) as i32,
                        (y * f64::from(scale.gui_scaled_height()) / f64::from(height.max(1)))
                            as i32,
                    );
                    buttons.mouse_input();
                }
                WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                    chosen = chosen.or(buttons.clicked(mouse));
                }
                WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
                    buttons.tab(!modifiers.contains(Modifiers::Shift));
                }
                WindowEvent::Key(key, _, Action::Press | Action::Repeat, _) => match key {
                    Key::Left => buttons.arrow(Direction::Left),
                    Key::Right => buttons.arrow(Direction::Right),
                    Key::Up => buttons.arrow(Direction::Up),
                    Key::Down => buttons.arrow(Direction::Down),
                    Key::Enter | Key::KpEnter | Key::Space => {
                        chosen = chosen.or(buttons.pressed());
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        match chosen {
            Some(TitleAction::Multiplayer) => return Ok(Choice::Join(server)),
            Some(TitleAction::Quit) => return Ok(Choice::Quit),
            // The other screens are not here yet.
            _ => {}
        }
        play::fit_swapchain(client)?;
        if gui_size(client) != screen.size() {
            screen = new_screen(gui_size(client));
        }
        gui.draw_title(&mut client.backend, &client.surface, &screen, mouse)?;
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

const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
