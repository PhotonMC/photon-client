use photon_core::ids::EntityTypeId;
use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_experience_orb_tick_probe;

const EXPERIENCE_ORB_ENTITY_TYPE_ID: EntityTypeId = EntityTypeId(49);
const EXPERIENCE_ORB_GRAVITY: f64 = 0.03;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = args
        .next()
        .unwrap_or_else(|| "P08ExperienceOrbTick".to_owned());
    let trace =
        connect_and_world_experience_orb_tick_probe(address, &username, Duration::from_secs(60))?;

    let tick = trace
        .experience_orb_tick_observations
        .first()
        .ok_or("vanilla Experience Orb did not receive a client-world tick")?;
    if tick.entity_type_id != EXPERIENCE_ORB_ENTITY_TYPE_ID || tick.tick_count != 2 {
        return Err(
            "Experience Orb tick observation has an unexpected entity or tick count".into(),
        );
    }
    let add = trace
        .add_entity_observations
        .iter()
        .find(|observation| observation.entity_id == tick.entity_id)
        .ok_or("Experience Orb add-entity observation is missing")?;
    if !add.applied_to_world || add.entity_type_id != EXPERIENCE_ORB_ENTITY_TYPE_ID {
        return Err("Experience Orb add-entity was not applied to ClientWorld".into());
    }
    if tick.before_position != (add.x, add.y, add.z)
        || tick.before_motion != add.movement
        || tick.after_first_tick_position != tick.before_position
        || tick.after_first_tick_motion != tick.before_motion
    {
        return Err("Experience Orb first client tick changed its initial state".into());
    }

    let gravity_motion = (
        tick.before_motion.0,
        tick.before_motion.1 - EXPERIENCE_ORB_GRAVITY,
        tick.before_motion.2,
    );
    let air_friction = f64::from(0.98_f32);
    let expected_position = (
        tick.before_position.0 + gravity_motion.0,
        tick.before_position.1 + gravity_motion.1,
        tick.before_position.2 + gravity_motion.2,
    );
    let expected_motion = (
        gravity_motion.0 * air_friction,
        gravity_motion.1 * air_friction,
        gravity_motion.2 * air_friction,
    );
    if tick.after_second_tick_position != expected_position
        || tick.after_second_tick_motion != expected_motion
    {
        return Err(format!(
            "Experience Orb tick mismatch: before_position={:?}, first_position={:?}, after_position={:?}, expected_position={expected_position:?}, before_motion={:?}, first_motion={:?}, after_motion={:?}, expected_motion={expected_motion:?}",
            tick.before_position,
            tick.after_first_tick_position,
            tick.after_second_tick_position,
            tick.before_motion,
            tick.after_first_tick_motion,
            tick.after_second_tick_motion,
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("experience_orb_add_entity_id={}", tick.entity_id);
    println!("experience_orb_add_position={:?}", tick.before_position);
    println!("experience_orb_add_motion={:?}", tick.before_motion);
    println!(
        "experience_orb_after_first_tick_position={:?}",
        tick.after_first_tick_position
    );
    println!(
        "experience_orb_after_first_tick_motion={:?}",
        tick.after_first_tick_motion
    );
    println!(
        "experience_orb_after_second_tick_position={:?}",
        tick.after_second_tick_position
    );
    println!(
        "experience_orb_after_second_tick_motion={:?}",
        tick.after_second_tick_motion
    );
    println!("experience_orb_tick_count={}", tick.tick_count);
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
