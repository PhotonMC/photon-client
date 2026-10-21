use super::*;

/// `hud_sleep`: the veil over the view for a sleep timer.
pub(super) fn hud_sleep_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    photon_gui::hud::sleep::draw(
        &mut frame,
        &ctx.atlas,
        (int("width"), int("height")),
        int("timer"),
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// `subtitles`: the sounds heard at their times, the overlay drawn from a listener's place.
pub(super) fn subtitles_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_core::math::Vec3;
    use photon_gui::hud::subtitles::{Listener, Subtitles};
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let vec = |value: &Value| -> Result<Vec3, String> {
        let part = |index: usize| value[index].as_f64().ok_or("coordinate");
        Ok(Vec3::new(part(0)?, part(1)?, part(2)?))
    };
    let lang = photon_game::text::Lang::load(&ctx.root);
    let mut subtitles = Subtitles::default();
    for heard in row["heard"].as_array().ok_or("heard")? {
        let place = vec(&heard["position"])?;
        let range = if heard["infinite"].as_bool().unwrap_or(false) {
            f32::INFINITY
        } else {
            heard["range"].as_f64().ok_or("range")? as f32
        };
        subtitles.heard(
            heard["key"].as_str().ok_or("key")?,
            (place.x, place.y, place.z),
            range,
            heard["at"].as_u64().ok_or("at")?,
        );
    }
    let (forward, up) = (vec(&row["forward"])?, vec(&row["up"])?);
    let listener = Listener::new(vec(&row["position"])?, forward, up);
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    // `Options.getBackgroundColor(0.8F)` with the background for the chat only: that opacity.
    let background = ((0.8_f32 * 255.0) as u32) << 24;
    subtitles.draw(
        &mut frame,
        (&ctx.atlas, &ctx.font, &lang),
        (int("width"), int("height")),
        listener,
        row["now"].as_u64().ok_or("now")?,
        (
            row["display_time"].as_f64().ok_or("display_time")?,
            background,
        ),
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// The id a Java field name (`SystemToastId`) says.
fn toast_id(name: &str) -> photon_gui::toasts::SystemToastId {
    use photon_gui::toasts::SystemToastId as Id;
    const IDS: [(&str, Id); 11] = [
        ("NARRATOR_TOGGLE", Id::NarratorToggle),
        ("WORLD_BACKUP", Id::WorldBackup),
        ("PACK_LOAD_FAILURE", Id::PackLoadFailure),
        ("WORLD_ACCESS_FAILURE", Id::WorldAccessFailure),
        ("PACK_COPY_FAILURE", Id::PackCopyFailure),
        ("FILE_DROP_FAILURE", Id::FileDropFailure),
        ("PERIODIC_NOTIFICATION", Id::PeriodicNotification),
        ("LOW_DISK_SPACE", Id::LowDiskSpace),
        ("CHUNK_LOAD_FAILURE", Id::ChunkLoadFailure),
        ("CHUNK_SAVE_FAILURE", Id::ChunkSaveFailure),
        ("UNSECURE_SERVER_WARNING", Id::UnsecureServerWarning),
    ];
    IDS.iter()
        .find(|(own, _)| *own == name)
        .map_or(Id::FriendSystemNotification, |(_, id)| *id)
}

/// The title and the message of a toast event, as literal text.
fn toast_words(
    event: &Value,
) -> (
    photon_game::text::Component,
    Option<photon_game::text::Component>,
) {
    use photon_game::text::Component;
    (
        Component::literal(event["title"].as_str().unwrap_or_default()),
        event["message"].as_str().map(Component::literal),
    )
}

/// What a row of the `toasts` case fixes for the whole script.
struct ToastRow {
    music: photon_gui::toasts::MusicToast,
    width: i32,
    hidden: bool,
    pause_screen: bool,
}

/// An advancement toast event: the toast of the advancement the event holds.
fn add_advancement_toast(
    manager: &mut photon_gui::toasts::ToastManager,
    event: &Value,
    env: &photon_gui::toasts::ToastEnv<'_>,
) -> Result<(), String> {
    let shown = super::stats::advancement_from(&event["advancement"])?;
    let display = shown.display.as_ref().ok_or("display")?;
    manager.add(Box::new(photon_gui::toasts::AdvancementToast::new(
        display, env,
    )));
    Ok(())
}

/// A tutorial toast event: the toast is made and kept for the events that follow, which move its
/// progress or hide it.
fn tutorial_event(
    manager: &mut photon_gui::toasts::ToastManager,
    event: &Value,
    env: &photon_gui::toasts::ToastEnv<'_>,
) -> Result<(), String> {
    use photon_game::text::Component;
    use photon_gui::toasts::{TutorialIcon, TutorialToast, TutorialWords};
    let icon = [
        TutorialIcon::MovementKeys,
        TutorialIcon::Mouse,
        TutorialIcon::Tree,
        TutorialIcon::RecipeBook,
        TutorialIcon::WoodenPlanks,
        TutorialIcon::SocialInteractions,
        TutorialIcon::RightClick,
    ][event["icon"].as_u64().ok_or("icon")? as usize];
    match event["do"].as_str().unwrap_or_default() {
        "tutorial" => {
            let title = Component::literal(event["title"].as_str().ok_or("title")?);
            let message = event["message"].as_str().map(Component::literal);
            let words = TutorialWords {
                title: &title,
                message: message.as_ref(),
                progressable: event["progressable"].as_bool().ok_or("progressable")?,
                time_to_display_ms: event["time_ms"].as_i64().ok_or("time_ms")? as i32,
            };
            manager.add(Box::new(TutorialToast::new(env, icon, &words)));
        }
        "tutorial_progress" => {
            let value = event["value"].as_f64().ok_or("value")? as f32;
            manager
                .tutorial_mut(icon)
                .ok_or("no tutorial toast")?
                .update_progress(value);
        }
        _ => manager
            .tutorial_mut(icon)
            .ok_or("no tutorial toast")?
            .hide(),
    }
    Ok(())
}

/// The toast a packet asks for: an advancement's or a recipe's.
fn add_packet_toast(
    manager: &mut photon_gui::toasts::ToastManager,
    event: &Value,
    env: &photon_gui::toasts::ToastEnv<'_>,
) -> Result<(), String> {
    match event["do"].as_str().unwrap_or_default() {
        "recipe" => {
            add_recipe_toast(manager, event, env);
            Ok(())
        }
        "advancement" => add_advancement_toast(manager, event, env),
        _ => tutorial_event(manager, event, env),
    }
}

/// A recipe toast event: the item a recipe's station and result name, or none. The oracle
/// draws no items, so any item stands for the named one.
fn add_recipe_toast(
    manager: &mut photon_gui::toasts::ToastManager,
    event: &Value,
    env: &photon_gui::toasts::ToastEnv<'_>,
) {
    let item = |key: &str| (!event[key].is_null()).then_some(photon_core::ids::ItemId(1));
    manager.add_or_update_recipe(env, (item("station"), item("result")));
}

/// The recipe the recipe toast shows, if there is one; the reference's JSON leaves out a null.
fn note_recipe_index(entry: &mut Value, manager: &mut photon_gui::toasts::ToastManager) {
    if let Some(index) = manager.recipe_index() {
        entry["recipe_index"] = json!(index);
    }
}

/// One toast event: the manager's answer, with the sounds it made and what it drew.
fn toast_event(
    manager: &mut photon_gui::toasts::ToastManager,
    event: &Value,
    (ctx, env, row): (&GuiContext, &photon_gui::toasts::ToastEnv<'_>, &ToastRow),
) -> Result<Value, String> {
    use photon_gui::toasts::SystemToast;
    let mut entry = json!({"do": event["do"]});
    let mut sounds: Vec<&str> = Vec::new();
    let id = toast_id(event["id"].as_str().unwrap_or_default());
    match event["do"].as_str().unwrap_or_default() {
        "system" => {
            let (title, message) = toast_words(event);
            manager.add_system(SystemToast::new(id, env, &title, message.as_ref()));
        }
        "system_update" => {
            let (title, message) = toast_words(event);
            manager.add_or_update_system(id, env, (&title, message.as_ref()));
        }
        "force_hide" => manager.force_hide_system(id),
        "advancement" | "recipe" | "tutorial" | "tutorial_progress" | "tutorial_hide" => {
            add_packet_toast(manager, event, env)?;
        }
        "note_colours" => {
            let ticks = event["ticks"].as_array().ok_or("ticks")?;
            let colours: Vec<i32> = ticks
                .iter()
                .map(|tick| {
                    photon_gui::toasts::music_note_colour(tick.as_f64().unwrap_or(0.0) as f32)
                        as i32
                })
                .collect();
            entry["colours"] = json!(colours);
        }
        "show_now_playing" => manager.show_now_playing(env.display_time),
        "hide_now_playing" => manager.hide_now_playing(),
        "update" => sounds = manager.update((event["at"].as_i64().ok_or("at")?, 0), env),
        "draw" => {
            let mut frame = GuiFrame {
                log: Some(Vec::new()),
                ..GuiFrame::default()
            };
            let shown = (row.hidden, row.music, row.pause_screen);
            let mut no_items = super::no_items;
            manager.draw(
                (&mut frame, &mut no_items),
                &ctx.atlas,
                row.width,
                shown,
                env,
            );
            entry["ops"] = json!(gui_ops(&frame.log.take().unwrap_or_default()));
        }
        other => return Err(format!("unknown toast event {other}")),
    }
    entry["sounds"] = json!(sounds);
    note_recipe_index(&mut entry, manager);
    Ok(entry)
}

/// `toasts`: a script of toast events, each answered with the sounds it made and what it drew.
pub(super) fn toasts_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_gui::toasts::{MusicToast, ToastEnv, ToastManager};
    let lang = photon_game::text::Lang::load(&ctx.root);
    let music = match row["music_toast"].as_str().ok_or("music_toast")? {
        "NEVER" => MusicToast::Never,
        "PAUSE" => MusicToast::Pause,
        _ => MusicToast::PauseAndToast,
    };
    let setup = ToastRow {
        music,
        width: row["width"].as_i64().ok_or("width")? as i32,
        hidden: row["hud_hidden"].as_bool().ok_or("hud_hidden")?,
        pause_screen: row["pause_screen"].as_bool().ok_or("pause_screen")?,
    };
    let song = row["song"].as_str();
    let env = ToastEnv {
        font: &ctx.font,
        lang: &lang,
        display_time: row["display_time"].as_f64().ok_or("display_time")?,
        current_song: song.map(|song| song.strip_prefix("minecraft:").unwrap_or(song)),
        note_colour: 0xFFFF_FFFF,
    };
    let mut manager = ToastManager::new(music);
    manager.notes.last_change = i64::MAX / 2;
    let mut events = Vec::new();
    for event in row["events"].as_array().ok_or("events")? {
        events.push(toast_event(&mut manager, event, (ctx, &env, &setup))?);
    }
    Ok(json!({"label": row["label"], "events": events}))
}

/// What `simple_performance_impactors` reads, from a row's `impactors`.
fn debug_impactors(each: &Value) -> photon_gui::hud::debug::Impactors {
    use photon_gui::hud::debug::{Clouds, Filtering, Impactors};
    let number = |key: &str| each[key].as_i64().unwrap_or(0) as i32;
    Impactors {
        improved_transparency: each["transparency"].as_bool().unwrap_or(false),
        clouds: [Clouds::Off, Clouds::Fast, Clouds::Fancy][number("clouds") as usize],
        biome_blend: number("blend"),
        filtering: [Filtering::None, Filtering::Rgss, Filtering::Anisotropic]
            [number("filtering") as usize],
        // `Options.maxAnisotropyValue`.
        anisotropy: (1 << number("bit")).min(number("limit")),
    }
}

/// What `system_specs` reads, from a row's `system`.
fn debug_system(each: &Value) -> photon_gui::hud::debug::SystemSpecs {
    use photon_gui::hud::debug::{DeviceType, SystemSpecs};
    let text = |key: &str| each[key].as_str().unwrap_or_default().to_owned();
    let number = |key: &str| each[key].as_i64().unwrap_or(0) as i32;
    SystemSpecs {
        runtime: ("Java".to_owned(), text("runtime")),
        cpu: text("cpu"),
        display: (number("width"), number("height")),
        vendor: text("vendor"),
        device_name: text("name"),
        device_type: DeviceType::from_vulkan(number("type")),
        backend: text("backend"),
        driver_info: text("driver"),
    }
}

/// The block a row put in the view: its place and a state such as `minecraft:oak_stairs[half=top]`.
fn debug_target(row: &Value) -> Option<photon_gui::hud::debug::TargetedBlock> {
    let target = row.get("target")?;
    let coordinate = |key: &str| target[key].as_i64().map(|value| value as i32);
    let position = (coordinate("x")?, coordinate("y")?, coordinate("z")?);
    let text = target["state"].as_str()?;
    let (name, properties) = text.split_once('[').unwrap_or((text, "]"));
    let chosen = properties
        .trim_end_matches(']')
        .split(',')
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect();
    let block = photon_data::block::registry::block(name)?;
    photon_gui::hud::debug::TargetedBlock::of(position, block.state_id(&chosen)?)
}

/// What the entries read, from a `hud_debug_overlay` row.
fn debug_input(row: &Value) -> Result<photon_gui::hud::debug::Input, String> {
    use photon_gui::hud::debug::{self, CameraEntity, FrameStats, Input, PresentMode, ServerInfo};
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let flag = |key: &str| row[key].as_bool().unwrap_or(false);
    let launched = row["launched"].as_str().ok_or("launched")?;
    Ok(Input {
        impactors: row.get("impactors").map(debug_impactors),
        system: row.get("system").map(debug_system),
        // The game's own memory is the machine's, which a row cannot pin.
        memory: None,
        gpu_utilization: row
            .get("gpu")
            .and_then(|_| crate::f64_field(row, "gpu").ok()),
        day_count: row
            .get("day")
            .and_then(Value::as_i64)
            .map(|ticks| (ticks / 24000) as i32),
        version: debug::version_line("26.2", launched),
        frame: FrameStats {
            fps: int("fps"),
            // `FramerateLimitTracker.getFramerateLimit`: a minimised window draws ten.
            limit: if flag("iconified") { 10 } else { int("limit") },
            present_mode: match row["present"].as_str().ok_or("present")? {
                "IMMEDIATE" => Some(PresentMode::Immediate),
                "MAILBOX" => Some(PresentMode::Mailbox),
                "FIFO" => Some(PresentMode::Fifo),
                "FIFO_RELAXED" => Some(PresentMode::FifoRelaxed),
                _ => None,
            },
        },
        server: Some(ServerInfo {
            brand: row["brand"].as_str().map(str::to_owned),
            stepping: flag("stepping"),
            frozen: flag("frozen"),
            sent: crate::f32_field(row, "sent")?,
            received: crate::f32_field(row, "received")?,
        }),
        camera: Some(CameraEntity {
            position: (
                crate::f64_field(row, "x")?,
                crate::f64_field(row, "y")?,
                crate::f64_field(row, "z")?,
            ),
            yaw: crate::f32_field(row, "yaw")?,
            pitch: crate::f32_field(row, "pitch")?,
            dimension: "minecraft:overworld".to_owned(),
            build_height: (-64, 384),
            target: debug_target(row),
            light: (int("sky"), int("block")),
            biome: "minecraft:plains".to_owned(),
        }),
    })
}

/// `hud_debug_overlay`: the overlay's lines and their draw for the default profile.
pub(super) fn hud_debug_overlay_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_gui::hud::debug::{self, EntryList, FooterKeys};
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let flag = |key: &str| row[key].as_bool().unwrap_or(false);
    let mut list = EntryList::default();
    list.set_overlay_visible(flag("overlay"));
    for extra in row["extra"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        list.set_status(extra, debug::Status::InOverlay);
    }
    let input = debug_input(row)?;
    let footer = list.is_overlay_visible().then(|| {
        debug::footer(
            &FooterKeys::defaults(),
            (false, false, false, false),
            (false, false),
        )
    });
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    if debug::shown(&list, flag("reduced"), (flag("hidden"), false)) {
        let (left, right) = debug::lines(&list, flag("reduced"), &input, footer);
        debug::draw(
            &mut frame,
            &ctx.atlas,
            &ctx.font,
            int("width"),
            (&left, &right),
        );
    }
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// `hud_debug_lines`: the columns drawn from lines given, the game's `extractLines` for each.
pub(super) fn hud_debug_lines_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let lines = |key: &str| -> Vec<String> {
        row[key]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|line| line.as_str().map(str::to_owned))
            .collect()
    };
    let (left, right) = (lines("left"), lines("right"));
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    photon_gui::hud::debug::draw(
        &mut frame,
        &ctx.atlas,
        &ctx.font,
        row["width"].as_i64().unwrap_or(0) as i32,
        (&left, &right),
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
