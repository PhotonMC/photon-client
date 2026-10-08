use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustWorldLight".to_owned());
    let trace = connect_and_probe(address, &username, Duration::from_secs(20))?;

    let initial_light_update_count = trace
        .chunk_observations
        .iter()
        .map(|chunk| chunk.sky_update_count + chunk.block_update_count)
        .sum::<usize>();
    if initial_light_update_count == 0 || trace.world.loaded_light_section_count() == 0 {
        return Err("vanilla chunk light data was not applied to ClientWorld".into());
    }

    let min_y = trace.world.dimension().min_y;
    let max_y = min_y + trace.world.dimension().height - 1;
    let mut sky_sample = None;
    let mut first_block_sample = None;
    let mut nonzero_block_sample = None;
    for chunk in &trace.chunk_observations {
        for local_z in 0..16 {
            for local_x in 0..16 {
                let x = chunk.x * 16 + local_x;
                let z = chunk.z * 16 + local_z;
                for y in (min_y..=max_y).step_by(16) {
                    if sky_sample.is_none() {
                        sky_sample = trace
                            .world
                            .sky_light_at(x, y, z)
                            .map(|level| (x, y, z, level));
                    }
                    if let Some(level) = trace.world.block_light_at(x, y, z) {
                        if first_block_sample.is_none() {
                            first_block_sample = Some((x, y, z, level));
                        }
                        if level > 0 && nonzero_block_sample.is_none() {
                            nonzero_block_sample = Some((x, y, z, level));
                        }
                    }
                    if sky_sample.is_some() && nonzero_block_sample.is_some() {
                        break;
                    }
                }
                if sky_sample.is_some() && nonzero_block_sample.is_some() {
                    break;
                }
            }
            if sky_sample.is_some() && nonzero_block_sample.is_some() {
                break;
            }
        }
        if sky_sample.is_some() && nonzero_block_sample.is_some() {
            break;
        }
    }
    let Some(sky_sample) = sky_sample else {
        return Err("vanilla sky light data was not queryable in ClientWorld".into());
    };
    let block_sample = nonzero_block_sample.or(first_block_sample);

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("initial_light_update_count={initial_light_update_count}");
    println!(
        "light_update_packet_count={}",
        trace.light_update_observations.len()
    );
    println!(
        "light_update_applied_sections={}",
        trace
            .light_update_observations
            .iter()
            .map(|observation| observation.applied_sky_sections + observation.applied_block_sections)
            .sum::<usize>()
    );
    println!(
        "loaded_light_section_count={}",
        trace.world.loaded_light_section_count()
    );
    println!("sky_light_sample={sky_sample:?}");
    println!("block_light_sample={block_sample:?}");
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
