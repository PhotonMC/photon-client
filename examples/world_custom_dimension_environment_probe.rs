use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_world_dimension_type_bounds_probe;
use voxel_client::world::{CardinalLightType, DimensionSkybox};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "CustomDimensionEnvironmentProbe".to_owned());
    let trace =
        connect_and_world_dimension_type_bounds_probe(address, &username, Duration::from_secs(60))?;
    let Some(respawn) = trace.respawn_observations.last() else {
        return Err("vanilla did not provide a respawn packet during the probe".into());
    };
    let Some(registry_environment) = usize::try_from(respawn.spawn_info.dimension_type_id)
        .ok()
        .and_then(|id| trace.dimension_type_registry_environments.get(id))
        .and_then(Option::as_ref)
    else {
        return Err("custom dimension type had no serialized environment fields".into());
    };
    let environment = trace.world.dimension_environment();

    if !respawn.dimension_changed
        || respawn.spawn_info.dimension != "p08:test"
        || respawn.spawn_info.dimension_type_id != 2
        || environment != registry_environment
        || !environment.has_fixed_time()
        || environment.has_skylight()
        || !environment.has_ceiling()
        || !environment.has_ender_dragon_fight()
        || environment.coordinate_scale().to_bits() != 2.5_f64.to_bits()
        || environment.logical_height() != 128
        || environment.ambient_light().to_bits() != 0.35_f32.to_bits()
        || environment.skybox() != DimensionSkybox::End
        || environment.cardinal_light_type() != CardinalLightType::Nether
    {
        return Err(format!(
            "custom dimension environment differed from the vanilla baseline: respawn={respawn:?}, registry={registry_environment:?}, world={environment:?}"
        )
        .into());
    }

    println!("connection=ok");
    println!("respawn_dimension={}", respawn.spawn_info.dimension);
    println!(
        "respawn_dimension_type_id={}",
        respawn.spawn_info.dimension_type_id
    );
    println!(
        "dimension_environment_registry_entry_count={}",
        trace.dimension_type_registry_environments.len()
    );
    println!(
        "dimension_environment_resolved_count={}",
        trace
            .dimension_type_registry_environments
            .iter()
            .filter(|environment| environment.is_some())
            .count()
    );
    println!("has_fixed_time={}", environment.has_fixed_time());
    println!("has_skylight={}", environment.has_skylight());
    println!("has_ceiling={}", environment.has_ceiling());
    println!(
        "has_ender_dragon_fight={}",
        environment.has_ender_dragon_fight()
    );
    println!("coordinate_scale={}", environment.coordinate_scale());
    println!("logical_height={}", environment.logical_height());
    println!("ambient_light={}", environment.ambient_light());
    println!("skybox={}", environment.skybox().as_str());
    println!(
        "cardinal_light={}",
        environment.cardinal_light_type().as_str()
    );
    println!("world_dimension={}", trace.world.dimension_key());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
