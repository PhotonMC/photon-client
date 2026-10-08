use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_world_entity_metadata_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08StingerCount".to_owned());
    let trace =
        connect_and_world_entity_metadata_probe(address, &username, Duration::from_secs(60))?;
    let Some(player) = trace.world.local_player() else {
        return Err("vanilla did not create the local player in ClientWorld".into());
    };
    if player.stinger_count() <= 0 {
        return Err("vanilla stinger count was not applied to the local player".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_metadata_packet_count={}",
        trace.entity_metadata_observations.len()
    );
    println!("stinger_count={}", player.stinger_count());
    println!("local_player_id={}", player.id());
    println!("local_player_position={:?}", player.position());
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
