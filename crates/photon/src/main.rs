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
mod indexed;
mod keyring_store;
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
mod resources;
mod screens;
mod screenshot;
mod script;
mod sign_in;
mod skins;
mod sound;
mod system_info;
mod view;
mod window_probe;
mod world;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init();
    content::use_installed();
    if env::args().nth(1).as_deref() == Some("window") {
        return window_probe::run_window(env::args().skip(2));
    }
    if env::args().nth(1).as_deref() == Some("play") {
        return play::run_play(env::args().skip(2));
    }
    match env::args().nth(1).as_deref() {
        None => return menu::run_menu(std::iter::empty()),
        Some("menu") => return menu::run_menu(env::args().skip(2)),
        Some("sign-in") => return sign_in::run_sign_in(),
        Some("sign-out") => return sign_in::run_sign_out(),
        Some("install") => return content::run_install(),
        Some("--version") => {
            println!("photon {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        _ => {}
    }
    Err(format!(
        "unknown command {:?}; use `menu`, `play`, `sign-in`, `sign-out`, `install`, `window` or `--version`",
        env::args().nth(1)
    )
    .into())
}
