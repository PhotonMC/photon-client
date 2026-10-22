use super::*;

pub(super) fn options_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use photon_gui::options::OptionsScreen;
    use photon_gui::widgets::slider::SliderKey;
    let lang = photon_game::text::Lang::load(root);
    let size = (int("width"), int("height"));
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut fov = int("fov");
    let mut screen = OptionsScreen::new(
        font,
        &lang,
        size,
        fov,
        row["in_world"].as_bool().unwrap_or(false),
    );
    for step in row["steps"].as_array().ok_or("steps")? {
        let at = (
            step["x"].as_f64().unwrap_or(0.0),
            step["y"].as_f64().unwrap_or(0.0),
        );
        let input = match step["do"].as_str().unwrap_or_default() {
            "key" => match step["key"].as_str().unwrap_or_default() {
                "tab" => {
                    screen.tab(true);
                    Default::default()
                }
                "shift+tab" => {
                    screen.tab(false);
                    Default::default()
                }
                "left" => screen.key_pressed(&lang, SliderKey::Left),
                "right" => screen.key_pressed(&lang, SliderKey::Right),
                "up" => screen.key_pressed(&lang, SliderKey::Up),
                "down" => screen.key_pressed(&lang, SliderKey::Down),
                "enter" | "space" => screen.key_pressed(&lang, SliderKey::Select),
                other => return Err(format!("unknown key {other}")),
            },
            "click" => screen.clicked(&lang, at),
            "drag" => screen.dragged(&lang, at.0),
            "release" => {
                screen.released(&lang);
                Default::default()
            }
            other => return Err(format!("unknown step {other}")),
        };
        fov = input.fov.unwrap_or(fov);
    }
    screen.draw(&mut frame, atlas, font, mouse);
    Ok(json!({
        "label": row["label"],
        "fov": fov,
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn video_settings_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use photon_gui::options::video::VideoSettingsScreen;
    use photon_render::window::scale::WindowScale;
    let lang = photon_game::text::Lang::load(root);
    let framebuffer = (
        row["framebuffer"][0].as_i64().unwrap_or(0) as i32,
        row["framebuffer"][1].as_i64().unwrap_or(0) as i32,
    );
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut window = WindowScale::new(framebuffer.0, framebuffer.1);
    let mut gui_scale = int("gui_scale");
    window.set_gui_scale(window.calculate_scale(gui_scale, false));
    let max_scale = window.calculate_scale(0, false);
    let mut screen = VideoSettingsScreen::new(
        font,
        &lang,
        (window.gui_scaled_width(), window.gui_scaled_height()),
        row["in_world"].as_bool().unwrap_or(false),
        gui_scale,
        max_scale,
        super::stored::stored_values(row),
    );
    let mut last = (0.0, 0.0);
    let mut clock = 0_u64;
    for step in row["steps"].as_array().ok_or("steps")? {
        if step["do"] == "frame" {
            clock = step["ms"].as_u64().unwrap_or(0);
            let mut thrown = GuiFrame {
                millis: clock,
                ..GuiFrame::default()
            };
            screen.draw(&mut thrown, atlas, font, mouse);
            continue;
        }
        // The reference draws a frame between one event and the next.
        let mut between = GuiFrame {
            millis: clock,
            ..GuiFrame::default()
        };
        screen.draw(&mut between, atlas, font, mouse);
        let at = (
            step["x"].as_f64().unwrap_or(last.0),
            step["y"].as_f64().unwrap_or(last.1),
        );
        // `Minecraft.resizeGui`: the window takes the scale.
        let mut resize = |new: i32| {
            gui_scale = new;
            window.set_gui_scale(window.calculate_scale(new, false));
            (window.gui_scaled_width(), window.gui_scaled_height())
        };
        match step["do"].as_str().unwrap_or_default() {
            "click" => {
                screen.clicked(
                    &lang,
                    at,
                    step["shift"].as_bool().unwrap_or(false),
                    &mut resize,
                );
            }
            "key" => match menu_key(step)? {
                MenuKey::Tab(forward) => screen.tab(forward),
                MenuKey::Key(key) => {
                    screen.key_pressed(&lang, key, false, &mut resize);
                }
            },
            "drag" => screen.dragged(&lang, at, at.1 - last.1),
            "release" => screen.released(&lang),
            "scroll" => screen.scrolled(
                &lang,
                at,
                step["dy"].as_f64().unwrap_or(0.0),
                step["ctrl"].as_bool().unwrap_or(false),
                &mut resize,
            ),
            other => return Err(format!("unknown step {other}")),
        }
        last = at;
    }
    frame.millis = row["ms"].as_u64().unwrap_or(0);
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

/// The options a settings row sets.
fn row_options(row: &Value) -> Result<photon_game::options::Options, String> {
    let mut options = photon_game::options::Options::default();
    let set = |key: &str, field: &mut f64| {
        if let Some(value) = row["options"][key].as_f64() {
            *field = value;
        }
    };
    set("scale", &mut options.chat_scale);
    set("opacity", &mut options.chat_opacity);
    set("width", &mut options.chat_width);
    set("height_focused", &mut options.chat_height_focused);
    set("height_unfocused", &mut options.chat_height_unfocused);
    set("line_spacing", &mut options.chat_line_spacing);
    set("background_opacity", &mut options.text_background_opacity);
    set("effects", &mut options.screen_effect_scale);
    set("sensitivity", &mut options.sensitivity);
    set("wheel", &mut options.mouse_wheel_sensitivity);
    if let Some(value) = row["options"]["blurriness"].as_i64() {
        options.menu_background_blurriness = value as i32;
    }
    options.invert_mouse_x = row["options"]["invert_x"].as_bool().unwrap_or(false);
    options.invert_mouse_y = row["options"]["invert_y"].as_bool().unwrap_or(false);
    for source in photon_game::sounds::SoundSource::ALL {
        if let Some(volume) = row["options"]["volumes"][source.name()].as_f64() {
            options.sound_volumes[source as usize] = volume;
        }
    }
    options.show_subtitles = row["options"]["subtitles"].as_bool().unwrap_or(false);
    options.directional_audio = row["options"]["directional"].as_bool().unwrap_or(false);
    if let Some(value) = row["options"]["music_frequency"]
        .as_str()
        .and_then(photon_game::options::MusicFrequency::from_serialized)
    {
        options.music_frequency = value;
    }
    if let Some(value) = row["options"]["music_toast"]
        .as_str()
        .and_then(photon_game::options::MusicToast::from_serialized)
    {
        options.music_toast = value;
    }
    if let Some(mask) = row["options"]["model_parts"].as_i64() {
        options.model_parts = mask as i32;
    }
    if let Some(hand) = row["options"]["main_hand"].as_str() {
        options.main_hand = photon_game::options::MainHand::from_serialized(&hand.to_lowercase())
            .ok_or("main_hand")?;
    }
    Ok(options)
}

use photon_gui::options::accessibility::AccessibilityScreen;
use photon_gui::options::chat::ChatOptionsScreen;
use photon_gui::options::font::FontOptionsScreen;
use photon_gui::options::mouse::{ControlsScreen, MouseSettingsScreen};
use photon_gui::options::skin::SkinCustomizationScreen;
use photon_gui::options::sound::SoundOptionsScreen;

// The settings screens that have the same inputs.
enum Shell {
    Chat(ChatOptionsScreen),
    Accessibility(AccessibilityScreen),
    Mouse(MouseSettingsScreen),
    Controls(ControlsScreen),
    Sound(SoundOptionsScreen),
    Skin(SkinCustomizationScreen),
    Font(FontOptionsScreen),
}
macro_rules! each {
    ($shell:expr, $screen:ident => $body:expr) => {
        match $shell {
            Shell::Chat($screen) => $body,
            Shell::Accessibility($screen) => $body,
            Shell::Mouse($screen) => $body,
            Shell::Controls($screen) => $body,
            Shell::Sound($screen) => $body,
            Shell::Skin($screen) => $body,
            Shell::Font($screen) => $body,
        }
    };
}

/// The screen of a settings operation, over the options the row has.
fn settings_shell(
    operation: &str,
    row: &Value,
    (font, lang): (&photon_render::gui::font::Font, &photon_game::text::Lang),
    (size, in_world): ((i32, i32), bool),
) -> Shell {
    let options = row_options(row).unwrap_or_default();
    match operation {
        "skin_customization_screen" => Shell::Skin(SkinCustomizationScreen::new(
            font, lang, size, in_world, &options,
        )),
        "font_options_screen" => {
            Shell::Font(FontOptionsScreen::new(font, lang, size, in_world, &options))
        }
        "chat_options_screen" => {
            Shell::Chat(ChatOptionsScreen::new(font, lang, size, in_world, &options))
        }
        "accessibility_options_screen" => Shell::Accessibility(AccessibilityScreen::new(
            font,
            lang,
            size,
            in_world,
            (
                &options,
                row["options"]["minecart"].as_bool().unwrap_or(false),
            ),
        )),
        "mouse_settings_screen" => Shell::Mouse(MouseSettingsScreen::new(
            font, lang, size, in_world, &options,
        )),
        "sound_options_screen" => {
            Shell::Sound(sound_screen(font, lang, (size, in_world), &options))
        }
        _ => Shell::Controls(ControlsScreen::new(font, lang, size, in_world)),
    }
}

pub(super) fn chat_options_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
    operation: &str,
) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use photon_render::window::scale::WindowScale;
    let lang = photon_game::text::Lang::load(root);
    let framebuffer = (
        row["framebuffer"][0].as_i64().unwrap_or(0) as i32,
        row["framebuffer"][1].as_i64().unwrap_or(0) as i32,
    );
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut window = WindowScale::new(framebuffer.0, framebuffer.1);
    window.set_gui_scale(window.calculate_scale(int("gui_scale"), false));
    let size = (window.gui_scaled_width(), window.gui_scaled_height());
    let in_world = row["in_world"].as_bool().unwrap_or(false);
    let mut screen = settings_shell(operation, row, (font, &lang), (size, in_world));
    let mut last = (0.0, 0.0);
    let mut clock = 0_u64;
    for step in row["steps"].as_array().ok_or("steps")? {
        if step["do"] == "frame" {
            clock = step["ms"].as_u64().unwrap_or(0);
            let mut thrown = GuiFrame {
                millis: clock,
                ..GuiFrame::default()
            };
            each!(&screen, s => s.draw(&mut thrown, atlas, font, mouse));
            continue;
        }
        // The reference draws a frame between one event and the next.
        let mut between = GuiFrame {
            millis: clock,
            ..GuiFrame::default()
        };
        each!(&screen, s => s.draw(&mut between, atlas, font, mouse));
        let at = (
            step["x"].as_f64().unwrap_or(last.0),
            step["y"].as_f64().unwrap_or(last.1),
        );
        match step["do"].as_str().unwrap_or_default() {
            "click" => {
                let shift = step["shift"].as_bool().unwrap_or(false);
                each!(&mut screen, s => { s.clicked(&lang, at, shift); });
            }
            "key" => match menu_key(step)? {
                MenuKey::Tab(forward) => each!(&mut screen, s => s.tab(forward)),
                MenuKey::Key(key) => {
                    each!(&mut screen, s => { s.key_pressed(&lang, key, false); });
                }
            },
            "drag" => each!(&mut screen, s => s.dragged(&lang, at, at.1 - last.1)),
            "release" => each!(&mut screen, s => s.released(&lang)),
            "scroll" => {
                let dy = step["dy"].as_f64().unwrap_or(0.0);
                each!(&mut screen, s => s.scrolled(at, dy));
            }
            other => return Err(format!("unknown step {other}")),
        }
        last = at;
    }
    frame.millis = row["ms"].as_u64().unwrap_or(0);
    each!(&screen, s => s.draw(&mut frame, atlas, font, mouse));
    Ok(json!({
        "label": row["label"],
        "width": window.gui_scaled_width(),
        "height": window.gui_scaled_height(),
        "gui_scale": int("gui_scale"),
        "scroll": each!(&screen, s => s.scroll_amount()),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn lang_text_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let languages = &mut ctx.languages;
    let source = match &languages {
        Some(source) => source,
        None => {
            languages.insert(photon_game::language::LanguageSource::open(root).ok_or("languages")?)
        }
    };
    let lang = source.lang(row["language"].as_str().unwrap_or("en_us"));
    let keys = row["keys"].as_array().ok_or("keys")?;
    Ok(json!({
        "label": row["label"],
        "texts": keys.iter().map(|key| lang.text(key.as_str().unwrap_or_default())).collect::<Vec<_>>(),
    }))
}

pub(super) fn language_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let languages = &mut ctx.languages;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use photon_gui::options::language::LanguageSelectScreen;
    use photon_render::window::scale::WindowScale;
    struct NoClipboard;
    impl photon_gui::widgets::edit_box::Clipboard for NoClipboard {
        fn get(&mut self) -> String {
            String::new()
        }
        fn set(&mut self, _text: &str) {}
    }
    let source = match &languages {
        Some(source) => source,
        None => {
            languages.insert(photon_game::language::LanguageSource::open(root).ok_or("languages")?)
        }
    };
    let code = row["language"].as_str().unwrap_or("en_us");
    let lang = source.lang(code);
    let framebuffer = (
        row["framebuffer"][0].as_i64().unwrap_or(0) as i32,
        row["framebuffer"][1].as_i64().unwrap_or(0) as i32,
    );
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut window = WindowScale::new(framebuffer.0, framebuffer.1);
    window.set_gui_scale(window.calculate_scale(int("gui_scale"), false));
    let mut screen = LanguageSelectScreen::new(
        font,
        &lang,
        (window.gui_scaled_width(), window.gui_scaled_height()),
        row["in_world"].as_bool().unwrap_or(false),
        source
            .languages()
            .into_iter()
            .filter(|(code, _)| {
                row["codes"].as_array().is_none_or(|codes| {
                    codes.is_empty() || codes.iter().any(|listed| listed == code.as_str())
                })
            })
            .collect(),
        code,
    );
    let mut last = (0.0, 0.0);
    for step in row["steps"].as_array().ok_or("steps")? {
        let at = (
            step["x"].as_f64().unwrap_or(last.0),
            step["y"].as_f64().unwrap_or(last.1),
        );
        match step["do"].as_str().unwrap_or_default() {
            "click" => {
                screen.clicked(font, at, step["double"].as_bool().unwrap_or(false));
            }
            "char" => {
                if let Some(character) = step["char"].as_str().and_then(|text| text.chars().next())
                {
                    screen.char_typed(font, character);
                }
            }
            "key" => {
                let name = step["key"].as_str().unwrap_or_default();
                let key = match name {
                    "backspace" => glfw::Key::Backspace,
                    "enter" => glfw::Key::Enter,
                    "space" => glfw::Key::Space,
                    "tab" | "shift+tab" => glfw::Key::Tab,
                    "left" => glfw::Key::Left,
                    "right" => glfw::Key::Right,
                    "up" => glfw::Key::Up,
                    "down" => glfw::Key::Down,
                    other => return Err(format!("unknown key {other}")),
                };
                let shift = name.starts_with("shift+");
                screen.key_pressed(font, key, (false, shift), &mut NoClipboard);
            }
            "drag" => screen.dragged(at, at.1 - last.1),
            "release" => screen.released(),
            "scroll" => screen.scrolled(at, step["dy"].as_f64().unwrap_or(0.0)),
            other => return Err(format!("unknown step {other}")),
        }
        last = at;
    }
    screen.draw(&mut frame, atlas, font, mouse);
    Ok(json!({
        "label": row["label"],
        "width": window.gui_scaled_width(),
        "height": window.gui_scaled_height(),
        "gui_scale": int("gui_scale"),
        "scroll": screen.scroll_amount(),
        "search": screen.search_text(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// The sound options screen, offering only the device the options name.
fn sound_screen(
    font: &photon_render::gui::font::Font,
    lang: &photon_game::text::Lang,
    (size, in_world): ((i32, i32), bool),
    options: &photon_game::options::Options,
) -> photon_gui::options::sound::SoundOptionsScreen {
    use photon_gui::options::sound::SoundOptionsScreen;
    let devices = vec![options.sound_device.clone()];
    SoundOptionsScreen::new(font, lang, size, in_world, (options, devices))
}
