use super::*;

/// `key_binds_screen`: the key binds screen over the row's bindings, after its steps. Keys
/// have no system names here, as GLFW is not running in the reference either.
pub(super) fn key_binds_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_data::keys::{InputKey, key_of};
    use photon_gui::options::keybinds::{BindKey, KeyBindsScreen};
    use photon_gui::widgets::slider::SliderKey;
    use photon_render::window::scale::WindowScale;
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let lang = photon_game::text::Lang::load(&ctx.root);
    let pair = |key: &str| {
        (
            row[key][0].as_i64().unwrap_or(0) as i32,
            row[key][1].as_i64().unwrap_or(0) as i32,
        )
    };
    let (framebuffer, mouse) = (pair("framebuffer"), pair("mouse"));
    let gui_scale = row["gui_scale"].as_i64().unwrap_or(0) as i32;
    let mut window = WindowScale::new(framebuffer.0, framebuffer.1);
    window.set_gui_scale(window.calculate_scale(gui_scale, false));
    let mut options = photon_game::options::Options::default();
    if let Some(bindings) = row["bindings"].as_object() {
        for (mapping, key) in bindings {
            let key = key.as_str().and_then(key_of).ok_or("binding")?;
            options.set_key_binding(mapping, key);
        }
    }
    let size = (window.gui_scaled_width(), window.gui_scaled_height());
    let in_world = row["in_world"].as_bool().unwrap_or(false);
    let mut screen = KeyBindsScreen::new(
        (font, &lang),
        size,
        in_world,
        options.key_bindings(),
        |_| None,
    );
    let mut last = (0.0, 0.0);
    let mut clock = 0_u64;
    for step in row["steps"].as_array().ok_or("steps")? {
        // The reference draws a frame between one event and the next.
        let mut between = GuiFrame {
            millis: clock,
            ..GuiFrame::default()
        };
        screen.draw(&mut between, atlas, font, mouse);
        if step["do"] == "frame" {
            // A frame drawn and thrown away, at a time (the tooltips wait by it).
            clock = step["ms"].as_u64().unwrap_or(0);
            let mut thrown = GuiFrame {
                millis: clock,
                ..GuiFrame::default()
            };
            screen.draw(&mut thrown, atlas, font, mouse);
            continue;
        }
        let at = (
            step["x"].as_f64().unwrap_or(last.0),
            step["y"].as_f64().unwrap_or(last.1),
        );
        match step["do"].as_str().unwrap_or_default() {
            "click" => {
                let button = step["button"].as_i64().unwrap_or(0) as i32;
                screen.clicked(&lang, at, button);
            }
            "key" => {
                let name = step["key"].as_str().unwrap_or_default();
                let (code, kind) = match name {
                    "tab" => (258, BindKey::Tab(true)),
                    "shift+tab" => (258, BindKey::Tab(false)),
                    "left" => (263, BindKey::Menu(SliderKey::Left)),
                    "right" => (262, BindKey::Menu(SliderKey::Right)),
                    "up" => (265, BindKey::Menu(SliderKey::Up)),
                    "down" => (264, BindKey::Menu(SliderKey::Down)),
                    "enter" => (257, BindKey::Menu(SliderKey::Select)),
                    "space" => (32, BindKey::Menu(SliderKey::Select)),
                    "escape" => (256, BindKey::Escape),
                    other => {
                        let code = other.strip_prefix("code:").ok_or("key")?;
                        (code.parse().map_err(|_| "key code")?, BindKey::Other)
                    }
                };
                screen.key_pressed(&lang, InputKey::Keysym(code), kind);
            }
            "scroll" => screen.scrolled(at, step["dy"].as_f64().unwrap_or(0.0)),
            other => return Err(format!("unknown step {other}")),
        }
        last = at;
    }
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        millis: row["ms"].as_u64().unwrap_or(0),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, atlas, font, mouse);
    Ok(json!({
        "label": row["label"],
        "width": window.gui_scaled_width(),
        "height": window.gui_scaled_height(),
        "gui_scale": gui_scale,
        "scroll": screen.scroll_amount(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
