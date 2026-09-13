use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_dimension_respawn_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "PlayerRespawn".to_owned());
    let trace =
        connect_and_world_dimension_respawn_probe(address, &username, Duration::from_secs(60))?;
    let Some(login) = trace.login_observation.as_ref() else {
        return Err("vanilla did not provide a login packet during the probe".into());
    };
    let Some(respawn) = trace.respawn_observations.last() else {
        return Err("vanilla did not provide a respawn packet during the probe".into());
    };
    let Some(player) = trace.world.local_player() else {
        return Err("Rust world did not create a local player state".into());
    };

    if !respawn.local_player_recreated
        || respawn.local_player_id != Some(login.player_id)
        || player.id() != login.player_id
        || player.generation() == 0
    {
        return Err(format!(
            "local player replacement did not preserve Java identity semantics: {respawn:?}, player={player:?}"
        )
        .into());
    }

    println!("connection=ok");
    println!("login_player_id={}", login.player_id);
    println!("respawn_packet_count={}", trace.respawn_observations.len());
    println!("respawn_dimension={}", respawn.spawn_info.dimension);
    println!("respawn_data_to_keep={}", respawn.data_to_keep);
    println!("keep_entity_data={}", respawn.keep_entity_data);
    println!(
        "keep_attribute_modifiers={}",
        respawn.keep_attribute_modifiers
    );
    println!("local_player_id={}", player.id());
    println!("local_player_recreated={}", respawn.local_player_recreated);
    println!("local_player_generation={}", player.generation());
    println!("local_player_position={:?}", player.position());
    println!("local_player_motion={:?}", player.motion());
    println!("local_player_rotation={:?}", player.rotation());
    println!(
        "local_player_entity_data_present={}",
        player.entity_data().is_some()
    );
    println!(
        "player_position_observation_count={}",
        trace.player_position_observations.len()
    );
    println!(
        "player_position_all_applied_to_world={}",
        trace
            .player_position_observations
            .iter()
            .all(|observation| observation.applied_to_world)
    );
    println!("dimension_changed={}", respawn.dimension_changed);
    println!("cleared_chunk_count={}", respawn.cleared_chunk_count);
    println!("cleared_entity_count={}", respawn.cleared_entity_count);
    println!("world_dimension={}", trace.world.dimension_key());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_entity_count={}", trace.world.loaded_entity_count());
    Ok(())
}
