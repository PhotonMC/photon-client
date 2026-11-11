use super::*;
use crate::case_ids::reference_class_name;

pub(super) fn block_case(request: &Value) -> Result<Value, String> {
    match string(request, "operation")? {
        "registry_names" => registry_names(request),
        "placement_rules" => placement_rules(request),
        "menu_providers" => menu_providers(request),
        _ => block_result(request),
    }
}

fn block_result(request: &Value) -> Result<Value, String> {
    use photon_data::block::registry;
    let operation = string(request, "operation")?;
    let rows = request["rows"]
        .as_array()
        .ok_or("block rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        match operation {
            "state_definitions" => {
                let blocks: Vec<Value> = crate::case_ids::blocks()
                    .enumerate()
                    .map(|(id, block)| {
                        let properties: Vec<Value> = block
                            .properties
                            .iter()
                            .map(|property| {
                                json!({
                                    "name": property.name,
                                    "kind": match property.kind {
                                        registry::PropertyKind::Boolean => "boolean",
                                        registry::PropertyKind::Integer => "integer",
                                        registry::PropertyKind::Enum => "enum",
                                    },
                                    "values": property.values,
                                })
                            })
                            .collect();
                        json!({
                            "name": block.name,
                            "id": id,
                            "properties": properties,
                            "default": block.default,
                            "state_count": block.state_count(),
                            "first_state_id": crate::case_ids::wire_state(block.first_state_id()),
                            "default_state_id": crate::case_ids::wire_state(block.default_state_id()),
                        })
                    })
                    .collect();
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "block_count": crate::case_ids::blocks().count(),
                    "state_count": crate::case_ids::blocks()
                        .map(registry::Block::state_count)
                        .sum::<u32>(),
                    "blocks": blocks,
                }));
            }
            "offset_properties" => {
                use photon_data::block::definition::OffsetType;
                let blocks: Vec<Value> = crate::case_ids::blocks()
                    .filter(|block| block.offset_type != OffsetType::None)
                    .map(|block| {
                        json!({
                            "name": block.name,
                            "kind": if block.offset_type == OffsetType::Xyz { "xyz" } else { "xz" },
                            "max_horizontal": format!(
                                "{:08x}",
                                photon_world::block::max_horizontal_offset(block).to_bits()
                            ),
                            "max_vertical": format!(
                                "{:08x}",
                                photon_world::block::max_vertical_offset(block).to_bits()
                            ),
                        })
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "blocks": blocks}));
            }
            "fluid_flow_properties" => {
                // `FlowingFluid.isSolidFace`'s sturdy faces (never for ice)
                // and `isSolid`, as runs of equal values.
                let count = crate::case_ids::state_count();
                let value = |id: u32| {
                    let state = photon_world::block::state(crate::case_ids::block_state(id as i32))
                        .expect("a registry state");
                    let ice = state
                        .block()
                        .class
                        .is(photon_data::block::classes::BlockClass::IceBlock);
                    let sturdy = if ice { 0 } else { state.sturdy_faces };
                    sturdy | u8::from(state.solid) << 6
                };
                let mut runs = Vec::new();
                let mut last = None;
                for id in 0..count {
                    let current = value(id);
                    if last != Some(current) {
                        runs.push(format!("{id}:{current}"));
                        last = Some(current);
                    }
                }
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "state_count": count,
                    "runs": runs,
                }));
            }
            "motion_blocking" => {
                let count = crate::case_ids::state_count();
                let mut words = vec![0_u64; count.div_ceil(64) as usize];
                for id in 0..count {
                    if photon_world::motion_blocking(crate::case_ids::block_state(id as i32)) {
                        words[id as usize / 64] |= 1 << (id % 64);
                    }
                }
                let words: Vec<String> = words.iter().map(|word| format!("{word:016x}")).collect();
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "state_count": count,
                    "words": words,
                }));
            }
            "fluid_overlay_blocks" => {
                let blocks: Vec<&str> = crate::case_ids::blocks()
                    .filter(|block| photon_world::block::shows_fluid_overlay(block))
                    .map(|block| block.name)
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "blocks": blocks}));
            }
            "block_declarations" => out.push(block_declarations(label)),
            "item_declarations" => out.push(item_declarations(label)),
            "block_behaviours" => {
                use photon_data::block::classes::Method;
                // The class whose own method the block runs, as the
                // reference's dispatch resolves it.
                let owner = |block: &registry::Block, method: Method| {
                    block
                        .class
                        .lineage()
                        .find(|class| class.declares().contains(&method))
                        .map_or_else(|| "BlockBehaviour".to_owned(), reference_class_name)
                };
                let blocks: Vec<Value> = crate::case_ids::blocks()
                    .map(|block| {
                        let replaceable = if block.replaceable { "1" } else { "0" };
                        json!({
                            "name": block.name,
                            "class": crate::case_ids::block_class_name(block),
                            "placement": owner(block, Method::GetStateForPlacement),
                            "can_be_replaced": owner(block, Method::CanBeReplaced),
                            "can_survive": owner(block, Method::CanSurvive),
                            "use_without_item": owner(block, Method::UseWithoutItem),
                            "use_item_on": owner(block, Method::UseItemOn),
                            "update_shape": owner(block, Method::UpdateShape),
                            "replaceable": replaceable.repeat(block.state_count() as usize),
                            "stairs": block
                                .class
                                .is(photon_data::block::classes::BlockClass::StairBlock),
                            "item": block.item_name(),
                        })
                    })
                    .collect();
                let items: Vec<Value> = crate::case_ids::items()
                    .map(|item| {
                        use photon_data::item::classes::ItemMethod;
                        let class = crate::case_ids::reference_class(item.class);
                        let owner = |method: ItemMethod| format!("{:?}", class.owner_of(method));
                        let mut described = json!({
                            "name": item.name,
                            "class": format!("{class:?}"),
                            "use_on": owner(ItemMethod::UseOn),
                            "use": owner(ItemMethod::Use),
                            "default_use_pass": item.use_does_nothing(),
                        });
                        if let Some(block) = item.block {
                            described["block"] = json!(block);
                        }
                        described
                    })
                    .collect();
                out.push(
                    json!({"label": label, "outcome": "listed", "blocks": blocks, "items": items}),
                );
            }
            "destroy_properties" => out.push(destroy_properties(label)),
            "item_tools" => {
                use photon_data::item::registry::RuleBlocks;
                let items: Vec<Value> = crate::case_ids::items()
                    .map(|item| {
                        let mut described = json!({"name": item.name});
                        if let Some(tool) = item.tool {
                            let rules: Vec<Value> = tool
                                .rules
                                .iter()
                                .map(|rule| {
                                    let mut written = json!({});
                                    written["blocks"] = match rule.blocks {
                                        RuleBlocks::Tag(tag) => json!(format!("#{tag}")),
                                        RuleBlocks::Blocks(names) => json!(names),
                                    };
                                    if let Some(speed) = rule.speed {
                                        written["speed"] = json!(format!("{:08x}", speed.to_bits()));
                                    }
                                    if let Some(correct) = rule.correct_for_drops {
                                        written["correct_for_drops"] = json!(correct);
                                    }
                                    written
                                })
                                .collect();
                            described["tool"] = json!({
                                "rules": rules,
                                "default_mining_speed": format!("{:08x}", tool.default_mining_speed.to_bits()),
                                "damage_per_block": tool.damage_per_block,
                                "can_destroy_blocks_in_creative": tool.can_destroy_blocks_in_creative,
                            });
                        }
                        described
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "items": items}));
            }
            "item_stacking" => {
                use photon_data::item::classes::ItemMethod;
                let items: Vec<Value> = crate::case_ids::items()
                    .map(|item| {
                        let owner = |method: ItemMethod| {
                            format!(
                                "{:?}",
                                crate::case_ids::reference_class(item.class).owner_of(method)
                            )
                        };
                        let mut described = json!({
                            "name": item.name,
                            "max_stack_size": item.max_stack_size,
                            "max_damage": item.max_damage,
                            "stacked_on_other": owner(ItemMethod::OverrideStackedOnOther),
                            "other_stacked_on_me": owner(ItemMethod::OverrideOtherStackedOnMe),
                        });
                        if let Some(equip) = &item.equippable {
                            described["equip_slot"] = json!(equip.slot);
                            described["player_can_equip"] = json!(equip.player);
                        }
                        if let Some(remainder) = &item.crafting_remainder {
                            described["crafting_remainder"] = json!(remainder.item);
                            described["crafting_remainder_count"] = json!(remainder.count);
                        }
                        described
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "items": items}));
            }
            "item_components" => out.push(item_components(label)),
            "tooltip_registries" => out.push(tooltip_registries(label)),
            "potions" => out.push(potions(label)),
            "creative_tabs" => {
                use photon_data::item::creative_tabs::{CREATIVE_TABS, DEFAULT_TAB, TabKind};
                let data_root = crate::jar_root().join("data");
                let registries = photon_game::registries::Registries::vanilla(&data_root);
                let stack = |entry: &photon_data::item::creative::CreativeStack| {
                    crate::creative_op::stack_json(entry, &registries)
                };
                let stacks = |list: &[_]| list.iter().map(&stack).collect::<Vec<_>>();
                let tabs: Vec<Value> = CREATIVE_TABS
                    .iter()
                    .enumerate()
                    .map(|(index, tab)| {
                        use photon_game::menus::creative::{ordered, ordered_search_items};
                        json!({
                            "name": tab.name,
                            "type": match tab.kind {
                                TabKind::Category => "CATEGORY",
                                TabKind::Inventory => "INVENTORY",
                                TabKind::Hotbar => "HOTBAR",
                                TabKind::Search => "SEARCH",
                            },
                            "row": if tab.top { "TOP" } else { "BOTTOM" },
                            "column": tab.column,
                            "aligned_right": tab.aligned_right,
                            "show_title": tab.show_title,
                            "can_scroll": tab.can_scroll,
                            "should_display": tab.should_display,
                            "background": tab.background,
                            "title": tab.title,
                            "icon": stack(&tab.icon),
                            "items": stacks(ordered(index)),
                            "search_items": stacks(ordered_search_items(index)),
                        })
                    })
                    .collect();
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "default_tab": DEFAULT_TAB,
                    "tabs": tabs,
                }));
            }
            "light_properties" => {
                let blocks: Vec<Value> = crate::case_ids::blocks()
                    .map(|block| {
                        let mut states: Vec<String> = (0..block.state_count())
                            .map(|offset| {
                                let t = photon_world::block::state(block.state_at(offset))
                                    .expect("a registry state");
                                format!(
                                    "{}|{}|{:08x}|{}|{}|{}|{}|{}|{}",
                                    t.light_emission,
                                    t.emissive_rendering,
                                    t.shade_brightness.to_bits(),
                                    t.view_blocking || t.near_plane_view_blocking,
                                    t.light_dampening,
                                    t.solid_render,
                                    t.collision_full_block,
                                    t.use_shape_for_light_occlusion,
                                    t.can_occlude
                                )
                            })
                            .collect();
                        if states.iter().all(|state| *state == states[0]) {
                            states.truncate(1);
                        }
                        json!({"name": block.name, "states": states})
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "blocks": blocks}));
            }
            "face_occlusion" => out.push(face_occlusion(label)?),
            "collision_shapes" => out.push(shape_table(label, false)),
            "outline_shapes" => out.push(shape_table(label, true)),
            "movement_properties" => out.push(movement_properties(label)?),
            "block_tints" => {
                let tinted: Vec<Value> = registry::TINTS
                    .iter()
                    .map(|(name, layers)| {
                        json!({
                            "name": name,
                            "layers": layers
                                .iter()
                                .map(|colour| format!("{colour:08x}"))
                                .collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "tinted": tinted}));
            }
            other => out.push(creative_operation(other, label, row)?),
        }
    }
    Ok(json!({"rows": out}))
}

/// The names of the registries whose wire ids the client reads, one row per label.
fn registry_names(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("block rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(json!({
            "label": string(row, "label")?,
            "stat_types": photon_data::stat::type_names(),
            "custom_stats": in_version(photon_data::stat::custom_names()),
            "custom_stat_formatters": formatters_in_version(),
        }));
    }
    Ok(json!({"rows": out}))
}

/// Which block states have a menu provider, as the bits of every state id.
fn menu_providers(request: &Value) -> Result<Value, String> {
    let count = crate::case_ids::state_count();
    let mut words = vec![0_u64; count.div_ceil(64) as usize];
    for id in 0..count {
        if photon_world::block::menu_provider::has_menu_provider(crate::case_ids::block_state(
            id as i32,
        )) {
            words[id as usize / 64] |= 1 << (id % 64);
        }
    }
    let words: Vec<String> = words.iter().map(|word| format!("{word:016x}")).collect();
    let rows: Vec<Value> = request["rows"]
        .as_array()
        .ok_or("block rows must be an array")?
        .iter()
        .map(|row| {
            json!({
                "label": row["label"],
                "outcome": "listed",
                "state_count": count,
                "words": words,
                "failed": [],
            })
        })
        .collect();
    Ok(json!({"rows": rows}))
}

/// `destroy_properties`: each block's destroy time, tool rule, air flag and fluid per state.
fn destroy_properties(label: &str) -> Value {
    let blocks: Vec<Value> = crate::case_ids::blocks()
        .map(|block| {
            let mut states: Vec<String> = (0..block.state_count())
                .map(|offset| {
                    let state = photon_world::block::state(block.state_at(offset))
                        .expect("a registry state");
                    format!(
                        "{:08x}|{}|{}|{}",
                        block.destroy_time.to_bits(),
                        block.requires_correct_tool_for_drops,
                        block.air,
                        crate::case_ids::wire_state(state.fluid.legacy_block())
                    )
                })
                .collect();
            if states.iter().all(|state| *state == states[0]) {
                states.truncate(1);
            }
            json!({
                "name": block.name,
                "game_master": block.class.implements(
                    photon_data::block::classes::BlockInterface::GameMasterBlock,
                ),
                "states": states,
            })
        })
        .collect();
    json!({"label": label, "outcome": "listed", "blocks": blocks})
}

/// The operations on the creative inventory and the hotbar file.
fn creative_operation(operation: &str, label: &str, row: &Value) -> Result<Value, String> {
    match operation {
        "creative_tabs_operator" => Ok(crate::creative_op::operator_tabs(label)),
        "creative_tag_search" => creative_tag_search(label, row),
        "spectator_menu" => crate::spectator_op::spectator_menu(label, row),
        "hotbar_entry" => crate::creative_op::hotbar_entry(label, row),
        other => Err(format!("unknown block operation: {other}")),
    }
}

/// The statistic names the oracle's version has: 26.3 added sleeping in a straw bed.
fn in_version(names: &[&'static str]) -> Vec<&'static str> {
    names
        .iter()
        .copied()
        .filter(|name| !is_new_stat(name))
        .collect()
}

/// The formatters of the custom statistics the oracle's version has, one per statistic.
fn formatters_in_version() -> Vec<&'static str> {
    let formatters = photon_data::stat::custom_formatter_names();
    photon_data::stat::custom_names()
        .iter()
        .zip(formatters)
        .filter(|(name, _)| !is_new_stat(name))
        .map(|(_, formatter)| *formatter)
        .collect()
}

fn is_new_stat(name: &str) -> bool {
    crate::case_ids::before_26_3() && name.ends_with("sleep_in_straw_bed")
}
