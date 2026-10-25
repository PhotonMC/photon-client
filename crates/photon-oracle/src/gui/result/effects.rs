use super::*;

/// `hud_effects`: the effect icons of a player with the row's effects.
pub(super) fn hud_effects_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let int = |value: &Value, key: &str| value[key].as_i64().unwrap_or(0) as i32;
    let mut effects = Vec::new();
    for described in row["effects"].as_array().into_iter().flatten() {
        let name = described["effect"].as_str().ok_or("effect")?;
        let id = photon_data::item::effect::effect_id(name)
            .ok_or_else(|| format!("unknown effect {name}"))?;
        // The packet's flags: ambient, visible, then the icon.
        let flag = |key: &str, bit: u8| {
            if described[key].as_bool().unwrap_or(false) {
                bit
            } else {
                0
            }
        };
        let flags = flag("ambient", 1) | 2 | flag("show_icon", 4);
        effects.push(photon_world::ClientMobEffectState::from_packet(
            id,
            int(described, "amplifier"),
            int(described, "duration"),
            flags,
        ));
    }
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    photon_gui::hud::effects::draw(&mut frame, &ctx.atlas, int(row, "width"), &effects);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
