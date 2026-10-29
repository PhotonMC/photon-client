use super::*;

/// `loading_overlay`: the loading overlay at a pinned time, progress and fade starts.
pub(super) fn loading_overlay_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let flag = |key: &str| row[key].as_bool().unwrap_or(false);
    let mut overlay =
        photon_gui::loading_overlay::LoadingOverlay::new(flag("fade_in"), flag("dark"));
    overlay.set_current_progress(row["progress"].as_f64().unwrap_or(0.0) as f32);
    overlay.set_fade_starts(
        row["fade_in_start"].as_i64(),
        row["fade_out_start"].as_i64(),
    );
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let clear = overlay.draw(
        &mut frame,
        &ctx.atlas,
        (int("width"), int("height")),
        (
            row["ms"].as_i64().unwrap_or(0),
            row["actual"].as_f64().unwrap_or(0.0) as f32,
        ),
        &mut |_| {},
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        "clear": clear_bits(clear),
        "progress": crate::declarations::bits(overlay.current_progress()),
    }))
}

/// `ARGB.setVector4fFromARGB32` on a zeroed vector: red, green, blue and alpha as bit strings.
fn clear_bits(colour: Option<u32>) -> Vec<String> {
    let channel = |shift: u32| f32::from(((colour.unwrap_or(0) >> shift) & 0xFF) as u8) / 255.0;
    [16, 8, 0, 24]
        .map(|shift| crate::declarations::bits(channel(shift)))
        .to_vec()
}
