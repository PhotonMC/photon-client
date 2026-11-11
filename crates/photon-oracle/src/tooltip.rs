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
    let blocks: Vec<Value> = crate::case_ids::blocks()
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
            let canonical = photon_data::block::registry::block_index(block.name).unwrap_or(index);
            let tagged = |tag: &str| tags.contains(canonical, tag);
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

/// `attribute_modifiers` as its codec writes it.
fn modifiers_json(modifiers: &[photon_data::item::component::AttributeModifier]) -> Value {
    use photon_data::item::component::{AttributeDisplay, Operation, SlotGroup};
    Value::Array(
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
    )
}

/// A default component value as the component's own codec writes it in JSON.
pub(super) fn component_json(component: &photon_data::item::component::Component) -> Value {
    use photon_data::item::component::{Component, Rarity};
    match component {
        Component::ItemName(key) => json!({"translate": key}),
        Component::Rarity(rarity) => json!(match rarity {
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::Epic => "epic",
        }),
        Component::AttributeModifiers(modifiers) => modifiers_json(modifiers),
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
        Component::PaintingVariant(name) | Component::BlockTransformer(name) => json!(name),
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
        Component::PotionDurationScale(_)
        | Component::UseCooldown(_)
        | Component::UseRemainder(_)
        | Component::Fireworks { .. }
        | Component::Instrument(_)
        | Component::JukeboxPlayable(_)
        | Component::OminousBottleAmplifier(_) => single_value_json(component),
        Component::BlockState(properties) => Value::Object(
            properties
                .iter()
                .map(|(key, value)| ((*key).to_owned(), json!(value)))
                .collect(),
        ),
    }
}

/// [`component_json`] of the components that hold a number or a name.
fn single_value_json(component: &photon_data::item::component::Component) -> Value {
    use photon_data::item::component::Component;
    match component {
        Component::PotionDurationScale(scale) => json!(f64::from(*scale)),
        Component::UseCooldown(seconds) => json!({"seconds": f64::from(*seconds)}),
        Component::UseRemainder(item) => json!({"id": item}),
        Component::Fireworks { flight_duration } => json!({"flight_duration": flight_duration}),
        Component::Instrument(name) | Component::JukeboxPlayable(name) => json!(name),
        Component::OminousBottleAmplifier(amplifier) => json!(amplifier),
        other => unreachable!("{other:?} is not a single value"),
    }
}

/// `item_declarations`: the declarations as the oracle describes them.
/// An item's `equippable` component as the declarations case writes it.
fn describe_equippable(equip: &photon_data::item::definition::Equip) -> Value {
    let mut described = json!({
        "slot": equip.slot,
        "player": equip.player,
        "swappable": equip.swappable,
        "equip_on_interact": equip.equip_on_interact,
    });
    if !equip.allowed.is_empty() {
        described["allowed"] = json!(equip.allowed);
    }
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
    if let Some(sound) = item.kinetic_sound {
        described["kinetic_sound"] = json!(sound);
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

/// The item classes [`crate::case_ids::oracle_version`]'s Java has, in its order.
fn item_classes() -> Vec<Value> {
    crate::declaration_order::item_classes(crate::case_ids::items())
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
                "parent": class.parent().map_or(String::new(), |parent| {
                    format!("{:?}", crate::case_ids::reference_class(parent))
                }),
                "declares": declares,
            })
        })
        .collect()
}

/// What burns in a furnace or a brewing stand. 26.3 keeps it as a component of the item, and the
/// ingredients a stand takes come from the server's recipes; before, the oracle's Java reads the
/// client's own fuel and ingredient lists.
fn describe_fuel(item: &photon_data::item::definition::Item, described: &mut Value) {
    if crate::case_ids::before_26_3() {
        if item.brewing_ingredient {
            described["brewing_ingredient"] = json!(true);
        }
        if item.burn_time > 0 {
            described["burn_time"] = json!(item.burn_time);
        }
        return;
    }
    if item.burn_time > 0 {
        described["cooking_fuel"] = json!(true);
    }
    if item.name == "minecraft:blaze_powder" {
        described["brewing_fuel"] = json!(true);
    }
}

/// The component types of the oracle's version, in its registry order.
fn present_component_names() -> Vec<&'static str> {
    let component_map = crate::case_ids::registries();
    let mut present: Vec<(u32, &str)> = photon_data::item::component_types::NAMES
        .iter()
        .enumerate()
        .filter_map(|(index, name)| {
            component_map
                .component_types
                .to_wire(index as u32)
                .map(|wire| (wire, *name))
        })
        .collect();
    present.sort_unstable();
    present.into_iter().map(|(_, name)| name).collect()
}

pub(super) fn item_declarations(label: &str) -> Value {
    let classes = item_classes();
    let data_root = crate::jar_root().join("data");
    let registries = photon_game::registries::Registries::vanilla(&data_root);
    let items: Vec<Value> = crate::case_ids::items()
        .map(|item| {
            let mut described = json!({
                "name": item.name,
                "class": format!("{:?}", crate::case_ids::reference_class(item.class)),
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
            describe_fuel(item, &mut described);
            if let Some(block) = item.block {
                described["block"] = json!(block);
            }
            let (components, wire) = default_components(item, &registries);
            described["components"] = components;
            described["component_wire"] = wire;
            described
        })
        .collect();
    let component_types = present_component_names();
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

/// `potions`: every mob effect and potion as the data tables declare them.
pub(super) fn potions(label: &str) -> Value {
    use photon_data::item::effect::{attribute_id, effect_id};
    use photon_data::item::mob_effects::MOB_EFFECTS;
    use photon_data::item::potions::POTIONS;
    let effects: Vec<Value> = MOB_EFFECTS
        .iter()
        .map(|effect| {
            json!({
                "name": effect.name,
                "color": effect.color,
                "description_id": effect.description_id(),
                "style": effect.style(),
                "category": format!("{:?}", effect.category).to_uppercase(),
                "modifiers": effect.modifiers.iter().map(|modifier| json!({
                    "attribute": attribute_id(modifier.attribute),
                    "amount": modifier.amount,
                    "operation": modifier.operation,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    let potions: Vec<Value> = POTIONS
        .iter()
        .map(|potion| {
            json!({
                "name": potion.name,
                "potion_name": potion.potion_name(),
                "effects": potion.effects.iter().map(|effect| json!({
                    "effect": effect_id(effect.effect),
                    "amplifier": effect.amplifier,
                    "duration": effect.duration,
                    "visible": effect.visible,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({
        "label": label,
        "outcome": "listed",
        "effects": effects,
        "potions": potions,
    })
}

/// `tooltip_registries`: the data-driven registries as the game loads them
/// from the data pack, and the attributes' declarations.
pub(super) fn tooltip_registries(label: &str) -> Value {
    use photon_data::item::attributes::ATTRIBUTES;
    use photon_game::registries::{DescribedEntry, Registries};
    let data_root = crate::jar_root().join("data");
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
