use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::{Recorder, query_status_and_ping_recorded};
use photon_protocol::phase::ConnectionProtocol;

/// Counts the packets of the exchange.
#[derive(Default)]
struct Packets(usize);

impl Recorder for Packets {
    fn packet(&mut self, _phase: ConnectionProtocol, _id: i32, _body_length: usize) {
        self.0 += 1;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let mut packets = Packets::default();
    let status = query_status_and_ping_recorded(address, Duration::from_secs(20), &mut packets)?;

    println!("status_connection=ok");
    println!("status_response_length={}", status.status_response.len());
    println!("status_response={}", status.status_response);
    println!("pong_time={}", status.pong_time);
    println!("received_packet_count={}", packets.0);
    Ok(())
}
