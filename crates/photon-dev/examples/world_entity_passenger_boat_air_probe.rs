use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_core::math::{MTH_DEG_TO_RAD, mth_cos_f64, mth_sin_f64};
use photon_dev::connect_and_world_entity_passengers_probe;
use photon_world::BoatInput;

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: i32 = 89;
const VANILLA_BOAT_AIR_FRICTION: f32 = 0.9_f32;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "P08BoatAir85".to_owned());
    let mut trace =
        connect_and_world_entity_passengers_probe(address, &username, Duration::from_secs(120))?;
    let local_player_id = trace
        .world
        .local_player()
        .map(|player| player.id())
        .ok_or("world did not create the local player state")?;

    let vehicle_id = trace
        .world
        .entities()
        .all()
        .filter(|entity| entity.type_id == VANILLA_OAK_BOAT_ENTITY_TYPE_ID)
        .filter(|entity| entity.passenger_ids().contains(&local_player_id))
        .map(|entity| entity.id())
        .max()
        .ok_or("vanilla did not retain the local player as an oak boat passenger")?;
    let before_vehicle = trace
        .world
        .entities()
        .at(vehicle_id)
        .ok_or("world lost the selected oak boat")?;
    if !before_vehicle.is_no_gravity() {
        return Err("selected oak boat did not retain NoGravity metadata".into());
    }
    let before_position = before_vehicle.position();
    let before_motion = before_vehicle.motion();
    let before_yaw = before_vehicle.rotation().0;
    let before_delta_rotation = before_vehicle.boat_delta_rotation();
    let floated_motion = (
        before_motion.0 * f64::from(VANILLA_BOAT_AIR_FRICTION),
        before_motion.1,
        before_motion.2 * f64::from(VANILLA_BOAT_AIR_FRICTION),
    );
    let expected_delta_rotation = before_delta_rotation * 0.9_f32;
    let expected_yaw = before_yaw + expected_delta_rotation;
    let acceleration = 0.04_f32;
    let x_acceleration = mth_sin_f64(f64::from(-expected_yaw * MTH_DEG_TO_RAD)) * acceleration;
    let z_acceleration = mth_cos_f64(f64::from(expected_yaw * MTH_DEG_TO_RAD)) * acceleration;
    let expected_motion = (
        floated_motion.0 + f64::from(x_acceleration),
        floated_motion.1,
        floated_motion.2 + f64::from(z_acceleration),
    );
    let expected_position = (
        before_position.0 + expected_motion.0,
        before_position.1 + expected_motion.1,
        before_position.2 + expected_motion.2,
    );
    if !trace
        .world
        .set_boat_input(vehicle_id, BoatInput::new(false, false, true, false))
    {
        return Err("world rejected forward input for the local oak boat".into());
    }

    trace.world.tick();

    let after_vehicle = trace
        .world
        .entities()
        .at(vehicle_id)
        .ok_or("world lost the oak boat after tick")?;
    if after_vehicle.motion().0.to_bits() != expected_motion.0.to_bits()
        || after_vehicle.motion().1.to_bits() != expected_motion.1.to_bits()
        || after_vehicle.motion().2.to_bits() != expected_motion.2.to_bits()
        || after_vehicle.position().0.to_bits() != expected_position.0.to_bits()
        || after_vehicle.position().1.to_bits() != expected_position.1.to_bits()
        || after_vehicle.position().2.to_bits() != expected_position.2.to_bits()
        || after_vehicle.boat_delta_rotation().to_bits() != expected_delta_rotation.to_bits()
        || after_vehicle.paddle_state() != (true, true)
    {
        return Err(format!(
            "air boat tick mismatch: before_position={before_position:?}, before_motion={before_motion:?}, expected_position={expected_position:?}, expected_motion={expected_motion:?}, got_position={:?}, got_motion={:?}, expected_delta_rotation={expected_delta_rotation}, got_delta_rotation={}",
            after_vehicle.position(),
            after_vehicle.motion(),
            after_vehicle.boat_delta_rotation(),
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("local_player_id={local_player_id}");
    println!("vehicle_id={vehicle_id}");
    println!("vehicle_type_id={}", after_vehicle.type_id);
    println!("no_gravity={}", after_vehicle.is_no_gravity());
    println!("before_position={before_position:?}");
    println!("after_position={:?}", after_vehicle.position());
    println!("before_motion={before_motion:?}");
    println!("after_motion={:?}", after_vehicle.motion());
    println!("before_delta_rotation={before_delta_rotation}");
    println!(
        "after_delta_rotation={}",
        after_vehicle.boat_delta_rotation()
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
