use super::*;

/// Clicks in the player's own inventory menu, as the client predicts them
/// and sends them.
pub(super) fn container_clicks_result(request: &Value) -> Result<Value, String> {
    use photon_game::menus::container::{ClickType, ContainerMenu, Player};
    use photon_world::ClientItemStackState;
    let stack = |described: &Value| -> Result<ClientItemStackState, String> {
        let name = described["item"].as_str().ok_or("item must be a string")?;
        let id = photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        let count = described["count"]
            .as_i64()
            .ok_or("count must be a number")? as i32;
        let stack = ClientItemStackState::new(count, id);
        let Some(hex_patch) = described["patch"].as_str() else {
            return Ok(stack);
        };
        let bytes = hex_bytes(hex_patch)?;
        let patch = photon_game::item_components::read_patch(&mut bytes.as_slice())
            .map_err(|_| format!("malformed patch {hex_patch}"))?;
        Ok(ClientItemStackState::from_wire(
            count,
            id,
            patch.added.iter().map(|(kind, _)| *kind).collect(),
            patch.removed,
        )
        .with_patch_bytes(bytes))
    };
    let describe = |stack: Option<&ClientItemStackState>| -> Value {
        match stack {
            None => Value::Null,
            Some(stack) => json!({
                "item": photon_data::item::registry::ITEMS[stack.item_registry_id().0 as usize].name,
                "count": stack.item_count(),
            }),
        }
    };
    let enchantments = std::sync::Arc::new(vanilla_enchantments());
    let traits = std::sync::Arc::new(vanilla_enchantment_traits());
    let item_tags = std::sync::Arc::new(vanilla_item_tags()?);
    let lang = std::sync::Arc::new(photon_game::text::Lang::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.data/minecraft/assets/jar/assets"),
    ));
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = row["label"].as_str().ok_or("label must be a string")?;
        let mut menu = match row.get("menu").and_then(Value::as_str) {
            None => ContainerMenu::default(),
            Some(_) if row.get("mount").is_some() => mount_menu(&row["mount"])?,
            Some(name) => menu_named(name, row)?,
        };
        menu.item_tags = item_tags.clone();
        menu.enchantments = enchantments.clone();
        menu.enchantment_traits = traits.clone();
        menu.lang = lang.clone();
        for slot in row["slots"].as_array().ok_or("slots must be an array")? {
            let index = slot["slot"].as_u64().ok_or("slot must be a number")? as usize;
            menu.set_slot(index, Some(stack(slot)?));
        }
        if let Some(name) = row.get("item_name").and_then(Value::as_str) {
            menu.set_item_name(name);
        }
        if menu.kind == photon_game::menus::container::MenuKind::Anvil
            && let Some(data) = row.get("data").and_then(Value::as_array)
        {
            // What the server last sent of the cost, which the slots would make again.
            for (index, value) in data.iter().enumerate() {
                menu.data[index] = value.as_i64().unwrap_or(0) as i16;
            }
        }
        if row.get("carried").is_some() {
            menu.carried = Some(stack(&row["carried"])?);
        }
        if menu.kind == photon_game::menus::container::MenuKind::Merchant {
            // Setting the payments made the result, as the reference's slots do.
            menu.update_sell_item();
            if let Some(trade) = row.get("select").and_then(Value::as_u64) {
                menu.selection_hint = trade as usize;
                menu.update_sell_item();
                menu.try_move_items(trade as usize);
            }
        }
        menu.state_id = row["state_id"]
            .as_i64()
            .ok_or("state_id must be a number")? as i32;
        let selected = row["selected"]
            .as_u64()
            .ok_or("selected must be a number")? as usize;
        let player = Player {
            selected,
            infinite_materials: row["game_mode"].as_str() == Some("creative"),
        };
        let mut clicks = Vec::new();
        for click in row["clicks"].as_array().ok_or("clicks must be an array")? {
            let name = click["type"].as_str().ok_or("type must be a string")?;
            if name == "SCROLL" || name == "UNHOVER" {
                clicks.push(bundle_mouse_step(&mut menu, click, selected)?);
                continue;
            }
            let kind = ClickType::from_name(name).ok_or("unknown click type")?;
            let (swings, body) = menu.handle_click(
                click["slot"].as_i64().ok_or("slot must be a number")? as i32,
                click["button"].as_i64().ok_or("button must be a number")? as i32,
                kind,
                player,
            );
            let mut sent = Vec::new();
            for _ in 0..swings {
                sent.push(json!({"type": "minecraft:swing", "bytes": "00"}));
            }
            sent.push(json!({"type": "minecraft:container_click", "bytes": hex(&body)}));
            let slots: Vec<Value> = menu
                .slots
                .iter()
                .map(|slot| describe(slot.as_ref()))
                .collect();
            let mut described = json!({"sent": sent, "slots": slots, "selected": selected});
            if let Some(selections) = bundle_selections(&menu) {
                described["bundle_selections"] = selections;
            }
            if !menu.sounds().is_empty() {
                described["sounds"] = json!(menu.sounds());
            }
            if menu.carried.is_some() {
                described["carried"] = describe(menu.carried.as_ref());
            }
            clicks.push(described);
        }
        out.push(json!({"label": label, "outcome": "clicked", "clicks": clicks}));
    }
    Ok(json!({"rows": out}))
}

/// The vanilla enchantments, in network order, as the data pack has them.
fn vanilla_enchantment_entries() -> Vec<photon_game::registries::EnchantmentEntry> {
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    photon_game::registries::Registries::vanilla(&data_root).enchantments
}

/// The vanilla enchantments by network id, as the data pack names them.
fn vanilla_enchantments() -> Vec<String> {
    vanilla_enchantment_entries()
        .into_iter()
        .map(|entry| entry.name)
        .collect()
}

/// What menus read of the vanilla enchantments.
fn vanilla_enchantment_traits() -> Vec<photon_game::menus::container::EnchantmentTraits> {
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    photon_game::menus::container::EnchantmentTraits::of(
        &photon_game::registries::Registries::vanilla(&data_root),
    )
}

/// The hash the click packet gives each component a stack's patch adds,
/// and the components it removes, by name.
pub(super) fn component_hashes_result(request: &Value) -> Result<Value, String> {
    use photon_data::item::component_types::NAMES;
    use photon_game::item_components::{hash::component_hash, read_patch};
    let enchantments = vanilla_enchantments();
    let name = |kind: i32| NAMES[kind as usize].to_owned();
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = row["label"].as_str().ok_or("label must be a string")?;
        let bytes = hex_bytes(row["patch"].as_str().ok_or("patch must be a string")?)?;
        let patch =
            read_patch(&mut bytes.as_slice()).map_err(|_| format!("{label}: malformed patch"))?;
        let added: serde_json::Map<String, Value> = patch
            .added
            .iter()
            .map(|(kind, value)| {
                (
                    name(*kind),
                    json!(component_hash(*kind, value, &enchantments)),
                )
            })
            .collect();
        let mut removed: Vec<String> = patch.removed.iter().map(|&kind| name(kind)).collect();
        removed.sort();
        out.push(json!({"label": label, "added": added, "removed": removed}));
    }
    Ok(json!({"rows": out}))
}

/// The screen's bundle actions over a slot, as a click row's step: the wheel by
/// whole steps (`button`), or the mouse leaving it; what it selects and sends.
fn bundle_mouse_step(
    menu: &mut photon_game::menus::container::ContainerMenu,
    click: &Value,
    selected: usize,
) -> Result<Value, String> {
    use photon_gui::screen::ScreenAction;
    let slot = click["slot"].as_u64().ok_or("slot must be a number")? as usize;
    let mut mouse = photon_gui::screen::bundle_mouse::BundleMouse::default();
    if click["type"] == "SCROLL" {
        let wheel = click["button"].as_i64().ok_or("button must be a number")?;
        mouse.scrolled(wheel as f64, Some(slot), menu);
    } else {
        mouse.hover_changed(Some(slot), None, menu);
    }
    let mut sent = Vec::new();
    for action in mouse.drain() {
        if let ScreenAction::SelectBundleItem { slot, selected } = action
            && menu.select_bundle_item(slot as usize, selected)
        {
            let mut body = Vec::new();
            photon_core::wire::write_varint(slot, &mut body);
            photon_core::wire::write_varint(selected, &mut body);
            sent.push(json!({"type": "minecraft:bundle_item_selected", "bytes": hex(&body)}));
        }
    }
    let slots: Vec<Value> = menu
        .slots
        .iter()
        .map(|stack| match stack {
            None => Value::Null,
            Some(stack) => json!({
                "item": photon_data::item::registry::ITEMS[stack.item_registry_id().0 as usize].name,
                "count": stack.item_count(),
            }),
        })
        .collect();
    let mut described = json!({"sent": sent, "slots": slots, "selected": selected});
    if let Some(selections) = bundle_selections(menu) {
        described["bundle_selections"] = selections;
    }
    if let Some(carried) = &menu.carried {
        described["carried"] = json!({
            "item": photon_data::item::registry::ITEMS[carried.item_registry_id().0 as usize].name,
            "count": carried.item_count(),
        });
    }
    Ok(described)
}

/// The bundles' selected items: `[slot, index]` for each that has one, -1
/// for the carried stack; `None` when none has.
fn bundle_selections(menu: &photon_game::menus::container::ContainerMenu) -> Option<Value> {
    let mut selections: Vec<Value> = menu
        .slots
        .iter()
        .enumerate()
        .filter_map(|(slot, stack)| {
            let selected = stack.as_ref()?.bundle_selected();
            (selected != -1).then(|| json!([slot, selected]))
        })
        .collect();
    if let Some(selected) = menu
        .carried
        .as_ref()
        .map(|stack| stack.bundle_selected())
        .filter(|&selected| selected != -1)
    {
        selections.push(json!([-1, selected]));
    }
    (!selections.is_empty()).then(|| json!(selections))
}

/// The menu `handleMountScreenOpen` builds: the mount's family and chest columns, and what the
/// menu reads of its entity.
pub(super) fn mount_menu(
    mount: &Value,
) -> Result<photon_game::menus::container::ContainerMenu, String> {
    use photon_game::menus::container::{ContainerMenu, Mount, MountEntity, MountFamily};
    let name = mount["entity"].as_str().ok_or("entity must be a string")?;
    let family = MountFamily::of_entity(name).ok_or_else(|| format!("{name} is not a mount"))?;
    let columns = mount["columns"]
        .as_u64()
        .ok_or("columns must be a number")? as u8;
    let baby = mount["baby"].as_bool().ok_or("baby must be a bool")?;
    let tame = mount["tame"].as_bool().ok_or("tame must be a bool")?;
    let kind = photon_game::menus::container::MenuKind::Mount(Mount { family, columns });
    let mut menu = ContainerMenu::for_menu(kind, 4, Vec::new(), None, 0);
    menu.mount = MountEntity {
        entity_id: EntityId(0),
        entity_type: photon_data::entity_type::id(name).ok_or("unknown entity")?,
        can_use: MountEntity::can_use_slots(name, baby, tame),
        entity_tags: std::sync::Arc::new(vanilla_entity_type_tags()?),
    };
    Ok(menu)
}
