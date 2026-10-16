use photon_core::ids::EntityTypeId;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_entity_passengers_probe;

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:oak_boat");
const VANILLA_ZOMBIE_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:zombie");
const VANILLA_OAK_BOAT_RIDE_HEIGHT: f64 = 0.1875;
const VANILLA_ZOMBIE_BABY_VEHICLE_ATTACHMENT_Y: f64 = 0.1875;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "P08BabyAttach26".to_owned());
    let mut trace =
        connect_and_world_entity_passengers_probe(address, &username, Duration::from_secs(120))?;

    let (vehicle_id, passenger_id) = trace
        .world
        .entities()
        .all()
        .filter(|entity| entity.type_id == VANILLA_OAK_BOAT_ENTITY_TYPE_ID)
        .filter_map(|vehicle| {
            vehicle
                .passenger_ids()
                .iter()
                .copied()
                .find(|&candidate| {
                    trace
                        .world
                        .entities()
                        .at(candidate)
                        .is_some_and(|passenger| {
                            passenger.type_id == VANILLA_ZOMBIE_ENTITY_TYPE_ID
                                && passenger.is_baby()
                        })
                })
                .map(|passenger_id| (vehicle.id(), passenger_id))
        })
        .max_by_key(|(vehicle_id, _)| *vehicle_id)
        .ok_or("vanilla did not retain a baby Zombie passenger on an oak boat")?;

    trace
        .world
        .entities()
        .at(vehicle_id)
        .ok_or("world lost the selected oak boat")?;
    let before_passenger = trace
        .world
        .entities()
        .at(passenger_id)
        .ok_or("world lost the selected baby Zombie")?;
    if !before_passenger.is_baby() {
        return Err("selected Zombie did not retain baby metadata".into());
    }
    let before_position = before_passenger.position();
    trace.world.tick();

    let after_vehicle = trace
        .world
        .entities()
        .at(vehicle_id)
        .ok_or("world lost the oak boat after tick")?;
    let after_passenger = trace
        .world
        .entities()
        .at(passenger_id)
        .ok_or("world lost the baby Zombie after tick")?;
    let expected_position = (
        after_vehicle.position().0,
        after_vehicle.position().1 + VANILLA_OAK_BOAT_RIDE_HEIGHT
            - VANILLA_ZOMBIE_BABY_VEHICLE_ATTACHMENT_Y,
        after_vehicle.position().2,
    );
    if after_passenger.position() != expected_position {
        return Err(format!(
            "baby Zombie vehicle attachment mismatch: expected={expected_position:?}, before={before_position:?}, after={:?}",
            after_passenger.position()
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("vehicle_id={vehicle_id}");
    println!("vehicle_type_id={}", after_vehicle.type_id);
    println!("vehicle_position={:?}", after_vehicle.position());
    println!("passenger_id={passenger_id}");
    println!("passenger_type_id={}", after_passenger.type_id);
    println!("passenger_is_baby={}", after_passenger.is_baby());
    println!(
        "passenger_position={before_position:?}->{:?}",
        after_passenger.position()
    );
    println!("expected_position={expected_position:?}");
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
