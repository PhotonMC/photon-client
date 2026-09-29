use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_block_entity_update_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustBlockEntityUpdate".to_owned());
    let trace =
        connect_and_world_block_entity_update_probe(address, &username, Duration::from_secs(60))?;
    let applied_count = trace
        .block_entity_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    let target = trace.world.block_entity_at(25, 80, -36);
    if trace.block_entity_observations.is_empty() || applied_count == 0 || target.is_none() {
        return Err("vanilla standalone block entity data was not applied to ClientWorld".into());
    }

    let first = &trace.block_entity_observations[0];
    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "block_entity_data_packet_count={}",
        trace.block_entity_observations.len()
    );
    println!("block_entity_data_applied_count={applied_count}");
    println!(
        "first_block_entity_data=x:{},y:{},z:{},type_id:{},tag_length:{},applied:{}",
        first.x, first.y, first.z, first.type_id, first.tag_length, first.applied_to_world
    );
    println!(
        "target_block_entity={:?}",
        target.map(|entity| (entity.type_id, entity.tag.as_ref().map(Vec::len)))
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
