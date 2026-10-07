use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_position_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "EntityMotion".to_owned());
    let trace =
        connect_and_world_entity_position_probe(address, &username, Duration::from_secs(60))?;

    let initial_motion = trace.add_entity_observations.iter().find(|observation| {
        observation.movement != (0.0, 0.0, 0.0)
            && observation.applied_to_world
            && trace
                .world
                .entities()
                .at(observation.entity_id)
                .is_some_and(|entity| entity.motion() == observation.movement)
    });
    let applied_motion = trace.entity_motion_observations.iter().find(|observation| {
        observation.applied_to_world
            && observation.applied_motion == Some((observation.x, observation.y, observation.z))
    });
    if initial_motion.is_none() && applied_motion.is_none() {
        return Err("vanilla entity motion was not applied to ClientWorld".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "add_entity_packet_count={}",
        trace.add_entity_observations.len()
    );
    println!(
        "add_entity_initial_motion_count={}",
        trace
            .add_entity_observations
            .iter()
            .filter(|observation| observation.movement != (0.0, 0.0, 0.0))
            .count()
    );
    println!(
        "entity_motion_packet_count={}",
        trace.entity_motion_observations.len()
    );
    println!(
        "entity_motion_applied_count={}",
        trace
            .entity_motion_observations
            .iter()
            .filter(|observation| observation.applied_to_world)
            .count()
    );
    if let Some(observation) = initial_motion {
        println!(
            "initial_motion_entity=id:{},motion:{:?},world_motion:{:?}",
            observation.entity_id,
            observation.movement,
            trace
                .world
                .entities()
                .at(observation.entity_id)
                .map(|entity| entity.motion())
        );
    }
    if let Some(observation) = applied_motion {
        println!(
            "set_entity_motion=id:{},motion:({},{},{}),world_motion:{:?}",
            observation.entity_id,
            observation.x,
            observation.y,
            observation.z,
            observation.applied_motion
        );
    }
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
