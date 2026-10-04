mod animated_atlas;
mod armor;
mod armor_stands;
mod assets;
mod atmosphere;
mod axolotls;
mod bears;
mod boats;
mod controls;
mod copper_golems;
mod dolphins;
mod dropped_items;
mod farm_animals;
mod felines;
mod fish;
mod foxes;
mod frogs;
mod golems;
mod guardians;
mod gui;
mod happy_ghasts;
mod held_items;
mod hoglins;
mod hud_input;
mod illagers;
mod item_layers;
mod mobs;
mod monsters;
mod nautilus;
mod overlays;
mod pandas;
mod play;
mod ppm;
mod profile;
mod ravagers;
mod screens;
mod script;
mod sections;
mod sheep;
mod sky;
mod squids;
mod striders;
mod turtles;
mod view;
mod villagers;
mod window_probe;
mod wolves;

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    if env::args().nth(1).as_deref() == Some("window") {
        return window_probe::run_window(env::args().skip(2));
    }
    if env::args().nth(1).as_deref() == Some("play") {
        return play::run_play(env::args().skip(2));
    }
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "VoxelClient".to_owned());
    let ticks: u32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(20);
    let timeout_seconds: u64 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(20);
    if timeout_seconds == 0 {
        return Err("timeout_seconds must be greater than zero".into());
    }

    let mut trace = connect_and_probe(address, &username, Duration::from_secs(timeout_seconds))?;
    for _ in 0..ticks {
        trace.world.tick();
    }

    println!("client_connection=ok");
    println!("play_login={}", trace.play_login);
    println!("world_constructed=true");
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!(
        "update_tags_packet_count={}",
        trace.update_tags_packet_count
    );
    println!(
        "block_tag_vine_climbable={}",
        trace.world.block_state_registry().is_climbable_state(8389)
    );
    println!(
        "block_tag_vine_glide_through={}",
        trace
            .world
            .block_state_registry()
            .is_glide_through_state(8389)
    );
    println!("ticks_run={ticks}");
    println!("world_game_time={}", trace.world.game_time());
    let local_player_previous_position = trace
        .world
        .local_player()
        .map(|player| player.previous_position())
        .unwrap_or((0.0, 0.0, 0.0));
    let local_player_interpolated_position = trace
        .world
        .local_player()
        .map(|player| player.interpolated_position(0.5))
        .unwrap_or((0.0, 0.0, 0.0));
    println!("local_player_previous_position={local_player_previous_position:?}");
    println!("local_player_interpolated_position_half={local_player_interpolated_position:?}");
    println!(
        "local_player_tick_count={}",
        trace
            .world
            .local_player()
            .map(|player| player.tick_count())
            .unwrap_or(0)
    );
    Ok(())
}
