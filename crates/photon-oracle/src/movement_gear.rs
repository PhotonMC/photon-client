//! The armour and off-hand stacks a movement row puts on the player.

use super::*;

/// The container slots a row may fill besides the main hand: its key and the slot it names.
const WORN: [(&str, i16); 2] = [("chest", 6), ("off_hand", 45)];

/// Put the row's worn items into the player's inventory, as the server's slot packets do.
pub(super) fn worn_items(
    world: &mut photon_world::ClientWorld,
    player: &photon_world::local_player::LocalPlayer,
    breaking: &Value,
) -> Result<(), String> {
    for (key, slot) in WORN {
        let Some(name) = breaking[key].as_str() else {
            continue;
        };
        let item = photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        world.apply_local_player_container_slot(
            0,
            0,
            slot,
            Some(&photon_world::ClientItemStackState::new(1, item)),
        );
    }
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
    game_mode: &photon_game::game_mode::GameMode,
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
    Ok(json!({
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
        "off_hand_count": hand_count(world, 45),
        "watched_states": watched,
    }))
}

/// The registry name of the item in a hand slot.
fn hand_item(world: &photon_world::ClientWorld, slot: i16) -> String {
    let stack = world
        .local_player()
        .and_then(|state| state.inventory_slot(slot));
    let name = stack.and_then(|stack| photon_game::item::definition(stack.item_registry_id()));
    name.map_or_else(|| "minecraft:air".to_owned(), |item| item.name.to_owned())
}
