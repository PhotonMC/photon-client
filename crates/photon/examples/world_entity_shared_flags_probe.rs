use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use photon_net::connection::connect_and_world_entity_metadata_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25568".to_owned())
        .parse()?;
    let username = env::args().nth(2).unwrap_or_else(|| "P08Flags".to_owned());
    let trace =
        connect_and_world_entity_metadata_probe(address, &username, Duration::from_secs(60))?;
    let flagged_entity = trace
        .world
        .entities()
        .all()
        .find(|entity| entity.is_invisible() && entity.is_glowing());
    let flagged = flagged_entity
        .map(|entity| {
            (
                "remote_entity",
                Some(entity.type_id),
                entity.shared_flags(),
                entity.is_on_fire(),
                entity.is_shift_key_down(),
                entity.is_sprinting(),
                entity.is_swimming(),
                entity.is_invisible(),
                entity.is_glowing(),
                entity.is_fall_flying(),
            )
        })
        .or_else(|| {
            trace
                .world
                .local_player()
                .filter(|player| player.is_invisible() && player.is_glowing())
                .map(|player| {
                    (
                        "local_player",
                        None,
                        player.shared_flags(),
                        player.is_on_fire(),
                        player.is_shift_key_down(),
                        player.is_sprinting(),
                        player.is_swimming(),
                        player.is_invisible(),
                        player.is_glowing(),
                        player.is_fall_flying(),
                    )
                })
        });
    let Some((
        target,
        type_id,
        shared_flags,
        on_fire,
        shift_key_down,
        sprinting,
        swimming,
        invisible,
        glowing,
        fall_flying,
    )) = flagged
    else {
        return Err("vanilla shared flags were not applied to a flagged entity".into());
    };

    println!("connection=ok");
    println!("play_login={}", trace.play_login);
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!(
        "entity_metadata_packet_count={}",
        trace.entity_metadata_observations.len()
    );
    println!("flagged_target={target}");
    match type_id {
        Some(type_id) => println!("flagged_entity_type_id={type_id}"),
        None => println!("flagged_entity_type_id=player"),
    }
    println!("shared_flags={shared_flags}");
    println!("on_fire={on_fire}");
    println!("shift_key_down={shift_key_down}");
    println!("sprinting={sprinting}");
    println!("swimming={swimming}");
    println!("invisible={invisible}");
    println!("glowing={glowing}");
    println!("fall_flying={fall_flying}");
    println!("world_entity_count={}", trace.world.entities().count());
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
