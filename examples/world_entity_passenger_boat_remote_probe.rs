use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_passengers_probe;
use voxel_world::BoatInput;

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: i32 = 89;
const VANILLA_ZOMBIE_ENTITY_TYPE_ID: i32 = 151;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "P08BoatRemote26".to_owned());
    let mut trace =
        connect_and_world_entity_passengers_probe(address, &username, Duration::from_secs(120))?;
    let local_player_id = trace
        .world
        .local_player()
        .map(|player| player.id())
        .ok_or("vanilla did not create the local player state")?;
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
                    candidate != local_player_id
                        && trace.world.entity_at(candidate).is_some_and(|passenger| {
                            passenger.type_id == VANILLA_ZOMBIE_ENTITY_TYPE_ID
                        })
                })
                .map(|passenger_id| (vehicle.id(), passenger_id))
        })
        .max_by_key(|(vehicle_id, _)| *vehicle_id)
        .ok_or("vanilla did not retain a remote Zombie passenger on an oak boat")?;
    let before_vehicle = trace
        .world
        .entity_at(vehicle_id)
        .ok_or("world lost the selected remote oak boat")?;
    if before_vehicle.passenger_ids().contains(&local_player_id) {
        return Err("selected boat unexpectedly has the local player as passenger".into());
    }
    let before_position = before_vehicle.position();
    let before_rotation = before_vehicle.rotation();
    let before_delta_rotation = before_vehicle.boat_delta_rotation();
    let before_paddles = before_vehicle.paddle_state();
    let before_motion = before_vehicle.motion();

    if !trace
        .world
        .set_boat_input(vehicle_id, BoatInput::new(false, true, false, false))
    {
        return Err("world rejected input for the remote oak boat".into());
    }
    trace.world.tick();

    let after_vehicle = trace
        .world
        .entity_at(vehicle_id)
        .ok_or("world lost the remote oak boat after tick")?;
    if after_vehicle.position() != before_position
        || after_vehicle.rotation() != before_rotation
        || after_vehicle.boat_delta_rotation() != before_delta_rotation
        || after_vehicle.paddle_state() != before_paddles
        || after_vehicle.motion() != (0.0, 0.0, 0.0)
    {
        return Err(format!(
            "remote boat tick mismatch: before_position={before_position:?}, after_position={:?}, before_motion={before_motion:?}, after_motion={:?}, before_rotation={before_rotation:?}, after_rotation={:?}, before_delta_rotation={before_delta_rotation}, after_delta_rotation={}, before_paddles={before_paddles:?}, after_paddles={:?}",
            after_vehicle.position(),
            after_vehicle.motion(),
            after_vehicle.rotation(),
            after_vehicle.boat_delta_rotation(),
            after_vehicle.paddle_state(),
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("local_player_id={local_player_id}");
    println!("vehicle_id={vehicle_id}");
    println!("vehicle_type_id={}", after_vehicle.type_id);
    println!("passenger_id={passenger_id}");
    println!("passenger_type_id={VANILLA_ZOMBIE_ENTITY_TYPE_ID}");
    println!("before_position={before_position:?}");
    println!("after_position={:?}", after_vehicle.position());
    println!("before_motion={before_motion:?}");
    println!("after_motion={:?}", after_vehicle.motion());
    println!("before_rotation={before_rotation:?}");
    println!("after_rotation={:?}", after_vehicle.rotation());
    println!("before_delta_rotation={before_delta_rotation}");
    println!(
        "after_delta_rotation={}",
        after_vehicle.boat_delta_rotation()
    );
    println!("before_paddles={before_paddles:?}");
    println!("after_paddles={:?}", after_vehicle.paddle_state());
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
