use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_net::connection::connect_and_world_falling_block_tick_probe;

const FALLING_BLOCK_ENTITY_TYPE_ID: i32 = 51;
const FALLING_BLOCK_GRAVITY: f64 = 0.04;
const FALLING_BLOCK_AIR_FRICTION: f64 = 0.98_f32 as f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "P08FallTick".to_owned());
    let trace =
        connect_and_world_falling_block_tick_probe(address, &username, Duration::from_secs(60))?;

    let tick = trace
        .world_entity_tick_observations
        .first()
        .ok_or("vanilla FallingBlock did not receive a client-world tick")?;
    if tick.entity_type_id != FALLING_BLOCK_ENTITY_TYPE_ID || tick.tick_count != 1 {
        return Err("FallingBlock tick observation has an unexpected entity or tick count".into());
    }
    let add = trace
        .add_entity_observations
        .iter()
        .find(|observation| observation.entity_id == tick.entity_id)
        .ok_or("FallingBlock add-entity observation is missing")?;
    if !add.applied_to_world || add.entity_type_id != FALLING_BLOCK_ENTITY_TYPE_ID {
        return Err("FallingBlock add-entity was not applied to ClientWorld".into());
    }
    if tick.before_position != (add.x, add.y, add.z)
        || tick.before_motion != add.movement
        || tick.before_rotation != (add.y_rot, add.x_rot, add.y_head_rot)
    {
        return Err("FallingBlock tick did not start from the add-entity state".into());
    }

    let gravity_motion = (
        tick.before_motion.0,
        tick.before_motion.1 - FALLING_BLOCK_GRAVITY,
        tick.before_motion.2,
    );
    let expected_position = (
        tick.before_position.0 + gravity_motion.0,
        tick.before_position.1 + gravity_motion.1,
        tick.before_position.2 + gravity_motion.2,
    );
    let expected_motion = (
        gravity_motion.0 * FALLING_BLOCK_AIR_FRICTION,
        gravity_motion.1 * FALLING_BLOCK_AIR_FRICTION,
        gravity_motion.2 * FALLING_BLOCK_AIR_FRICTION,
    );
    if tick.after_position != expected_position
        || tick.after_motion != expected_motion
        || tick.after_rotation != tick.before_rotation
    {
        return Err(format!(
            "FallingBlock free-flight tick mismatch: before_position={:?}, after_position={:?}, expected_position={expected_position:?}, before_motion={:?}, after_motion={:?}, expected_motion={expected_motion:?}, before_rotation={:?}, after_rotation={:?}",
            tick.before_position,
            tick.after_position,
            tick.before_motion,
            tick.after_motion,
            tick.before_rotation,
            tick.after_rotation,
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("falling_block_add_entity_id={}", tick.entity_id);
    println!("falling_block_add_position={:?}", tick.before_position);
    println!("falling_block_add_motion={:?}", tick.before_motion);
    println!(
        "falling_block_after_tick_position={:?}",
        tick.after_position
    );
    println!("falling_block_after_tick_motion={:?}", tick.after_motion);
    println!(
        "falling_block_before_tick_rotation={:?}",
        tick.before_rotation
    );
    println!(
        "falling_block_after_tick_rotation={:?}",
        tick.after_rotation
    );
    println!("falling_block_tick_count={}", tick.tick_count);
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
