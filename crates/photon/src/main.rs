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
mod self_update;
mod settings;
mod sign_in;
mod skins;
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
    match env::args().nth(1).as_deref() {
        None => {
            updates::start();
            return self_update::restart_if_requested(menu::run_menu(std::iter::empty()));
        }
        Some("menu") => {
            updates::start();
            return self_update::restart_if_requested(menu::run_menu(env::args().skip(2)));
        }
        Some("update") => return updates::run_update(),
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
        "unknown command {:?}; use `menu`, `play`, `sign-in`, `sign-out`, `install`, `update`, `window` or `--version`",
        env::args().nth(1)
    )
    .into())
}
