use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_snowball_tick_probe;

const SNOWBALL_ENTITY_TYPE_ID: i32 = 120;
const SNOWBALL_GRAVITY: f64 = 0.03;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "SnowballTick".to_owned());
    let trace = connect_and_world_snowball_tick_probe(address, &username, Duration::from_secs(10))?;

    let tick = trace
        .world_entity_tick_observations
        .first()
        .ok_or("vanilla snowball did not receive a client-world tick")?;
    if tick.entity_type_id != SNOWBALL_ENTITY_TYPE_ID || tick.tick_count != 1 {
        return Err("snowball tick observation has an unexpected entity or tick count".into());
    }
    let add = trace
        .add_entity_observations
        .iter()
        .find(|observation| observation.entity_id == tick.entity_id)
        .ok_or("snowball add-entity observation is missing")?;
    if !add.applied_to_world || add.entity_type_id != SNOWBALL_ENTITY_TYPE_ID {
        return Err("snowball add-entity was not applied to ClientWorld".into());
    }
    if tick.before_position != (add.x, add.y, add.z) || tick.before_motion != add.movement {
        return Err("snowball tick did not start from the add-entity state".into());
    }

    let air_drag = f64::from(0.99_f32);
    let expected_motion = (
        tick.before_motion.0 * air_drag,
        (tick.before_motion.1 - SNOWBALL_GRAVITY) * air_drag,
        tick.before_motion.2 * air_drag,
    );
    let expected_position = (
        tick.before_position.0 + expected_motion.0,
        tick.before_position.1 + expected_motion.1,
        tick.before_position.2 + expected_motion.2,
    );
    if tick.after_motion != expected_motion || tick.after_position != expected_position {
        let before_block = (
            tick.before_position.0.floor() as i32,
            tick.before_position.1.floor() as i32,
            tick.before_position.2.floor() as i32,
        );
        let expected_block = (
            expected_position.0.floor() as i32,
            expected_position.1.floor() as i32,
            expected_position.2.floor() as i32,
        );
        return Err(format!(
            "snowball free-flight tick differs from the Java update order: before_position={:?}, before_motion={:?}, after_position={:?}, expected_position={:?}, after_motion={:?}, expected_motion={:?}, before_block_state={:?}, expected_block_state={:?}",
            tick.before_position,
            tick.before_motion,
            tick.after_position,
            expected_position,
            tick.after_motion,
            expected_motion,
            trace.world.block_state_at(before_block.0, before_block.1, before_block.2),
            trace.world.block_state_at(expected_block.0, expected_block.1, expected_block.2)
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("snowball_add_entity_id={}", tick.entity_id);
    println!("snowball_add_position={:?}", tick.before_position);
    println!("snowball_add_motion={:?}", tick.before_motion);
    println!("snowball_after_tick_position={:?}", tick.after_position);
    println!("snowball_after_tick_motion={:?}", tick.after_motion);
    println!("snowball_before_tick_rotation={:?}", tick.before_rotation);
    println!("snowball_after_tick_rotation={:?}", tick.after_rotation);
    println!("snowball_tick_count={}", tick.tick_count);
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
