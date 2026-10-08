use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_world_block_entity_update_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustSpawnerState".to_owned());
    let trace =
        connect_and_world_block_entity_update_probe(address, &username, Duration::from_secs(60))?;
    let applied_count = trace
        .block_entity_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    let entity = trace
        .world
        .block_entity_at(25, 80, -36)
        .ok_or("vanilla spawner block entity was not applied to ClientWorld")?;
    if entity.type_id != 9 || applied_count == 0 {
        return Err("vanilla block entity update did not describe a mob spawner".into());
    }
    let state = entity
        .spawner_state()
        .ok_or("spawner update tag did not expose semantic state")?;
    if state.min_spawn_delay() != 200
        || state.max_spawn_delay() != 800
        || state.spawn_count() != 4
        || state.max_nearby_entities() != 6
        || state.required_player_range() != 16
        || state.spawn_range() != 4
    {
        return Err("vanilla spawner defaults did not reach ClientWorld".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("block_entity_data_applied_count={applied_count}");
    println!("spawner_type_id={}", entity.type_id);
    println!(
        "spawner_state=delay:{} min:{} max:{} count:{} nearby:{} range:{} required:{}",
        state.delay(),
        state.min_spawn_delay(),
        state.max_spawn_delay(),
        state.spawn_count(),
        state.max_nearby_entities(),
        state.spawn_range(),
        state.required_player_range()
    );
    println!(
        "world_block_entity_count={}",
        trace.world.loaded_block_entity_count()
    );
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
