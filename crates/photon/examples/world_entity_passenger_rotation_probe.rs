use std::env;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use photon_net::connection::{
    connect_and_remote_player_passenger_rotation_mover_probe,
    connect_and_world_entity_passenger_rotation_probe,
};

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: i32 = 89;
const VANILLA_PLAYER_ENTITY_TYPE_ID: i32 = 156;
const VANILLA_BOAT_PASSENGER_MAX_ROTATION: f32 = 105.0;

fn wrap_degrees(value: f32) -> f32 {
    let wrapped = value % 360.0;
    if wrapped >= 180.0 {
        wrapped - 360.0
    } else if wrapped < -180.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let observer_username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08BoatRotObs26".to_owned());
    let mover_username = env::args()
        .nth(3)
        .unwrap_or_else(|| "P08BoatRotMove26".to_owned());

    let observer_address = address;
    let observer_username_for_thread = observer_username.clone();
    let observer = thread::spawn(move || {
        connect_and_world_entity_passenger_rotation_probe(
            observer_address,
            &observer_username_for_thread,
            Duration::from_secs(120),
        )
    });
    thread::sleep(Duration::from_millis(750));
    let mover_trace = connect_and_remote_player_passenger_rotation_mover_probe(
        address,
        &mover_username,
        Duration::from_secs(60),
    )?;
    let trace = observer
        .join()
        .map_err(|_| "vanilla boat passenger rotation observer probe panicked")??;

    if mover_trace.movement_packets_sent < 1 {
        return Err(
            "vanilla boat passenger rotation mover did not send its movement packet".into(),
        );
    }
    let observation = trace
        .entity_passenger_tick_observations
        .iter()
        .find(|observation| observation.passenger_type_id == VANILLA_PLAYER_ENTITY_TYPE_ID)
        .ok_or("vanilla did not produce a remote Player passenger tick observation")?;
    let vehicle = trace
        .add_entity_observations
        .iter()
        .find(|entity| entity.entity_id == observation.vehicle_id)
        .ok_or("vanilla did not add the boat passenger vehicle")?;
    if vehicle.entity_type_id != VANILLA_OAK_BOAT_ENTITY_TYPE_ID {
        return Err(format!(
            "passenger vehicle type changed: expected oak boat {}, got {}",
            VANILLA_OAK_BOAT_ENTITY_TYPE_ID, vehicle.entity_type_id
        )
        .into());
    }
    let passenger_relative_yaw =
        wrap_degrees(observation.after_passenger_rotation.0 - vehicle.y_rot);
    if !(-VANILLA_BOAT_PASSENGER_MAX_ROTATION..=VANILLA_BOAT_PASSENGER_MAX_ROTATION)
        .contains(&passenger_relative_yaw)
    {
        return Err(format!(
            "remote Player rotation exceeded the boat clamp: vehicle_yaw={}, before={:?}, relative_yaw={}, got={:?}",
            vehicle.y_rot,
            observation.before_passenger_rotation,
            passenger_relative_yaw,
            observation.after_passenger_rotation,
        )
        .into());
    }
    if observation.after_passenger_rotation.2 != observation.after_passenger_rotation.0 {
        return Err(format!(
            "remote Player head yaw did not follow its clamped yaw: got={:?}",
            observation.after_passenger_rotation
        )
        .into());
    }
    if observation.after_passenger_body_y_rotation != vehicle.y_rot {
        return Err(format!(
            "remote Player body yaw did not follow the boat: expected={}, got={}",
            vehicle.y_rot, observation.after_passenger_body_y_rotation
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
    println!("vehicle_id={}", observation.vehicle_id);
    println!("vehicle_type_id={}", vehicle.entity_type_id);
    println!("vehicle_y_rot={}", vehicle.y_rot);
    println!("passenger_id={}", observation.passenger_id);
    println!("passenger_type_id={}", observation.passenger_type_id);
    println!(
        "passenger_rotation={:?}->{:?}",
        observation.before_passenger_rotation, observation.after_passenger_rotation
    );
    println!(
        "passenger_body_y_rotation={}->{}",
        observation.before_passenger_body_y_rotation, observation.after_passenger_body_y_rotation
    );
    println!(
        "passenger_previous_rotation_after_tick={:?}",
        trace
            .world
            .entities()
            .at(observation.passenger_id)
            .map(|entity| entity.previous_rotation())
    );
    println!(
        "passenger_observation_count={}",
        trace.entity_passenger_tick_observations.len()
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
