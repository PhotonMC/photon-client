use std::env;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use voxel_net::connection::{
    connect_and_remote_player_motion_mover_probe, connect_and_world_entity_passenger_tick_probe,
};

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: i32 = 89;
const VANILLA_PLAYER_ENTITY_TYPE_ID: i32 = 156;
const VANILLA_OAK_BOAT_PASSENGER_RIDE_HEIGHT: f64 = 0.1875;
const VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y: f64 = 0.6;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let observer_username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08PlayerPosObs".to_owned());
    let mover_username = env::args()
        .nth(3)
        .unwrap_or_else(|| "P08PlayerPosMove".to_owned());

    let observer_address = address;
    let observer_username_for_thread = observer_username.clone();
    let observer = thread::spawn(move || {
        connect_and_world_entity_passenger_tick_probe(
            observer_address,
            &observer_username_for_thread,
            Duration::from_secs(120),
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
        .map_err(|_| "vanilla player passenger observer probe panicked")??;

    if mover_trace.movement_packets_sent < 1 {
        return Err("vanilla player passenger mover did not send its movement packet".into());
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
        .ok_or("vanilla did not add the Player passenger vehicle")?;
    if vehicle.entity_type_id != VANILLA_OAK_BOAT_ENTITY_TYPE_ID {
        return Err(format!(
            "passenger vehicle type changed: expected oak boat {}, got {}",
            VANILLA_OAK_BOAT_ENTITY_TYPE_ID, vehicle.entity_type_id
        )
        .into());
    }
    let expected_position = (
        vehicle.x,
        vehicle.y + VANILLA_OAK_BOAT_PASSENGER_RIDE_HEIGHT - VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y,
        vehicle.z,
    );
    if observation.after_passenger_position != expected_position {
        return Err(format!(
            "remote Player was not positioned from its vehicle attachment: expected={expected_position:?}, got={:?}",
            observation.after_passenger_position
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
    println!(
        "vehicle_position=({}, {}, {})",
        vehicle.x, vehicle.y, vehicle.z
    );
    println!("passenger_id={}", observation.passenger_id);
    println!("passenger_type_id={}", observation.passenger_type_id);
    println!(
        "passenger_position={:?}->{:?}",
        observation.before_passenger_position, observation.after_passenger_position
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
