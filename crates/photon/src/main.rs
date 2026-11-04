mod ambient;
mod api;
mod assets;
mod bench;
mod content;
mod controls;
mod equipment;
mod frame_limiter;
mod gpu_meter;
mod gui;
mod hud_input;
mod identity_proof;
mod indexed;
mod keyring_store;
mod launch_link;
mod links_config;
mod logger;
mod menu;
mod mobs;
mod music;
mod objects;
mod overlays;
mod packs;
mod particles;
mod play;
mod ppm;
mod profile;
mod profile_login;
mod register_links;
mod resources;
mod screens;
mod screenshot;
mod script;
mod self_update;
mod settings;
mod sign_in;
mod signin_link;
mod skins;
mod social;
mod sound;
mod system_info;
mod updates;
mod view;
mod window_probe;
mod world;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init();
    self_update::apply_pending();
    content::use_installed();
    if env::args().nth(1).as_deref() == Some("window") {
        return window_probe::run_window(env::args().skip(2));
    }
    if env::args().nth(1).as_deref() == Some("play") {
        updates::start();
        let result = play::run_play(env::args().skip(2));
        return self_update::restart_if_requested(result);
    }
    if let Some(command) = env::args().nth(1)
        && let Some(result) = sign_in::run_account_command(&command)
    {
        return result;
    }
    if let Some(result) = run_menu_command(env::args().nth(1).as_deref()) {
        return result;
    }
    Err(format!(
        "unknown command {:?}; use `menu`, `play`, `sign-in`, `sign-out`, `identity-proof`, `open-link`, `register-links`, `install`, `update`, `window` or `--version`",
        env::args().nth(1)
    )
    .into())
}

/// `photon open-link <link>`: the menu opens with the website's launch link to spend.
fn open_link() -> Result<(), Box<dyn std::error::Error>> {
    let link = env::args()
        .nth(2)
        .ok_or("open-link needs a photon:// link")?;
    launch_link::remember(&link)?;
    open_menu(std::iter::empty())
}

/// The commands that open the menu or work without a window; nothing for any other command.
fn run_menu_command(command: Option<&str>) -> Option<Result<(), Box<dyn std::error::Error>>> {
    let result = match command {
        None => open_menu(std::iter::empty()),
        Some("menu") => open_menu(env::args().skip(2)),
        Some("open-link") => open_link(),
        Some("register-links") => register_links::run_register(),
        Some("update") => updates::run_update(),
        Some("install") => content::run_install(),
        Some("--version") => {
            println!("photon {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => return None,
    };
    Some(result)
}

/// The menu, with the update check running beside it.
fn open_menu(args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    updates::start();
    self_update::restart_if_requested(menu::run_menu(args))
}
