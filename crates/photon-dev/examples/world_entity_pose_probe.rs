use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_entity_metadata_probe;
use photon_world::ClientEntityPose;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args().nth(2).unwrap_or_else(|| "P08Pose".to_owned());
    let trace =
        connect_and_world_entity_metadata_probe(address, &username, Duration::from_secs(60))?;
    let Some(entity) = trace
        .world
        .entities()
        .all()
        .find(|entity| entity.pose() != ClientEntityPose::Standing)
    else {
        return Err("vanilla pose entity was not applied to ClientWorld".into());
    };

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_metadata_packet_count={}",
        trace.entity_metadata_observations.len()
    );
    println!("pose={:?}", entity.pose());
    println!("pose_id={}", entity.pose().id());
    println!("entity_type_id={}", entity.type_id);
    println!("entity_position={:?}", entity.position());
    println!(
        "entity_metadata_length={}",
        entity.metadata().map_or(0, <[u8]>::len)
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
