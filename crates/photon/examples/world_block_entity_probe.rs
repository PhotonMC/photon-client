use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustBlockEntity".to_owned());
    let trace = connect_and_probe(address, &username, Duration::from_secs(60))?;
    let records = trace
        .chunk_observations
        .iter()
        .flat_map(|chunk| chunk.block_entities.iter())
        .collect::<Vec<_>>();
    if records.is_empty() || trace.world.loaded_block_entity_count() == 0 {
        return Err("vanilla chunk block entities were not applied to ClientWorld".into());
    }

    let first = records[0];
    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("chunk_block_entity_count={}", records.len());
    println!(
        "block_entity_applied_count={}",
        records
            .iter()
            .filter(|record| record.applied_to_world)
            .count()
    );
    println!(
        "first_block_entity=x:{},y:{},z:{},type_id:{},tag_length:{}",
        first.x, first.y, first.z, first.type_id, first.tag_length
    );
    println!(
        "world_block_entity_count={}",
        trace.world.loaded_block_entity_count()
    );
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
