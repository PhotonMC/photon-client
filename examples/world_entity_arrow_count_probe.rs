use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_entity_metadata_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08EntityArrowCount".to_owned());
    let trace =
        connect_and_world_entity_metadata_probe(address, &username, Duration::from_secs(60))?;
    let Some(entity) = trace.world.entities().find(|entity| {
        let (x, _, z) = entity.position();
        entity.type_id == 70 && x == 55.5 && z == -35.5 && entity.arrow_count() > 0
    }) else {
        return Err("vanilla living entity arrow count was not applied to ClientWorld".into());
    };

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_metadata_packet_count={}",
        trace.entity_metadata_observations.len()
    );
    println!("entity_arrow_count={}", entity.arrow_count());
    println!("entity_type_id={}", entity.type_id);
    println!("entity_position={:?}", entity.position());
    println!(
        "entity_metadata_length={}",
        entity.metadata().map_or(0, <[u8]>::len)
    );
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
