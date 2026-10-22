//! The options a settings row stores before its screen opens.

use serde_json::Value;

/// The quality options the row stores, as the file would have them when the screen opens.
pub(super) fn stored_values(row: &Value) -> photon_gui::options::video::VideoValues {
    use photon_game::options::{
        CloudStatus, GraphicsPreset, PrioritizeChunkUpdates, QualityOptions, TextureFiltering,
    };
    let stored = &row["options"]["quality"];
    let int = |key: &str, default: i32| stored[key].as_i64().map_or(default, |value| value as i32);
    let flag = |key: &str, default: bool| stored[key].as_bool().unwrap_or(default);
    let fallback = QualityOptions::default();
    let quality = QualityOptions {
        graphics_preset: stored["preset"]
            .as_str()
            .and_then(|name| {
                GraphicsPreset::ALL
                    .into_iter()
                    .find(|p| p.serialized() == name)
            })
            .unwrap_or(fallback.graphics_preset),
        biome_blend_radius: int("blend", fallback.biome_blend_radius),
        prioritize_chunk_updates: PrioritizeChunkUpdates::ALL[int("priority", 0) as usize],
        simulation_distance: int("simulation", fallback.simulation_distance),
        cloud_status: CloudStatus::ALL[int("clouds", 2) as usize],
        mipmap_levels: int("mipmaps", fallback.mipmap_levels),
        entity_shadows: flag("shadows", fallback.entity_shadows),
        entity_distance_scaling: stored["entity_distance"]
            .as_f64()
            .unwrap_or(fallback.entity_distance_scaling),
        cloud_range: int("cloud_range", fallback.cloud_range),
        cutout_leaves: flag("leaves", fallback.cutout_leaves),
        improved_transparency: flag("transparency", fallback.improved_transparency),
        weather_radius: int("weather", fallback.weather_radius),
        max_anisotropy_bit: int("anisotropy", fallback.max_anisotropy_bit),
        texture_filtering: TextureFiltering::ALL[int("filtering", 0) as usize],
    };
    photon_gui::options::video::VideoValues {
        quality,
        ..photon_gui::options::video::VideoValues::default()
    }
}
