use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_dimension_type_bounds_probe;
use photon_world::DimensionBounds;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "CustomDimensionProbe".to_owned());
    let trace =
        connect_and_world_dimension_type_bounds_probe(address, &username, Duration::from_secs(60))?;
    let Some(respawn) = trace.respawn_observations.last() else {
        return Err("vanilla did not provide a respawn packet during the probe".into());
    };
    let Some(registry_bounds) = usize::try_from(respawn.spawn_info.dimension_type_id)
        .ok()
        .and_then(|id| trace.dimension_type_registry_bounds.get(id))
        .and_then(|bounds| *bounds)
    else {
        return Err("custom dimension type had no serialized registry bounds".into());
    };
    let expected_bounds = DimensionBounds::new(-32, 192);

    if trace.known_pack_count != 1
        || trace.selected_known_pack_count != 0
        || trace.dimension_type_registry_bounds.len() != 5
        || trace
            .dimension_type_registry_bounds
            .iter()
            .filter(|bounds| bounds.is_some())
            .count()
            != 5
        || !respawn.dimension_changed
        || !respawn.applied_to_world
        || respawn.spawn_info.dimension != "p08:test"
        || respawn.spawn_info.dimension_type_id != 2
        || registry_bounds != expected_bounds
        || trace.world.dimension_key() != "p08:test"
        || trace.world.dimension() != expected_bounds
        || trace.world.loaded_chunk_count() != 157
        || trace.world.loaded_section_count() != 1884
    {
        return Err(format!(
            "custom dimension state differed from the vanilla baseline: respawn={respawn:?}, registry_bounds={registry_bounds:?}, world_dimension={}, world_bounds={:?}, registry_entries={}, resolved_entries={}, chunks={}, sections={}",
            trace.world.dimension_key(),
            trace.world.dimension(),
            trace.dimension_type_registry_bounds.len(),
            trace
                .dimension_type_registry_bounds
                .iter()
                .filter(|bounds| bounds.is_some())
                .count(),
            trace.world.loaded_chunk_count(),
            trace.world.loaded_section_count(),
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("known_pack_count={}", trace.known_pack_count);
    println!(
        "selected_known_pack_count={}",
        trace.selected_known_pack_count
    );
    println!(
        "dimension_type_registry_entry_count={}",
        trace.dimension_type_registry_bounds.len()
    );
    println!(
        "dimension_type_registry_resolved_count={}",
        trace
            .dimension_type_registry_bounds
            .iter()
            .filter(|bounds| bounds.is_some())
            .count()
    );
    println!(
        "login_dimension={}",
        trace
            .login_observation
            .as_ref()
            .map(|login| login.spawn_info.dimension.as_str())
            .unwrap_or("unknown")
    );
    println!("respawn_packet_count={}", trace.respawn_observations.len());
    println!("respawn_dimension={}", respawn.spawn_info.dimension);
    println!(
        "respawn_dimension_type_id={}",
        respawn.spawn_info.dimension_type_id
    );
    println!(
        "registry_bounds=min_y:{},height:{}",
        registry_bounds.min_y, registry_bounds.height
    );
    println!("respawn_data_to_keep={}", respawn.data_to_keep);
    println!("dimension_changed={}", respawn.dimension_changed);
    println!("cleared_chunk_count={}", respawn.cleared_chunk_count);
    println!("cleared_entity_count={}", respawn.cleared_entity_count);
    println!("world_dimension={}", trace.world.dimension_key());
    println!(
        "world_bounds=min_y:{},height:{}",
        trace.world.dimension().min_y,
        trace.world.dimension().height
    );
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("world_entity_count={}", trace.world.entities().count());
    println!("chunk_packet_count={}", trace.chunk_packet_count);
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
