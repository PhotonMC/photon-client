//! The local player's effects as the fog and the lightmap read them: how an effect fades in and
//! out as it is ticked, the night vision's flicker, and the water vision.

use serde_json::{Value, json};

use super::pose::entity;
use crate::bits::f32_bits;

/// The real `MobEffectInstance` ticked on a client: nausea and darkness, which fade, and others,
/// that start faded in or fade in over the effect's own time, short, long and without end.
pub fn mob_effect_blend() -> Vec<Value> {
    let row = |label: &str, effect: &str, duration: i64, blend: bool, ticks: i64| json!({"label": label, "effect": effect, "duration": duration, "amplifier": 0, "blend": blend, "ticks": ticks});
    let rows = vec![
        row("darkness fades in", "minecraft:darkness", 400, true, 60),
        row("darkness starts faded in", "minecraft:darkness", 400, false, 30),
        row("darkness fades out near its end", "minecraft:darkness", 40, true, 60),
        row("darkness without end", "minecraft:darkness", -1, true, 40),
        row("nausea fades in slowly", "minecraft:nausea", 600, true, 200),
        row("nausea fades out before its end", "minecraft:nausea", 100, true, 120),
        row("nausea starts faded in", "minecraft:nausea", 100, false, 120),
        row("blindness has no fade", "minecraft:blindness", 30, true, 40),
        row("night vision has no fade", "minecraft:night_vision", 50, true, 60),
    ];
    vec![entity("mob-effect-blend", "mob_effect_blend", rows)]
}

/// The real `GameRenderer.nightVisionScale` and `LocalPlayer.getWaterVision`.
pub fn view_scales() -> Vec<Value> {
    let mut rows = Vec::new();
    for (duration, partial) in
        [(-1, 0.0), (400, 0.5), (201, 0.0), (200, 0.0), (150, 0.25), (37, 0.75), (1, 0.0), (0, 0.5)]
    {
        rows.push(json!({
            "kind": "night_vision", "label": format!("night vision with {duration} left at {partial}"),
            "duration": duration, "partial": f32_bits(partial),
        }));
    }
    for (time, eyes) in
        [(0, true), (50, true), (99, true), (100, true), (250, true), (599, true), (600, true), (300, false)]
    {
        rows.push(json!({"kind": "water_vision", "label": format!("{time} ticks under water, eyes in {eyes}"), "time": time, "eyes_in_water": eyes}));
    }
    vec![entity("view-scales", "view_scales", rows)]
}
