use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_metadata_probe;
use voxel_world::ClientPlayerGameMode;

const CHANGE_GAME_MODE_EVENT_ID: u8 = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25571".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P09GameMode".to_owned());
    let trace =
        connect_and_world_entity_metadata_probe(address, &username, Duration::from_secs(60))?;

    let expected_modes = [
        ClientPlayerGameMode::Survival,
        ClientPlayerGameMode::Adventure,
        ClientPlayerGameMode::Spectator,
        ClientPlayerGameMode::Creative,
    ];
    let change_events: Vec<_> = trace
        .game_event_observations
        .iter()
        .filter(|observation| observation.packet.event_id == CHANGE_GAME_MODE_EVENT_ID)
        .collect();
    if change_events.len() < expected_modes.len() {
        return Err(format!(
            "vanilla sent too few CHANGE_GAME_MODE events: expected at least {}, got {:?}",
            expected_modes.len(),
            trace.game_event_observations
        )
        .into());
    }
    for (observation, expected_mode) in change_events.iter().zip(expected_modes) {
        if !observation.applied_to_world || observation.local_game_mode != Some(expected_mode) {
            return Err(format!(
                "game-mode event was not applied in order: expected {}, observation={observation:?}",
                expected_mode.name()
            )
            .into());
        }
    }

    let applied_abilities = trace
        .player_abilities_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    if applied_abilities < expected_modes.len() + 1 {
        return Err(format!(
            "vanilla abilities updates were incomplete: {}/{} applied, observations={:?}",
            applied_abilities,
            expected_modes.len() + 1,
            trace.player_abilities_observations
        )
        .into());
    }

    let player = trace
        .world
        .local_player()
        .ok_or("Rust world did not create a local player state")?;
    let abilities = player.abilities();
    if !trace.play_login
        || !trace.player_loaded_sent
        || player.game_mode() != ClientPlayerGameMode::Creative
        || !abilities.invulnerable()
        || abilities.flying()
        || !abilities.may_fly()
        || !abilities.instabuild()
        || !abilities.may_build()
        || abilities.flying_speed().to_bits() != 0.05_f32.to_bits()
        || abilities.walking_speed().to_bits() != 0.1_f32.to_bits()
    {
        return Err(format!(
            "final creative game-mode state differed from vanilla: player={player:?}, game_events={:?}, abilities={:?}",
            trace.game_event_observations,
            trace.player_abilities_observations
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("change_game_mode_event_count={}", change_events.len());
    println!("change_game_mode_events={change_events:?}");
    println!(
        "player_abilities_packet_count={}",
        trace.player_abilities_observations.len()
    );
    println!("player_abilities_applied_count={applied_abilities}");
    println!(
        "player_abilities_packets={:?}",
        trace.player_abilities_observations
    );
    println!("final_game_mode={}", player.game_mode().name());
    println!("final_abilities={abilities:?}");
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
