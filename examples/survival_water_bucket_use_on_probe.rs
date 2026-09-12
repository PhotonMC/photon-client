use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_survival_water_bucket_use_on_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustWaterUse1".to_owned());
    let trace = connect_and_survival_water_bucket_use_on_probe(
        address,
        &username,
        Duration::from_secs(30),
    )?;

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "carried_item_packets_sent={}",
        trace.carried_item_packets_sent
    );
    println!(
        "use_item_on_packets_sent={}",
        trace.use_item_on_packets_sent
    );
    println!("use_item_packets_sent={}", trace.use_item_packets_sent);
    println!(
        "container_slot_observations={:?}",
        trace.container_slot_observations
    );
    println!(
        "container_content_observations={:?}",
        trace.container_content_observations
    );
    println!(
        "survival_water_bucket_target_update={:?}",
        trace.survival_water_bucket_target_update
    );
    if let Some(update) = trace.survival_water_bucket_target_update.as_ref() {
        println!(
            "survival_water_bucket_target_world_state={:?}",
            trace.world.block_state_at(update.x, update.y, update.z)
        );
    }
    println!(
        "survival_water_bucket_neighbor_update={:?}",
        trace.survival_water_bucket_neighbor_update
    );
    if let Some(update) = trace.survival_water_bucket_neighbor_update.as_ref() {
        println!(
            "survival_water_bucket_neighbor_world_state={:?}",
            trace.world.block_state_at(update.x, update.y, update.z)
        );
    }
    println!(
        "block_update_observations={:?}",
        trace.block_update_observations
    );
    println!(
        "block_changed_ack_sequences={:?}",
        trace.block_changed_ack_sequences
    );
    println!(
        "system_chat_observations={:?}",
        trace.system_chat_observations
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
