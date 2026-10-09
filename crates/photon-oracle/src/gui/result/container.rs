use super::*;

/// A stack as a row describes it: its item, count, and a written book's
/// pages or the component patch as the wire carries it.
pub(super) fn row_stack(slot: &Value) -> Result<photon_world::ClientItemStackState, String> {
    let item =
        photon_game::item::id_of(slot["item"].as_str().ok_or("item")?).ok_or("unknown item")?;
    let mut stack =
        photon_world::ClientItemStackState::new(slot["count"].as_i64().unwrap_or(1) as i32, item);
    if let Some(pages) = slot["book_pages"].as_array() {
        stack = stack.with_patch_bytes(written_book_patch(pages));
    }
    if let Some(hex_patch) = slot["patch"].as_str() {
        let bytes = crate::hex::hex_bytes(hex_patch)?;
        let patch = photon_game::item_components::read_patch(&mut bytes.as_slice())
            .map_err(|_| format!("malformed patch {hex_patch}"))?;
        stack.set_patch(&patch);
    }
    Ok(stack)
}

pub(super) fn container_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    use photon_render::gui::screen::Screen;
    use photon_render::gui::screen::generic::{EnchantText, GenericScreen, Titles};
    let name = row["menu"].as_str().ok_or("menu")?;
    // Item cooldowns the server set, then the player's ticks since.
    let mut cooldowns = photon_game::item::cooldowns::ItemCooldowns::default();
    for cooldown in row["cooldowns"].as_array().into_iter().flatten() {
        cooldowns.add(
            photon_game::item::cooldowns::namespaced(cooldown["group"].as_str().ok_or("group")?),
            cooldown["duration"].as_i64().ok_or("duration")? as i32,
        );
    }
    for _ in 0..row["cooldown_ticks"].as_i64().unwrap_or(0) {
        cooldowns.tick();
    }
    let mut menu = if row.get("mount").is_some() {
        crate::mount_menu(&row["mount"])?
    } else {
        crate::menu_named(name, row)?
    };
    for slot in row["slots"].as_array().ok_or("slots")? {
        let index = slot["slot"].as_u64().ok_or("slot")? as usize;
        menu.set_slot(index, Some(row_stack(slot)?));
    }
    // Setting the payments made the merchant's result, as the reference's slots do.
    if menu.kind == photon_game::menus::container::MenuKind::Merchant {
        menu.update_sell_item();
    }
    // The stonecutter's entries for the item in its slot, each
    // showing an item (which the screen draws and this run does not
    // record).
    if let (Some(inputs), Some(Some(input))) = (
        row["stonecutter_inputs"].as_array(),
        menu.slots
            .first()
            .map(|slot| slot.as_ref().map(|s| s.item_registry_id())),
    ) {
        menu.options = inputs
            .iter()
            .filter(|name| {
                name.as_str()
                    .and_then(photon_game::item::id_of)
                    .is_some_and(|id| id == input)
            })
            .map(|_| 1)
            .collect();
    }
    let lang = std::sync::Arc::new(photon_game::text::Lang::load(root));
    let shared_font = std::sync::Arc::new(Font::load(root).ok_or("font")?);
    let titles = Titles {
        title: lang.text(row["title_key"].as_str().ok_or("title_key")?),
        inventory: lang.text("container.inventory"),
        font: shared_font.clone(),
        repair_cost: lang.text("container.repair.cost"),
        repair_expensive: lang.text("container.repair.expensive"),
        patterns: std::sync::Arc::default(),
        lang: lang.clone(),
        alt_font: std::sync::Arc::new(Font::load_alt(root).ok_or("alt font")?),
        enchantments: std::sync::Arc::new(Vec::<EnchantText>::new()),
        recipe: std::sync::Arc::default(),
    };
    let (width, height) = (int("width"), int("height"));
    let mut screen: Box<dyn Screen> =
        if menu.kind == photon_game::menus::container::MenuKind::Lectern {
            Box::new(photon_render::gui::screen::book::BookScreen::new(
                width,
                height,
                lang.clone(),
                shared_font.clone(),
                !matches!(row["game_mode"].as_str(), Some("adventure" | "spectator")),
            ))
        } else {
            Box::new(GenericScreen::new(menu.kind, width, height, titles))
        };
    let mouse = (f64::from(int("mouse_x")), f64::from(int("mouse_y")));
    // The smithing table's empty slots show their icons, and a beacon's buttons their state,
    // once it has ticked.
    if matches!(
        menu.kind,
        photon_game::menus::container::MenuKind::Smithing
            | photon_game::menus::container::MenuKind::Beacon
    ) {
        screen.tick(&menu);
    }
    let mut steps = Vec::new();
    let mut keys = vec![String::new()];
    keys.extend(
        row["keys"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|k| k.as_str().map(str::to_owned)),
    );
    let mut item_drawer =
        |_: &photon_world::ClientItemStackState,
         _: f32,
         _: f32,
         _: &mut Vec<photon_render::gui::GuiItemVertex>| {};
    let mut player = |_: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
        Vec::<photon_render::gui::GuiItemVertex>::new()
    };
    // A tooltip is only its first line, the item's name, which is
    // what the anvil's name box takes.
    let tooltips = row["tooltips"].as_bool().unwrap_or(true);
    let mut tooltip = |stack: &photon_world::ClientItemStackState, _: bool| {
        if !tooltips {
            return Vec::new();
        }
        let name = photon_data::item::registry::ITEMS[stack.item_registry_id() as usize]
            .name
            .trim_start_matches("minecraft:");
        let text = lang
            .get(&format!("item.minecraft.{name}"))
            .or_else(|| lang.get(&format!("block.minecraft.{name}")))
            .unwrap_or(name)
            .to_owned();
        vec![photon_render::gui::chat::plain_runs(&text)]
    };
    let book = photon_world::RecipeBook::default();
    let tags = photon_world::block::tags::ItemTags::default();
    for key in keys {
        if let Some(text) = key.strip_prefix("type:") {
            for character in text.chars() {
                screen.char_typed(character);
            }
        }
        let mut drawn = GuiFrame {
            log: Some(Vec::new()),
            // The game's clock has not advanced: no partial tick.
            cooldowns: cooldowns.percents(0.0),
            ..GuiFrame::default()
        };
        let mut draw = photon_render::gui::screen::ScreenDraw {
            frame: &mut drawn,
            atlas,
            font,
            item: &mut item_drawer,
            player: &mut player,
            tooltip: &mut tooltip,
            lang: &lang,
            book: &book,
            tags: &tags,
            control_held: false,
            partial: 0.0,
            widget_tooltips: tooltips,
        };
        screen.build(&mut draw, &menu, mouse);
        steps.push(json!({
            "key": key,
            "ops": gui_ops(&drawn.log.take().unwrap_or_default()),
        }));
    }
    Ok(json!({"label": row["label"], "steps": steps}))
}
