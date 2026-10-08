use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_world_entity_remove_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "EntityRemove".to_owned());
    let trace = connect_and_world_entity_remove_probe(address, &username, Duration::from_secs(60))?;
    let applied_count: usize = trace
        .remove_entities_observations
        .iter()
        .map(|observation| observation.applied_entity_count)
        .sum();
    let removed_known_entity = trace
        .remove_entities_observations
        .iter()
        .flat_map(|observation| observation.entity_ids.iter().copied())
        .any(|entity_id| {
            trace
                .add_entity_observations
                .iter()
                .any(|observation| observation.entity_id == entity_id)
                && trace.world.entities().at(entity_id).is_none()
        });
    if trace.remove_entities_observations.is_empty() || applied_count == 0 || !removed_known_entity
    {
        return Err("vanilla entity removal was not applied to ClientWorld".into());
    }

    let first = &trace.remove_entities_observations[0];
    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "remove_entities_packet_count={}",
        trace.remove_entities_observations.len()
    );
    println!("removed_entity_id_count={}", first.entity_ids.len());
    println!("remove_entities_applied_count={applied_count}");
    println!("first_removed_entity_ids={:?}", first.entity_ids);
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
