use super::stats::update_from;
use super::*;

use photon_game::advancements::ClientAdvancements;
use photon_game::text::Lang;
use photon_gui::screen::advancements::AdvancementsScreen;

fn pair(value: &Value) -> (i32, i32) {
    (
        value[0].as_i64().unwrap_or(0) as i32,
        value[1].as_i64().unwrap_or(0) as i32,
    )
}

/// What a case's screen is shown over: the client copy, the language and the pointer.
struct Scene<'a> {
    advancements: ClientAdvancements,
    lang: &'a Lang,
    mouse: (i32, i32),
}

fn draw(ctx: &GuiContext, scene: &Scene, screen: &mut AdvancementsScreen) -> Vec<Value> {
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let drawing = (&ctx.atlas, &ctx.font, scene.lang);
    screen.draw(&mut frame, drawing, scene.mouse, &mut no_items);
    gui_ops(&frame.log.take().unwrap_or_default())
}

/// One step of a row: an update or a tab, a press, a drag, the wheel, a tick or a resize.
fn run_step(
    ctx: &GuiContext,
    scene: &mut Scene,
    screen: &mut AdvancementsScreen,
    step: &Value,
) -> Result<(), String> {
    let fonts = (&ctx.font, scene.lang);
    let number = |key: &str| step[key].as_f64().ok_or_else(|| key.to_owned());
    let point = (
        step["x"].as_f64().unwrap_or(0.0) as i32,
        step["y"].as_f64().unwrap_or(0.0) as i32,
    );
    match step["do"].as_str().ok_or("step")? {
        "update" => {
            scene.advancements.update(&update_from(step)?);
            screen.sync(&mut scene.advancements, fonts);
        }
        "tab" => {
            let known = step["id"]
                .as_str()
                .filter(|id| scene.advancements.get(id).is_some());
            scene.advancements.select_tab(known);
            screen.sync(&mut scene.advancements, fonts);
        }
        "click" => {
            screen.clicked(&mut scene.advancements, fonts, point);
            screen.released();
        }
        "drag" => screen.dragged((number("dx")?, number("dy")?)),
        "release" => screen.released(),
        "scroll" => {
            screen.scrolled((number("sx")?, number("sy")?));
        }
        "tick" => screen.tick(point),
        "resize" => screen.resize(pair(&json!([step["width"], step["height"]]))),
        "frame" => {
            draw(ctx, scene, screen);
        }
        _ => return Err("unknown step".into()),
    }
    Ok(())
}

/// `advancements_screen`: the screen over a client copy, after its updates, clicks, drags, scrolls,
/// ticks and resizes.
pub(super) fn advancements_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let lang = Lang::load(&ctx.root);
    let mut scene = Scene {
        advancements: ClientAdvancements::default(),
        lang: &lang,
        mouse: pair(&row["mouse"]),
    };
    for update in row["before"].as_array().ok_or("before")? {
        scene.advancements.update(&update_from(update)?);
    }
    let size = (
        row["width"].as_i64().unwrap_or(0) as i32,
        row["height"].as_i64().unwrap_or(0) as i32,
    );
    let (mut screen, _) =
        AdvancementsScreen::new(&mut scene.advancements, (&ctx.font, &lang), size, true);
    for step in row["steps"].as_array().ok_or("steps")? {
        run_step(ctx, &mut scene, &mut screen, step)?;
    }
    Ok(json!({"label": row["label"], "ops": draw(ctx, &scene, &mut screen)}))
}
