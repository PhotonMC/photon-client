use super::*;

pub(super) fn map_data_case() -> Result<Value, String> {
    let mut rows = vec![json!({
        "label": "colors",
        "colors": (0..=255u8)
            .map(|packed| photon_data::map::packed_color(packed) as i32)
            .collect::<Vec<_>>(),
    })];
    // 26.3 numbers five more structures after the trial chambers.
    let present = if crate::case_ids::before_26_3() {
        photon_data::map::decoration_types().len() - 5
    } else {
        photon_data::map::decoration_types().len()
    };
    for (id, kind) in photon_data::map::decoration_types()[..present]
        .iter()
        .enumerate()
    {
        let mut row = json!({"label": kind.name, "id": id, "asset": kind.asset,
            "show_on_item_frame": kind.show_on_item_frame,
            "track_count": kind.track_count});
        if crate::case_ids::before_26_3() {
            row["map_color"] = json!(legacy_map_color(kind.name));
        }
        rows.push(row);
    }
    Ok(json!({"rows": rows}))
}

/// The colour 26.2 gives a decoration type's exploration map marker (`MapDecorationType.mapColor`),
/// or -1 for the ones that are no map element.
fn legacy_map_color(name: &str) -> i32 {
    match name.strip_prefix("minecraft:").unwrap_or(name) {
        "mansion" => 5_393_476,
        "monument" => 3_830_373,
        "trial_chambers" => 12_741_452,
        "village_desert" | "village_plains" | "village_savanna" | "village_snowy"
        | "village_taiga" | "jungle_temple" | "swamp_hut" => 10_066_329,
        _ => -1,
    }
}
