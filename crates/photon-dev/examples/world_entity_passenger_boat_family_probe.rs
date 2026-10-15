use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_entity_passenger_tick_probe;

const VANILLA_SNOWBALL_ENTITY_TYPE_ID: i32 = 120;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "PassBoatFamily26".to_owned());
    let expected_vehicle_type_id: i32 = env::args()
        .nth(3)
        .unwrap_or_else(|| "90".to_owned())
        .parse()?;
    let expected_ride_height: f64 = env::args()
        .nth(4)
        .unwrap_or_else(|| "0.1875".to_owned())
        .parse()?;
    let expected_single_passenger_z_offset: f64 = env::args()
        .nth(5)
        .unwrap_or_else(|| f64::from(0.15_f32).to_string())
        .parse()?;

    let trace = connect_and_world_entity_passenger_tick_probe(
        address,
        &username,
        Duration::from_secs(120),
    )?;
    let observation = trace
        .entity_passenger_tick_observations
        .iter()
        .find(|observation| observation.passenger_type_id == VANILLA_SNOWBALL_ENTITY_TYPE_ID)
        .ok_or("vanilla did not produce a snowball passenger tick observation")?;
    let vehicle = trace
        .add_entity_observations
        .iter()
        .find(|entity| entity.entity_id == observation.vehicle_id)
        .ok_or("vanilla did not add the passenger vehicle")?;
    if vehicle.entity_type_id != expected_vehicle_type_id {
        return Err(format!(
            "passenger vehicle type changed: expected {}, got {}",
            expected_vehicle_type_id, vehicle.entity_type_id
        )
        .into());
    }

    let expected_position = (
        vehicle.x,
        vehicle.y + expected_ride_height,
        vehicle.z + expected_single_passenger_z_offset,
    );
    if observation.after_passenger_position != expected_position {
        return Err(format!(
            "boat family passenger seat differs: expected={expected_position:?}, got={:?}",
            observation.after_passenger_position
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
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
        "passenger_motion={:?}->{:?}",
        observation.before_passenger_motion, observation.after_passenger_motion
    );
    println!(
        "passenger_tick_observation_count={}",
        trace.entity_passenger_tick_observations.len()
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
