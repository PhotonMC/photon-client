use super::*;
use crate::case_ids::block_state;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The chat box's reading of commands typed against a tree: where each node
/// read, what could not be read, the colours and the lines under the text.
pub(super) fn command_parse_result(
    request: &Value,
    root: &std::path::Path,
) -> Result<Value, String> {
    use photon_game::commands::{format_text, usage_lines};
    use photon_game::text::{Lang, Style};
    let bytes: Vec<u8> = (0..request["tree"].as_str().ok_or("tree")?.len() / 2)
        .map(|index| {
            u8::from_str_radix(
                &request["tree"].as_str().unwrap_or_default()[index * 2..index * 2 + 2],
                16,
            )
            .map_err(|_| "tree hex")
        })
        .collect::<Result<_, _>>()?;
    let tree = photon_net::connection::parse_commands_packet(&bytes)
        .map_err(|error| format!("{error:?}"))?;
    let lang = Lang::load(root);
    let plain = |component: &photon_game::text::Component| lang.plain(component);
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let text = row["text"].as_str().ok_or("text")?;
        let start = usize::from(text.starts_with('/'));
        let selectors = row["selectors"].as_bool().unwrap_or(true);
        let parse = tree.parse(text, start, selectors);
        let mut nodes = Vec::new();
        let mut context = Some(&parse.context);
        while let Some(current) = context {
            for (node, range) in &current.nodes {
                nodes.push(json!([
                    tree.node(*node).usage_text(),
                    range.start,
                    range.end
                ]));
            }
            context = current.child.as_deref();
        }
        let arguments: Vec<Value> = parse
            .context
            .last_child()
            .arguments
            .iter()
            .map(|(name, range)| json!([name, range.start, range.end]))
            .collect();
        let exceptions: Vec<Value> = parse
            .exceptions
            .iter()
            .map(|(_, error)| json!([plain(&error.message), error.cursor()]))
            .collect();
        let mut runs: Vec<(String, i64)> = Vec::new();
        for (part, colour) in format_text(&parse, text, 0) {
            match runs.last_mut() {
                Some(last) if last.1 == i64::from(colour) => last.0.push_str(&part),
                _ => runs.push((part, i64::from(colour))),
            }
        }
        let view = usage_lines(
            &tree,
            &parse,
            text.chars().count(),
            row["suggestions_empty"].as_bool().unwrap_or(true),
        );
        let _ = Style::default();
        rows.push(json!({
            "label": row["label"],
            "nodes": nodes,
            "arguments": arguments,
            "cursor": parse.cursor(),
            "executable": parse.context.last_child().executable,
            "exceptions": exceptions,
            "runs": runs.iter().map(|(text, colour)| json!([text, colour])).collect::<Vec<_>>(),
            "usage": view.lines.iter().map(plain).collect::<Vec<_>>(),
            "usage_start": view.start,
        }));
    }
    Ok(json!({"rows": rows}))
}

/// The command tree a request's packet payload holds.
fn tree_of(request: &Value) -> Result<photon_game::commands::CommandTree, String> {
    let hex = request["tree"].as_str().ok_or("tree")?;
    let bytes: Vec<u8> = (0..hex.len() / 2)
        .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).map_err(|_| "tree hex"))
        .collect::<Result<_, _>>()?;
    photon_net::connection::parse_commands_packet(&bytes).map_err(|error| format!("{error:?}"))
}

/// What running each command for the player needs first: nothing, a whole command, a signature
/// or a permission.
pub(super) fn command_verify_result(request: &Value) -> Result<Value, String> {
    let tree = tree_of(request)?;
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let text = row["text"].as_str().ok_or("text")?;
        let selectors = row["selectors"].as_bool().unwrap_or(true);
        let check = match tree.check(text, selectors) {
            photon_game::commands::CommandCheck::NoIssues => "NO_ISSUES",
            photon_game::commands::CommandCheck::ParseErrors => "PARSE_ERRORS",
            photon_game::commands::CommandCheck::SignatureRequired => "SIGNATURE_REQUIRED",
            photon_game::commands::CommandCheck::PermissionsRequired => "PERMISSIONS_REQUIRED",
        };
        rows.push(json!({"label": row["label"], "check": check}));
    }
    Ok(json!({"rows": rows}))
}

/// The tree's own suggestions for text at a cursor, and whether the server
/// would be asked.
pub(super) fn command_suggest_result(request: &Value) -> Result<Value, String> {
    use photon_game::commands::SuggestContext;
    let hex = request["tree"].as_str().ok_or("tree")?;
    let bytes: Vec<u8> = (0..hex.len() / 2)
        .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).map_err(|_| "tree hex"))
        .collect::<Result<_, _>>()?;
    let tree = photon_net::connection::parse_commands_packet(&bytes)
        .map_err(|error| format!("{error:?}"))?;
    let names = |value: &Value| -> Vec<String> {
        value
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|name| name.as_str().map(str::to_owned))
            .collect()
    };
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let text = row["text"].as_str().ok_or("text")?;
        let cursor = row["cursor"]
            .as_u64()
            .map_or_else(|| text.chars().count(), |cursor| cursor as usize);
        let (players, teams) = (names(&row["players"]), names(&row["teams"]));
        let selected = names(&row["selected_entities"]);
        let selectors = row["selectors"].as_bool().unwrap_or(true);
        let parse = tree.parse(text, usize::from(text.starts_with('/')), selectors);
        let found = tree.suggestions(
            &parse,
            cursor,
            &SuggestContext {
                players: &players,
                teams: &teams,
                selected_entities: &selected,
                selectors,
            },
        );
        let start = if found.matches.is_empty() {
            -1
        } else {
            found.start as i64
        };
        rows.push(json!({
            "label": row["label"],
            "matches": found.matches,
            "start": start,
            "asked": found.ask_server,
        }));
    }
    Ok(json!({"rows": rows}))
}

/// The families whose answer is one function of the request.
fn plain_family(family: &str, request: &Value) -> Option<Result<Value, String>> {
    Some(match family {
        "wire" => wire_result(request),
        "timer" => timer_result(request),
        "math" => math_result(request),
        "utf8" => utf8_result(request),
        "binary" => binary_result(request),
        "nbt" => nbt_result(request),
        "resources" => resources_result(request),
        "window" => window_result(request),
        "atlas" => atlas_result(request),
        _ => return None,
    })
}

/// The families of the game's content, each one function of the request.
fn content_family(family: &str, request: &Value) -> Option<Result<Value, String>> {
    Some(match family {
        "block" => block_case(request),
        "registry" => registry::registry_result(request),
        "image" => image_result(request),
        "model" => model_result(request),
        "json" => json_result(request),
        "movement" => movement_result(request),
        "entity" => entity_family_result(request),
        "gui" => gui::gui_result(request),
        _ => return None,
    })
}

pub(super) fn respond(request: Value) -> Result<Value, String> {
    if request["schema_version"].as_u64() != Some(1) {
        return Err("unsupported schema_version".into());
    }
    let case_id = string(&request, "case_id")?;
    let family = string(&request, "family")?;
    if let Some(result) =
        plain_family(family, &request).or_else(|| content_family(family, &request))
    {
        return Ok(json!({"schema_version": 1, "case_id": case_id, "result": result?}));
    }
    let result = match family {
        "sound" => match string(&request, "operation")? {
            "packets" => sound_packets_result(&request)?,
            "decode" => sound_decode_result(&request)?,
            "engine" => sound_engine_result(&request)?,
            "block_sounds" => {
                let bits = |value: f32| format!("{:08x}", value.to_bits());
                let mut rows: Vec<Value> = photon_data::block_sound::sound_types()
                    .map(|row| {
                        json!({"label": format!("type:{}", row.name), "kind": "type", "name": row.name,
                            "volume": bits(row.volume), "pitch": bits(row.pitch),
                            "break": row.break_sound(), "step": row.step_sound(), "place": row.place_sound(),
                            "hit": row.hit_sound(), "fall": row.fall_sound()})
                    })
                    .collect();
                let overrides: Vec<(u32, &str)> = photon_data::block_sound::state_overrides()
                    .map(|(state, row)| (state, row.name))
                    .collect();
                for (name, row) in photon_data::block_sound::blocks() {
                    let own: Vec<Value> = overrides
                        .iter()
                        .filter(|(state, _)| {
                            photon_world::block::state_or_air(block_state(*state as i32))
                                .block()
                                .name
                                == name
                        })
                        .map(|(state, type_name)| json!([state, type_name]))
                        .collect();
                    rows.push(
                        json!({"label": format!("block:{name}"), "kind": "block", "name": name,
                        "type": row.name, "overrides": own}),
                    );
                }
                json!({"rows": rows})
            }
            "music_manager" => crate::music::music_manager_result(&request)?,
            "damage_effects" => {
                let mut rows: Vec<Value> = photon_data::damage_effect::effect_sounds()
                    .map(|(name, sound)| {
                        json!({"label": format!("effect:{name}"), "kind": "effect", "name": name,
                            "sound": sound})
                    })
                    .collect();
                rows.extend(
                    photon_data::damage_effect::type_effects().map(|(name, effect)| {
                        json!({"label": format!("type:{name}"), "kind": "type", "name": name,
                        "effect": effect})
                    }),
                );
                json!({"rows": rows})
            }
            _ => sound_registry_result(&request)?,
        },
        _ => return Err("unknown behavior family".into()),
    };
    Ok(json!({"schema_version": 1, "case_id": case_id, "result": result}))
}

/// Answers every input line. Cases are independent, so they are answered in
/// parallel; the output keeps the input order.
pub(super) fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lines: Vec<String> = io::stdin().lock().lines().collect::<Result<_, _>>()?;
    let responses = answer_all(&lines).map_err(io::Error::other)?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    for response in responses {
        writeln!(output, "{response}")?;
    }
    output.flush()?;
    Ok(())
}

/// Answers every line on plain threads pulling the next unanswered line.
///
/// Not rayon workers: production code builds shared caches lazily, some with
/// rayon inside the initializer. Rayon workers blocked on such an
/// initialization would leave its parallel work nobody to run it.
fn answer_all(lines: &[String]) -> Result<Vec<String>, String> {
    let next = AtomicUsize::new(0);
    let answers: Vec<Slot> = lines.iter().map(|_| Mutex::new(None)).collect();
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| answer_until_done(lines, &next, &answers));
        }
    });
    answers
        .into_iter()
        .map(|slot| {
            slot.into_inner()
                .ok()
                .flatten()
                .unwrap_or_else(|| Err("no answer".to_owned()))
        })
        .collect()
}

/// One line's answer, filled in by whichever worker took the line.
type Slot = Mutex<Option<Result<String, String>>>;

/// One worker: takes the next unanswered line until none are left.
fn answer_until_done(lines: &[String], next: &AtomicUsize, answers: &[Slot]) {
    loop {
        let index = next.fetch_add(1, Ordering::Relaxed);
        let Some(line) = lines.get(index) else {
            return;
        };
        let result = answer(line).map_err(|message| format!("line {}: {message}", index + 1));
        if let Ok(mut slot) = answers[index].lock() {
            *slot = Some(result);
        }
    }
}

fn answer(line: &str) -> Result<String, String> {
    let request = serde_json::from_str(line).map_err(|error| error.to_string())?;
    let response = respond(request)?;
    serde_json::to_string(&response).map_err(|error| error.to_string())
}

/// The raw registry id of an item named with or without its namespace.
fn raw_item_id(name: &str) -> Option<i32> {
    let name = format!("minecraft:{}", name.trim_start_matches("minecraft:"));
    photon_game::item::id_of(&name).map(|id| id.0)
}

/// The menu a scenario row describes: its kind, recipe sets, data and XP
/// level, as the client holds them before its slots are set.
pub(super) fn menu_named(
    name: &str,
    row: &Value,
) -> Result<photon_game::menus::container::ContainerMenu, String> {
    use photon_game::menus::container::{ContainerMenu, MenuKind, Smelter};
    let kind = match name.strip_prefix("minecraft:").unwrap_or(name) {
        "crafting" => MenuKind::Crafting,
        "anvil" => MenuKind::Anvil,
        "enchantment" => MenuKind::Enchantment,
        "brewing_stand" => MenuKind::Brewing,
        "beacon" => MenuKind::Beacon,
        "crafter_3x3" => MenuKind::Crafter,
        "lectern" => MenuKind::Lectern,
        "merchant" => MenuKind::Merchant,
        "grindstone" => MenuKind::Grindstone,
        "loom" => MenuKind::Loom,
        "smithing" => MenuKind::Smithing,
        "cartography_table" => MenuKind::Cartography,
        "stonecutter" => MenuKind::Stonecutter,
        "furnace" => MenuKind::Furnace(Smelter::Furnace),
        "blast_furnace" => MenuKind::Furnace(Smelter::BlastFurnace),
        "smoker" => MenuKind::Furnace(Smelter::Smoker),
        "generic_3x3" => MenuKind::Dispenser,
        "hopper" => MenuKind::Hopper,
        "shulker_box" => MenuKind::ShulkerBox,
        chest => MenuKind::Chest(
            chest
                .strip_prefix("generic_9x")
                .and_then(|rows| rows.parse().ok())
                .ok_or_else(|| format!("unknown menu {name}"))?,
        ),
    };
    let id = row["container_id"]
        .as_i64()
        .ok_or("container_id must be a number")? as i32;
    let mut menu = ContainerMenu::for_menu(kind, id, Vec::new(), None, 0);
    if let Some(sets) = row.get("recipe_sets").and_then(Value::as_object) {
        // In the order the menu reads its sets.
        menu.accepted = kind
            .input_sets()
            .iter()
            .map(|name| {
                std::sync::Arc::new(
                    sets.get(*name)
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|item| raw_item_id(item.as_str()?))
                        .collect(),
                )
            })
            .collect();
    }
    if let Some(inputs) = row.get("stonecutter_inputs").and_then(Value::as_array) {
        menu.accepted = vec![std::sync::Arc::new(
            inputs
                .iter()
                .filter_map(|item| raw_item_id(item.as_str()?))
                .collect(),
        )];
    }
    if let Some(data) = row.get("data").and_then(Value::as_array) {
        menu.data = data
            .iter()
            .filter_map(|value| Some(value.as_i64()? as i16))
            .collect();
    }
    // What the server sends once the screen is open lands in the same data.
    for pair in row
        .get("data_after")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let (Some(index), Some(value)) = (pair[0].as_u64(), pair[1].as_i64()) else {
            continue;
        };
        if menu.data.len() <= index as usize {
            menu.data.resize(index as usize + 1, 0);
        }
        menu.data[index as usize] = value as i16;
    }
    if let Some(level) = row.get("xp_level").and_then(Value::as_i64) {
        menu.xp_level = level as i32;
    }
    menu.item_tags = std::sync::Arc::new(vanilla_item_tags()?);
    if kind == MenuKind::Merchant {
        let item = |pair: &Value| -> Result<photon_game::menus::merchant::ItemCost, String> {
            let name = pair[0].as_str().ok_or("item must be a string")?;
            Ok(photon_game::menus::merchant::ItemCost {
                item: photon_game::item::id_of(name)
                    .ok_or_else(|| format!("unknown item {name}"))?,
                count: pair[1].as_i64().ok_or("count must be a number")? as i32,
                components: Vec::new(),
            })
        };
        let mut offers = Vec::new();
        for offer in row["offers"].as_array().into_iter().flatten() {
            let cost_a = item(&offer["buy"])?;
            let result = item(&offer["sell"])?;
            offers.push(photon_game::menus::merchant::MerchantOffer {
                cost_a,
                cost_b: offer.get("buy_b").map(item).transpose()?,
                result: result.stack(result.count),
                uses: offer["uses"].as_i64().ok_or("uses")? as i32,
                max_uses: offer["max_uses"].as_i64().ok_or("max_uses")? as i32,
                xp: offer["xp"].as_i64().ok_or("xp")? as i32,
                special_price_diff: offer["special"].as_i64().ok_or("special")? as i32,
                demand: offer["demand"].as_i64().ok_or("demand")? as i32,
                price_multiplier: offer["multiplier"].as_f64().ok_or("multiplier")? as f32,
            });
        }
        menu.offers = std::sync::Arc::new(offers);
        if let Some(state) = row.get("merchant") {
            menu.trader = photon_game::menus::merchant::Trader {
                level: state["level"].as_i64().unwrap_or(0) as i32,
                xp: state["xp"].as_i64().unwrap_or(0) as i32,
                show_progress: state["progress"].as_bool().unwrap_or(false),
                can_restock: state["restock"].as_bool().unwrap_or(false),
            };
        }
    }
    if let Some(inputs) = row.get("furnace_inputs").and_then(Value::as_array) {
        menu.accepted = vec![std::sync::Arc::new(
            inputs
                .iter()
                .filter_map(|name| photon_game::item::id_of(name.as_str()?).map(|id| id.0))
                .collect(),
        )];
    }
    Ok(menu)
}
