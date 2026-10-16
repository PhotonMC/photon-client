use std::env;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use photon_core::math::{mth_lerp_f64, mth_rot_lerp_f64};
use photon_dev::{
    connect_and_remote_player_interpolation_observer_probe,
    connect_and_remote_player_interpolation_probe,
};
use photon_net::connection::EntityInterpolationObservation;

fn expected_positions(observation: &EntityInterpolationObservation) -> Vec<(f64, f64, f64)> {
    let mut current = observation.before_position;
    let mut expected = Vec::with_capacity(3);
    for steps in [3.0_f64, 2.0, 1.0] {
        current = (
            mth_lerp_f64(1.0 / steps, current.0, observation.target_position.0),
            mth_lerp_f64(1.0 / steps, current.1, observation.target_position.1),
            mth_lerp_f64(1.0 / steps, current.2, observation.target_position.2),
        );
        expected.push(current);
    }
    expected
}

fn expected_rotations(observation: &EntityInterpolationObservation) -> Vec<(f32, f32)> {
    let mut current = observation.before_rotation;
    let mut expected = Vec::with_capacity(3);
    for steps in [3.0_f64, 2.0, 1.0] {
        let y_rot = mth_rot_lerp_f64(
            1.0 / steps,
            f64::from(current.0),
            f64::from(observation.target_rotation.0),
        ) as f32;
        let x_rot = mth_lerp_f64(
            1.0 / steps,
            f64::from(current.1),
            f64::from(observation.target_rotation.1),
        ) as f32;
        current = ((y_rot % 360.0), (x_rot % 360.0).clamp(-90.0, 90.0));
        expected.push(current);
    }
    expected
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let observer_username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08InterpObs".to_owned());
    let mover_username = env::args()
        .nth(3)
        .unwrap_or_else(|| "P08InterpMove".to_owned());

    let observer_address = address;
    let observer_username_for_thread = observer_username.clone();
    let observer = thread::spawn(move || {
        connect_and_remote_player_interpolation_observer_probe(
            observer_address,
            &observer_username_for_thread,
            Duration::from_secs(60),
        )
    });
    thread::sleep(Duration::from_millis(750));
    let mover_trace = connect_and_remote_player_interpolation_probe(
        address,
        &mover_username,
        Duration::from_secs(60),
    )?;
    let trace = observer
        .join()
        .map_err(|_| "vanilla observer probe panicked")??;

    if mover_trace.movement_packets_sent < 3 {
        return Err(format!(
            "vanilla mover sent too few movement packets: {}",
            mover_trace.movement_packets_sent
        )
        .into());
    }
    let observation = trace
        .entity_interpolation_observations
        .first()
        .ok_or("vanilla did not produce a remote Player interpolation observation")?;
    if observation.entity_type_id != photon_data::entity_type::named("minecraft:player") {
        return Err(format!(
            "interpolation observation used unexpected entity type {}",
            observation.entity_type_id
        )
        .into());
    }
    if observation.positions_after_ticks.len() != 3 || observation.rotations_after_ticks.len() != 3
    {
        return Err("remote Player interpolation did not record three ticks".into());
    }
    let expected_positions = expected_positions(observation);
    if observation.positions_after_ticks != expected_positions {
        return Err(format!(
            "remote Player positions differ from Java three-step interpolation: expected {expected_positions:?}, got {:?}",
            observation.positions_after_ticks
        )
        .into());
    }
    let expected_rotations = expected_rotations(observation);
    if observation.rotations_after_ticks != expected_rotations {
        return Err(format!(
            "remote Player rotations differ from Java three-step interpolation: expected {expected_rotations:?}, got {:?}",
            observation.rotations_after_ticks
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "mover_movement_packets_sent={}",
        mover_trace.movement_packets_sent
    );
    println!("remote_player_id={}", observation.entity_id);
    println!("remote_player_interpolation_packet_count=1");
    println!(
        "remote_player_interpolation_before_position={:?}",
        observation.before_position
    );
    println!(
        "remote_player_interpolation_target_position={:?}",
        observation.target_position
    );
    println!(
        "remote_player_interpolation_after_tick_positions={:?}",
        observation.positions_after_ticks
    );
    println!(
        "remote_player_interpolation_before_rotation={:?}",
        observation.before_rotation
    );
    println!(
        "remote_player_interpolation_target_rotation={:?}",
        observation.target_rotation
    );
    println!(
        "remote_player_interpolation_after_tick_rotations={:?}",
        observation.rotations_after_ticks
    );
    println!(
        "remote_player_interpolation_tick_count={}",
        observation.tick_count
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
