use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_creative_snowball_use_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustSnowball1".to_owned());
    let trace =
        connect_and_creative_snowball_use_probe(address, &username, Duration::from_secs(20))?;

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "snowball_item_registry_id={:?}",
        trace.snowball_item_registry_id
    );
    println!(
        "creative_mode_slot_packets_sent={}",
        trace.creative_mode_slot_packets_sent
    );
    println!(
        "carried_item_packets_sent={}",
        trace.carried_item_packets_sent
    );
    println!("use_item_packets_sent={}", trace.use_item_packets_sent);
    println!(
        "add_entity_observations={:?}",
        trace.add_entity_observations
    );
    println!(
        "block_changed_ack_sequences={:?}",
        trace.block_changed_ack_sequences
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
