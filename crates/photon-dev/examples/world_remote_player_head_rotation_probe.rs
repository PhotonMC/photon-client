use photon_core::ids::EntityTypeId;
use std::env;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use photon_dev::{
    connect_and_remote_player_head_rotation_observer_probe,
    connect_and_remote_player_head_rotation_probe,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let observer_username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08HeadObs".to_owned());
    let mover_username = env::args()
        .nth(3)
        .unwrap_or_else(|| "P08HeadMove".to_owned());

    let observer_address = address;
    let observer_username_for_thread = observer_username.clone();
    let observer = thread::spawn(move || {
        connect_and_remote_player_head_rotation_observer_probe(
            observer_address,
            &observer_username_for_thread,
            Duration::from_secs(60),
        )
    });
    thread::sleep(Duration::from_millis(750));
    let mover_trace = connect_and_remote_player_head_rotation_probe(
        address,
        &mover_username,
        Duration::from_secs(60),
    )?;
    let trace = observer
        .join()
        .map_err(|_| "vanilla observer probe panicked")??;

    let Some(observer_player) = trace.world.local_player() else {
        return Err("vanilla did not create the observer local player in ClientWorld".into());
    };
    let observer_id = observer_player.id();
    let Some(remote_id) = trace
        .add_entity_observations
        .iter()
        .find(|observation| {
            observation.entity_type_id == EntityTypeId(156)
                && observation.entity_id != observer_id
                && observation.applied_to_world
        })
        .map(|observation| observation.entity_id)
    else {
        return Err("vanilla did not add the mover as a remote player entity".into());
    };
    let remote_head_packets: Vec<_> = trace
        .entity_head_rotation_observations
        .iter()
        .filter(|observation| observation.entity_id == remote_id && observation.applied_to_world)
        .collect();
    if remote_head_packets.len() < 2 {
        return Err(format!(
            "vanilla sent too few applied remote player head packets: {}",
            remote_head_packets.len()
        )
        .into());
    }
    let distinct_head_targets = remote_head_packets
        .windows(2)
        .filter(|window| window[0].y_head_rot != window[1].y_head_rot)
        .count();
    if distinct_head_targets == 0 {
        return Err("vanilla remote player head packets did not change target rotation".into());
    }
    let head_targets: Vec<_> = remote_head_packets
        .iter()
        .map(|observation| observation.y_head_rot)
        .collect();
    let Some(remote_before_ticks) = trace.world.entities().at(remote_id) else {
        return Err("remote player disappeared before ClientWorld tick test".into());
    };
    let head_before_ticks = remote_before_ticks.rotation().2;
    let expected_head_rotation = remote_head_packets
        .last()
        .expect("remote head packet count checked")
        .y_head_rot;

    let mut world = trace.world;
    world.tick();
    world.tick();
    world.tick();
    let Some(remote_after_ticks) = world.entities().at(remote_id) else {
        return Err("remote player disappeared while ticking ClientWorld".into());
    };
    let head_after_three_ticks = remote_after_ticks.rotation().2;
    if (head_after_three_ticks - expected_head_rotation).abs() > 0.0001 {
        return Err(format!(
            "remote player head rotation did not converge: expected {expected_head_rotation}, got {head_after_three_ticks}"
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "mover_movement_packets_sent={}",
        mover_trace.movement_packets_sent
    );
    println!("observer_local_player_id={observer_id}");
    println!("remote_player_id={remote_id}");
    println!(
        "remote_player_head_rotation_packet_count={}",
        remote_head_packets.len()
    );
    println!("remote_player_head_rotation_targets={head_targets:?}");
    println!("remote_player_head_rotation_before_ticks={head_before_ticks}");
    println!("remote_player_head_rotation_target={expected_head_rotation}");
    println!("remote_player_head_rotation_after_three_ticks={head_after_three_ticks}");
    println!("world_entity_count={}", world.entities().count());
    println!("world_chunk_count={}", world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
