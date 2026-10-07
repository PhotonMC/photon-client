use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_ticking_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "EntityTicking".to_owned());
    let trace =
        connect_and_world_entity_ticking_probe(address, &username, Duration::from_secs(60))?;

    let ticking_count = trace
        .world
        .entities()
        .all()
        .filter(|entity| entity.is_ticking())
        .count();
    let tracked_count = trace.world.entities().count();
    let stopped_count = tracked_count.saturating_sub(ticking_count);
    let stopped_unload_count = trace
        .chunk_unload_observations
        .iter()
        .filter(|observation| observation.stopped_entity_count > 0)
        .count();
    let removed_count = trace
        .chunk_unload_observations
        .iter()
        .filter(|observation| observation.removed_from_world)
        .count();

    if removed_count == 0 {
        return Err(format!(
            "vanilla did not provide an applied chunk unload: movement={}, chunks={}, unloads={}, tracked={}, ticking={}, stopped={}, observations={:?}",
            trace.movement_packets_sent,
            trace.chunk_packet_count,
            trace.chunk_unload_observations.len(),
            tracked_count,
            ticking_count,
            stopped_count,
            trace.chunk_unload_observations
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "chunk_unload_packet_count={}",
        trace.chunk_unload_observations.len()
    );
    println!("chunk_unload_applied_count={removed_count}");
    println!("world_entity_count={tracked_count}");
    println!("ticking_entity_count={ticking_count}");
    println!("tracked_stopped_entity_count={stopped_count}");
    println!("unloads_with_stopped_entities={stopped_unload_count}");
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
