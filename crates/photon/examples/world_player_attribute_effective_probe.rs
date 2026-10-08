use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_world_dimension_type_bounds_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "AttributeEffectiveProbe".to_owned());
    let trace =
        connect_and_world_dimension_type_bounds_probe(address, &username, Duration::from_secs(60))?;
    let Some(respawn) = trace.respawn_observations.last() else {
        return Err("vanilla did not provide a respawn packet during the probe".into());
    };
    let Some(player) = trace.world.local_player() else {
        return Err("Rust world did not create a local player state".into());
    };
    let Some(attribute) = player
        .attributes()
        .iter()
        .find(|attribute| attribute.attribute_id() == 26)
    else {
        return Err("vanilla did not provide movement_speed attribute id 26".into());
    };
    let expected_effective_bits = 0x3fe4_28f5_c28f_5c29;
    let modifier = |id: &str| {
        attribute
            .modifiers()
            .iter()
            .find(|modifier| modifier.id() == id)
    };

    if !respawn.dimension_changed
        || respawn.data_to_keep != 3
        || respawn.spawn_info.dimension != "p08:test"
        || attribute.base_value().to_bits() != 0x3fd0_0000_0000_0000
        || attribute.modifiers().len() != 3
        || modifier("p08:oracle_add")
            .map(|modifier| (modifier.amount().to_bits(), modifier.operation()))
            != Some((0x3fb9_9999_9999_999a, 0))
        || modifier("p08:oracle_base")
            .map(|modifier| (modifier.amount().to_bits(), modifier.operation()))
            != Some((0x3fc9_9999_9999_999a, 1))
        || modifier("p08:oracle_total")
            .map(|modifier| (modifier.amount().to_bits(), modifier.operation()))
            != Some((0x3fe0_0000_0000_0000, 2))
        || attribute.effective_value(0.0, 1024.0).to_bits() != expected_effective_bits
    {
        return Err(format!(
            "effective movement_speed state differed from the vanilla oracle: respawn={respawn:?}, attribute={attribute:?}, player={player:?}"
        )
        .into());
    }

    println!("connection=ok");
    println!("respawn_dimension={}", respawn.spawn_info.dimension);
    println!("respawn_data_to_keep={}", respawn.data_to_keep);
    println!("attribute_id={}", attribute.attribute_id());
    println!("attribute_base={}", attribute.base_value());
    println!("attribute_modifier_count={}", attribute.modifiers().len());
    println!(
        "attribute_effective={}",
        attribute.effective_value(0.0, 1024.0)
    );
    println!(
        "attribute_effective_bits={:016x}",
        attribute.effective_value(0.0, 1024.0).to_bits()
    );
    for modifier in attribute.modifiers() {
        println!(
            "modifier={} amount={} operation={}",
            modifier.id(),
            modifier.amount(),
            modifier.operation()
        );
    }
    println!("world_dimension={}", trace.world.dimension_key());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!(
        "local_player_attribute_update_count={}",
        trace
            .attribute_update_observations
            .iter()
            .filter(|observation| observation.entity_id == player.id())
            .count()
    );
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
