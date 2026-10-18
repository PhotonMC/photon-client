use super::*;

/// `hud_sleep`: the veil over the view for a sleep timer.
pub(super) fn hud_sleep_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    photon_gui::hud::sleep::draw(
        &mut frame,
        &ctx.atlas,
        (int("width"), int("height")),
        int("timer"),
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// `subtitles`: the sounds heard at their times, the overlay drawn from a listener's place.
pub(super) fn subtitles_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_core::math::Vec3;
    use photon_gui::hud::subtitles::{Listener, Subtitles};
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let vec = |value: &Value| -> Result<Vec3, String> {
        let part = |index: usize| value[index].as_f64().ok_or("coordinate");
        Ok(Vec3::new(part(0)?, part(1)?, part(2)?))
    };
    let lang = photon_game::text::Lang::load(&ctx.root);
    let mut subtitles = Subtitles::default();
    for heard in row["heard"].as_array().ok_or("heard")? {
        let place = vec(&heard["position"])?;
        let range = if heard["infinite"].as_bool().unwrap_or(false) {
            f32::INFINITY
        } else {
            heard["range"].as_f64().ok_or("range")? as f32
        };
        subtitles.heard(
            heard["key"].as_str().ok_or("key")?,
            (place.x, place.y, place.z),
            range,
            heard["at"].as_u64().ok_or("at")?,
        );
    }
    let (forward, up) = (vec(&row["forward"])?, vec(&row["up"])?);
    let listener = Listener::new(vec(&row["position"])?, forward, up);
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    // `Options.getBackgroundColor(0.8F)` with the background for the chat only: that opacity.
    let background = ((0.8_f32 * 255.0) as u32) << 24;
    subtitles.draw(
        &mut frame,
        (&ctx.atlas, &ctx.font, &lang),
        (int("width"), int("height")),
        listener,
        row["now"].as_u64().ok_or("now")?,
        (
            row["display_time"].as_f64().ok_or("display_time")?,
            background,
        ),
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
