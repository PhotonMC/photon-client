use super::*;

pub(super) fn titles_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use voxel_render::gui::hud::titles::Titles;
    let mut titles = Titles::default();
    titles.set_times(int("fade_in"), int("stay"), int("fade_out"));
    let runs = |text: &str| voxel_render::gui::chat::plain_runs(text);
    if let Some(title) = row["title"].as_str() {
        titles.set_title(runs(title));
        for _ in 0..int("elapsed") {
            titles.tick();
        }
    }
    if let Some(subtitle) = row["subtitle"].as_str() {
        titles.set_subtitle(runs(subtitle));
    }
    if let Some(overlay) = row["overlay"].as_str() {
        titles.set_overlay(runs(overlay));
        for _ in 0..int("overlay_elapsed") {
            titles.tick();
        }
    }
    let partial = row["partial"].as_f64().unwrap_or(0.0) as f32;
    titles.draw(
        &mut frame,
        atlas,
        font,
        (int("width"), int("height")),
        partial,
    );

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn sidebar_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use voxel_render::gui::hud::sidebar;
    let lang = voxel_game::text::Lang::new([]);
    let board = scoreboard_from(row)?;
    let style = voxel_game::text::Style::default();
    if let Some((name, objective)) =
        board.sidebar_objective(row["local"].as_str().unwrap_or_default())
    {
        let lines: Vec<sidebar::SidebarLine> = board
            .sidebar_lines(name)
            .iter()
            .map(|(name, score)| (lang.runs(name, &style), lang.runs(score, &style)))
            .collect();
        sidebar::draw(
            &mut frame,
            atlas,
            font,
            (int("width"), int("height")),
            &lang.runs(&objective.display_name, &style),
            &lines,
        );
    }

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn hud_hotbar_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use voxel_render::gui::hud::{Hud, HudInput};
    let flag = |key: &str| row[key].as_bool().unwrap_or(false);
    let speed = row["attack_speed"].as_f64().unwrap_or(4.0);
    let delay = voxel_world::local_player::attack_strength_delay(speed);
    let ticks = row["attack_ticks"].as_i64().unwrap_or(0) as f32;
    let attack_indicator =
        voxel_game::options::AttackIndicator::ALL[int("attack_indicator") as usize];
    let input = HudInput {
        first_person: !flag("third_person"),
        spectator: flag("spectator"),
        // The reference draws the crosshair and the hotbar alone here.
        survival: false,
        selected_slot: int("selected_slot"),
        health: 20.0,
        max_health: 20.0,
        absorption: 0.0,
        invulnerable_time: 0,
        heart_type: voxel_render::gui::HeartType::Normal,
        regeneration: false,
        hardcore: false,
        armor: 0,
        food: 20,
        saturation: 5.0,
        hunger: false,
        air_supply: 300,
        max_air_supply: 300,
        eye_in_water: false,
        experience_progress: 0.0,
        experience_level: 0,
        experience_prioritised: false,
        locator: None,
        attack_strength: (ticks / delay).clamp(0.0, 1.0),
        attack_delay: delay,
        attack_indicator,
        target_is_living: row["target"] == "living",
        hotbar: Default::default(),
        offhand: None,
    };
    let mut draw_item = |_: &voxel_world::ClientItemStackState,
                         _: f32,
                         _: f32,
                         _: &mut Vec<voxel_render::gui::GuiItemVertex>| {};
    Hud::default().draw(
        &mut frame,
        atlas,
        font,
        int("width"),
        int("height"),
        &input,
        0,
        &mut draw_item,
    );

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn hud_status_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    use voxel_render::gui::HeartType;
    use voxel_render::gui::hud::{Hud, HudInput};
    let number = |key: &str| row[key].as_f64().unwrap_or(0.0);
    let has = |name: &str| {
        row["effects"]
            .as_array()
            .is_some_and(|effects| effects.iter().any(|effect| effect == name))
    };
    let float = |key: &str| -> Result<f32, String> {
        u32::from_str_radix(row[key].as_str().ok_or(key)?, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let mut hud = Hud::default();
    let mut ticks = 0;
    let mut frames = Vec::new();
    for step in row["frames"].as_array().ok_or("frames")? {
        let tick = step["tick"].as_i64().unwrap_or(0);
        while ticks < tick {
            hud.tick();
            ticks += 1;
        }
        let input = HudInput {
            first_person: true,
            spectator: false,
            survival: true,
            selected_slot: 0,
            health: u32::from_str_radix(step["health"].as_str().ok_or("health")?, 16)
                .map(f32::from_bits)
                .map_err(|error| error.to_string())?,
            max_health: number("max_health") as f32,
            absorption: float("absorption")?,
            invulnerable_time: step["invulnerable"].as_i64().unwrap_or(0) as i32,
            heart_type: HeartType::for_player(
                has("poison"),
                has("wither"),
                int("frozen_ticks") >= 140,
            ),
            regeneration: has("regeneration"),
            hardcore: row["hardcore"].as_bool().unwrap_or(false),
            armor: number("armor").floor() as i32,
            food: int("food"),
            saturation: float("saturation")?,
            hunger: has("hunger"),
            air_supply: int("air"),
            max_air_supply: 300,
            eye_in_water: row["eye_in_water"].as_bool().unwrap_or(false),
            experience_progress: float("xp_progress")?,
            experience_level: int("xp_level"),
            experience_prioritised: false,
            locator: None,
            attack_strength: 1.0,
            attack_delay: 5.0,
            attack_indicator: voxel_game::options::AttackIndicator::Off,
            target_is_living: false,
            hotbar: Default::default(),
            offhand: None,
        };
        let mut drawn = GuiFrame {
            log: Some(Vec::new()),
            ..GuiFrame::default()
        };
        let now = step["ms"].as_u64().unwrap_or(0) as u128;
        let size = (int("width"), int("height"));
        hud.player_health(&mut drawn, atlas, size.0, size.1, &input, now);
        hud.contextual_bar(&mut drawn, atlas, font, size, &input);
        frames.push(Value::Array(gui_ops(&drawn.log.take().unwrap_or_default())));
    }
    Ok(json!({"label": row["label"], "frames": frames}))
}

pub(super) fn boss_bar_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use voxel_render::gui::hud::boss_bar::BossBars;
    let mut bars = BossBars::default();
    for (index, bar) in row["bars"].as_array().ok_or("bars")?.iter().enumerate() {
        bars.add(
            index as u128 + 1,
            voxel_render::gui::chat::plain_runs(bar["name"].as_str().unwrap_or_default()),
            bar["progress"].as_f64().unwrap_or(0.0) as f32,
            bar["color"].as_u64().unwrap_or(0) as u8,
            bar["overlay"].as_u64().unwrap_or(0) as u8,
            0,
            0,
        );
    }
    bars.draw(&mut frame, atlas, font, (int("width"), int("height")), 0);

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn locator_bar_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use voxel_render::gui::hud::locator::{
        self, LocatorCamera, LocatorInput, Viewer, WaypointStyles,
    };
    let numbers = |value: &Value| -> Vec<f64> {
        value
            .as_array()
            .map(|list| list.iter().filter_map(Value::as_f64).collect())
            .unwrap_or_default()
    };
    let mut waypoints = voxel_game::waypoint::WaypointManager::default();
    for packet in row["packets"].as_array().ok_or("packets")? {
        let hex = packet.as_str().ok_or("packet")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| error.to_string())?;
        let (operation, waypoint) = voxel_net::connection::parse_waypoint_packet(&bytes)
            .map_err(|error| format!("{error:?}"))?;
        waypoints.apply(operation, waypoint);
    }
    let camera = &row["camera"];
    let at = numbers(&camera["pos"]);
    let feet = numbers(&row["player"]["pos"]);
    let uuid = row["player"]["uuid"].as_array().ok_or("uuid")?;
    let half = |index: usize| uuid[index].as_i64().unwrap_or(0) as u64;
    let input = LocatorInput {
        waypoints,
        camera: LocatorCamera {
            y_rot: camera["yaw"].as_f64().unwrap_or(0.0) as f32,
            x_rot: camera["pitch"].as_f64().unwrap_or(0.0) as f32,
            position: [at[0], at[1], at[2]],
            fov: camera["fov"].as_f64().unwrap_or(70.0) as f32,
            size: (camera["aspect"].as_f64().unwrap_or(1.0) as f32, 1.0),
            z_near: 0.05,
            z_far: camera["far"].as_f64().unwrap_or(768.0) as f32,
        },
        viewer: Viewer {
            uuid: u128::from(half(0)) << 64 | u128::from(half(1)),
            position: [feet[0], feet[1], feet[2]],
        },
    };
    let size = (int("width"), int("height"));
    locator::background(&mut frame, atlas, size);
    locator::draw_dots(&mut frame, atlas, &input, &WaypointStyles::load(root), size);

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn tab_list_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use voxel_render::gui::hud::tab_list::{self, HealthStates, TabPlayer, TabScores};
    let runs = |key: &str| {
        row[key]
            .as_str()
            .map(voxel_render::gui::chat::plain_runs)
            .unwrap_or_default()
    };
    let board = scoreboard_from(row)?;
    let lang = voxel_game::text::Lang::load(root);
    let displays: Vec<Option<voxel_game::text::Component>> = row["players"]
        .as_array()
        .ok_or("players")?
        .iter()
        .map(|player| {
            player["display"]
                .as_str()
                .map(voxel_game::text::Component::literal)
        })
        .collect();
    let players = row["players"].as_array().ok_or("players")?;
    let (entries, objective) = tab_list::entries(
        players
            .iter()
            .zip(&displays)
            .map(|(player, display)| TabPlayer {
                name: player["name"].as_str().unwrap_or_default(),
                display_name: display.as_ref(),
                game_mode: player["mode"].as_i64().unwrap_or(0) as i32,
                latency: player["latency"].as_i64().unwrap_or(0) as i32,
                list_order: player["order"].as_i64().unwrap_or(0) as i32,
            }),
        &board,
        &lang,
    );
    let mut health = HealthStates::default();
    tab_list::draw(
        &mut frame,
        atlas,
        font,
        int("width"),
        &entries,
        (&runs("header"), &runs("footer")),
        TabScores {
            objective,
            health: &mut health,
            gui_ticks: 0,
            lang: &lang,
        },
    );

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
