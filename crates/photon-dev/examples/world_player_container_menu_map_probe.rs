use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_dev::connect_and_world_player_menu_probe;

fn is_hopper(id: i32) -> bool {
    photon_data::menu_type::id("minecraft:hopper") == Some(id)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args()
        .nth(2)
        .unwrap_or_else(|| "RustHopperMenu".to_owned());
    let trace = connect_and_world_player_menu_probe(address, &username, Duration::from_secs(60))?;

    let open = trace
        .open_screen_observations
        .iter()
        .find(|observation| observation.applied_to_world)
        .ok_or("vanilla did not provide an applied open-screen packet")?;
    if open.container_id == 0 || !is_hopper(open.menu_type_id) || open.title_nbt.is_empty() {
        return Err("vanilla open-screen fields did not describe a hopper menu".into());
    }

    let content = trace
        .container_content_observations
        .iter()
        .find(|observation| {
            observation.container_id == open.container_id && observation.applied_to_world
        })
        .ok_or("vanilla did not provide applied hopper-container content")?;
    if content.item_count != 41 || content.items.len() != 41 {
        return Err("vanilla hopper content did not contain 41 menu slots".into());
    }
    let hopper_item = content.items[0]
        .as_ref()
        .ok_or("vanilla hopper slot 0 was empty")?;
    if hopper_item.item_count() != 7 || hopper_item.item_registry_id().0 != 1 {
        return Err("vanilla hopper slot 0 did not contain stone x7".into());
    }

    let container = trace
        .world
        .local_player()
        .and_then(|player| player.open_container())
        .ok_or("Rust world did not retain the open hopper container")?;
    if container.container_id() != open.container_id
        || container.menu_type_id() != open.menu_type_id
        || container.top_slot_count() != Some(5)
        || container.slot_count() != 41
        || container.state_id() != content.state_id
    {
        return Err("Rust hopper menu metadata did not match vanilla".into());
    }
    let world_item = container.slot(0).ok_or("Rust hopper slot 0 was empty")?;
    if world_item.item_count() != 7 || world_item.item_registry_id().0 != 1 {
        return Err("Rust hopper slot 0 did not contain stone x7".into());
    }

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "open_screen_observations={:?}",
        trace.open_screen_observations
    );
    println!(
        "hopper_container_content_state={} item_count={}",
        content.state_id, content.item_count
    );
    println!(
        "world_open_container=id:{} menu_type:{} top_slots:{:?} slots:{} state:{} slot0:{}x{}",
        container.container_id(),
        container.menu_type_id(),
        container.top_slot_count(),
        container.slot_count(),
        container.state_id(),
        world_item.item_count(),
        world_item.item_registry_id()
    );
    println!(
        "container_close_observations={:?}",
        trace.container_close_observations
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
