use super::*;

/// `DecimalFormat("#.##")`: up to two decimals, none when whole.
pub fn format_decimal(value: f64) -> String {
    let text = format!("{value:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" || text.is_empty() {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

pub(super) fn rarity_colour(rarity: i32) -> &'static str {
    match rarity {
        1 => "yellow",
        2 => "aqua",
        3 => "light_purple",
        _ => "white",
    }
}

pub(super) fn item_name_of(stack: &Stack) -> &'static str {
    usize::try_from(stack.item_registry_id())
        .ok()
        .and_then(|id| photon_data::item::registry::ITEMS.get(id))
        .map_or("", |item| item.name)
}

/// The description id a name is built on: the key of the item's own name.
pub(super) fn description_id(values: &Values) -> Option<String> {
    match values.item_name()?.content {
        crate::text::Content::Translatable { key, .. } => Some(key),
        crate::text::Content::Literal(_) => None,
    }
}

/// `PotionContents.getName`.
pub(super) fn potion_name(
    prefix: &str,
    contents: &crate::item_components::PotionContents,
) -> Component {
    let suffix = contents
        .custom_name
        .clone()
        .or_else(|| {
            contents
                .potion
                .and_then(|id| POTIONS.get(usize::try_from(id).ok()?))
                .map(|potion| potion.potion_name().to_owned())
        })
        .unwrap_or_else(|| "empty".to_owned());
    Component::translatable(format!("{prefix}{suffix}"))
}

/// `ItemStack.getHoverName`.
pub fn hover_name(stack: &Stack, values: &Values) -> Component {
    if let Some(name) = values.custom_name().or_else(|| book_title(values)) {
        return name;
    }
    let item = item_name_of(stack);
    let base = description_id(values);
    let named = |suffix: &str| {
        base.as_ref()
            .map(|id| Component::translatable(format!("{id}{suffix}")))
    };
    match item {
        "minecraft:potion"
        | "minecraft:splash_potion"
        | "minecraft:lingering_potion"
        | "minecraft:tipped_arrow" => {
            if let (Some(contents), Some(id)) = (values.potion_contents(), &base) {
                return potion_name(&format!("{id}.effect."), &contents);
            }
        }
        "minecraft:shield" => {
            if let (Some(dye), Some(id)) = (values.raw(ids::BASE_COLOR), &base)
                && let Some(dye) = crate::item_components::varint_value(dye)
            {
                return Component::translatable(format!("{id}.{}", dye_name(dye)));
            }
        }
        "minecraft:compass" if values.has(ids::LODESTONE_TRACKER) => {
            return Component::translatable("item.minecraft.lodestone_compass");
        }
        _ => {}
    }
    let _ = named;
    values.item_name().unwrap_or_else(Component::empty)
}

/// `DyeColor.getName` by id.
pub(super) fn dye_name(id: i32) -> &'static str {
    crate::item::default_components::DYE_NAMES
        .get(usize::try_from(id).unwrap_or(0))
        .copied()
        .unwrap_or("white")
}

pub(super) fn shape_name(shape: i32) -> Component {
    let name = SHAPES
        .get(usize::try_from(shape).unwrap_or(0))
        .copied()
        .unwrap_or("small_ball");
    Component::translatable(format!("item.minecraft.firework_star.shape.{name}"))
}

pub(super) fn colour_names(colours: &[i32], mut line: Component) -> Component {
    for (index, colour) in colours.iter().enumerate() {
        if index > 0 {
            line = line.append(Component::literal(", "));
        }
        let name = FIREWORK_COLOURS
            .iter()
            .position(|firework| firework == &(colour & 0xFF_FFFF))
            .map_or_else(
                || "item.minecraft.firework_star.custom_color".to_owned(),
                |dye| format!("item.minecraft.firework_star.{}", dye_name(dye as i32)),
            );
        line = line.append(Component::translatable(name));
    }
    line
}

/// `FireworkExplosion.addAdditionalTooltip`.
pub(super) fn explosion_details(explosion: &Explosion, out: &mut Vec<Component>) {
    if !explosion.colors.is_empty() {
        out.push(colour_names(
            &explosion.colors,
            Component::empty().with_colour("gray"),
        ));
    }
    if !explosion.fade_colors.is_empty() {
        out.push(colour_names(
            &explosion.fade_colors,
            Component::translatable("item.minecraft.firework_star.fade_to")
                .append(Component::literal(" "))
                .with_colour("gray"),
        ));
    }
    if explosion.trail {
        out.push(Component::translatable("item.minecraft.firework_star.trail").with_colour("gray"));
    }
    if explosion.twinkle {
        out.push(
            Component::translatable("item.minecraft.firework_star.flicker").with_colour("gray"),
        );
    }
}

/// `StringUtil.formatTickDuration` at twenty ticks a second.
pub(super) fn format_ticks(ticks: i32) -> String {
    let seconds = ticks.div_euclid(20);
    let (minutes, seconds) = (seconds / 60, seconds % 60);
    let (hours, minutes) = (minutes / 60, minutes % 60);
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

/// `PotionContents.addPotionTooltip`: `(effect id, amplifier, duration)`.
pub(super) fn potion_tooltip(effects: &[(i32, i32, i32)], scale: f32, out: &mut Vec<Component>) {
    let mut modifiers: Vec<(i32, f64, i32)> = Vec::new();
    for &(effect, amplifier, duration) in effects {
        let Some(entry) = usize::try_from(effect)
            .ok()
            .and_then(|id| MOB_EFFECTS.get(id))
        else {
            continue;
        };
        for modifier in entry.modifiers {
            modifiers.push((
                attribute_id(modifier.attribute).unwrap_or(-1),
                modifier.amount * f64::from(amplifier + 1),
                modifier.operation,
            ));
        }
        let mut line = Component::translatable(entry.description_id());
        if amplifier > 0 {
            line = Component::translatable_with(
                "potion.withAmplifier",
                vec![
                    line,
                    Component::translatable(format!("potion.potency.{amplifier}")),
                ],
            );
        }
        // `MobEffectInstance.endsWithin(20)`: not one that never ends.
        if duration == -1 || duration > 20 {
            let time = if duration == -1 {
                Component::translatable("effect.duration.infinite")
            } else {
                Component::literal(format_ticks((duration as f32 * scale).floor() as i32))
            };
            line = Component::translatable_with("potion.withDuration", vec![line, time]);
        }
        out.push(line.with_colour(if entry.style() == "RED" {
            "red"
        } else {
            "blue"
        }));
    }
    if effects.is_empty() {
        out.push(Component::translatable("effect.none").with_colour("gray"));
    }
    if !modifiers.is_empty() {
        out.push(Component::empty());
        out.push(Component::translatable("potion.whenDrank").with_colour("dark_purple"));
        for (attribute, amount, operation) in modifiers {
            let mut display = amount;
            if operation == 1 || operation == 2 {
                display *= 100.0;
            }
            let name = attribute_name(attribute);
            let (key, shown, colour) = if amount > 0.0 {
                ("plus", display, "blue")
            } else if amount < 0.0 {
                ("take", -display, "red")
            } else {
                continue;
            };
            out.push(
                Component::translatable_with(
                    format!("attribute.modifier.{key}.{operation}"),
                    vec![Component::literal(format_decimal(shown)), name],
                )
                .with_colour(colour),
            );
        }
    }
}

pub(super) fn attribute_name(attribute: i32) -> Component {
    usize::try_from(attribute)
        .ok()
        .and_then(|id| ATTRIBUTES.get(id))
        .map_or_else(Component::empty, |entry| {
            Component::translatable(entry.description_id())
        })
}

pub(super) fn colour_of(formatting: &str) -> &'static str {
    match formatting {
        "RED" => "red",
        "GRAY" => "gray",
        _ => "blue",
    }
}

pub(super) fn json_component(value: &JsonValue) -> Option<Component> {
    Component::from_json(value)
}

/// `ItemStack.addAttributeTooltips`.
pub(super) fn attribute_tooltips(values: &Values, player: bool, out: &mut Vec<Component>) {
    const GROUPS: [&str; 11] = [
        "any", "mainhand", "offhand", "hand", "feet", "legs", "chest", "head", "armor", "body",
        "saddle",
    ];
    let entries = values.attribute_modifiers();
    for (slot, group) in GROUPS.iter().enumerate() {
        let mut first = true;
        for entry in entries.iter().filter(|entry| entry.slot == slot as i32) {
            let mut lines = Vec::new();
            match &entry.display {
                AttributeDisplay::Hidden => continue,
                AttributeDisplay::Override(component) => lines.push(component.clone()),
                AttributeDisplay::Default => {
                    let attribute = usize::try_from(entry.attribute)
                        .ok()
                        .and_then(|id| ATTRIBUTES.get(id));
                    let mut amount = entry.amount;
                    let mut with_base = false;
                    if player && entry.id == "minecraft:base_attack_damage" {
                        amount += PLAYER_ATTACK_DAMAGE;
                        with_base = true;
                    } else if player && entry.id == "minecraft:base_attack_speed" {
                        amount += PLAYER_ATTACK_SPEED;
                        with_base = true;
                    }
                    let mut display = amount;
                    if entry.operation == 1 || entry.operation == 2 {
                        display = amount * 100.0;
                    } else if attribute.is_some_and(|a| a.name == "minecraft:knockback_resistance")
                    {
                        display = amount * 10.0;
                    }
                    let name = attribute_name(entry.attribute);
                    let (up, down) = attribute.map_or(("BLUE", "RED"), |a| a.styles());
                    let arg = |value: f64| Component::literal(format_decimal(value));
                    if with_base {
                        lines.push(
                            Component::literal(" ")
                                .append(Component::translatable_with(
                                    format!("attribute.modifier.equals.{}", entry.operation),
                                    vec![arg(display), name],
                                ))
                                .with_colour("dark_green"),
                        );
                    } else if amount > 0.0 {
                        lines.push(
                            Component::translatable_with(
                                format!("attribute.modifier.plus.{}", entry.operation),
                                vec![arg(display), name],
                            )
                            .with_colour(colour_of(up)),
                        );
                    } else if amount < 0.0 {
                        lines.push(
                            Component::translatable_with(
                                format!("attribute.modifier.take.{}", entry.operation),
                                vec![arg(-display), name],
                            )
                            .with_colour(colour_of(down)),
                        );
                    }
                }
            }
            if first {
                out.push(Component::empty());
                out.push(
                    Component::translatable(format!("item.modifiers.{group}")).with_colour("gray"),
                );
                first = false;
            }
            out.extend(lines);
        }
    }
}

/// `Enchantment.getFullname`.
pub(super) fn enchantment_line(registries: &Registries, id: i32, level: i32) -> Option<Component> {
    let entry = registries.enchantments.get(usize::try_from(id).ok()?)?;
    let description = json_component(entry.description.as_ref()?)?;
    let mut line =
        description.with_style(&Style::coloured(if entry.curse { "red" } else { "gray" }));
    if level != 1 || entry.max_level != 1 {
        line = line
            .append(Component::literal(" "))
            .append(Component::translatable(format!(
                "enchantment.level.{level}"
            )));
    }
    Some(line)
}

/// `ItemEnchantments.addToTooltip`: the enchantments the tooltip order names
/// first, in its order, then the rest.
pub(super) fn enchantment_tooltip(
    registries: &Registries,
    entries: &[(i32, i32)],
    out: &mut Vec<Component>,
) {
    let mut ordered: Vec<(i32, i32, i32)> = registries
        .enchantments
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.tooltip_order >= 0)
        .map(|(id, entry)| (entry.tooltip_order, id as i32, 0))
        .collect();
    ordered.sort();
    for (_, id, _) in ordered {
        if let Some(&(_, level)) = entries.iter().find(|(entry, _)| *entry == id)
            && level > 0
        {
            out.extend(enchantment_line(registries, id, level));
        }
    }
    for &(id, level) in entries {
        let in_order = registries
            .enchantments
            .get(usize::try_from(id).unwrap_or(usize::MAX))
            .is_some_and(|entry| entry.tooltip_order >= 0);
        if !in_order {
            out.extend(enchantment_line(registries, id, level));
        }
    }
}

/// `ItemStack.getRarity`.
pub fn rarity(values: &Values) -> i32 {
    let base = values.rarity();
    if values.enchantments(ids::ENCHANTMENTS).is_empty() {
        return base;
    }
    match base {
        0 | 1 => 2,
        2 => 3,
        other => other,
    }
}

/// `ItemStack.getStyledHoverName`.
pub fn styled_name(stack: &Stack, values: &Values) -> Component {
    let mut name = Component::empty()
        .append(hover_name(stack, values))
        .with_colour(rarity_colour(rarity(values)));
    if values.has(ids::CUSTOM_NAME) {
        name = name.italic();
    }
    name
}
