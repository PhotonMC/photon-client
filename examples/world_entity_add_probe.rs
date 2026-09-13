use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_entity_add_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustEntityAdd".to_owned());
    let trace = connect_and_world_entity_add_probe(address, &username, Duration::from_secs(60))?;
    let applied_count = trace
        .add_entity_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    if trace.add_entity_observations.is_empty()
        || applied_count == 0
        || trace.world.loaded_entity_count() == 0
    {
        return Err("vanilla entity add was not applied to ClientWorld".into());
    }

    let first = &trace.add_entity_observations[0];
    let entity = trace.world.entity_at(first.entity_id);
    if entity.is_none() {
        return Err("applied vanilla entity is not queryable in ClientWorld".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "add_entity_packet_count={}",
        trace.add_entity_observations.len()
    );
    println!("add_entity_applied_count={applied_count}");
    println!(
        "first_entity=id:{},type_id:{},x:{},y:{},z:{},data:{},applied:{}",
        first.entity_id,
        first.entity_type_id,
        first.x,
        first.y,
        first.z,
        first.data,
        first.applied_to_world
    );
    println!(
        "world_entity={:?}",
        entity.map(|value| (value.type_id, value.position(), value.data))
    );
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
