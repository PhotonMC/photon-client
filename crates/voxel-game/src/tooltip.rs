//! An item's tooltip: `ItemStack.getTooltipLines`, as the components a stack
//! carries and the item's defaults give it.
//!
//! The name, then what each component says in the reference's order.
//! Ported: the name (custom, item, book title, potion, tipped arrow, shield,
//! player head, compass), painting variants, disc fragments, instruments,
//! fireworks and stars, potion and ominous bottle effects, jukebox songs,
//! armour trims, enchantments, dye, lore, attribute modifiers, unbreakable,
//! banner patterns, suspicious stew and beehive honey. Not ported: smithing
//! templates, map, container, book and bee lines, the adventure predicates
//! and the advanced lines (the `F3+H` toggle is off).

use crate::item_components::ids;
use crate::item_values::{AttributeDisplay, Explosion, PaintingVariant, Values};
use crate::registries::Registries;
use crate::text::{Component, Lang, Style};
use voxel_core::json::JsonValue;
use voxel_data::item::attributes::ATTRIBUTES;
use voxel_data::item::effect::{attribute_id, effect_id};
use voxel_data::item::mob_effects::MOB_EFFECTS;
use voxel_data::item::potions::POTIONS;
use voxel_world::ClientItemStackState as Stack;

/// `TooltipFlag`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    /// `isCreative`: the creative inventory's hovered slot.
    pub creative: bool,
}

/// The two base attributes a player's attribute modifiers are added to.
const PLAYER_ATTACK_DAMAGE: f64 = 1.0;
const PLAYER_ATTACK_SPEED: f64 = 4.0;

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

fn rarity_colour(rarity: i32) -> &'static str {
    match rarity {
        1 => "yellow",
        2 => "aqua",
        3 => "light_purple",
        _ => "white",
    }
}

fn item_name_of(stack: &Stack) -> &'static str {
    usize::try_from(stack.item_registry_id())
        .ok()
        .and_then(|id| voxel_data::item::registry::ITEMS.get(id))
        .map_or("", |item| item.name)
}

/// The description id a name is built on: the key of the item's own name.
fn description_id(values: &Values) -> Option<String> {
    match values.item_name()?.content {
        crate::text::Content::Translatable { key, .. } => Some(key),
        crate::text::Content::Literal(_) => None,
    }
}

/// `PotionContents.getName`.
fn potion_name(prefix: &str, contents: &crate::item_components::PotionContents) -> Component {
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
fn hover_name(stack: &Stack, values: &Values) -> Component {
    if let Some(name) = values.custom_name() {
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
fn dye_name(id: i32) -> &'static str {
    const NAMES: [&str; 16] = [
        "white",
        "orange",
        "magenta",
        "light_blue",
        "yellow",
        "lime",
        "pink",
        "gray",
        "light_gray",
        "cyan",
        "purple",
        "blue",
        "brown",
        "green",
        "red",
        "black",
    ];
    NAMES
        .get(usize::try_from(id).unwrap_or(0))
        .copied()
        .unwrap_or("white")
}

/// `DyeColor.fireworkColor` by id.
const FIREWORK_COLOURS: [i32; 16] = [
    15_790_320, 15_435_844, 12_801_229, 6_719_955, 14_602_026, 4_312_372, 14_188_952, 4_408_131,
    11_250_603, 2_651_799, 8_073_150, 2_437_522, 5_320_730, 3_887_386, 11_743_532, 1_973_019,
];

const SHAPES: [&str; 5] = ["small_ball", "large_ball", "star", "creeper", "burst"];

fn shape_name(shape: i32) -> Component {
    let name = SHAPES
        .get(usize::try_from(shape).unwrap_or(0))
        .copied()
        .unwrap_or("small_ball");
    Component::translatable(format!("item.minecraft.firework_star.shape.{name}"))
}

fn colour_names(colours: &[i32], mut line: Component) -> Component {
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
fn explosion_details(explosion: &Explosion, out: &mut Vec<Component>) {
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
fn format_ticks(ticks: i32) -> String {
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
fn potion_tooltip(effects: &[(i32, i32, i32)], scale: f32, out: &mut Vec<Component>) {
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

fn attribute_name(attribute: i32) -> Component {
    usize::try_from(attribute)
        .ok()
        .and_then(|id| ATTRIBUTES.get(id))
        .map_or_else(Component::empty, |entry| {
            Component::translatable(entry.description_id())
        })
}

fn colour_of(formatting: &str) -> &'static str {
    match formatting {
        "RED" => "red",
        "GRAY" => "gray",
        _ => "blue",
    }
}

fn json_component(value: &JsonValue) -> Option<Component> {
    Component::from_json(value)
}

/// `ItemStack.addAttributeTooltips`.
fn attribute_tooltips(values: &Values, player: bool, out: &mut Vec<Component>) {
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
fn enchantment_line(registries: &Registries, id: i32, level: i32) -> Option<Component> {
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
fn enchantment_tooltip(registries: &Registries, entries: &[(i32, i32)], out: &mut Vec<Component>) {
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
    if values.custom_name().is_some() {
        name = name.italic();
    }
    name
}

/// `ItemStack.getTooltipLines`. `player` is whether a player is at hand,
/// which the attribute lines add its base values for.
#[expect(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    reason = "kept whole to read against the reference method line by line"
)]
pub fn lines(stack: &Stack, player: bool, flags: Flags, registries: &Registries) -> Vec<Component> {
    let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
    let (hide, hidden) = values.tooltip_display();
    if !flags.creative && hide {
        return Vec::new();
    }
    let shows = |component: i32| !hide && !hidden.contains(&component);
    let mut out = vec![styled_name(stack, &values)];
    let item = item_name_of(stack);

    // `HangingEntityItem.appendHoverText`.
    if item == "minecraft:painting" && shows(ids::PAINTING_VARIANT) {
        match values.painting_variant() {
            Some(PaintingVariant::Registry(id)) => {
                if let Some(variant) = registries.painting_variants.get(id as usize) {
                    out.extend(variant.title.as_ref().and_then(json_component));
                    out.extend(variant.author.as_ref().and_then(json_component));
                    out.push(Component::translatable_with(
                        "painting.dimensions",
                        vec![
                            Component::literal(variant.width.to_string()),
                            Component::literal(variant.height.to_string()),
                        ],
                    ));
                }
            }
            Some(PaintingVariant::Direct {
                width,
                height,
                title,
                author,
            }) => {
                out.extend(title);
                out.extend(author);
                out.push(Component::translatable_with(
                    "painting.dimensions",
                    vec![
                        Component::literal(width.to_string()),
                        Component::literal(height.to_string()),
                    ],
                ));
            }
            None if flags.creative => {
                out.push(Component::translatable("painting.random").with_colour("gray"));
            }
            None => {}
        }
    }
    // `DiscFragmentItem.appendHoverText`.
    if item == "minecraft:disc_fragment_5"
        && let Some(id) = description_id(&values)
    {
        out.push(Component::translatable(format!("{id}.desc")).with_colour("gray"));
    }
    if shows(ids::INSTRUMENT)
        && let Some(id) = values.holder_id(ids::INSTRUMENT)
        && let Some(entry) = registries.instruments.get(id as usize)
        && let Some(description) = entry.description.as_ref().and_then(json_component)
    {
        out.push(description.with_style(&Style::coloured("gray")));
    }
    if shows(ids::FIREWORKS)
        && let Some((flight, explosions)) = values.fireworks()
    {
        if flight > 0 {
            out.push(
                Component::translatable("item.minecraft.firework_rocket.flight")
                    .append(Component::literal(" "))
                    .append(Component::literal(flight.to_string()))
                    .with_colour("gray"),
            );
        }
        let mut current: Option<(&Explosion, i32)> = None;
        let flush = |current: Option<(&Explosion, i32)>, out: &mut Vec<Component>| {
            if let Some((explosion, count)) = current {
                out.push(if count == 1 {
                    Component::translatable_with(
                        "item.minecraft.firework_rocket.single_star",
                        vec![shape_name(explosion.shape)],
                    )
                    .with_colour("gray")
                } else {
                    Component::translatable_with(
                        "item.minecraft.firework_rocket.multiple_stars",
                        vec![
                            Component::literal(count.to_string()),
                            shape_name(explosion.shape),
                        ],
                    )
                    .with_colour("gray")
                });
                let mut details = Vec::new();
                explosion_details(explosion, &mut details);
                out.extend(
                    details
                        .into_iter()
                        .map(|line| Component::literal(" ").append(line).with_colour("gray")),
                );
            }
        };
        for explosion in &explosions {
            current = match current {
                None => Some((explosion, 1)),
                Some((present, count)) if present == explosion => Some((present, count + 1)),
                other => {
                    flush(other, &mut out);
                    Some((explosion, 1))
                }
            };
        }
        flush(current, &mut out);
    }
    if shows(ids::FIREWORK_EXPLOSION)
        && let Some(explosion) = values.firework_explosion()
    {
        out.push(shape_name(explosion.shape).with_colour("gray"));
        explosion_details(&explosion, &mut out);
    }
    if shows(ids::POTION_CONTENTS)
        && let Some(contents) = values.potion_contents()
    {
        let mut effects: Vec<(i32, i32, i32)> = contents
            .potion
            .and_then(|id| POTIONS.get(usize::try_from(id).ok()?))
            .map(|potion| {
                potion
                    .effects
                    .iter()
                    .map(|effect| {
                        (
                            effect_id(effect.effect).unwrap_or(-1),
                            effect.amplifier,
                            effect.duration,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        effects.extend(
            contents
                .effects
                .iter()
                .map(|&(effect, amplifier, duration, _)| (effect, amplifier, duration)),
        );
        potion_tooltip(&effects, values.potion_duration_scale(), &mut out);
    }
    if shows(ids::JUKEBOX_PLAYABLE)
        && let Some(id) = values.holder_id(ids::JUKEBOX_PLAYABLE)
        && let Some(entry) = registries.jukebox_songs.get(id as usize)
        && let Some(description) = entry.description.as_ref().and_then(json_component)
    {
        out.push(description.with_style(&Style::coloured("gray")));
    }
    if shows(ids::TRIM)
        && let Some((material, pattern)) = values.trim()
        && let (Some(material), Some(pattern)) = (
            registries.trim_materials.get(material as usize),
            registries.trim_patterns.get(pattern as usize),
        )
    {
        out.push(
            Component::translatable("item.minecraft.smithing_template.upgrade").with_colour("gray"),
        );
        let material = material
            .description
            .as_ref()
            .and_then(json_component)
            .unwrap_or_else(Component::empty);
        let pattern = pattern
            .description
            .as_ref()
            .and_then(json_component)
            .unwrap_or_else(Component::empty);
        out.push(Component::literal(" ").append(pattern.with_style(&material.style)));
        out.push(Component::literal(" ").append(material));
    }
    if shows(ids::STORED_ENCHANTMENTS) {
        enchantment_tooltip(
            registries,
            &values.enchantments(ids::STORED_ENCHANTMENTS),
            &mut out,
        );
    }
    if shows(ids::ENCHANTMENTS) {
        enchantment_tooltip(
            registries,
            &values.enchantments(ids::ENCHANTMENTS),
            &mut out,
        );
    }
    if shows(ids::DYED_COLOR) && values.dyed_color().is_some() {
        out.push(
            Component::translatable("item.dyed")
                .with_colour("gray")
                .italic(),
        );
    }
    if shows(ids::LORE) {
        let lore = Style {
            colour: crate::text::named_colour("dark_purple"),
            italic: Some(true),
            ..Style::default()
        };
        out.extend(values.lore().into_iter().map(|line| line.with_style(&lore)));
    }
    if shows(ids::BANNER_PATTERNS) {
        for (pattern, dye) in values.banner_layers().into_iter().take(6) {
            if let Some(entry) = registries.banner_patterns.get(pattern as usize) {
                out.push(
                    Component::translatable(format!("{}.{}", entry.translation_key, dye_name(dye)))
                        .with_colour("gray"),
                );
            }
        }
    }
    attribute_tooltips_if(shows(ids::ATTRIBUTE_MODIFIERS), &values, player, &mut out);
    if shows(ids::INTANGIBLE_PROJECTILE) && values.has(ids::INTANGIBLE_PROJECTILE) {
        out.push(Component::translatable("item.intangible").with_colour("gray"));
    }
    if shows(ids::UNBREAKABLE) && values.has(ids::UNBREAKABLE) {
        out.push(Component::translatable("item.unbreakable").with_colour("blue"));
    }
    if shows(ids::OMINOUS_BOTTLE_AMPLIFIER)
        && let Some(amplifier) = values
            .raw(ids::OMINOUS_BOTTLE_AMPLIFIER)
            .and_then(crate::item_components::varint_value)
        && let Some(bad_omen) = MOB_EFFECTS
            .iter()
            .position(|effect| effect.name == "minecraft:bad_omen")
    {
        potion_tooltip(&[(bad_omen as i32, amplifier, 120_000)], 1.0, &mut out);
    }
    if flags.creative && shows(ids::SUSPICIOUS_STEW_EFFECTS) {
        let effects: Vec<(i32, i32, i32)> = values
            .suspicious_stew()
            .into_iter()
            .map(|(effect, duration)| (effect, 0, duration))
            .collect();
        if !effects.is_empty() {
            potion_tooltip(&effects, 1.0, &mut out);
        }
    }
    if shows(ids::BLOCK_STATE)
        && let Some((_, level)) = values
            .block_state()
            .into_iter()
            .find(|(key, _)| key == "honey_level")
    {
        out.push(
            Component::translatable_with(
                "container.beehive.honey",
                vec![Component::literal(level), Component::literal("5")],
            )
            .with_colour("gray"),
        );
    }
    out
}

fn attribute_tooltips_if(shown: bool, values: &Values, player: bool, out: &mut Vec<Component>) {
    if shown {
        attribute_tooltips(values, player, out);
    }
}

/// The tooltip's lines as the search reads them: formatting stripped, each
/// trimmed, the empty ones left out.
pub fn search_lines(stack: &Stack, lang: &Lang, registries: &Registries) -> Vec<String> {
    lines(stack, false, Flags { creative: true }, registries)
        .iter()
        .map(|line| lang.plain(line).trim().to_owned())
        .filter(|line| !line.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The vanilla data pack next to the assets, when there is one.
    fn registries() -> Registries {
        let assets = std::path::PathBuf::from(std::env::var("VOXEL_ASSETS").unwrap_or_default());
        Registries::vanilla(&assets.with_file_name("data"))
    }

    fn lang() -> Lang {
        Lang::load(std::path::Path::new(
            &std::env::var("VOXEL_ASSETS").unwrap_or_default(),
        ))
    }

    #[test]
    fn decimals_are_cut_to_two_places() {
        assert_eq!(format_decimal(7.0), "7");
        assert_eq!(format_decimal(1.6), "1.6");
        assert_eq!(format_decimal(-0.4), "-0.4");
        assert_eq!(format_decimal(0.125), "0.12");
        assert_eq!(format_decimal(-0.0), "0");
    }

    #[test]
    fn ticks_read_as_minutes_and_seconds() {
        assert_eq!(format_ticks(3600), "03:00");
        assert_eq!(format_ticks(20), "00:01");
        assert_eq!(format_ticks(20 * 3600), "01:00:00");
    }

    #[test]
    fn a_plain_item_has_only_its_name() {
        let stone = crate::item::id_of("minecraft:stone").expect("an item");
        let lines = lines(
            &Stack::new(1, stone),
            false,
            Flags::default(),
            &registries(),
        );
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn every_creative_stack_has_a_name_and_no_panic() {
        let lang = lang();
        if lang.get("item.minecraft.stone").is_none() {
            return;
        }
        let registries = registries();
        for tab in crate::creative_menu::shown_tabs() {
            for &index in voxel_data::item::creative_tabs::CREATIVE_TABS[tab].items {
                let stack = crate::creative_menu::table_stack(index);
                let text = search_lines(&stack, &lang, &registries);
                assert!(!text.is_empty(), "{}", item_name_of(&stack));
            }
        }
    }
}
