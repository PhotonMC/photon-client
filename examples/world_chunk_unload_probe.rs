use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_chunk_unload_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustChunkUnload".to_owned());
    let trace = connect_and_world_chunk_unload_probe(address, &username, Duration::from_secs(20))?;

    let removed_count = trace
        .chunk_unload_observations
        .iter()
        .filter(|observation| observation.removed_from_world)
        .count();
    if trace.chunk_unload_observations.is_empty() || removed_count == 0 {
        eprintln!(
            "chunk unload missing: movement_packets_sent={}, chunk_packets={}, unload_observations={}, world_chunks={}",
            trace.movement_packets_sent,
            trace.chunk_packet_count,
            trace.chunk_unload_observations.len(),
            trace.world.loaded_chunk_count()
        );
        return Err("vanilla did not provide a chunk unload applied to ClientWorld".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("movement_packets_sent={}", trace.movement_packets_sent);
    println!(
        "chunk_unload_observations={:?}",
        trace.chunk_unload_observations
    );
    println!("chunk_unload_applied_count={removed_count}");
    println!("chunk_packet_count={}", trace.chunk_packet_count);
    println!("decoded_chunk_count={}", trace.chunk_observations.len());
    println!(
        "chunk_batch_finished_count={}",
        trace.chunk_batch_finished_count
    );
    if let Some(first_chunk) = trace.chunk_observations.first() {
        println!(
            "first_chunk=x:{},z:{},present_after_unload={}",
            first_chunk.x,
            first_chunk.z,
            trace.world.contains_chunk(first_chunk.x, first_chunk.z),
        );
    }
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
