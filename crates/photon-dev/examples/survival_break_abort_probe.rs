use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_survival_break_abort_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustSurvival1".to_owned());
    let trace =
        connect_and_survival_break_abort_probe(address, &username, Duration::from_secs(20))?;

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "player_action_packets_sent={}",
        trace.player_action_packets_sent
    );
    println!(
        "block_changed_ack_sequences={:?}",
        trace.block_changed_ack_sequences
    );
    println!(
        "block_destruction_observations={:?}",
        trace.block_destruction_observations
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
