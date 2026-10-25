use super::*;
use photon_world::ClientMobEffectState;

/// The row's effects as the server's packets give them.
fn row_effects(row: &Value) -> Result<Vec<ClientMobEffectState>, String> {
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
        effects.push(ClientMobEffectState::from_packet(
            id,
            int(described, "amplifier"),
            int(described, "duration"),
            flags,
        ));
    }
    Ok(effects)
}

/// The effects' icons on the HUD, or their list beside an inventory.
pub(super) fn effects_case(
    ctx: &mut GuiContext,
    row: &Value,
    operation: &str,
) -> Result<Value, String> {
    if operation == "hud_effects" {
        hud_effects_case(ctx, row)
    } else {
        inventory_effects_case(ctx, row)
    }
}

/// `hud_effects`: the effect icons of a player with the row's effects.
fn hud_effects_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let effects = row_effects(row)?;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let width = row["width"].as_i64().unwrap_or(0) as i32;
    photon_gui::hud::effects::draw(&mut frame, &ctx.atlas, width, &effects);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// `inventory_effects`: the effects listed beside an inventory image placed as the row says, then
/// the tooltip a cut entry under the mouse asks for.
fn inventory_effects_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let effects = row_effects(row)?;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let mut layout = photon_gui::screen::container::ContainerLayout::new(
        int("width"),
        int("height"),
        (int("image_width"), int("image_height")),
    );
    layout.left_override = Some(int("left"));
    let mut item = no_items;
    let mut player = |_: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
        photon_render::gui::GuiFigure::default()
    };
    let mut entity = |_: EntityId, _: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
        Vec::<photon_render::gui::GuiItemVertex>::new()
    };
    let mut tooltip = |_: &photon_world::ClientItemStackState, _: bool| Vec::new();
    let (book, tags) = (
        photon_world::RecipeBook::default(),
        photon_world::block::tags::ItemTags::default(),
    );
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let mut draw = photon_gui::screen::ScreenDraw {
        viewer: photon_gui::screen::PlayerView {
            effects: &effects,
            tick_rate: row["tick_rate"].as_f64().unwrap_or(20.0) as f32,
            ..photon_gui::screen::PlayerView::default()
        },
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
    let mouse = (f64::from(int("mouse_x")), f64::from(int("mouse_y")));
    let asked = photon_gui::screen::effects::draw(&mut draw, &layout, mouse);
    photon_gui::screen::effects::draw_tooltip(&mut draw, &layout, asked, mouse);
    Ok(json!({"label": row["label"], "ops": gui_ops(&frame.log.take().unwrap_or_default())}))
}
