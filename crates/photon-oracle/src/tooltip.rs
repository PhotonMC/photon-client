use super::*;

/// `movement_properties`: the declarations, the state cache and the
/// vanilla tags, as the oracle describes them.
pub(super) fn movement_properties(label: &str) -> Result<Value, String> {
    use photon_data::block::classes::BlockClass;
    use photon_data::block::definition::Fluid;
    let tags = vanilla_tags()?;
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let fluid_name = |fluid: Fluid| match fluid {
        Fluid::Empty => "minecraft:empty",
        Fluid::Water => "minecraft:water",
        Fluid::FlowingWater => "minecraft:flowing_water",
        Fluid::Lava => "minecraft:lava",
        Fluid::FlowingLava => "minecraft:flowing_lava",
    };
    let blocks: Vec<Value> = photon_data::block::registry::BLOCKS
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let first = block.first_state_id();
            let mut fluids: Vec<String> = (0..block.state_count())
                .map(|offset| {
                    let state = photon_world::block::state(BlockStateId(first.0 + offset as i32))
                        .expect("a registry state");
                    let fluid = state.fluid;
                    format!(
                        "{}|{}|{}|{:08x}|{}|{}",
                        fluid_name(fluid.fluid),
                        fluid.amount,
                        fluid.falling,
                        fluid.own_height().to_bits(),
                        fluid.is_source(),
                        state.suffocating
                    )
                })
                .collect();
            if fluids.iter().all(|fluid| *fluid == fluids[0]) {
                fluids.truncate(1);
            }
            let tagged = |tag: &str| tags.contains(index, tag);
            json!({
                "name": block.name,
                "friction": bits(block.friction),
                "speed_factor": bits(block.speed_factor),
                "jump_factor": bits(block.jump_factor),
                "bounce_restitution": bits(block.bounce_restitution),
                "climbable": tagged("minecraft:climbable"),
                "fence": tagged("minecraft:fences"),
                "wall": tagged("minecraft:walls"),
                "can_glide_through": tagged("minecraft:can_glide_through"),
                "suppresses_bounce": tagged("minecraft:suppresses_bounce"),
                "fence_gate": block.class.is(BlockClass::FenceGateBlock),
                "trapdoor": block.class.is(BlockClass::TrapDoorBlock),
                "ladder": block.class.is(BlockClass::LadderBlock),
                "fluids": fluids,
            })
        })
        .collect();
    Ok(json!({"label": label, "outcome": "listed", "blocks": blocks}))
}

/// A default component value as the component's own codec writes it in JSON.
pub(super) fn component_json(component: &photon_data::item::component::Component) -> Value {
    use photon_data::item::component::{AttributeDisplay, Component, Operation, Rarity, SlotGroup};
    match component {
        Component::ItemName(key) => json!({"translate": key}),
        Component::Rarity(rarity) => json!(match rarity {
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::Epic => "epic",
        }),
        Component::AttributeModifiers(modifiers) => Value::Array(
            modifiers
                .iter()
                .map(|modifier| {
                    let mut entry = json!({
                        "type": modifier.attribute,
                        "id": modifier.id,
                        "amount": modifier.amount,
                        "operation": match modifier.operation {
                            Operation::AddValue => "add_value",
                            Operation::AddMultipliedBase => "add_multiplied_base",
                            Operation::AddMultipliedTotal => "add_multiplied_total",
                        },
                        "slot": match modifier.slot {
                            SlotGroup::Any => "any",
                            SlotGroup::MainHand => "mainhand",
                            SlotGroup::OffHand => "offhand",
                            SlotGroup::Hand => "hand",
                            SlotGroup::Feet => "feet",
                            SlotGroup::Legs => "legs",
                            SlotGroup::Chest => "chest",
                            SlotGroup::Head => "head",
                            SlotGroup::Armor => "armor",
                            SlotGroup::Body => "body",
                            SlotGroup::Saddle => "saddle",
                        },
                    });
                    if modifier.display == AttributeDisplay::Hidden {
                        entry["display"] = json!({"type": "hidden"});
                    }
                    entry
                })
                .collect(),
        ),
        Component::EnchantmentGlintOverride(value) => json!(value),
        Component::MapColor(color) => json!(color),
        Component::PotionContents(None) => json!({}),
        Component::PotionContents(Some(potion)) => json!({"potion": potion}),
        Component::StoredEnchantments(enchantments) => Value::Object(
            enchantments
                .iter()
                .map(|(name, level)| ((*name).to_owned(), json!(level)))
                .collect(),
        ),
        Component::PaintingVariant(name) => json!(name),
        Component::SuspiciousStewEffects(effects) => Value::Array(
            effects
                .iter()
                .map(|(name, duration)| json!({"id": name, "duration": duration}))
                .collect(),
        ),
        Component::BannerPatterns(layers) => Value::Array(
            layers
                .iter()
                .map(|(pattern, color)| json!({"color": color, "pattern": pattern}))
                .collect(),
        ),
        Component::TooltipDisplay(hidden) => json!({"hidden_components": hidden}),
        Component::PotionDurationScale(scale) => json!(f64::from(*scale)),
        Component::Fireworks { flight_duration } => json!({"flight_duration": flight_duration}),
        Component::Instrument(name) | Component::JukeboxPlayable(name) => json!(name),
        Component::BlockState(properties) => Value::Object(
            properties
                .iter()
                .map(|(key, value)| ((*key).to_owned(), json!(value)))
                .collect(),
        ),
        Component::OminousBottleAmplifier(amplifier) => json!(amplifier),
    }
}

/// An item's default components the client reads, as the codec's JSON and
/// as the network's bytes, keyed by component type name.
pub(super) fn default_components(
    item: &photon_data::item::definition::Item,
    registries: &photon_game::registries::Registries,
) -> (Value, Value) {
    use photon_data::item::component::Component;
    use photon_data::item::component_types::NAMES;
    let mut json_map = serde_json::Map::new();
    let mut wire_map = serde_json::Map::new();
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let mut put = |id: i32, json: Value, bytes: &[u8]| {
        let name = NAMES[id as usize];
        json_map.insert(name.to_owned(), json);
        wire_map.insert(name.to_owned(), json!(hex(bytes)));
    };
    let declared_name = item
        .components
        .iter()
        .any(|component| matches!(component, Component::ItemName(_)));
    if !declared_name {
        let key = photon_game::item::default_components::description_key(item);
        put(
            photon_data::item::component_types::ITEM_NAME,
            json!({"translate": key}),
            &photon_game::item::default_components::translatable(&key),
        );
    }
    for component in item.components {
        if let Some((id, bytes)) =
            photon_game::item::default_components::encode(component, Some(registries))
        {
            put(id, component_json(component), &bytes);
        }
    }
    (Value::Object(json_map), Value::Object(wire_map))
}

/// `item_declarations`: the declarations as the oracle describes them.
/// An item's `equippable` component as the declarations case writes it.
fn describe_equippable(equip: &photon_data::item::definition::Equip) -> Value {
    let mut described = json!({
        "slot": equip.slot,
        "player": equip.player,
        "swappable": equip.swappable,
    });
    if let Some(asset) = equip.asset {
        described["asset"] = json!(asset);
    }
    if let Some(overlay) = equip.camera_overlay {
        described["camera_overlay"] = json!(overlay);
    }
    described
}

/// An item's default consume seconds, animation and equippable, as the declarations case writes them.
fn describe_consume(item: &photon_data::item::definition::Item, described: &mut Value) {
    if let Some(consume) = &item.consumable {
        // Java writes the float's own shortest text, which reads back as this double.
        described["consume_seconds"] = json!(
            consume
                .seconds
                .to_string()
                .parse::<f64>()
                .unwrap_or_default()
        );
        described["use_animation"] = json!(consume.animation);
        described["consume_sound"] = json!(consume.sound);
        described["consume_particles"] = json!(consume.particles);
    }
    if let Some(sound) = item.kinetic_hit_sound {
        described["kinetic_hit_sound"] = json!(sound);
    }
    if item.minimum_attack_charge > 0.0 {
        described["minimum_attack_charge"] = json!(
            item.minimum_attack_charge
                .to_string()
                .parse::<f64>()
                .unwrap_or_default()
        );
    }
    if let Some(sound) = item.piercing_sound {
        described["piercing_sound"] = json!(sound);
    }
    // Every item has the component; the table lists only those that differ from the default.
    let effects = item
        .use_effects
        .unwrap_or(photon_data::item::definition::UseEffects::DEFAULT);
    described["use_can_sprint"] = json!(effects.can_sprint);
    described["use_speed"] = json!(effects.speed.to_string().parse::<f64>().unwrap_or_default());
    if let Some(equip) = &item.equippable {
        described["equippable"] = describe_equippable(equip);
    }
}

/// A tool component as the declarations list it.
fn describe_tool(tool: &photon_data::item::definition::Tool) -> Value {
    use photon_data::item::definition::RuleBlocks;
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
    json!({
        "rules": rules,
        "default_mining_speed": format!("{:08x}", tool.default_mining_speed.to_bits()),
        "damage_per_block": tool.damage_per_block,
        "can_destroy_blocks_in_creative": tool.can_destroy_blocks_in_creative,
    })
}

pub(super) fn item_declarations(label: &str) -> Value {
    use photon_data::item::classes::ItemClass;
    let classes: Vec<Value> = ItemClass::ALL
        .iter()
        .map(|class| {
            let declares: Vec<String> = class
                .declares()
                .iter()
                .map(|method| {
                    let name = format!("{method:?}");
                    name[..1].to_lowercase() + &name[1..]
                })
                .collect();
            json!({
                "name": format!("{class:?}"),
                "parent": class.parent().map_or(String::new(), |parent| format!("{parent:?}")),
                "declares": declares,
            })
        })
        .collect();
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    let registries = photon_game::registries::Registries::vanilla(&data_root);
    let items: Vec<Value> = photon_data::item::registry::ITEMS
        .iter()
        .map(|item| {
            let mut described = json!({
                "name": item.name,
                "class": format!("{:?}", item.class),
                "max_stack_size": item.max_stack_size,
                "max_damage": item.max_damage,
                "consumable": item.consumable.is_some(),
                "blocks_attacks": item.blocks_attacks,
                "kinetic_weapon": item.kinetic_weapon,
                "glider": item.glider,
            });
            describe_consume(item, &mut described);
            if let Some(remainder) = &item.crafting_remainder {
                described["crafting_remainder"] =
                    json!({"item": remainder.item, "count": remainder.count});
            }
            if let Some(tool) = item.tool {
                described["tool"] = describe_tool(tool);
            }
            if item.burn_time > 0 {
                described["burn_time"] = json!(item.burn_time);
            }
            if item.brewing_ingredient {
                described["brewing_ingredient"] = json!(true);
            }
            if let Some(block) = item.block {
                described["block"] = json!(block);
            }
            let (components, wire) = default_components(item, &registries);
            described["components"] = components;
            described["component_wire"] = wire;
            described
        })
        .collect();
    let component_types: Vec<&str> = photon_data::item::component_types::NAMES.to_vec();
    json!({
        "label": label,
        "outcome": "listed",
        "classes": classes,
        "items": items,
        "component_types": component_types,
    })
}

/// A parsed JSON value as a `serde_json` one, for the oracle's comparison.
pub(super) fn serde_of(value: &photon_core::json::JsonValue) -> Value {
    use photon_core::json::JsonValue;
    match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(flag) => json!(flag),
        JsonValue::Number(text) => text.parse::<i64>().map_or_else(
            |_| json!(text.parse::<f64>().unwrap_or(0.0)),
            |number| json!(number),
        ),
        JsonValue::String(units) => json!(String::from_utf16_lossy(units)),
        JsonValue::Array(items) => Value::Array(items.iter().map(serde_of).collect()),
        JsonValue::Object(object) => Value::Object(
            object
                .entries()
                .map(|(key, value)| (String::from_utf16_lossy(key), serde_of(value)))
                .collect(),
        ),
    }
}

/// `tooltip_registries`: the data-driven registries as the game loads them
/// from the data pack, and the attributes' declarations.
pub(super) fn tooltip_registries(label: &str) -> Value {
    use photon_data::item::attributes::ATTRIBUTES;
    use photon_game::registries::{DescribedEntry, Registries};
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    let registries = Registries::vanilla(&data_root);
    let component =
        |value: &Option<photon_core::json::JsonValue>| value.as_ref().map_or(Value::Null, serde_of);
    let enchantments: Vec<Value> = registries
        .enchantments
        .iter()
        .enumerate()
        .map(|(id, entry)| {
            json!({
                "name": entry.name,
                "id": id,
                "description": component(&entry.description),
                "curse": entry.curse,
                "max_level": entry.max_level,
                "tooltip_order": entry.tooltip_order,
            })
        })
        .collect();
    let paintings: Vec<Value> = registries
        .painting_variants
        .iter()
        .enumerate()
        .map(|(id, entry)| {
            let mut described = json!({
                "name": entry.name,
                "id": id,
                "width": entry.width,
                "height": entry.height,
            });
            if entry.title.is_some() {
                described["title"] = component(&entry.title);
            }
            if entry.author.is_some() {
                described["author"] = component(&entry.author);
            }
            described
        })
        .collect();
    let described = |entries: &[DescribedEntry]| -> Vec<Value> {
        entries
            .iter()
            .enumerate()
            .map(|(id, entry)| {
                json!({
                    "name": entry.name,
                    "id": id,
                    "description": component(&entry.description),
                })
            })
            .collect()
    };
    let banners: Vec<Value> = registries
        .banner_patterns
        .iter()
        .enumerate()
        .map(|(id, entry)| {
            json!({"name": entry.name, "id": id, "translation_key": entry.translation_key})
        })
        .collect();
    let attributes: Vec<Value> = ATTRIBUTES
        .iter()
        .map(|entry| {
            let (up, down) = entry.styles();
            json!({
                "name": entry.name,
                "description_id": entry.description_id(),
                "style_up": up,
                "style_down": down,
            })
        })
        .collect();
    json!({
        "label": label,
        "outcome": "listed",
        "enchantments": enchantments,
        "painting_variants": paintings,
        "instruments": described(&registries.instruments),
        "jukebox_songs": described(&registries.jukebox_songs),
        "banner_patterns": banners,
        "trim_materials": described(&registries.trim_materials),
        "trim_patterns": described(&registries.trim_patterns),
        "attributes": attributes,
    })
}
