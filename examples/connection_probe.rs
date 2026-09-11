use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_probe;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address: SocketAddr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = env::args().nth(2).unwrap_or_else(|| "RustProbe".to_owned());
    let trace = connect_and_probe(address, &username, Duration::from_secs(20))?;

    println!("connection=ok");
    println!("compression_threshold={:?}", trace.compression_threshold);
    println!("login_finished={}", trace.login_finished);
    println!("configuration_finished={}", trace.configuration_finished);
    println!("enabled_feature_count={}", trace.enabled_feature_count);
    println!("known_pack_count={}", trace.known_pack_count);
    println!("registry_packet_count={}", trace.registry_packet_count);
    println!("registry_entry_count={}", trace.registry_entry_count);
    println!(
        "update_tags_packet_count={}",
        trace.update_tags_packet_count
    );
    println!("play_login={}", trace.play_login);
    println!(
        "player_position_observations={:?}",
        trace.player_position_observations
    );
    println!("player_loaded_sent={}", trace.player_loaded_sent);
    println!("movement_packets_sent={}", trace.movement_packets_sent);
    println!(
        "player_action_packets_sent={}",
        trace.player_action_packets_sent
    );
    println!(
        "use_item_on_packets_sent={}",
        trace.use_item_on_packets_sent
    );
    println!(
        "block_changed_ack_sequences={:?}",
        trace.block_changed_ack_sequences
    );
    println!("chunk_packet_count={}", trace.chunk_packet_count);
    println!("decoded_chunk_count={}", trace.chunk_observations.len());
    if let Some(chunk) = trace.chunk_observations.first() {
        println!(
            "first_chunk=x:{},z:{},heightmaps:{},heightmap_observations:{:?},section_bytes:{},block_entities:{},sky_mask_words:{},block_mask_words:{},empty_sky_mask_words:{},empty_block_mask_words:{},sky_updates:{:?},block_updates:{:?}",
            chunk.x,
            chunk.z,
            chunk.heightmap_count,
            chunk.heightmap_observations,
            chunk.section_data_length,
            chunk.block_entity_count,
            chunk.sky_mask_word_count,
            chunk.block_mask_word_count,
            chunk.empty_sky_mask_word_count,
            chunk.empty_block_mask_word_count,
            chunk.sky_update_lengths,
            chunk.block_update_lengths,
        );
        println!(
            "first_chunk_section_count={}",
            chunk.section_observations.len()
        );
        if let Some(section) = chunk.section_observations.first() {
            println!(
                "first_section=non_empty:{},fluid:{},block_bits:{},block_palette:{},block_palette_ids:{:?},block_words:{},block_first_values:{:?},block_distinct:{},biome_bits:{},biome_palette:{},biome_palette_ids:{:?},biome_words:{},biome_first_values:{:?},biome_distinct:{}",
                section.non_empty_block_count,
                section.fluid_count,
                section.block_state_bits_per_entry,
                section.block_state_palette_entry_count,
                section.block_state_palette_ids,
                section.block_state_storage_word_count,
                section.block_state_first_values,
                section.block_state_distinct_value_count,
                section.biome_bits_per_entry,
                section.biome_palette_entry_count,
                section.biome_palette_ids,
                section.biome_storage_word_count,
                section.biome_first_values,
                section.biome_distinct_value_count,
            );
        }
    }
    println!(
        "chunk_batch_finished_count={}",
        trace.chunk_batch_finished_count
    );
    println!("received_packet_count={}", trace.packets.len());
    Ok(())
}
