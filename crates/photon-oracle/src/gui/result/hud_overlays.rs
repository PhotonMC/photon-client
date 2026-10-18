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
            manager.draw(&mut frame, &ctx.atlas, row.width, shown, env);
            entry["ops"] = json!(gui_ops(&frame.log.take().unwrap_or_default()));
        }
        other => return Err(format!("unknown toast event {other}")),
    }
    entry["sounds"] = json!(sounds);
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
