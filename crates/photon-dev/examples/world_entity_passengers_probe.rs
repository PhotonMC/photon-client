use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_entity_passengers_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "PassengerObserver".to_owned());
    let trace =
        connect_and_world_entity_passengers_probe(address, &username, Duration::from_secs(120))?;

    let (mounted_index, mounted) = trace
        .entity_passengers_observations
        .iter()
        .enumerate()
        .find(|(_, observation)| {
            observation.applied_to_world && !observation.passenger_ids.is_empty()
        })
        .ok_or("vanilla did not produce an applied non-empty passenger-list packet")?;
    let applied_vehicle_passengers = mounted
        .applied_vehicle_passengers
        .as_deref()
        .ok_or("passenger-list vehicle was not present in the client world")?;
    if applied_vehicle_passengers != mounted.passenger_ids.as_slice() {
        return Err(format!(
            "applied passenger order differs: packet={:?}, world={applied_vehicle_passengers:?}",
            mounted.passenger_ids
        )
        .into());
    }
    for &(passenger_id, vehicle_id) in &mounted.applied_passenger_vehicles {
        if vehicle_id != Some(mounted.vehicle_id) {
            return Err(format!(
                "passenger {passenger_id} has vehicle {vehicle_id:?}, expected {}",
                mounted.vehicle_id
            )
            .into());
        }
    }

    let dismounted = trace
        .entity_passengers_observations
        .iter()
        .skip(mounted_index + 1)
        .find(|observation| {
            observation.applied_to_world
                && observation.vehicle_id == mounted.vehicle_id
                && observation.passenger_ids.is_empty()
        })
        .ok_or("vanilla did not produce the empty passenger-list replacement")?;
    if dismounted.applied_vehicle_passengers.as_deref() != Some(&[][..]) {
        return Err("empty passenger-list packet left stale passengers in the world".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("vehicle_id={}", mounted.vehicle_id);
    println!("mounted_passengers={:?}", mounted.passenger_ids);
    println!("mounted_world_passengers={applied_vehicle_passengers:?}");
    println!("dismounted_passengers={:?}", dismounted.passenger_ids);
    println!(
        "dismounted_world_passengers={:?}",
        dismounted.applied_vehicle_passengers
    );
    println!(
        "passenger_packet_count={}",
        trace.entity_passengers_observations.len()
    );
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
