use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_teleport_delta_probe;

const RELATIVE_DELTA_MASK: i32 = (1 << 5) | (1 << 6) | (1 << 7);
const ROTATE_DELTA_FLAG: i32 = 1 << 8;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "TeleportDelta".to_owned());
    let trace = connect_and_world_entity_teleport_delta_probe(
        address,
        &username,
        Duration::from_secs(120),
    )?;
    let observation = trace
        .entity_teleport_observations
        .iter()
        .find(|observation| {
            observation.applied_to_world
                && observation.relative_flags & RELATIVE_DELTA_MASK == RELATIVE_DELTA_MASK
        })
        .ok_or("vanilla did not produce an applied relative entity teleport packet")?;
    let motion_before = observation
        .motion_before
        .ok_or("relative entity teleport did not retain the pre-teleport motion")?;
    let applied_motion = observation
        .applied_motion
        .ok_or("relative entity teleport did not expose applied motion")?;
    if observation.relative_flags & ROTATE_DELTA_FLAG != 0 {
        return Err("live relative teleport unexpectedly included ROTATE_DELTA".into());
    }
    if observation.delta_x != 0.0 || observation.delta_y != 0.0 || observation.delta_z != 0.0 {
        return Err("live relative teleport fixture changed its delta payload".into());
    }
    if applied_motion != motion_before {
        return Err(format!(
            "relative zero-delta teleport changed motion: before={motion_before:?}, applied={applied_motion:?}"
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("entity_id={}", observation.entity_id);
    println!(
        "teleport_position_payload={:?}",
        (observation.x, observation.y, observation.z)
    );
    println!(
        "teleport_delta_payload={:?}",
        (
            observation.delta_x,
            observation.delta_y,
            observation.delta_z
        )
    );
    println!("teleport_relative_flags={}", observation.relative_flags);
    println!("motion_before={motion_before:?}");
    println!("applied_motion={applied_motion:?}");
    println!("applied_position={:?}", observation.applied_position);
    println!("applied_rotation={:?}", observation.applied_rotation);
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
