//! The debug charts' operation.

use super::*;

/// `hud_debug_chart`: the ping chart over the samples given, as the game's `extractRenderState` draws it.
pub(super) fn hud_debug_chart_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let samples: Vec<i64> = row["samples"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_i64)
        .collect();
    // The log keeps the newest samples the chart has room for.
    let kept = samples
        .len()
        .saturating_sub(photon_gui::hud::debug_chart::CAPACITY as usize);
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    photon_gui::hud::debug_chart::draw(
        &mut frame,
        (&ctx.atlas, &ctx.font),
        photon_gui::hud::debug_chart::Kind::Ping,
        &samples[kept..],
        (
            int("left"),
            photon_gui::hud::debug_chart::width(int("width")),
            int("height"),
        ),
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
