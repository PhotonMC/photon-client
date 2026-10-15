use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_dimension_respawn_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "DimensionProbe".to_owned());
    let trace =
        connect_and_world_dimension_respawn_probe(address, &username, Duration::from_secs(60))?;
    let Some(respawn) = trace.respawn_observations.last() else {
        return Err("vanilla did not provide a respawn packet during the probe".into());
    };

    if !respawn.dimension_changed || !respawn.applied_to_world {
        return Err(format!("respawn did not replace the client world: {respawn:?}").into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "login_dimension={}",
        trace
            .login_observation
            .as_ref()
            .map(|login| login.spawn_info.dimension.as_str())
            .unwrap_or("unknown")
    );
    println!("respawn_packet_count={}", trace.respawn_observations.len());
    println!("respawn_dimension={}", respawn.spawn_info.dimension);
    println!(
        "respawn_dimension_type_id={}",
        respawn.spawn_info.dimension_type_id
    );
    println!("respawn_data_to_keep={}", respawn.data_to_keep);
    println!("dimension_changed={}", respawn.dimension_changed);
    println!("cleared_chunk_count={}", respawn.cleared_chunk_count);
    println!("cleared_entity_count={}", respawn.cleared_entity_count);
    println!("world_dimension={}", trace.world.dimension_key());
    println!(
        "world_bounds=min_y:{},height:{}",
        trace.world.dimension().min_y,
        trace.world.dimension().height
    );
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("world_entity_count={}", trace.world.entities().count());
    println!("chunk_packet_count={}", trace.chunk_packet_count);
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
