use super::*;

/// `sign_texts`: what a sign's text submits, each line's pose, place, runs, light and colours.
pub(super) fn sign_texts_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use voxel_render::block_entity::sign::{SignFace, SignKind, sign_text_submits};
    let lang = voxel_game::text::Lang::load(&ctx.root);
    let kind = if row["kind"] == "hanging" {
        SignKind::Hanging
    } else {
        SignKind::Standing
    };
    let wall = row["wall"]
        .as_str()
        .map(|name| voxel_core::direction::Direction::from_name(name).ok_or("wall"))
        .transpose()?;
    let filtering = row["filtering"].as_bool() == Some(true);
    // Each face's lines (written or filtered, as the row filters) in words.
    let face = |face: &Value| -> Result<([Vec<voxel_game::text::Run>; 4], usize, bool), String> {
        let key = if filtering && !face["filtered"].is_null() {
            "filtered"
        } else {
            "messages"
        };
        let mut lines: [Vec<voxel_game::text::Run>; 4] = Default::default();
        for (line, value) in lines.iter_mut().zip(face[key].as_array().ok_or("lines")?) {
            let json = voxel_core::json::parse(&value.to_string()).map_err(|_| "json")?;
            let component = voxel_game::text::Component::from_json(&json).ok_or("component")?;
            *line = lang.runs(&component, &voxel_game::text::Style::default());
        }
        let color = voxel_game::sign::DYES
            .iter()
            .position(|dye| face["color"] == *dye)
            .ok_or("dye")?;
        Ok((lines, color, face["glowing"].as_bool() == Some(true)))
    };
    let (front, back) = (face(&row["front"])?, face(&row["back"])?);
    let faces = [&front, &back].map(|(lines, color, glowing)| SignFace {
        lines,
        color: *color,
        glowing: *glowing,
    });
    let transformations = kind.transformations(wall, row["rotation"].as_i64().unwrap_or(0) as i32);
    let submits = sign_text_submits(
        &voxel_render::joml::Pose::default(),
        kind,
        (&transformations, faces),
        (
            row["draw_outline"].as_bool() == Some(true),
            row["light"].as_i64().unwrap_or(0) as i32,
        ),
        &ctx.font,
    );
    let submits: Vec<Value> = submits
        .iter()
        .map(|submit| {
            // The line as runs of one colour, as the oracle merges them.
            let mut runs: Vec<(String, i64)> = Vec::new();
            for run in &submit.line {
                let colour = run.style.colour.map_or(-1, i64::from);
                match runs.last_mut() {
                    Some((text, last)) if *last == colour => text.push_str(&run.text),
                    _ if run.text.is_empty() => {}
                    _ => runs.push((run.text.clone(), colour)),
                }
            }
            json!({
                "pose": crate::strider::matrix_bits(&submit.pose),
                "x": format!("{:08x}", submit.x.to_bits()),
                "y": format!("{:08x}", submit.y.to_bits()),
                "runs": runs.iter().map(|(text, colour)| json!([text, colour])).collect::<Vec<_>>(),
                "shadow": false,
                "mode": "POLYGON_OFFSET",
                "light": submit.light,
                "color": format!("{:x}", submit.color),
                "background": 0,
                "outline": format!("{:x}", submit.outline),
            })
        })
        .collect();
    Ok(json!({"label": row["label"], "submits": submits}))
}
