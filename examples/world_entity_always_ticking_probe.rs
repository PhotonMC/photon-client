use std::env;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use voxel_client::connection::connect_and_world_entity_add_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let first_username = env::args()
        .nth(2)
        .unwrap_or_else(|| "P08AlwaysA".to_owned());
    let second_username = env::args()
        .nth(3)
        .unwrap_or_else(|| "P08AlwaysB".to_owned());

    let first_address = address;
    let first_username_for_thread = first_username.clone();
    let first = thread::spawn(move || {
        connect_and_world_entity_add_probe(
            first_address,
            &first_username_for_thread,
            Duration::from_secs(60),
        )
    });
    thread::sleep(Duration::from_millis(750));

    let second =
        connect_and_world_entity_add_probe(address, &second_username, Duration::from_secs(60))?;
    let first = first.join().map_err(|_| "first vanilla probe panicked")??;

    let Some(remote) = first.world.entities().find(|entity| entity.type_id == 156) else {
        return Err("vanilla did not send the second player as entity type 156".into());
    };
    let remote_type_id = remote.type_id;
    let remote_id = remote.uuid;
    let position = remote.position();
    let chunk_x = (position.0.floor() as i32).div_euclid(16);
    let chunk_z = (position.2.floor() as i32).div_euclid(16);
    let tick_before_unload = remote.tick_count();
    if !remote.is_always_ticking() || !remote.is_ticking() {
        return Err("player entity was not always ticking before chunk unload".into());
    }

    let mut world = first.world;
    let removed = world.unload_chunk(chunk_x, chunk_z);
    world.tick();
    let Some(after_unload) = world.entities().find(|entity| entity.uuid == remote_id) else {
        return Err("remote player disappeared from ClientWorld during chunk unload".into());
    };
    if !removed
        || !after_unload.is_always_ticking()
        || !after_unload.is_ticking()
        || after_unload.tick_count() != tick_before_unload + 1
    {
        return Err(format!(
            "player always-ticking contract failed: removed={removed}, before={tick_before_unload}, after_ticking={}, after_always={}, after_ticks={}",
            after_unload.is_ticking(),
            after_unload.is_always_ticking(),
            after_unload.tick_count()
        )
        .into());
    }

    println!("connection=ok");
    println!("first_play_login={}", first.play_login);
    println!("second_play_login={}", second.play_login);
    println!(
        "first_add_entity_packet_count={}",
        first.add_entity_observations.len()
    );
    println!(
        "second_add_entity_packet_count={}",
        second.add_entity_observations.len()
    );
    println!("remote_player_type_id={remote_type_id}");
    println!("remote_player_uuid_most={}", remote_id.most);
    println!("remote_player_uuid_least={}", remote_id.least);
    println!("remote_player_chunk=x:{chunk_x},z:{chunk_z}");
    println!("chunk_removed={removed}");
    println!(
        "always_ticking_after_unload={}",
        after_unload.is_always_ticking()
    );
    println!("ticking_after_unload={}", after_unload.is_ticking());
    println!("tick_count_before_unload={tick_before_unload}");
    println!("tick_count_after_unload={}", after_unload.tick_count());
    println!("first_world_chunk_count={}", world.loaded_chunk_count());
    println!("first_world_entity_count={}", world.loaded_entity_count());
    println!(
        "second_world_entity_count={}",
        second.world.loaded_entity_count()
    );
    println!("first_received_packet_count={}", first.packets.len());
    println!("second_received_packet_count={}", second.packets.len());
    Ok(())
}
