#![expect(
    clippy::too_many_lines,
    reason = "a probe scenario reads as one sequence of steps"
)]

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_world_entity_passenger_animal_rotation_probe;

const VANILLA_OAK_BOAT_ENTITY_TYPE_ID: i32 = 89;
const VANILLA_COW_ENTITY_TYPE_ID: i32 = 30;
const VANILLA_SNOWBALL_ENTITY_TYPE_ID: i32 = 120;
const VANILLA_BOAT_PASSENGER_MAX_ROTATION: f32 = 105.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08AnimalRotObs26".to_owned());
    let trace = connect_and_world_entity_passenger_animal_rotation_probe(
        address,
        &username,
        Duration::from_secs(120),
    )?;
    let observation = trace
        .entity_passenger_tick_observations
        .iter()
        .find(|observation| observation.passenger_type_id == VANILLA_COW_ENTITY_TYPE_ID)
        .ok_or("vanilla did not produce a Cow passenger tick observation")?;
    let vehicle = trace
        .add_entity_observations
        .iter()
        .find(|entity| entity.entity_id == observation.vehicle_id)
        .ok_or("vanilla did not add the animal passenger vehicle")?;
    if vehicle.entity_type_id != VANILLA_OAK_BOAT_ENTITY_TYPE_ID {
        return Err(format!(
            "passenger vehicle type changed: expected oak boat {}, got {}",
            VANILLA_OAK_BOAT_ENTITY_TYPE_ID, vehicle.entity_type_id
        )
        .into());
    }
    let passenger_relation = trace
        .entity_passengers_observations
        .iter()
        .find(|relation| {
            relation.vehicle_id == observation.vehicle_id && relation.passenger_ids.len() == 2
        })
        .ok_or("vanilla did not produce the full two-passenger relation")?;
    if passenger_relation
        .passenger_ids
        .iter()
        .filter_map(|&id| trace.world.entities().at(id))
        .any(|entity| {
            entity.type_id != VANILLA_COW_ENTITY_TYPE_ID
                && entity.type_id != VANILLA_SNOWBALL_ENTITY_TYPE_ID
        })
    {
        return Err("vanilla two-passenger relation contained an unexpected entity".into());
    }
    if observation.after_passenger_rotation.0 != VANILLA_BOAT_PASSENGER_MAX_ROTATION {
        return Err(format!(
            "Cow yaw was not clamped before the animal offset: before={:?}, after={:?}",
            observation.before_passenger_rotation, observation.after_passenger_rotation
        )
        .into());
    }
    let expected_offset = if observation.passenger_id % 2 == 0 {
        90.0
    } else {
        270.0
    };
    let expected_head = VANILLA_BOAT_PASSENGER_MAX_ROTATION + expected_offset;
    if observation.after_passenger_rotation.2 != expected_head {
        return Err(format!(
            "Cow head yaw did not include the Java parity offset: id={}, expected={}, got={:?}",
            observation.passenger_id, expected_head, observation.after_passenger_rotation
        )
        .into());
    }
    if observation.after_passenger_body_y_rotation != expected_offset {
        return Err(format!(
            "Cow body yaw did not include the Java parity offset: id={}, expected={}, got={}",
            observation.passenger_id, expected_offset, observation.after_passenger_body_y_rotation
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("vehicle_id={}", observation.vehicle_id);
    println!("vehicle_type_id={}", vehicle.entity_type_id);
    println!("vehicle_y_rot={}", vehicle.y_rot);
    println!("passenger_ids={:?}", passenger_relation.passenger_ids);
    println!("passenger_id={}", observation.passenger_id);
    println!("passenger_type_id={}", observation.passenger_type_id);
    println!(
        "passenger_rotation={:?}->{:?}",
        observation.before_passenger_rotation, observation.after_passenger_rotation
    );
    println!(
        "passenger_body_y_rotation={}->{}",
        observation.before_passenger_body_y_rotation, observation.after_passenger_body_y_rotation
    );
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
