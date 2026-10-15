use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_entity_passenger_tick_probe;

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: i32 = 89;
const VANILLA_ZOMBIE_ENTITY_TYPE_ID: i32 = 151;
const VANILLA_BOAT_PASSENGER_MAX_ROTATION: f32 = 105.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08LivingTurn26".to_owned());
    let mut trace = connect_and_world_entity_passenger_tick_probe(
        address,
        &username,
        Duration::from_secs(120),
    )?;
    let observation = trace
        .entity_passenger_tick_observations
        .iter()
        .find(|observation| observation.passenger_type_id == VANILLA_ZOMBIE_ENTITY_TYPE_ID)
        .ok_or("vanilla did not produce a Zombie passenger tick observation")?;
    let vehicle = trace
        .world
        .entities()
        .at(observation.vehicle_id)
        .ok_or("world did not retain the boat vehicle")?;
    let vehicle_type_id = vehicle.type_id;
    let vehicle_y_rot = vehicle.rotation().0;
    let vehicle_id = observation.vehicle_id;
    let passenger_id = observation.passenger_id;
    if vehicle_type_id != VANILLA_OAK_BOAT_ENTITY_TYPE_ID {
        return Err(format!(
            "expected oak boat vehicle {}, got {}",
            VANILLA_OAK_BOAT_ENTITY_TYPE_ID, vehicle_type_id
        )
        .into());
    }

    let before = trace
        .world
        .entities()
        .at(passenger_id)
        .ok_or("world did not retain the Zombie passenger")?;
    let before_position = before.position();
    let before_rotation = before.rotation();
    let before_previous_rotation = before.previous_rotation();
    let before_body = before.body_y_rotation();
    if before_body != vehicle_y_rot {
        return Err(format!(
            "living passenger tick mismatch: rotation={before_rotation:?}, previous={before_previous_rotation:?}, body={before_body}, vehicle_yaw={vehicle_y_rot}"
        )
        .into());
    }

    assert!(trace.world.turn_entity(passenger_id, 20.0, 0.0));
    let after = trace
        .world
        .entities()
        .at(passenger_id)
        .ok_or("world lost the Zombie passenger after turn")?;
    if after.rotation()
        != (
            VANILLA_BOAT_PASSENGER_MAX_ROTATION,
            0.0,
            VANILLA_BOAT_PASSENGER_MAX_ROTATION,
        )
        || after.previous_rotation().1 != 0.0
        || after.body_y_rotation() != vehicle_y_rot
        || after.position() != before_position
    {
        return Err(format!(
            "living passenger turn mismatch: vehicle_yaw={vehicle_y_rot}, before_rotation={before_rotation:?}, before_previous={before_previous_rotation:?}, before_body={before_body}, after_rotation={:?}, after_previous={:?}, after_body={}, position={:?}",
            after.rotation(),
            after.previous_rotation(),
            after.body_y_rotation(),
            after.position(),
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("vehicle_id={vehicle_id}");
    println!("vehicle_type_id={vehicle_type_id}");
    println!("vehicle_y_rot={vehicle_y_rot}");
    println!("passenger_id={passenger_id}");
    println!("passenger_type_id={}", observation.passenger_type_id);
    println!("before_rotation={before_rotation:?}");
    println!("before_previous_rotation={before_previous_rotation:?}");
    println!("before_body_y_rotation={before_body}");
    println!("after_rotation={:?}", after.rotation());
    println!("after_previous_rotation={:?}", after.previous_rotation());
    println!("after_body_y_rotation={}", after.body_y_rotation());
    println!(
        "passenger_position={before_position:?}->{:?}",
        after.position()
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
