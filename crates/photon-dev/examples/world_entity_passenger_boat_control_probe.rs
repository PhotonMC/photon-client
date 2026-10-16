use photon_core::ids::EntityTypeId;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_entity_passenger_boat_control_probe;
use photon_world::BoatInput;

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:oak_boat");

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
    let local_player_id = trace
        .world
        .local_player()
        .map(|player| player.id())
        .ok_or("vanilla did not create the local player state")?;
    let vehicle_id = trace
        .world
        .entities()
        .all()
        .filter(|entity| entity.type_id == VANILLA_OAK_BOAT_ENTITY_TYPE_ID)
        .filter(|vehicle| vehicle.passenger_ids().contains(&local_player_id))
        .map(|vehicle| vehicle.id())
        .max()
        .ok_or("vanilla did not retain the local player on an oak boat")?;

    let before_vehicle = trace
        .world
        .entities()
        .at(vehicle_id)
        .ok_or("world lost the selected oak boat")?;
    let before_vehicle_rotation = before_vehicle.rotation();
    if before_vehicle_rotation.0 != 0.0 || before_vehicle_rotation.1 != 0.0 {
        return Err(format!(
            "control fixture was not at its pinned zero heading: vehicle={before_vehicle_rotation:?}"
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
        .entities()
        .at(vehicle_id)
        .ok_or("world lost the oak boat after control")?;
    if after_vehicle.boat_delta_rotation() != 1.0
        || after_vehicle.rotation().0 != 1.0
        || after_vehicle.paddle_state() != (true, false)
    {
        return Err(format!(
            "boat control mismatch: local_player_id={local_player_id}, before_vehicle={before_vehicle_rotation:?}, after_vehicle_rotation={:?}, delta_rotation={}, paddles={:?}",
            after_vehicle.rotation(),
            after_vehicle.boat_delta_rotation(),
            after_vehicle.paddle_state(),
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
    println!("local_player_id={local_player_id}");
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
