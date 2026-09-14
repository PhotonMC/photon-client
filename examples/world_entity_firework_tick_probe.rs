use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_firework_tick_probe;
use voxel_client::math::{mth_atan2, mth_rot_lerp_f32};

const FIREWORK_ENTITY_TYPE_ID: i32 = 53;
const FIREWORK_HORIZONTAL_ACCELERATION: f64 = 1.15;
const FIREWORK_VERTICAL_ACCELERATION: f64 = 0.04;
const PI_F32: f32 = f32::from_bits(0x4049_0fdb);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "P08Firework".to_owned());
    let trace = connect_and_world_firework_tick_probe(address, &username, Duration::from_secs(60))?;

    let tick = trace
        .world_entity_tick_observations
        .first()
        .ok_or("vanilla Firework Rocket did not receive a client-world tick")?;
    if tick.entity_type_id != FIREWORK_ENTITY_TYPE_ID || tick.tick_count != 1 {
        return Err(
            "Firework Rocket tick observation has an unexpected entity or tick count".into(),
        );
    }
    let add = trace
        .add_entity_observations
        .iter()
        .find(|observation| observation.entity_id == tick.entity_id)
        .ok_or("Firework Rocket add-entity observation is missing")?;
    if !add.applied_to_world || add.entity_type_id != FIREWORK_ENTITY_TYPE_ID {
        return Err("Firework Rocket add-entity was not applied to ClientWorld".into());
    }
    if tick.before_position != (add.x, add.y, add.z)
        || tick.before_motion != add.movement
        || tick.before_rotation != (add.y_rot, add.x_rot, add.y_head_rot)
    {
        return Err("Firework Rocket tick did not start from the add-entity state".into());
    }

    let movement = (
        tick.before_motion.0 * FIREWORK_HORIZONTAL_ACCELERATION,
        tick.before_motion.1 + FIREWORK_VERTICAL_ACCELERATION,
        tick.before_motion.2 * FIREWORK_HORIZONTAL_ACCELERATION,
    );
    let expected_position = (
        tick.before_position.0 + movement.0,
        tick.before_position.1 + movement.1,
        tick.before_position.2 + movement.2,
    );
    let horizontal_distance = (movement.0 * movement.0 + movement.2 * movement.2).sqrt();
    let target_x_rotation = (mth_atan2(movement.1, horizontal_distance) * f64::from(180.0_f32)
        / f64::from(PI_F32)) as f32;
    let target_y_rotation =
        (mth_atan2(movement.0, movement.2) * f64::from(180.0_f32) / f64::from(PI_F32)) as f32;
    let expected_rotation = (
        mth_rot_lerp_f32(0.2_f32, tick.before_rotation.0, target_y_rotation),
        mth_rot_lerp_f32(0.2_f32, tick.before_rotation.1, target_x_rotation),
        tick.before_rotation.2,
    );
    if tick.after_position != expected_position
        || tick.after_motion != movement
        || tick.after_rotation != expected_rotation
    {
        return Err(format!(
            "Firework Rocket free-flight tick mismatch: before_position={:?}, after_position={:?}, expected_position={expected_position:?}, before_motion={:?}, after_motion={:?}, expected_motion={movement:?}, before_rotation={:?}, after_rotation={:?}, expected_rotation={expected_rotation:?}",
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
    println!("firework_add_entity_id={}", tick.entity_id);
    println!("firework_add_position={:?}", tick.before_position);
    println!("firework_add_motion={:?}", tick.before_motion);
    println!("firework_after_tick_position={:?}", tick.after_position);
    println!("firework_after_tick_motion={:?}", tick.after_motion);
    println!("firework_before_tick_rotation={:?}", tick.before_rotation);
    println!("firework_after_tick_rotation={:?}", tick.after_rotation);
    println!("firework_tick_count={}", tick.tick_count);
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
