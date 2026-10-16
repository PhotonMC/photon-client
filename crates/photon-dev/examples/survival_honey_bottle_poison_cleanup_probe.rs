use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_survival_honey_bottle_poison_cleanup_probe;

fn is_poison(registry_id: i32) -> bool {
    photon_data::item::effect::effect_id("minecraft:poison") == Some(registry_id)
}

const HONEY_BOTTLE: photon_core::ids::ItemId =
    photon_data::item::registry::named("minecraft:honey_bottle");
const GLASS_BOTTLE: photon_core::ids::ItemId =
    photon_data::item::registry::named("minecraft:glass_bottle");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustHoneyPoison1".to_owned());
    let trace = connect_and_survival_honey_bottle_poison_cleanup_probe(
        address,
        &username,
        Duration::from_secs(30),
    )?;
    let Some(login) = trace.login_observation.as_ref() else {
        return Err("vanilla did not provide a login packet during the probe".into());
    };
    let Some(player) = trace.world.local_player() else {
        return Err("Rust world did not create a local player state".into());
    };
    let Some(local_update) = trace
        .mob_effect_update_observations
        .iter()
        .find(|observation| {
            observation.entity_id == login.player_id
                && is_poison(observation.effect_registry_id)
                && observation.applied_to_world
        })
    else {
        return Err("vanilla did not provide an applied local poison update".into());
    };
    let Some(local_remove) = trace
        .mob_effect_remove_observations
        .iter()
        .find(|observation| {
            observation.entity_id == login.player_id
                && is_poison(observation.effect_registry_id)
                && observation.applied_to_world
        })
    else {
        return Err("vanilla did not provide an applied local poison removal".into());
    };
    let Some(initial_content) = trace
        .container_content_observations
        .iter()
        .find(|observation| observation.container_id == 0 && observation.applied_to_world)
    else {
        return Err("vanilla did not provide applied player inventory content".into());
    };
    let Some(preparation_slot) = trace
        .container_slot_observations
        .iter()
        .find(|observation| {
            observation.container_id == 0
                && observation.slot == 36
                && observation.state_id == initial_content.state_id + 1
                && observation.item_registry_id == Some(HONEY_BOTTLE)
                && observation.applied_to_world
        })
    else {
        return Err("vanilla did not provide an applied honey bottle inventory slot".into());
    };
    let Some(remainder_slot) = trace
        .container_slot_observations
        .iter()
        .find(|observation| {
            observation.container_id == 0
                && observation.slot == 36
                && observation.state_id == initial_content.state_id + 2
                && observation.item_registry_id == Some(GLASS_BOTTLE)
                && observation.applied_to_world
        })
    else {
        return Err("vanilla did not provide an applied glass bottle inventory slot".into());
    };
    if local_update.amplifier != 0
        || local_update.duration_ticks != 1_200
        || local_update.flags != 0b1000
        || !local_remove.applied_to_world
        || !player.effects().is_empty()
        || initial_content.item_count != 46
        || initial_content.items.len() != 46
        || initial_content.items.iter().any(Option::is_some)
        || initial_content.carried_item.is_some()
        || preparation_slot.item.as_ref().map(|item| item.item_count()) != Some(1)
        || preparation_slot
            .item
            .as_ref()
            .map(|item| item.positive_component_type_ids())
            != Some([17].as_slice())
        || remainder_slot.item.as_ref().map(|item| item.item_count()) != Some(1)
        || remainder_slot
            .item
            .as_ref()
            .map(|item| item.positive_component_type_ids())
            != Some([].as_slice())
        || player.inventory_state_id() != remainder_slot.state_id
        || player.inventory_slot_count() != 46
        || player
            .inventory_slot(36)
            .map(|item| item.item_registry_id())
            != Some(GLASS_BOTTLE)
        || player.carried_item().is_some()
    {
        return Err(format!(
            "local player state did not match the vanilla cleanup flow: update={local_update:?}, remove={local_remove:?}, effects={:?}, content={initial_content:?}, preparation={preparation_slot:?}, remainder={remainder_slot:?}, inventory_state_id={}, inventory_slot_count={}, inventory_slot_36={:?}, carried={:?}",
            player.effects(),
            player.inventory_state_id(),
            player.inventory_slot_count(),
            player.inventory_slot(36),
            player.carried_item(),
        )
        .into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("login_player_id={}", login.player_id);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "carried_item_packets_sent={}",
        trace.carried_item_packets_sent
    );
    println!("use_item_packets_sent={}", trace.use_item_packets_sent);
    println!(
        "mob_effect_update_observations={:?}",
        trace.mob_effect_update_observations
    );
    println!(
        "mob_effect_remove_observations={:?}",
        trace.mob_effect_remove_observations
    );
    println!("local_player_effect_count={}", player.effects().len());
    println!(
        "local_player_inventory_state_id={}",
        player.inventory_state_id()
    );
    println!(
        "local_player_inventory_slot_count={}",
        player.inventory_slot_count()
    );
    println!(
        "local_player_inventory_slot_36={:?}",
        player.inventory_slot(36)
    );
    println!("local_player_carried_item={:?}", player.carried_item());
    println!(
        "container_slot_observations={:?}",
        trace.container_slot_observations
    );
    println!(
        "container_content_observations={:?}",
        trace.container_content_observations
    );
    println!(
        "player_inventory_observations={:?}",
        trace.player_inventory_observations
    );
    println!(
        "cursor_item_observations={:?}",
        trace.cursor_item_observations
    );
    println!(
        "block_changed_ack_sequences={:?}",
        trace.block_changed_ack_sequences
    );
    println!("chunk_packet_count={}", trace.chunk_packet_count);
    println!("decoded_chunk_count={}", trace.chunk_observations.len());
    println!(
        "chunk_batch_finished_count={}",
        trace.chunk_batch_finished_count
    );
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
