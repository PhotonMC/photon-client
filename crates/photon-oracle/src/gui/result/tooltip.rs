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

/// A stack's tooltip as a container screen draws it, with a bundle's image:
/// the lines from `tooltip::lines`, by the mouse on a GUI of the row's size.
pub(super) fn bundle_tooltip_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_game::tooltip::{Flags, lines, no_maps};
    let lang = photon_game::text::Lang::load(&ctx.root);
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    let registries = photon_game::registries::Registries::vanilla(&data_root);
    let mut stack = row_stack(row)?;
    if let Some(selected) = row["selected"].as_i64() {
        photon_game::menus::container::toggle_bundle_selected(&mut stack, selected as i32);
    }
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let layout =
        photon_gui::screen::container::ContainerLayout::new(int("width"), int("height"), (0, 0));
    let mut item = no_items;
    let mut player = |_: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
        Vec::<photon_render::gui::GuiItemVertex>::new()
    };
    let mut entity = |_: EntityId, _: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
        Vec::<photon_render::gui::GuiItemVertex>::new()
    };
    let mut tooltip = |stack: &photon_world::ClientItemStackState, _: bool| {
        lines(stack, false, Flags::default(), &registries, &no_maps)
            .iter()
            .map(|line| lang.runs(line, &photon_game::text::Style::default()))
            .collect::<Vec<_>>()
    };
    let (book, tags) = (
        photon_world::RecipeBook::default(),
        photon_world::block::tags::ItemTags::default(),
    );
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let mut draw = photon_gui::screen::ScreenDraw {
        held: (0, [false; 2]),
        frame: &mut frame,
        atlas: &ctx.atlas,
        font: &ctx.font,
        item: &mut item,
        player: &mut player,
        entity: &mut entity,
        tooltip: &mut tooltip,
        lang: &lang,
        book: &book,
        tags: &tags,
        control_held: false,
        partial: 0.0,
        widget_tooltips: true,
    };
    let text = (draw.tooltip)(&stack, false);
    let mouse = (f64::from(int("mouse_x")), f64::from(int("mouse_y")));
    photon_gui::screen::bundle_tooltip::stack_tooltip(&layout, &mut draw, &stack, &text, mouse);
    Ok(json!({"label": row["label"], "ops": gui_ops(&frame.log.take().unwrap_or_default())}))
}
