use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_entity_passenger_tick_probe;

const VANILLA_SNOWBALL_ENTITY_TYPE_ID: i32 = 120;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "PassTickObs26".to_owned());
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

    if observation.after_vehicle_tick_count - observation.before_vehicle_tick_count != 1 {
        return Err(format!(
            "vehicle tick count did not advance once: before={}, after={}",
            observation.before_vehicle_tick_count, observation.after_vehicle_tick_count
        )
        .into());
    }
    if observation.after_passenger_tick_count - observation.before_passenger_tick_count != 1 {
        return Err(format!(
            "passenger tick count did not advance once: before={}, after={}",
            observation.before_passenger_tick_count, observation.after_passenger_tick_count
        )
        .into());
    }
    if observation.before_passenger_motion.0 == 0.0 {
        return Err("snowball fixture did not carry horizontal pre-tick motion".into());
    }
    if observation.after_passenger_motion.0 != 0.0 || observation.after_passenger_motion.2 != 0.0 {
        return Err(format!(
            "passenger ride tick retained horizontal motion: before={:?}, after={:?}",
            observation.before_passenger_motion, observation.after_passenger_motion
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("vehicle_id={}", observation.vehicle_id);
    println!("passenger_id={}", observation.passenger_id);
    println!("passenger_type_id={}", observation.passenger_type_id);
    println!(
        "vehicle_tick_count={}->{}",
        observation.before_vehicle_tick_count, observation.after_vehicle_tick_count
    );
    println!(
        "passenger_tick_count={}->{}",
        observation.before_passenger_tick_count, observation.after_passenger_tick_count
    );
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
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
