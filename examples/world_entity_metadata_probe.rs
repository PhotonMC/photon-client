use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_entity_metadata_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "EntityMetadata".to_owned());
    let trace =
        connect_and_world_entity_metadata_probe(address, &username, Duration::from_secs(60))?;
    let applied = trace
        .entity_metadata_observations
        .iter()
        .find(|observation| observation.applied_to_world);
    let Some(first) = applied else {
        return Err("vanilla entity metadata was not applied to ClientWorld".into());
    };
    if trace.world.entity_metadata_at(first.entity_id).is_none() {
        return Err("applied entity metadata is not queryable in ClientWorld".into());
    }

    let applied_count = trace
        .entity_metadata_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_metadata_packet_count={}",
        trace.entity_metadata_observations.len()
    );
    println!("entity_metadata_applied_count={applied_count}");
    println!(
        "first_entity_metadata=id:{},length:{},applied:{}",
        first.entity_id, first.metadata_length, first.applied_to_world
    );
    println!(
        "world_entity_metadata_length={}",
        trace
            .world
            .entity_metadata_at(first.entity_id)
            .map_or(0, <[u8]>::len)
    );
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
