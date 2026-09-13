use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_player_menu_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08EntitySleepingPos".to_owned());
    let trace = connect_and_world_player_menu_probe(address, &username, Duration::from_secs(60))?;
    let Some(player) = trace.world.local_player() else {
        return Err("vanilla did not create the local player in ClientWorld".into());
    };
    let Some(sleeping_pos) = player.sleeping_pos() else {
        return Err("vanilla did not apply the player's sleeping position".into());
    };

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_metadata_packet_count={}",
        trace.entity_metadata_observations.len()
    );
    println!("sleeping_pos={sleeping_pos:?}");
    println!("is_sleeping={}", player.is_sleeping());
    println!("pose={:?}", player.pose());
    println!("local_player_position={:?}", player.position());
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
