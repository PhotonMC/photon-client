use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_entity_passenger_boat_control_probe;
use voxel_client::world::BoatInput;

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: i32 = 89;
const VANILLA_ZOMBIE_ENTITY_TYPE_ID: i32 = 151;
const VANILLA_OAK_BOAT_PASSENGER_RIDE_HEIGHT: f64 = 0.1875;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "P08BoatControl26".to_owned());
    let mut trace = connect_and_world_entity_passenger_boat_control_probe(
        address,
        &username,
        Duration::from_secs(120),
    )?;

    let (vehicle_id, passenger_id) = trace
        .world
        .entities()
        .filter(|entity| entity.type_id == VANILLA_OAK_BOAT_ENTITY_TYPE_ID)
        .filter_map(|vehicle| {
            vehicle
                .passenger_ids()
                .iter()
                .copied()
                .find(|&candidate| {
                    trace
                        .world
                        .entity_at(candidate)
                        .is_some_and(|passenger| passenger.type_id == VANILLA_ZOMBIE_ENTITY_TYPE_ID)
                })
                .map(|passenger_id| (vehicle.id(), passenger_id))
        })
        .max_by_key(|(vehicle_id, _)| *vehicle_id)
        .ok_or("vanilla did not retain a Zombie passenger on an oak boat")?;

    let before_vehicle = trace
        .world
        .entity_at(vehicle_id)
        .ok_or("world lost the selected oak boat")?;
    let before_passenger = trace
        .world
        .entity_at(passenger_id)
        .ok_or("world lost the selected Zombie passenger")?;
    let before_vehicle_rotation = before_vehicle.rotation();
    let before_passenger_rotation = before_passenger.rotation();
    let before_passenger_position = before_passenger.position();
    if before_vehicle_rotation.0 != 0.0 || before_passenger_rotation.0 != 0.0 {
        return Err(format!(
            "control fixture was not at its pinned zero heading: vehicle={before_vehicle_rotation:?}, passenger={before_passenger_rotation:?}"
        )
        .into());
    }

    if !trace
        .world
        .set_boat_input(vehicle_id, BoatInput::new(false, true, false, false))
    {
        return Err("world rejected the selected oak boat input".into());
    }
    trace.world.tick();

    let after_vehicle = trace
        .world
        .entity_at(vehicle_id)
        .ok_or("world lost the oak boat after control")?;
    let after_passenger = trace
        .world
        .entity_at(passenger_id)
        .ok_or("world lost the Zombie passenger after control")?;
    let expected_passenger_position = (
        after_vehicle.position().0,
        after_vehicle.position().1 + VANILLA_OAK_BOAT_PASSENGER_RIDE_HEIGHT,
        after_vehicle.position().2,
    );
    if after_vehicle.boat_delta_rotation() != 1.0
        || after_vehicle.rotation().0 != 1.0
        || after_vehicle.paddle_state() != (true, false)
        || after_passenger.rotation() != (1.0, 0.0, 1.0)
        || after_passenger.previous_rotation() != (0.0, 0.0, 0.0)
        || after_passenger.body_y_rotation() != 1.0
        || after_passenger.position() != expected_passenger_position
    {
        return Err(format!(
            "boat control mismatch: before_vehicle={before_vehicle_rotation:?}, before_passenger={before_passenger_rotation:?}, after_vehicle_rotation={:?}, delta_rotation={}, paddles={:?}, after_passenger_rotation={:?}, after_previous={:?}, after_body={}, expected_position={expected_passenger_position:?}, position={:?}",
            after_vehicle.rotation(),
            after_vehicle.boat_delta_rotation(),
            after_vehicle.paddle_state(),
            after_passenger.rotation(),
            after_passenger.previous_rotation(),
            after_passenger.body_y_rotation(),
            after_passenger.position(),
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("vehicle_id={vehicle_id}");
    println!("vehicle_type_id={}", after_vehicle.type_id);
    println!("before_vehicle_rotation={before_vehicle_rotation:?}");
    println!("after_vehicle_rotation={:?}", after_vehicle.rotation());
    println!("delta_rotation={}", after_vehicle.boat_delta_rotation());
    println!("paddle_state={:?}", after_vehicle.paddle_state());
    println!("passenger_id={passenger_id}");
    println!("passenger_type_id={}", after_passenger.type_id);
    println!("before_passenger_rotation={before_passenger_rotation:?}");
    println!("after_passenger_rotation={:?}", after_passenger.rotation());
    println!(
        "after_passenger_previous_rotation={:?}",
        after_passenger.previous_rotation()
    );
    println!(
        "after_passenger_body_y_rotation={}",
        after_passenger.body_y_rotation()
    );
    println!(
        "passenger_position={before_passenger_position:?}->{:?}",
        after_passenger.position()
    );
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
