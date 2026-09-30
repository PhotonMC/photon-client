#![expect(
    clippy::too_many_lines,
    reason = "a probe scenario reads as one sequence of steps"
)]

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
        .unwrap_or_else(|| "EntityRotation".to_owned());
    let trace =
        connect_and_world_entity_position_probe(address, &username, Duration::from_secs(60))?;

    let initial_rotation = trace.add_entity_observations.iter().find(|observation| {
        (observation.x_rot != 0.0 || observation.y_rot != 0.0 || observation.y_head_rot != 0.0)
            && observation.applied_to_world
            && observation.applied_rotation
                == Some((observation.y_rot, observation.x_rot, observation.y_head_rot))
    });
    let move_rotation = trace.entity_move_observations.iter().find(|observation| {
        observation.has_rotation
            && observation.applied_to_world
            && observation
                .y_rot
                .zip(observation.x_rot)
                .is_some_and(|(y_rot, x_rot)| {
                    observation
                        .applied_rotation
                        .is_some_and(|rotation| (rotation.0, rotation.1) == (y_rot, x_rot))
                })
    });
    let head_rotation = trace
        .entity_head_rotation_observations
        .iter()
        .find(|observation| {
            observation.applied_to_world
                && observation.applied_head_rotation == Some(observation.y_head_rot)
        });
    if initial_rotation.is_none() && move_rotation.is_none() && head_rotation.is_none() {
        return Err("vanilla entity rotation was not applied to ClientWorld".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "add_entity_packet_count={}",
        trace.add_entity_observations.len()
    );
    println!(
        "add_entity_nonzero_rotation_count={}",
        trace
            .add_entity_observations
            .iter()
            .filter(|observation| {
                observation.x_rot != 0.0
                    || observation.y_rot != 0.0
                    || observation.y_head_rot != 0.0
            })
            .count()
    );
    println!(
        "entity_move_rotation_packet_count={}",
        trace
            .entity_move_observations
            .iter()
            .filter(|observation| observation.has_rotation)
            .count()
    );
    println!(
        "entity_move_rotation_applied_count={}",
        trace
            .entity_move_observations
            .iter()
            .filter(|observation| observation.has_rotation && observation.applied_to_world)
            .count()
    );
    println!(
        "entity_position_sync_packet_count={}",
        trace.entity_position_sync_observations.len()
    );
    println!(
        "entity_position_sync_rotation_applied_count={}",
        trace
            .entity_position_sync_observations
            .iter()
            .filter(|observation| observation.applied_rotation.is_some())
            .count()
    );
    println!(
        "entity_teleport_packet_count={}",
        trace.entity_teleport_observations.len()
    );
    println!(
        "entity_teleport_rotation_applied_count={}",
        trace
            .entity_teleport_observations
            .iter()
            .filter(|observation| observation.applied_rotation.is_some())
            .count()
    );
    println!(
        "entity_head_rotation_packet_count={}",
        trace.entity_head_rotation_observations.len()
    );
    println!(
        "entity_head_rotation_applied_count={}",
        trace
            .entity_head_rotation_observations
            .iter()
            .filter(|observation| observation.applied_to_world)
            .count()
    );
    if let Some(observation) = initial_rotation {
        println!(
            "initial_rotation_entity=id:{},rotation:(y:{},x:{},head:{}),world_rotation:{:?}",
            observation.entity_id,
            observation.y_rot,
            observation.x_rot,
            observation.y_head_rot,
            observation.applied_rotation
        );
    }
    if let Some(observation) = move_rotation {
        println!(
            "move_rotation_entity=id:{},rotation:(y:{},x:{}),world_rotation:{:?}",
            observation.entity_id,
            observation.y_rot.unwrap_or_default(),
            observation.x_rot.unwrap_or_default(),
            observation.applied_rotation
        );
    }
    if let Some(observation) = head_rotation {
        println!(
            "head_rotation_entity=id:{},head:{},world_head:{}",
            observation.entity_id,
            observation.y_head_rot,
            observation.applied_head_rotation.unwrap_or_default()
        );
    }
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
