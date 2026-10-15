use photon_core::ids::EntityTypeId;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_entity_passenger_tick_probe;

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: EntityTypeId = EntityTypeId(89);
const VANILLA_SNOWBALL_ENTITY_TYPE_ID: EntityTypeId = EntityTypeId(120);
const VANILLA_OAK_BOAT_PASSENGER_RIDE_HEIGHT: f64 = 0.1875;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "PassMultiObs26".to_owned());
    let trace = connect_and_world_entity_passenger_tick_probe(
        address,
        &username,
        Duration::from_secs(120),
    )?;
    let relation = trace
        .entity_passengers_observations
        .iter()
        .find(|observation| {
            observation.passenger_ids.len() == 2
                && observation.passenger_ids.iter().all(|&entity_id| {
                    trace
                        .world
                        .entities()
                        .at(entity_id)
                        .is_some_and(|entity| entity.type_id == VANILLA_SNOWBALL_ENTITY_TYPE_ID)
                })
        })
        .ok_or_else(|| {
            let relations = trace
                .entity_passengers_observations
                .iter()
                .map(|observation| {
                    (
                        observation.vehicle_id,
                        observation.passenger_ids.clone(),
                        observation.applied_to_world,
                    )
                })
                .collect::<Vec<_>>();
            format!("vanilla did not produce a two-snowball passenger relation: {relations:?}")
        })?;
    let vehicle = trace
        .add_entity_observations
        .iter()
        .find(|entity| entity.entity_id == relation.vehicle_id)
        .ok_or("vanilla did not add the passenger vehicle")?;
    if vehicle.entity_type_id != VANILLA_OAK_BOAT_ENTITY_TYPE_ID {
        return Err(format!(
            "passenger vehicle type changed: expected oak boat {}, got {}",
            VANILLA_OAK_BOAT_ENTITY_TYPE_ID, vehicle.entity_type_id
        )
        .into());
    }

    let first = trace
        .world
        .entities()
        .at(relation.passenger_ids[0])
        .ok_or("first snowball passenger missing")?;
    let second = trace
        .world
        .entities()
        .at(relation.passenger_ids[1])
        .ok_or("second snowball passenger missing")?;
    let first_expected = (
        vehicle.x,
        vehicle.y + VANILLA_OAK_BOAT_PASSENGER_RIDE_HEIGHT,
        vehicle.z + f64::from(0.2_f32),
    );
    let second_expected = (
        vehicle.x,
        vehicle.y + VANILLA_OAK_BOAT_PASSENGER_RIDE_HEIGHT,
        vehicle.z + f64::from(-0.6_f32),
    );
    if first.position() != first_expected || second.position() != second_expected {
        return Err(format!(
            "snowball passenger seats differ: expected={first_expected:?}/{second_expected:?}, got={:?}/{:?}",
            first.position(), second.position()
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("vehicle_id={}", relation.vehicle_id);
    println!("vehicle_type_id={}", vehicle.entity_type_id);
    println!(
        "vehicle_position=({}, {}, {})",
        vehicle.x, vehicle.y, vehicle.z
    );
    println!("passenger_ids={:?}", relation.passenger_ids);
    println!("first_passenger_position={:?}", first.position());
    println!("second_passenger_position={:?}", second.position());
    println!(
        "passenger_observation_count={}",
        trace.entity_passengers_observations.len()
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
