use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_move_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustMoveProbe".to_owned());
    let trace = connect_and_move_probe(address, &username, Duration::from_secs(20))?;

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!(
        "player_position_observations={:?}",
        trace.player_position_observations
    );
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("movement_packets_sent={}", trace.movement_packets_sent);
    println!("chunk_packet_count={}", trace.chunk_packet_count);
    println!("decoded_chunk_count={}", trace.chunk_observations.len());
    println!(
        "chunk_batch_finished_count={}",
        trace.chunk_batch_finished_count
    );
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
