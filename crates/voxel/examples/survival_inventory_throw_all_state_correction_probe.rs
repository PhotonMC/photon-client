use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_survival_inventory_throw_all_state_correction_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "InvThrowAll1".to_owned());
    let trace = connect_and_survival_inventory_throw_all_state_correction_probe(
        address,
        &username,
        Duration::from_secs(45),
    )?;

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "container_click_packets_sent={}",
        trace.container_click_packets_sent
    );
    println!(
        "container_click_state_id_sent={:?}",
        trace.container_click_state_id_sent
    );
    println!(
        "container_slot_observations={:?}",
        trace.container_slot_observations
    );
    println!(
        "container_content_observations={:?}",
        trace.container_content_observations
    );
    println!(
        "add_entity_observations={:?}",
        trace.add_entity_observations
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
