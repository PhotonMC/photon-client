use std::env;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use photon_dev::{
    connect_and_remote_player_motion_mover_probe, connect_and_remote_player_motion_observer_probe,
};
use photon_net::connection::EntityMotionInterpolationObservation;

fn expected_motions(observation: &EntityMotionInterpolationObservation) -> Vec<(f64, f64, f64)> {
    let mut current = observation.applied_motion;
    let target = observation.target_motion;
    let mut expected = Vec::with_capacity(3);
    for steps in [3.0_f64, 2.0, 1.0] {
        current = (
            current.0 + (target.0 - current.0) / steps,
            current.1 + (target.1 - current.1) / steps,
            current.2 + (target.2 - current.2) / steps,
        );
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
        .unwrap_or_else(|| "P08MotionObs".to_owned());
    let mover_username = env::args()
        .nth(3)
        .unwrap_or_else(|| "P08MotionMove".to_owned());

    let observer_address = address;
    let observer_username_for_thread = observer_username.clone();
    let observer = thread::spawn(move || {
        connect_and_remote_player_motion_observer_probe(
            observer_address,
            &observer_username_for_thread,
            Duration::from_secs(60),
        )
    });
    thread::sleep(Duration::from_millis(750));
    let mover_trace = connect_and_remote_player_motion_mover_probe(
        address,
        &mover_username,
        Duration::from_secs(60),
    )?;
    let trace = observer
        .join()
        .map_err(|_| "vanilla motion observer probe panicked")??;

    if mover_trace.movement_packets_sent < 1 {
        return Err("vanilla motion mover did not send its movement packet".into());
    }
    let observation = trace
        .entity_motion_interpolation_observations
        .first()
        .ok_or("vanilla did not produce a remote Player motion observation")?;
    if observation.entity_type_id != photon_core::ids::EntityTypeId(156) {
        return Err(format!(
            "motion observation used unexpected entity type {}",
            observation.entity_type_id
        )
        .into());
    }
    if observation.applied_motion != observation.before_motion {
        return Err(format!(
            "remote Player motion changed immediately: before={:?}, applied={:?}",
            observation.before_motion, observation.applied_motion
        )
        .into());
    }
    if observation.motions_after_ticks.len() != 3 {
        return Err("remote Player motion observation did not record three ticks".into());
    }
    let expected = expected_motions(observation);
    if observation.motions_after_ticks != expected {
        return Err(format!(
            "remote Player motion differs from Java three-step interpolation: expected {expected:?}, got {:?}",
            observation.motions_after_ticks
        )
        .into());
    }
    if observation.motions_after_ticks.last().copied() != Some(observation.target_motion) {
        return Err("remote Player motion did not reach its target after three ticks".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "mover_movement_packets_sent={}",
        mover_trace.movement_packets_sent
    );
    println!("remote_player_id={}", observation.entity_id);
    println!(
        "remote_player_motion_before={:?}",
        observation.before_motion
    );
    println!(
        "remote_player_motion_target={:?}",
        observation.target_motion
    );
    println!(
        "remote_player_motion_applied_immediately={:?}",
        observation.applied_motion
    );
    println!(
        "remote_player_motion_after_tick_motions={:?}",
        observation.motions_after_ticks
    );
    println!("remote_player_motion_tick_count={}", observation.tick_count);
    println!(
        "entity_motion_packet_count={}",
        trace.entity_motion_observations.len()
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
