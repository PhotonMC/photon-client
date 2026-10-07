use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_metadata_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args().nth(2).unwrap_or_else(|| "P08Air".to_owned());
    let trace =
        connect_and_world_entity_metadata_probe(address, &username, Duration::from_secs(60))?;
    let Some(player) = trace.world.local_player() else {
        return Err("vanilla player was not applied to ClientWorld".into());
    };
    if player.air_supply() >= 300 {
        return Err(format!(
            "vanilla player air supply did not fall below the default: {}",
            player.air_supply()
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_metadata_packet_count={}",
        trace.entity_metadata_observations.len()
    );
    println!("air_supply={}", player.air_supply());
    println!("max_air_supply=300");
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
