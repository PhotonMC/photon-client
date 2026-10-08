use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_world_entity_position_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "EntityGround".to_owned());
    let trace =
        connect_and_world_entity_position_probe(address, &username, Duration::from_secs(60))?;

    let mut packet_count = 0;
    let mut applied_count = 0;
    let mut true_count = 0;
    let mut false_count = 0;
    let mut first_true = None;
    let mut first_false = None;

    for (path, on_ground, applied) in trace
        .entity_position_sync_observations
        .iter()
        .map(|observation| {
            (
                "position_sync",
                observation.on_ground,
                observation.applied_on_ground,
            )
        })
        .chain(
            trace
                .entity_teleport_observations
                .iter()
                .map(|observation| {
                    (
                        "teleport",
                        observation.on_ground,
                        observation.applied_on_ground,
                    )
                }),
        )
        .chain(
            trace
                .entity_move_observations
                .iter()
                .map(|observation| ("move", observation.on_ground, observation.applied_on_ground)),
        )
    {
        packet_count += 1;
        if applied == Some(on_ground) {
            applied_count += 1;
            if on_ground {
                true_count += 1;
                first_true.get_or_insert(path);
            } else {
                false_count += 1;
                first_false.get_or_insert(path);
            }
        }
    }

    if packet_count == 0 || applied_count != packet_count || true_count == 0 || false_count == 0 {
        return Err(format!(
            "incomplete entity ground state: packets={packet_count}, applied={applied_count}, true={true_count}, false={false_count}"
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_position_sync_packet_count={}",
        trace.entity_position_sync_observations.len()
    );
    println!(
        "entity_teleport_packet_count={}",
        trace.entity_teleport_observations.len()
    );
    println!(
        "entity_move_packet_count={}",
        trace.entity_move_observations.len()
    );
    println!("entity_ground_packet_count={packet_count}");
    println!("entity_ground_applied_count={applied_count}");
    println!("entity_ground_true_count={true_count}");
    println!("entity_ground_false_count={false_count}");
    println!("first_ground_true_path={}", first_true.unwrap_or("none"));
    println!("first_ground_false_path={}", first_false.unwrap_or("none"));
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
