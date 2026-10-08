use super::*;

/// `ItemStack.getTooltipLines` for a stack, with no player: each line as its
/// runs, neighbours of one style joined, the style written as the reference
/// adapter writes it (a colour or `-`, then the set flags).
pub(super) fn tooltip_lines_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_game::tooltip::{Flags, MapInfo, lines};
    let lang = photon_game::text::Lang::load(&ctx.root);
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    let registries = photon_game::registries::Registries::vanilla(&data_root);
    let stack = row_stack(row)?;
    let map = row.get("map").map(|map| MapInfo {
        scale: map["scale"].as_i64().unwrap_or(0) as i8,
        locked: map["locked"].as_bool().unwrap_or(false),
    });
    let flags = Flags {
        creative: row["creative"].as_bool().unwrap_or(false),
        advanced: row["advanced"].as_bool().unwrap_or(false),
    };
    let lines: Vec<Value> = lines(&stack, false, flags, &registries, &|_| map)
        .iter()
        .map(|line| {
            let mut runs: Vec<(String, String)> = Vec::new();
            for run in lang.runs(line, &photon_game::text::Style::default()) {
                let style = &run.style;
                let flag = |on: Option<bool>, letter: &'static str| {
                    if on == Some(true) { letter } else { "" }
                };
                let key = format!(
                    "{}{}{}{}{}{}",
                    style
                        .colour
                        .map_or("-".to_owned(), |colour| format!("{colour:06x}")),
                    flag(style.bold, "b"),
                    flag(style.italic, "i"),
                    flag(style.underlined, "u"),
                    flag(style.strikethrough, "s"),
                    flag(style.obfuscated, "o"),
                );
                match runs.last_mut() {
                    Some((text, last)) if *last == key => text.push_str(&run.text),
                    _ => runs.push((run.text.clone(), key)),
                }
            }
            json!(
                runs.into_iter()
                    .map(|(text, style)| json!([text, style]))
                    .collect::<Vec<_>>()
            )
        })
        .collect();
    Ok(json!({"label": row["label"], "lines": lines}))
}
