use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_add_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustEntityAdd".to_owned());
    let trace = connect_and_world_entity_add_probe(address, &username, Duration::from_secs(60))?;
    let applied_count = trace
        .add_entity_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    if trace.add_entity_observations.is_empty()
        || applied_count == 0
        || trace.world.entities().count() == 0
    {
        return Err("vanilla entity add was not applied to ClientWorld".into());
    }

    let Some(first) = trace.add_entity_observations.iter().find(|observation| {
        observation.applied_to_world
            && trace
                .world
                .entities()
                .at(observation.entity_id)
                .is_some_and(|entity| entity.uuid == observation.uuid)
    }) else {
        return Err("vanilla entity UUID was not preserved in ClientWorld".into());
    };
    let entity = trace
        .world
        .entities()
        .at(first.entity_id)
        .expect("matched entity UUID implies queryable entity");

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "add_entity_packet_count={}",
        trace.add_entity_observations.len()
    );
    println!("add_entity_applied_count={applied_count}");
    println!(
        "first_entity=id:{},uuid_most:{},uuid_least:{},type_id:{},x:{},y:{},z:{},data:{},applied:{}",
        first.entity_id,
        first.uuid.most,
        first.uuid.least,
        first.entity_type_id,
        first.x,
        first.y,
        first.z,
        first.data,
        first.applied_to_world
    );
    println!(
        "world_entity={:?}",
        Some((entity.uuid, entity.type_id, entity.position(), entity.data))
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
