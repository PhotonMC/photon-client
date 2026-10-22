mod ambient;
mod assets;
mod bench;
mod controls;
mod equipment;
mod frame_limiter;
mod gpu_meter;
mod gui;
mod hud_input;
mod logger;
mod menu;
mod mobs;
mod music;
mod objects;
mod overlays;
mod particles;
mod play;
mod ppm;
mod profile;
mod profile_login;
mod screens;
mod screenshot;
mod script;
mod sound;
mod system_info;
mod view;
mod window_probe;
mod world;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::init();
    if env::args().nth(1).as_deref() == Some("window") {
        return window_probe::run_window(env::args().skip(2));
    }
    if env::args().nth(1).as_deref() == Some("play") {
        return play::run_play(env::args().skip(2));
    }
    match env::args().nth(1).as_deref() {
        None => return menu::run_menu(std::iter::empty()),
        Some("menu") => return menu::run_menu(env::args().skip(2)),
        _ => {}
    }
    Err(format!(
        "unknown command {:?}; use `menu`, `play` or `window`",
        env::args().nth(1)
    )
    .into())
}
