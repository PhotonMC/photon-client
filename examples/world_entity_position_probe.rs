use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_entity_position_probe;

const CONTROLLED_POSITION: (f64, f64, f64) = (27.5, 81.0, -35.5);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "EntityPosition".to_owned());
    let trace =
        connect_and_world_entity_position_probe(address, &username, Duration::from_secs(60))?;
    let applied_sync = trace
        .entity_position_sync_observations
        .iter()
        .find(|observation| {
            observation.applied_to_world
                && observation.applied_position == Some(CONTROLLED_POSITION)
        });
    let applied_teleport = trace
        .entity_teleport_observations
        .iter()
        .find(|observation| {
            observation.applied_to_world
                && observation.applied_position == Some(CONTROLLED_POSITION)
        });
    let applied_move = trace.entity_move_observations.iter().find(|observation| {
        observation.applied_to_world && observation.applied_position == Some(CONTROLLED_POSITION)
    });
    let (entity_id, applied_position, applied_kind) = if let Some(observation) = applied_sync {
        (
            observation.entity_id,
            observation
                .applied_position
                .ok_or("missing applied position")?,
            "position_sync",
        )
    } else if let Some(observation) = applied_teleport {
        (
            observation.entity_id,
            observation
                .applied_position
                .ok_or("missing applied position")?,
            "teleport",
        )
    } else if let Some(observation) = applied_move {
        (
            observation.entity_id,
            observation
                .applied_position
                .ok_or("missing applied position")?,
            "move",
        )
    } else {
        return Err("vanilla controlled entity position was not applied to ClientWorld".into());
    };
    if applied_position != CONTROLLED_POSITION {
        return Err("controlled entity position application differs in ClientWorld".into());
    }
    let final_position = trace
        .world
        .entity_at(entity_id)
        .map(|entity| entity.position())
        .ok_or("applied entity position is not queryable in ClientWorld")?;

    let applied_sync_count = trace
        .entity_position_sync_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    let applied_teleport_count = trace
        .entity_teleport_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    let applied_move_count = trace
        .entity_move_observations
        .iter()
        .filter(|observation| observation.applied_to_world)
        .count();
    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_position_sync_packet_count={}",
        trace.entity_position_sync_observations.len()
    );
    println!("entity_position_sync_applied_count={applied_sync_count}");
    println!(
        "entity_teleport_packet_count={}",
        trace.entity_teleport_observations.len()
    );
    println!("entity_teleport_applied_count={applied_teleport_count}");
    println!(
        "entity_move_packet_count={}",
        trace.entity_move_observations.len()
    );
    println!("entity_move_applied_count={applied_move_count}");
    println!("last_controlled_entity_position_kind={applied_kind}");
    println!(
        "applied_entity_position=id:{},position:({},{},{})",
        entity_id, applied_position.0, applied_position.1, applied_position.2
    );
    println!(
        "world_entity_final_position=id:{},position:({},{},{})",
        entity_id, final_position.0, final_position.1, final_position.2
    );
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
