use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::query_status_and_ping;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let trace = query_status_and_ping(address, Duration::from_secs(20))?;

    println!("status_connection=ok");
    println!("status_response_length={}", trace.status_response.len());
    println!("status_response={}", trace.status_response);
    println!("pong_time={}", trace.pong_time);
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
