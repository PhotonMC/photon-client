//! `tutorial`: the tutorial's steps through scripts of events, answered with the step the options
//! keep and the tutorial toasts the manager holds.

use super::*;
use photon_game::options::TutorialStep;
use photon_gui::toasts::{Toast, ToastEnv, ToastManager, TutorialToast, TutorialWords};
use photon_gui::tutorial::{Command, Facts, ItemKind, Keys, Tutorial};
use photon_world::block::tags::ItemTags;

const FIND_TREE: &str = "minecraft:completes_find_tree_tutorial";

/// What a row says of the player, and the tags the data pack gives items and blocks.
struct Setting {
    inventory: Vec<i32>,
    mined: Vec<i32>,
    crafted: Vec<i32>,
    survival: bool,
    items: ItemTags,
    blocks: ItemTags,
}

impl Setting {
    fn holds(&self, items: &[i32], tag: &str) -> bool {
        items.iter().any(|item| self.items.contains(*item, tag))
    }
}

impl Facts for Setting {
    fn survival(&self) -> bool {
        self.survival
    }

    fn has_logs(&self) -> bool {
        self.holds(&self.inventory, "minecraft:logs")
    }

    fn has_tree_item(&self) -> bool {
        self.holds(&self.inventory, FIND_TREE)
    }

    fn has_planks(&self) -> bool {
        self.holds(&self.inventory, "minecraft:planks")
    }

    fn mined_trees_before(&self) -> bool {
        self.mined
            .iter()
            .any(|block| self.blocks.contains(*block, FIND_TREE))
    }

    fn crafted_planks_before(&self) -> bool {
        self.holds(&self.crafted, "minecraft:planks")
    }
}

fn block_id(name: &str) -> Result<i32, String> {
    crate::case_ids::blocks()
        .position(|block| block.name == name)
        .map(|index| index as i32)
        .ok_or_else(|| format!("unknown block {name}"))
}

fn item_id(name: &str) -> Result<i32, String> {
    photon_game::item::id_of(name)
        .map(|id| id.0)
        .ok_or_else(|| format!("unknown item {name}"))
}

fn names<'a>(row: &'a Value, key: &str) -> Result<Vec<&'a str>, String> {
    row[key]
        .as_array()
        .ok_or_else(|| format!("{key} must be an array"))?
        .iter()
        .map(|name| {
            name.as_str()
                .ok_or_else(|| format!("{key} must hold names"))
        })
        .collect()
}

fn setting(row: &Value) -> Result<Setting, String> {
    let blocks = crate::vanilla::vanilla_block_tags()?;
    Ok(Setting {
        inventory: names(row, "inventory")?
            .into_iter()
            .map(item_id)
            .collect::<Result<_, _>>()?,
        mined: names(row, "mined")?
            .into_iter()
            .map(block_id)
            .collect::<Result<_, _>>()?,
        crafted: names(row, "crafted")?
            .into_iter()
            .map(item_id)
            .collect::<Result<_, _>>()?,
        survival: row["mode"] == "SURVIVAL",
        items: crate::vanilla::vanilla_item_tags()?,
        blocks: ItemTags::of_registry(
            &blocks.registries,
            "minecraft:block",
            Some(&crate::case_ids::registries().blocks),
        )
        .ok_or("no block tags")?,
    })
}

fn keys_of(event: &Value) -> Keys {
    let held = |name: &str| {
        event["keys"]
            .as_array()
            .is_some_and(|keys| keys.iter().any(|key| key == name))
    };
    Keys {
        forward: held("forward"),
        backward: held("backward"),
        left: held("left"),
        right: held("right"),
        jump: held("jump"),
    }
}

/// What the game does with the tutorial's commands: toasts shown, hidden and moved on.
fn apply(manager: &mut ToastManager, env: &ToastEnv<'_>, commands: Vec<Command>) {
    use photon_game::text::Component;
    for command in commands {
        match command {
            Command::Show(spec) => {
                let title = Component::translatable(spec.title.key);
                let message = spec.message.map(|words| Component::translatable(words.key));
                let words = TutorialWords {
                    title: &title,
                    message: message.as_ref(),
                    progressable: spec.progressable,
                    time_to_display_ms: 0,
                };
                manager.add(Box::new(TutorialToast::new(env, spec.icon, &words)));
            }
            Command::Hide(icon) => {
                if let Some(toast) = manager.tutorial_mut(icon) {
                    toast.hide();
                }
            }
            Command::Progress(icon, progress) => {
                if let Some(toast) = manager.tutorial_mut(icon) {
                    toast.update_progress(progress);
                }
            }
            Command::SetStep(_) => {}
        }
    }
}

/// Ticks, each with the mouse moves and the keys of the event.
fn run_ticks(
    (tutorial, manager): (&mut Tutorial, &mut ToastManager),
    (setting, env): (&Setting, &ToastEnv<'_>),
    event: &Value,
) -> Result<(), String> {
    for _ in 0..event["n"].as_i64().ok_or("n")? {
        for turn in event["mouse"].as_array().ok_or("mouse")? {
            tutorial.on_mouse((turn[0].as_f64().ok_or("x")?, turn[1].as_f64().ok_or("y")?));
        }
        tutorial.on_input(keys_of(event));
        apply(manager, env, tutorial.tick(setting));
    }
    Ok(())
}

/// The events about the world and the inventory, and the commands they gave.
fn world_event(
    tutorial: &mut Tutorial,
    setting: &Setting,
    event: &Value,
) -> Result<Vec<Command>, String> {
    let text = |key: &str| {
        event[key]
            .as_str()
            .ok_or_else(|| format!("{key} must be a name"))
    };
    Ok(match event["do"].as_str().ok_or("do")? {
        "look" => {
            let block = block_id(text("block")?)?;
            tutorial.on_look_at_block(setting.blocks.contains(block, FIND_TREE))
        }
        "destroy" => {
            let block = block_id(text("block")?)?;
            let percent = event["percent"].as_f64().ok_or("percent")? as f32;
            tutorial.on_destroy_block(setting.blocks.contains(block, "minecraft:logs"), percent)
        }
        "inventory" => tutorial.on_open_inventory(),
        "get_item" => {
            let item = item_id(text("item")?)?;
            tutorial.on_get_item(ItemKind {
                log: setting.items.contains(item, "minecraft:logs"),
                tree_item: setting.items.contains(item, FIND_TREE),
                planks: setting.items.contains(item, "minecraft:planks"),
            })
        }
        "stop" => tutorial.stop(),
        other => return Err(format!("unknown tutorial event {other}")),
    })
}

/// One event of a script.
fn run_event(
    (tutorial, manager): (&mut Tutorial, &mut ToastManager),
    (setting, env): (&mut Setting, &ToastEnv<'_>),
    event: &Value,
) -> Result<Vec<&'static str>, String> {
    match event["do"].as_str().ok_or("do")? {
        "tick" => run_ticks((tutorial, manager), (setting, env), event)?,
        "mode" => setting.survival = event["value"] == "SURVIVAL",
        "update" => {
            return Ok(manager.update((event["at"].as_i64().ok_or("at")?, 0), env));
        }
        _ => apply(manager, env, world_event(tutorial, setting, event)?),
    }
    Ok(Vec::new())
}

fn toasts(manager: &ToastManager) -> Vec<Value> {
    manager
        .tutorial_toasts()
        .into_iter()
        .map(|(waiting, toast)| {
            json!({
                "waiting": waiting,
                "icon": toast.icon() as usize,
                "hidden": toast.wanted_visibility() == photon_gui::toasts::Visibility::Hide,
                "progress": format!("{:08x}", toast.progress().to_bits()),
                "progressable": toast.progressable(),
            })
        })
        .collect()
}

/// `toasts` and `tutorial`, which both drive the toast manager.
pub(super) fn toast_case(
    ctx: &mut GuiContext,
    row: &Value,
    operation: &str,
) -> Result<Value, String> {
    if operation == "tutorial" {
        tutorial_case(ctx, row)
    } else {
        super::hud_overlays::toasts_case(ctx, row)
    }
}

fn tutorial_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let lang = photon_game::text::Lang::load(&ctx.root);
    let env = ToastEnv {
        font: &ctx.font,
        lang: &lang,
        display_time: 1.0,
        current_song: None,
        note_colour: 0xFFFF_FFFF,
    };
    let mut setting = setting(row)?;
    let mut tutorial = Tutorial::new(TutorialStep::by_name(row["step"].as_str().ok_or("step")?));
    let mut manager = ToastManager::new(photon_gui::toasts::MusicToast::Never);
    let mut events = Vec::new();
    for event in row["events"].as_array().ok_or("events")? {
        let sounds = run_event((&mut tutorial, &mut manager), (&mut setting, &env), event)?;
        events.push(json!({
            "do": event["do"],
            "step": tutorial.step().name(),
            "sounds": sounds,
            "toasts": toasts(&manager),
        }));
    }
    Ok(json!({"label": row["label"], "events": events}))
}
