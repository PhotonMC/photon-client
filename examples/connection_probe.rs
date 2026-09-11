use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args().nth(2).unwrap_or_else(|| "RustProbe".to_owned());
    let trace = connect_and_probe(address, &username, Duration::from_secs(20))?;

    println!("connection=ok");
    println!("compression_threshold={:?}", trace.compression_threshold);
    println!("login_finished={}", trace.login_finished);
    println!("configuration_finished={}", trace.configuration_finished);
    println!("enabled_feature_count={}", trace.enabled_feature_count);
    println!("known_pack_count={}", trace.known_pack_count);
    println!("registry_packet_count={}", trace.registry_packet_count);
    println!("registry_entry_count={}", trace.registry_entry_count);
    println!(
        "update_tags_packet_count={}",
        trace.update_tags_packet_count
    );
    println!("play_login={}", trace.play_login);
    println!("chunk_packet_count={}", trace.chunk_packet_count);
    println!(
        "chunk_batch_finished_count={}",
        trace.chunk_batch_finished_count
    );
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
