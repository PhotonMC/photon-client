use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_creative_place_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustCreativePlaceProbe".to_owned());
    let trace = connect_and_creative_place_probe(address, &username, Duration::from_secs(20))?;

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("stone_item_registry_id={:?}", trace.stone_item_registry_id);
    println!(
        "creative_mode_slot_packets_sent={}",
        trace.creative_mode_slot_packets_sent
    );
    println!(
        "carried_item_packets_sent={}",
        trace.carried_item_packets_sent
    );
    println!(
        "use_item_on_packets_sent={}",
        trace.use_item_on_packets_sent
    );
    println!(
        "creative_place_target_update={:?}",
        trace.creative_place_target_update
    );
    println!(
        "creative_place_neighbor_update={:?}",
        trace.creative_place_neighbor_update
    );
    println!(
        "block_changed_ack_sequences={:?}",
        trace.block_changed_ack_sequences
    );
    println!(
        "block_update_observations={:?}",
        trace.block_update_observations
    );
    println!("chunk_packet_count={}", trace.chunk_packet_count);
    println!("decoded_chunk_count={}", trace.chunk_observations.len());
    println!(
        "chunk_batch_finished_count={}",
        trace.chunk_batch_finished_count
    );
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
