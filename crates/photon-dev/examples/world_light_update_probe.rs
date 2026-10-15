use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_light_update_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustWorldLightUpdate".to_owned());
    let trace = connect_and_world_light_update_probe(address, &username, Duration::from_secs(60))?;

    let applied_sections = trace
        .light_update_observations
        .iter()
        .map(|observation| observation.applied_sky_sections + observation.applied_block_sections)
        .sum::<usize>();
    if trace.light_update_observations.is_empty() || applied_sections == 0 {
        return Err("vanilla standalone light update was not applied to ClientWorld".into());
    }
    let target_block_light = trace.world.block_light_at(25, 80, -36);
    if target_block_light.is_none_or(|level| level == 0) {
        return Err("vanilla glowstone light was not queryable after standalone update".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "light_update_packet_count={}",
        trace.light_update_observations.len()
    );
    println!("light_update_applied_sections={applied_sections}");
    println!("target_block_light={target_block_light:?}");
    println!(
        "first_light_update={:?}",
        trace.light_update_observations.first().map(|observation| {
            (
                observation.x,
                observation.z,
                observation.applied_sky_sections,
                observation.applied_block_sections,
                observation.sky_update_lengths.clone(),
                observation.block_update_lengths.clone(),
            )
        })
    );
    println!(
        "loaded_light_section_count={}",
        trace.world.loaded_light_section_count()
    );
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
