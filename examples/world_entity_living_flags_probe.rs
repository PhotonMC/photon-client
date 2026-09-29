use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_metadata_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08EntityLivingFlags".to_owned());
    let trace =
        connect_and_world_entity_metadata_probe(address, &username, Duration::from_secs(60))?;
    let Some(entity) = trace.world.entities().find(|entity| {
        let (x, _, z) = entity.position();
        entity.type_id == 70 && x == 43.5 && z == -35.5
    }) else {
        return Err("vanilla living entity flags were not applied to ClientWorld".into());
    };
    if entity.living_flags() != 0 {
        return Err(format!(
            "unexpected vanilla default living flags: {}",
            entity.living_flags()
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_metadata_packet_count={}",
        trace.entity_metadata_observations.len()
    );
    println!("entity_living_flags={}", entity.living_flags());
    println!("is_using_item={}", entity.is_using_item());
    println!("used_item_is_off_hand={}", entity.used_item_is_off_hand());
    println!("is_auto_spin_attack={}", entity.is_auto_spin_attack());
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
