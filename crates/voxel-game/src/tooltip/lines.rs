use super::*;

/// `ItemStack.getTooltipLines`. `player` is whether a player is at hand,
/// which the attribute lines add its base values for.
#[expect(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    reason = "kept whole to read against the reference method line by line"
)]
pub fn lines(stack: &Stack, player: bool, flags: Flags, registries: &Registries) -> Vec<Component> {
    let values = Values::with_registries(stack.item_registry_id(), stack.patch_bytes(), registries);
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

pub(super) fn attribute_tooltips_if(
    shown: bool,
    values: &Values,
    player: bool,
    out: &mut Vec<Component>,
) {
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
