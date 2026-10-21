use super::*;

/// A single-precision number as the reference writes it: its shortest
/// decimal, which is the value the oracle reads back as a double. Of two
/// shortest decimals as near to the value, `Float.toString` takes the one
/// whose last digit is even, where Rust's takes the larger.
pub(super) fn num(value: f32) -> f64 {
    let shortest = value.to_string();
    let parsed: f64 = shortest.parse().unwrap_or(f64::from(value));
    if !shortest.contains('.') {
        return parsed;
    }
    let digits = shortest.len() - shortest.find('.').unwrap_or(0) - 1;
    let unit = 10f64.powi(-(digits as i32));
    let exact = f64::from(value);
    let last_odd = shortest
        .bytes()
        .last()
        .is_some_and(|digit| (digit - b'0') % 2 == 1);
    // The other decimal of the tie: as near, one unit the other side, still this float.
    let other = if parsed > exact {
        parsed - unit
    } else {
        parsed + unit
    };
    let other: f64 = format!("{other:.digits$}").parse().unwrap_or(parsed);
    let tie = ((other - exact).abs() - (parsed - exact).abs()).abs() < unit * 1e-6;
    if last_odd && tie && other as f32 == value {
        other
    } else {
        parsed
    }
}

/// One run of a text op as the oracle writes it.
fn run_json(run: &photon_render::gui::LoggedRun) -> Value {
    let mut json = json!({
        "text": run.text, "bold": run.bold, "italic": run.italic,
        "underlined": run.underlined, "strikethrough": run.strikethrough,
        "obfuscated": run.obfuscated,
    });
    if let Some(rgb) = run.colour {
        json["rgb"] = json!(rgb);
    }
    json
}

/// One underline or strikethrough rectangle of a text op.
fn effect_json(effect: &photon_render::gui::text_effect::TextEffect) -> Value {
    json!({
        "x0": num(effect.rect.0), "y0": num(effect.rect.1),
        "x1": num(effect.rect.2), "y1": num(effect.rect.3),
        "color": effect.colour as i32, "shadowColor": effect.shadow_colour as i32,
    })
}

/// The recorded drawing calls as the reference's oracle writes them.
pub(super) fn gui_ops(ops: &[photon_render::gui::DrawOp]) -> Vec<Value> {
    use photon_render::gui::DrawOp;
    // Fills before text, then top to bottom, left to right and by what is
    // left to tell two apart: the reference layers what overlaps, an order
    // for the GPU and not for what is drawn.
    let key = |op: &&DrawOp| -> (u8, [f64; 4], String, i64) {
        match op {
            DrawOp::Fill { rect, colour } => (
                0,
                [rect.1, rect.0, rect.2, rect.3].map(f64::from),
                String::new(),
                i64::from(*colour as i32),
            ),
            DrawOp::Sprite { rect, name, colour } => (
                1,
                [rect.1, rect.0, rect.2, rect.3].map(f64::from),
                name.clone(),
                i64::from(*colour as i32),
            ),
            DrawOp::Texture { rect, name, .. } => (
                1,
                [rect.1, rect.0, rect.2, rect.3].map(f64::from),
                name.clone(),
                -1,
            ),
            DrawOp::Text {
                x, y, colour, runs, ..
            } => (
                2,
                [f64::from(*y), f64::from(*x), 0.0, 0.0],
                runs.iter().map(|run| run.text.as_str()).collect::<String>(),
                i64::from(*colour as i32),
            ),
        }
    };
    let mut sorted: Vec<&DrawOp> = ops.iter().collect();
    sorted.sort_by(|a, b| {
        let (a, b) = (key(a), key(b));
        a.0.cmp(&b.0)
            .then_with(|| {
                a.1.iter()
                    .zip(&b.1)
                    .map(|(x, y)| x.total_cmp(y))
                    .find(|order| order.is_ne())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
    });
    sorted
        .into_iter()
        .map(|op| match op {
            DrawOp::Fill { rect, colour } => json!({
                "op": "fill", "x0": num(rect.0), "y0": num(rect.1),
                "x1": num(rect.2), "y1": num(rect.3),
                "color": *colour as i32,
            }),
            DrawOp::Texture { name, rect, uv, colour } => json!({
                "op": "texture", "name": name, "x0": num(rect.0), "y0": num(rect.1),
                "x1": num(rect.2), "y1": num(rect.3), "color": *colour as i32,
                "u0": f64::from(uv.0), "u1": f64::from(uv.1), "v0": f64::from(uv.2), "v1": f64::from(uv.3),
            }),
            DrawOp::Sprite { name, rect, colour } => json!({
                "op": "sprite", "name": name, "x0": num(rect.0), "y0": num(rect.1),
                "x1": num(rect.2), "y1": num(rect.3), "color": *colour as i32,
            }),
            DrawOp::Text {
                x,
                y,
                scale,
                colour,
                shadow,
                runs,
                effects,
                ..
            } => json!({
                "op": "text", "x": num(*x), "y": num(*y), "scale": num(*scale), "color": *colour as i32,
                "shadow": shadow,
                "runs": runs.iter().map(run_json).collect::<Vec<_>>(),
                "effects": effects.iter().map(effect_json).collect::<Vec<_>>(),
            }),
        })
        .collect()
}

/// The screens' drawing, recorded and compared with the reference's own.
/// A number format named in a request: `blank`, `style:<colour>` or
/// `fixed:<text>`.
pub(super) fn number_format(spec: &str) -> photon_game::scoreboard::NumberFormat {
    use photon_game::scoreboard::NumberFormat;
    if spec == "blank" {
        NumberFormat::Blank
    } else if let Some(colour) = spec.strip_prefix("style:") {
        NumberFormat::Styled(photon_game::text::Style::coloured(colour))
    } else {
        NumberFormat::Fixed(photon_game::text::Component::literal(
            spec.strip_prefix("fixed:").unwrap_or_default(),
        ))
    }
}

/// The scoreboard a gui request describes, built the way the packets build
/// the game's.
pub(super) fn scoreboard_from(row: &Value) -> Result<photon_game::scoreboard::Scoreboard, String> {
    use photon_game::scoreboard::{Scoreboard, ScoreboardEvent, TeamParameters};
    use photon_game::text::Component;
    let literal = |value: &Value| Component::literal(value.as_str().unwrap_or_default());
    let mut board = Scoreboard::default();
    for objective in row["objectives"].as_array().into_iter().flatten() {
        board.apply(ScoreboardEvent::Objective {
            name: objective["name"]
                .as_str()
                .ok_or("objective name")?
                .to_owned(),
            method: 0,
            display_name: literal(&objective["title"]),
            hearts: objective["hearts"].as_bool().unwrap_or(false),
            number_format: objective["format"].as_str().map(number_format),
        });
    }
    for score in row["scores"].as_array().into_iter().flatten() {
        board.apply(ScoreboardEvent::Score {
            owner: score["owner"].as_str().ok_or("score owner")?.to_owned(),
            objective: score["objective"]
                .as_str()
                .ok_or("score objective")?
                .to_owned(),
            value: score["value"].as_i64().unwrap_or(0) as i32,
            display: score["display"].as_str().map(Component::literal),
            number_format: score["format"].as_str().map(number_format),
        });
    }
    if let Some(slots) = row["slots"].as_object() {
        for (slot, objective) in slots {
            board.apply(ScoreboardEvent::Display {
                slot: slot.parse().map_err(|_| "slot")?,
                objective: objective.as_str().map(str::to_owned),
            });
        }
    }
    for team in row["teams"].as_array().into_iter().flatten() {
        board.apply(ScoreboardEvent::Team {
            name: team["name"].as_str().ok_or("team name")?.to_owned(),
            method: 0,
            parameters: Some(TeamParameters {
                display_name: literal(&team["name"]),
                prefix: literal(&team["prefix"]),
                suffix: literal(&team["suffix"]),
                colour: team["color"].as_u64().map(|colour| colour as u8),
            }),
            players: team["players"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|player| player.as_str().map(str::to_owned))
                .collect(),
        });
    }
    Ok(board)
}
