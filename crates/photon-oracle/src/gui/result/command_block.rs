use super::*;
use photon_game::text::Lang;
use photon_gui::screen::command_block::{CommandBlockAction, CommandBlockScreen};
use photon_world::CommandBlockMode;

fn mode_of(name: &str) -> CommandBlockMode {
    match name {
        "sequence" => CommandBlockMode::Sequence,
        "auto" => CommandBlockMode::Auto,
        _ => CommandBlockMode::Redstone,
    }
}

/// The keys a row may name.
const KEYS: [(&str, glfw::Key); 13] = [
    ("tab", glfw::Key::Tab),
    ("backspace", glfw::Key::Backspace),
    ("delete", glfw::Key::Delete),
    ("enter", glfw::Key::Enter),
    ("escape", glfw::Key::Escape),
    ("space", glfw::Key::Space),
    ("left", glfw::Key::Left),
    ("right", glfw::Key::Right),
    ("up", glfw::Key::Up),
    ("down", glfw::Key::Down),
    ("home", glfw::Key::Home),
    ("end", glfw::Key::End),
    ("a", glfw::Key::A),
];

fn key_of(name: &str) -> Result<glfw::Key, String> {
    KEYS.iter()
        .find(|(key, _)| *key == name)
        .map(|(_, key)| *key)
        .ok_or_else(|| format!("unknown key {name}"))
}

/// The commands the server has told of: none yet, as the reference's empty dispatcher.
fn no_commands() -> Result<std::sync::Arc<photon_game::commands::CommandTree>, String> {
    use photon_game::commands::{CommandTree, NodeKind, NodeStub};
    let root = NodeStub {
        kind: NodeKind::Root,
        children: Vec::new(),
        redirect: None,
        executable: false,
        custom_suggestions: false,
        restricted: false,
        type_id: 0,
    };
    CommandTree::build(vec![root], 0)
        .map(std::sync::Arc::new)
        .ok_or_else(|| "commands".to_owned())
}

/// What the screen did so far: whether it closed and what it sent.
#[derive(Default)]
struct Outcome {
    closed: bool,
    packets: Vec<Value>,
    /// The minecart whose screen it is, if it is one's.
    minecart: Option<i64>,
}

impl Outcome {
    fn settle(&mut self, action: CommandBlockAction) {
        match action {
            CommandBlockAction::Stay => {}
            CommandBlockAction::Close => self.closed = true,
            CommandBlockAction::Done(edit) => {
                self.closed = true;
                self.packets.push(self.packet(&edit));
            }
        }
    }

    /// `ServerboundSetCommandBlockPacket` or `ServerboundSetCommandMinecartPacket`.
    fn packet(&self, edit: &photon_gui::screen::command_block::CommandBlockEdit) -> Value {
        let mut packet = json!({"command": edit.command, "track_output": edit.track_output});
        if let Some(entity) = self.minecart {
            packet["entity"] = json!(entity);
        }
        if let Some(block) = &edit.block {
            packet["mode"] = json!(format!("{:?}", block.mode).to_lowercase());
            packet["conditional"] = json!(block.conditional);
            packet["automatic"] = json!(block.automatic);
        }
        packet
    }
}

/// The block's data as the screen is told of it.
struct Block {
    contents: photon_game::command_block::CommandBlockContents,
    mode: (CommandBlockMode, bool),
}

fn block_of(row: &Value) -> Block {
    Block {
        contents: photon_game::command_block::CommandBlockContents {
            command: row["command"].as_str().unwrap_or_default().to_owned(),
            track_output: row["track_output"].as_bool() == Some(true),
            last_output: row["last_output"]
                .as_str()
                .map(photon_game::text::Component::literal),
            automatic: row["automatic"].as_bool() == Some(true),
        },
        mode: (
            mode_of(row["mode"].as_str().unwrap_or_default()),
            row["conditional"].as_bool() == Some(true),
        ),
    }
}

/// One step of a row: text, a key, a click, the wheel, the block's data or a new size.
fn step(
    screen: &mut CommandBlockScreen,
    (lang, block): (&Lang, &Block),
    step: &Value,
    outcome: &mut Outcome,
) -> Result<(), String> {
    let at = (
        step["x"].as_f64().unwrap_or(0.0) as i32,
        step["y"].as_f64().unwrap_or(0.0) as i32,
    );
    match step["do"].as_str() {
        Some("type") => {
            for character in step["text"].as_str().unwrap_or_default().chars() {
                screen.char_typed(character);
            }
        }
        Some("key") => {
            let name = step["key"].as_str().unwrap_or_default();
            let control = name.starts_with("ctrl+");
            let name = name.trim_start_matches("ctrl+");
            let shift = name.starts_with("shift+");
            let key = key_of(name.trim_start_matches("shift+"))?;
            let action =
                screen.key_pressed(lang, key, (control, shift), &mut crate::gui::NoClipboard);
            outcome.settle(action);
        }
        Some("click") => {
            let shift = step["shift"].as_bool() == Some(true);
            outcome.settle(screen.clicked(lang, at, shift));
        }
        Some("wheel") => screen.scrolled(lang, step["amount"].as_f64().unwrap_or(0.0), at),
        Some("update") => screen.update_gui(lang, block.mode, &block.contents),
        Some("resize") => {
            let size = |key: &str| step[key].as_i64().unwrap_or(0) as i32;
            screen.resize(lang, (size("width"), size("height")));
        }
        other => return Err(format!("unknown step {other:?}")),
    }
    Ok(())
}

/// `command_block_screen`: the command block's screen after the block's data arrives, text, keys
/// and clicks, drawn at a time after it opened, whether it closed and the packets it would send.
pub(super) fn command_block_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    use photon_gui::screen::command_block::SuggestionSources;
    let lang = Lang::load(&ctx.root);
    let font = std::sync::Arc::new(Font::load(&ctx.root).ok_or("font")?);
    let int = |value: &Value| value.as_i64().unwrap_or(0) as i32;
    let block = block_of(row);
    let commands = no_commands()?;
    let size = (int(&row["width"]), int(&row["height"]));
    let mut outcome = Outcome::default();
    let mut screen = if row["minecart"].as_bool() == Some(true) {
        outcome.minecart = row["entity_id"].as_i64();
        CommandBlockScreen::minecart((&font, &lang), &block.contents, size)
    } else {
        CommandBlockScreen::new((&font, &lang), &block.contents, size)
    };
    screen.set_suggestion_sources(SuggestionSources {
        commands: Some(commands.clone()),
        selectors: true,
        ..SuggestionSources::default()
    });
    photon_gui::ui_sound::take();
    for each in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        step(&mut screen, (&lang, &block), each, &mut outcome)?;
    }
    let sounds = super::screens::asked_sounds();
    let mouse = (int(&row["mouse"][0]), int(&row["mouse"][1]));
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        millis: row["ms"].as_u64().unwrap_or(0),
        ..GuiFrame::default()
    };
    screen.draw(
        (&mut frame, &ctx.atlas, &ctx.font),
        mouse,
        (Some(&commands), &lang),
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        "closed": outcome.closed,
        "sounds": sounds,
        "packets": outcome.packets,
    }))
}
