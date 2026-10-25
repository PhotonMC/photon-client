//! The armour and off-hand stacks a movement row puts on the player.

use super::*;

/// The container slots a row may fill besides the main hand: its key and the slot it names.
const WORN: [(&str, i16); 2] = [("chest", 6), ("off_hand", 45)];

/// Put the row's worn items into the player's inventory, as the server's slot packets do.
pub(super) fn worn_items(
    world: &mut photon_world::ClientWorld,
    player: &mut photon_world::local_player::LocalPlayer,
    breaking: &Value,
) -> Result<(), String> {
    for (key, slot) in WORN {
        let Some(name) = breaking[key].as_str() else {
            continue;
        };
        let item = photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        let mut stack = photon_world::ClientItemStackState::new(1, item);
        if let Some(hex) = breaking[format!("{key}_patch")].as_str() {
            stack.set_patch(&wire_patch(hex)?);
        }
        world.apply_local_player_container_slot(0, 0, slot, Some(&stack));
    }
    // The session has synced the hands by the time a row's first keys are read.
    player.set_hand_state(photon_game::item::glider::hand_state(world, 0));
    add_rockets(world, player, breaking)
}

/// As the session does: the main hand the player ticks with is the world's inventory now.
pub(super) fn before_tick(
    world: &photon_world::ClientWorld,
    player: &mut photon_world::local_player::LocalPlayer,
    breaking: Option<&Value>,
) {
    if breaking.is_some() {
        player.set_hand_state(photon_game::item::glider::hand_state(world, 0));
    }
}

/// The level ticks the rockets after the player, who they then push.
pub(super) fn after_tick(
    world: &mut photon_world::ClientWorld,
    player: &mut photon_world::local_player::LocalPlayer,
    breaking: Option<&Value>,
) {
    if breaking.is_some_and(|breaking| breaking["rockets"].is_number()) {
        world.tick_entities();
        // The player is right-handed, and the rocket is in neither hand.
        world.boost_from_rockets(player, photon_world::RocketHand::default());
    }
}

/// Fireworks that fly with the player, as the server's use of a rocket makes them: added after the
/// player, their data naming it.
fn add_rockets(
    world: &mut photon_world::ClientWorld,
    player: &photon_world::local_player::LocalPlayer,
    breaking: &Value,
) -> Result<(), String> {
    let kind = photon_data::entity_type::id("minecraft:firework_rocket")
        .ok_or("no firework rocket type")?;
    let position = player.position();
    for index in 0..breaking["rockets"].as_i64().unwrap_or(0) {
        let id = EntityId(900 + index as i32);
        let entity = EntityInput {
            entity_id: id,
            uuid: Uuid::new(0, id.0 as u64),
            type_id: kind,
            x: position.x,
            y: position.y,
            z: position.z,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        };
        if !world.apply_entity(entity) {
            return Err("the rocket was rejected".into());
        }
        // The attached-to-target field (9): an optional unsigned int, the id plus one.
        let target = (player.entity_id().0 + 1) as u8;
        world.apply_entity_metadata(id, &[9, 19, target, 0xFF]);
    }
    Ok(())
}

/// The tick with where the row's rockets are, as bits.
pub(super) fn with_rockets(
    mut tick: Value,
    world: &photon_world::ClientWorld,
    breaking: Option<&Value>,
) -> Value {
    let count = breaking
        .and_then(|breaking| breaking["rockets"].as_i64())
        .unwrap_or(0);
    tick["rockets"] = (0..count)
        .filter_map(|index| world.entities().at(EntityId(900 + index as i32)))
        .map(|rocket| {
            let (x, y, z) = rocket.position();
            json!({"x": hex_bits(x), "y": hex_bits(y), "z": hex_bits(z)})
        })
        .collect();
    tick
}

fn hex_bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

/// The block events a row sends before the tick `index`; the first tick sets the game's clock.
pub(super) fn apply_block_events(
    world: &mut photon_world::ClientWorld,
    events: Option<&Vec<Value>>,
    index: usize,
) -> Result<(), String> {
    if index == 0 && events.is_some() {
        world.set_game_time(1000);
    }
    for event in events.into_iter().flatten() {
        if i32_field(event, "tick")? as usize == index {
            world.apply_block_event(
                (
                    i32_field(event, "x")?,
                    i32_field(event, "y")?,
                    i32_field(event, "z")?,
                ),
                i32_field(event, "b0")? as u8,
                i32_field(event, "b1")? as u8,
            );
        }
    }
    Ok(())
}

/// `Minecraft.tick` after the entities, for a row with block events: the block entities tick (the
/// moving pistons push the player), and the clock moves on.
pub(super) fn tick_block_entities(
    world: &mut photon_world::ClientWorld,
    player: &mut photon_world::local_player::LocalPlayer,
    events: Option<&Vec<Value>>,
) -> Result<(), String> {
    if events.is_none() {
        return Ok(());
    }
    let pushed = world.tick_after_entities(Some(player));
    world.set_game_time(world.game_time() + 1);
    pushed.map_err(|unsupported| unsupported.to_string())
}

/// What the tick's attack left of the breaking of blocks.
pub(super) fn breaking_report(
    world: &photon_world::ClientWorld,
    player: &photon_world::local_player::LocalPlayer,
    (game_mode, cooldowns): (
        &photon_game::game_mode::GameMode,
        &photon_game::item::cooldowns::ItemCooldowns,
    ),
    attack: &Value,
) -> Result<Value, String> {
    let stage = world
        .destroying_blocks()
        .get(&player.entity_id())
        .map_or(-1, |progress| progress.stage);
    let mut watched = Vec::new();
    for place in attack["watch"].as_array().ok_or("watch must be an array")? {
        watched.push(
            world
                .block_state_at(
                    i32_field(place, "x")?,
                    i32_field(place, "y")?,
                    i32_field(place, "z")?,
                )
                .map_or(0, |state| state.0),
        );
    }
    let mut report = json!({
        "destroying": game_mode.is_destroying(),
        "progress": format!("{:08x}", game_mode.destroy_progress().to_bits()),
        "destroy_ticks": format!("{:08x}", game_mode.destroy_ticks().to_bits()),
        "delay": game_mode.destroy_delay(),
        "miss_time": game_mode.miss_time(),
        "stage": stage,
        "sequence": world.prediction_sequence(),
        "attack_strength_ticker": player.attack_strength_ticker(),
        "right_click_delay": game_mode.right_click_delay(),
        "main_hand_count": hand_count(world, 36),
        "main_hand_item": hand_item(world, 36),
        "main_hand_potion": hand_potion(world),
        "main_hand_cooldown": world
            .local_player()
            .and_then(|state| state.inventory_slot(36))
            .is_some_and(|stack| stack.item_count() > 0 && cooldowns.is_on_cooldown(stack)),
        "off_hand_count": hand_count(world, 45),
        "watched_states": watched,
    });
    if attack.get("inventory").is_some() {
        report["inventory"] = json!(inventory_report(world));
    }
    Ok(report)
}

/// The slots of the inventory menu that hold something, as `slot:item:count`, when a row asks.
fn inventory_report(world: &photon_world::ClientWorld) -> Vec<String> {
    let Some(state) = world.local_player() else {
        return Vec::new();
    };
    (5..=45)
        .filter_map(|slot| {
            let stack = state.inventory_slot(slot)?;
            (stack.item_count() > 0).then(|| {
                let name = photon_game::item::definition(stack.item_registry_id())
                    .map_or("?", |row| row.name);
                format!("{slot}:{name}:{}", stack.item_count())
            })
        })
        .collect()
}

/// The registry name of the potion the main hand's `potion_contents` names, or empty.
fn hand_potion(world: &photon_world::ClientWorld) -> String {
    let Some(stack) = world
        .local_player()
        .and_then(|state| state.inventory_slot(36))
    else {
        return String::new();
    };
    photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
        .potion_contents()
        .and_then(|contents| contents.potion)
        .and_then(|index| photon_data::item::potions::POTIONS.get(usize::try_from(index).ok()?))
        .map_or_else(String::new, |potion| potion.name.to_owned())
}

/// The registry name of the item in a hand slot.
fn hand_item(world: &photon_world::ClientWorld, slot: i16) -> String {
    let stack = world
        .local_player()
        .and_then(|state| state.inventory_slot(slot));
    let name = stack.and_then(|stack| photon_game::item::definition(stack.item_registry_id()));
    name.map_or_else(|| "minecraft:air".to_owned(), |item| item.name.to_owned())
}

/// A component patch as the wire carries it, the same bytes Java decodes; the reader must end
/// exactly where they do.
fn wire_patch(hex: &str) -> Result<photon_world::item_components::Patch, String> {
    let bytes = hex_bytes(hex)?;
    let mut rest = bytes.as_slice();
    let patch = photon_world::item_components::read_patch(&mut rest)
        .map_err(|_| format!("malformed component patch {hex}"))?;
    if !rest.is_empty() {
        return Err(format!("component patch has trailing bytes {hex}"));
    }
    Ok(patch)
}

/// The stack a row puts in the main hand, in the world's inventory and in the player.
pub(super) fn hold_main_hand(
    world: &mut photon_world::ClientWorld,
    player: &mut photon_world::local_player::LocalPlayer,
    breaking: &Value,
) -> Result<(), String> {
    let Some(name) = breaking["main_hand"].as_str() else {
        return Ok(());
    };
    let item = photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
    let count = breaking["main_hand_count"].as_i64().unwrap_or(1) as i32;
    let mut stack = photon_world::ClientItemStackState::new(count, item);
    let mut added = Vec::new();
    if let Some(seconds) = breaking["main_hand_use_cooldown"].as_f64() {
        // `UseCooldown.STREAM_CODEC`: the seconds and no group.
        let mut value = (seconds as f32).to_be_bytes().to_vec();
        value.push(0);
        added.push((photon_data::item::component_types::USE_COOLDOWN, value));
    }
    let mut removed = Vec::new();
    if let Some(hex) = breaking["main_hand_patch"].as_str() {
        let patch = wire_patch(hex)?;
        added.extend(patch.added);
        removed = patch.removed;
    }
    if let Some(name) = breaking["main_hand_use_remainder"].as_str() {
        let remainder =
            photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        // `ItemStackTemplate.STREAM_CODEC`: the item, a count of one and an empty patch.
        let mut value = Vec::new();
        photon_core::wire::write_varint(remainder.0, &mut value);
        value.extend([1, 0, 0]);
        added.push((photon_data::item::component_types::USE_REMAINDER, value));
    }
    if let Some(name) = breaking["main_hand_custom_name"].as_str() {
        // A plain text component on the wire: a nameless string tag.
        let mut value = vec![8];
        value.extend((name.len() as u16).to_be_bytes());
        value.extend(name.as_bytes());
        added.push((photon_data::item::component_types::CUSTOM_NAME, value));
    }
    if !added.is_empty() || !removed.is_empty() {
        stack.set_patch(&photon_world::item_components::Patch { added, removed });
    }
    world.apply_local_player_container_slot(0, 0, 36, Some(&stack));
    player.set_main_hand_item(item);
    Ok(())
}

/// The mob effects a row gives the player, as the server's packets do.
pub(super) fn apply_effects(
    world: &mut photon_world::ClientWorld,
    player: &photon_world::local_player::LocalPlayer,
    breaking: &Value,
) -> Result<(), String> {
    for effect in breaking["effects"].as_array().into_iter().flatten() {
        let id = match string(effect, "effect")? {
            "minecraft:haste" => 2,
            "minecraft:mining_fatigue" => 3,
            "minecraft:conduit_power" => 28,
            other => return Err(format!("unknown effect {other}")),
        };
        world.apply_local_player_mob_effect(
            player.entity_id(),
            id,
            i32_field(effect, "amplifier")?,
            1_000_000,
            0,
        );
    }
    Ok(())
}

/// The instruments of the vanilla data pack, by registry id.
pub(super) fn vanilla_instruments() -> Vec<Option<photon_game::item::instrument::Instrument>> {
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    photon_game::item::instrument::instruments_of(&photon_game::registries::Registries::vanilla(
        &data_root,
    ))
}

/// Which enchantments of the vanilla data pack carry the trident spin attack effect, by registry id.
pub(super) fn vanilla_spin_attack() -> Vec<bool> {
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    photon_game::registries::Registries::vanilla(&data_root)
        .enchantments
        .iter()
        .map(|entry| entry.trident_spin_attack)
        .collect()
}

/// The entity events a tick brings to the player: a damage event, the death event and the end of
/// a used item (event 9).
pub(super) fn server_events(
    tick: &Value,
    (world, player, cooldowns): (
        &mut photon_world::ClientWorld,
        &mut photon_world::local_player::LocalPlayer,
        &mut photon_game::item::cooldowns::ItemCooldowns,
    ),
) {
    if let Some(damage) = tick.get("damage").and_then(Value::as_str) {
        player.handle_damage_event(damage);
    }
    if tick.get("death").is_some() {
        player.handle_death_event();
    }
    if tick.get("finish_use").is_some() {
        photon_game::game_mode::finish_use(world, player, cooldowns, 0);
    }
}

/// A sheep row's wool byte: the dye in the low four bits, sheared in bit four.
fn sheep_wool_byte(described: &Value) -> Option<u8> {
    let sheared = described.get("sheared").is_some();
    let dye = described.get("dye").and_then(Value::as_u64);
    (sheared || dye.is_some()).then(|| dye.unwrap_or(0) as u8 | if sheared { 16 } else { 0 })
}

/// The spawn flags a row can put on an entity, applied as the server's metadata would.
fn apply_spawn_flags(
    world: &mut photon_world::ClientWorld,
    id: EntityId,
    name: &str,
    described: &Value,
) -> Result<(), String> {
    if described.get("baby").is_some() {
        // A piglin keeps its own baby flag after the zombification one.
        let baby_id = if name == "minecraft:piglin" { 17 } else { 16 };
        world.apply_entity_metadata(id, &[baby_id, 8, 1, 255]);
        if !world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_baby())
        {
            return Err(format!("{name} did not take the baby flag"));
        }
    }
    if described.get("age_locked").is_some() {
        world.apply_entity_metadata(id, &[17, 8, 1, 255]);
        if !world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_age_locked())
        {
            return Err(format!("{name} did not take the age lock"));
        }
    }
    apply_species_flags(world, id, name, described)
}

/// The species' own flags: a brown mooshroom, a screaming goat, a tamed horse, a sheep's wool
/// and a golem without its pumpkin.
fn apply_species_flags(
    world: &mut photon_world::ClientWorld,
    id: EntityId,
    name: &str,
    described: &Value,
) -> Result<(), String> {
    if described.get("brown").is_some() {
        // The variant is an int (serializer 1), brown is one.
        let mut data = vec![photon_world::MOOSHROOM_TYPE_DATA_ID, 1];
        photon_core::wire::write_varint(1, &mut data);
        data.push(255);
        world.apply_entity_metadata(id, &data);
    }
    apply_pet_flags(world, id, described)?;
    if described.get("screaming").is_some() {
        world.apply_entity_metadata(id, &[photon_world::GOAT_SCREAMING_DATA_ID, 8, 1, 255]);
    }
    if described.get("tamed").is_some() {
        world.apply_entity_metadata(id, &[photon_world::HORSE_FLAGS_DATA_ID, 0, 2, 255]);
        if !world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.horse().is_some_and(|horse| horse.is_tamed()))
        {
            return Err(format!("{name} did not take the tame flag"));
        }
    }
    if let Some(wool) = sheep_wool_byte(described) {
        world.apply_entity_metadata(id, &[photon_world::SHEEP_WOOL_DATA_ID, 0, wool, 255]);
    }
    if described.get("bare").is_some() {
        world.apply_entity_metadata(id, &[photon_world::SNOW_GOLEM_PUMPKIN_DATA_ID, 0, 0, 255]);
    }
    Ok(())
}

/// A tamed pet's flags: tame (owned by the player, or by another uuid when the flag is a string),
/// sitting, a collar dye, a health below the maximum and a wolf armor on its body.
fn apply_pet_flags(
    world: &mut photon_world::ClientWorld,
    id: EntityId,
    described: &Value,
) -> Result<(), String> {
    if let Some(tame) = described.get("tame") {
        let sitting = u8::from(described.get("sitting").is_some());
        world.apply_entity_metadata(
            id,
            &[photon_world::TAMABLE_FLAGS_DATA_ID, 0, 4 | sitting, 255],
        );
        let owner = if tame.is_string() {
            0x7002
        } else {
            super::movement::ORACLE_PLAYER_UUID
        };
        let mut data = vec![photon_world::TAMABLE_OWNER_DATA_ID, 13, 1];
        data.extend_from_slice(&0_u64.to_be_bytes());
        data.extend_from_slice(&owner.to_be_bytes());
        data.push(255);
        world.apply_entity_metadata(id, &data);
    }
    if let Some(dye) = described.get("collar").and_then(Value::as_i64) {
        let mut data = vec![photon_world::WOLF_COLLAR_DATA_ID, 1];
        photon_core::wire::write_varint(dye as i32, &mut data);
        data.push(255);
        world.apply_entity_metadata(id, &data);
    }
    if let Some(health) = described.get("hurt").and_then(Value::as_f64) {
        let mut data = vec![9, 3];
        data.extend_from_slice(&(health as f32).to_be_bytes());
        data.push(255);
        world.apply_entity_metadata(id, &data);
    }
    if described.get("grounded").is_some() {
        world.apply_entity_ground_state(id, true);
    }
    apply_worn_items(world, id, described)
}

/// The `armored` (body slot, wolf armor unless it names an item), `saddled` (a saddle unless it
/// names an item), `offhand` and `mainhand` flags.
fn apply_worn_items(
    world: &mut photon_world::ClientWorld,
    id: EntityId,
    described: &Value,
) -> Result<(), String> {
    use photon_world::EquipmentSlot;
    let worn = [
        ("armored", EquipmentSlot::Body, "minecraft:wolf_armor"),
        ("saddled", EquipmentSlot::Saddle, "minecraft:saddle"),
        ("offhand", EquipmentSlot::OffHand, ""),
        ("mainhand", EquipmentSlot::MainHand, ""),
    ];
    for (key, slot, default) in worn {
        let Some(flag) = described.get(key) else {
            continue;
        };
        let name = flag.as_str().unwrap_or(default);
        let item = photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        let stack = photon_world::ClientItemStackState::new(1, item);
        world.entities_mut().apply_equipment(id, slot, Some(stack));
    }
    Ok(())
}

/// The entities the server told of, added to the level as `handleAddEntity` does; one may be
/// held on a lead by the player (`handleEntityLinkPacket`).
pub(super) fn told_entities(
    world: &mut photon_world::ClientWorld,
    player: &photon_world::local_player::LocalPlayer,
    row: &Value,
) -> Result<(), String> {
    use photon_world::EntityInput;
    for described in row["entities"].as_array().into_iter().flatten() {
        let name = string(described, "type")?;
        let id = entity_id_field(described, "id")?;
        let accepted = world.apply_entity(EntityInput {
            entity_id: id,
            uuid: Uuid::new(0, id.0 as u64),
            type_id: photon_data::entity_type::id(name)
                .ok_or_else(|| format!("unknown entity type {name}"))?,
            x: bits64(described, "x")?,
            y: bits64(described, "y")?,
            z: bits64(described, "z")?,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: described.get("data").and_then(Value::as_i64).unwrap_or(0) as i32,
        });
        if !accepted {
            return Err(format!("entity {name} was rejected"));
        }
        // The reference's entity random starts from nowhere in particular; the port seeds it from the id.
        world.fix_entity_sound_seed(i64::from(id.0));
        apply_spawn_flags(world, id, name, described)?;
        if let Some(holder) = described.get("leash_holder") {
            // `true` is the player; a number is the entity holding the lead.
            let holder = holder
                .as_i64()
                .map_or(player.entity_id(), |holder| EntityId(holder as i32));
            world.entities_mut().set_leash_holder(id, holder);
        }
    }
    Ok(())
}
