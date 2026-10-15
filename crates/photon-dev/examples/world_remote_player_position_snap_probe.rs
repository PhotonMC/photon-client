use std::env;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use photon_dev::{
    connect_and_remote_player_position_snap_mover_probe,
    connect_and_remote_player_position_snap_observer_probe,
};
use photon_net::connection::EntityPositionSnapObservation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let observer_username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08SnapObs".to_owned());
    let mover_username = env::args()
        .nth(3)
        .unwrap_or_else(|| "P08SnapMove".to_owned());

    let observer_address = address;
    let observer_username_for_thread = observer_username.clone();
    let observer = thread::spawn(move || {
        connect_and_remote_player_position_snap_observer_probe(
            observer_address,
            &observer_username_for_thread,
            Duration::from_secs(60),
        )
    });
    thread::sleep(Duration::from_millis(750));
    let mover_trace = connect_and_remote_player_position_snap_mover_probe(
        address,
        &mover_username,
        Duration::from_secs(60),
    )?;
    let trace = observer
        .join()
        .map_err(|_| "vanilla observer probe panicked")??;

    if mover_trace.movement_packets_sent < 1 {
        return Err("vanilla snap mover did not send its movement packet".into());
    }
    let observation = trace
        .entity_position_snap_observations
        .first()
        .ok_or("vanilla did not produce a large remote Player snap observation")?;
    validate_snap(observation)?;

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "mover_movement_packets_sent={}",
        mover_trace.movement_packets_sent
    );
    println!(
        "remote_player_position_snap_packet_kind={}",
        observation.packet_kind
    );
    println!("remote_player_id={}", observation.entity_id);
    println!(
        "remote_player_position_snap_before_position={:?}",
        observation.before_position
    );
    println!(
        "remote_player_position_snap_target_position={:?}",
        observation.target_position
    );
    println!(
        "remote_player_position_snap_before_rotation={:?}",
        observation.before_rotation
    );
    println!(
        "remote_player_position_snap_target_rotation={:?}",
        observation.target_rotation
    );
    println!(
        "remote_player_position_snap_applied_position={:?}",
        observation.applied_position
    );
    println!(
        "remote_player_position_snap_applied_rotation={:?}",
        observation.applied_rotation
    );
    println!(
        "remote_player_position_snap_previous_position={:?}",
        observation.previous_position
    );
    println!(
        "remote_player_position_snap_previous_rotation={:?}",
        observation.previous_rotation
    );
    println!(
        "remote_player_position_snap_interpolation_steps={}",
        observation.interpolation_steps
    );
    println!(
        "entity_position_sync_packet_count={}",
        trace.entity_position_sync_observations.len()
    );
    println!(
        "entity_teleport_packet_count={}",
        trace.entity_teleport_observations.len()
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}

fn validate_snap(
    observation: &EntityPositionSnapObservation,
) -> Result<(), Box<dyn std::error::Error>> {
    if observation.entity_type_id != 156 {
        return Err(format!(
            "snap observation used unexpected entity type {}",
            observation.entity_type_id
        )
        .into());
    }
    let delta_x = observation.target_position.0 - observation.before_position.0;
    let delta_y = observation.target_position.1 - observation.before_position.1;
    let delta_z = observation.target_position.2 - observation.before_position.2;
    if delta_x * delta_x + delta_y * delta_y + delta_z * delta_z <= 4096.0 {
        return Err("snap observation did not cross Java's interpolation threshold".into());
    }
    if observation.applied_position != observation.target_position
        || observation.previous_position != observation.target_position
    {
        return Err(format!(
            "large remote Player update did not snap immediately: target={:?}, applied={:?}, previous={:?}",
            observation.target_position,
            observation.applied_position,
            observation.previous_position
        )
        .into());
    }
    if observation.applied_rotation
        != (
            observation.target_rotation.0,
            observation.target_rotation.1,
            observation.applied_rotation.2,
        )
    {
        return Err("large remote Player body rotation was not applied with the snap".into());
    }
    if observation.interpolation_steps != 0 {
        return Err(format!(
            "large remote Player update left {} interpolation steps",
            observation.interpolation_steps
        )
        .into());
    }
    Ok(())
}
