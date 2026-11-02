//! The debug charts' operation.

use super::*;

/// `hud_debug_chart`: a chart over the samples given, as the game's `extractRenderState` draws it.
pub(super) fn hud_debug_chart_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_gui::hud::debug_chart::{CAPACITY, DIMENSIONS, Kind, draw, width};
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let samples: Vec<[i64; DIMENSIONS]> = row["samples"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|sample| {
            let mut out = [0; DIMENSIONS];
            match sample.as_array() {
                Some(values) => values
                    .iter()
                    .zip(&mut out)
                    .for_each(|(value, slot)| *slot = value.as_i64().unwrap_or(0)),
                None => out[0] = sample.as_i64().unwrap_or(0),
            }
            out
        })
        .collect();
    // The log keeps the newest samples the chart has room for.
    let kept = samples.len().saturating_sub(CAPACITY as usize);
    let kind = match row["kind"].as_str() {
        Some("fps") => Kind::Fps {
            limit: int("limit"),
        },
        Some("tps") => Kind::Tps {
            mspt: row["mspt"].as_f64().unwrap_or(50.0) as f32,
        },
        Some("bandwidth") => Kind::Bandwidth,
        _ => Kind::Ping,
    };
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let place = (int("left"), width(int("width")), int("height"));
    draw(
        &mut frame,
        (&ctx.atlas, &ctx.font),
        kind,
        &samples[kept..],
        place,
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
