//! An item's default components as the network writes them: the typed values
//! an item declares ([`voxel_data::item::component::Component`]) encoded the
//! way each component's stream codec writes its value, so they read through
//! the same code as the components a server sends.

use voxel_core::wire::write_varint;
use voxel_data::item::component::{
    AttributeDisplay, AttributeModifier, Component, Operation, Rarity, SlotGroup,
};
use voxel_data::item::component_types as ids;
use voxel_data::item::definition::Item;
use voxel_data::item::potions::POTIONS;

use crate::item_components::Patch;
use crate::registries::Registries;

/// The translation key an item's name defaults to: its block's for a block
/// item (`Item.getDescriptionId`).
pub fn description_key(item: &Item) -> String {
    let (kind, name) = match item.block {
        Some(block) => ("block", block),
        None => ("item", item.name),
    };
    let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
    format!("{kind}.{namespace}.{path}")
}

fn string(out: &mut Vec<u8>, text: &str) {
    write_varint(text.len() as i32, out);
    out.extend_from_slice(text.as_bytes());
}

/// A translatable text component as the network writes it: a nameless
/// compound holding a `translate` string.
pub fn translatable(key: &str) -> Vec<u8> {
    let mut out = vec![0x0a, 0x08];
    out.extend_from_slice(&(b"translate".len() as u16).to_be_bytes());
    out.extend_from_slice(b"translate");
    out.extend_from_slice(&(key.len() as u16).to_be_bytes());
    out.extend_from_slice(key.as_bytes());
    out.push(0x00);
    out
}

fn rarity(rarity: Rarity) -> i32 {
    match rarity {
        Rarity::Common => 0,
        Rarity::Uncommon => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 3,
    }
}

fn operation(operation: Operation) -> i32 {
    match operation {
        Operation::AddValue => 0,
        Operation::AddMultipliedBase => 1,
        Operation::AddMultipliedTotal => 2,
    }
}

fn slot(slot: SlotGroup) -> i32 {
    match slot {
        SlotGroup::Any => 0,
        SlotGroup::MainHand => 1,
        SlotGroup::OffHand => 2,
        SlotGroup::Hand => 3,
        SlotGroup::Feet => 4,
        SlotGroup::Legs => 5,
        SlotGroup::Chest => 6,
        SlotGroup::Head => 7,
        SlotGroup::Armor => 8,
        SlotGroup::Body => 9,
        SlotGroup::Saddle => 10,
    }
}

fn attribute_modifier(modifier: &AttributeModifier, out: &mut Vec<u8>) {
    let id = voxel_data::item::effect::attribute_id(modifier.attribute).unwrap_or(-1);
    write_varint(id, out);
    string(out, modifier.id);
    out.extend_from_slice(&modifier.amount.to_be_bytes());
    write_varint(operation(modifier.operation), out);
    write_varint(slot(modifier.slot), out);
    write_varint(
        match modifier.display {
            AttributeDisplay::Default => 0,
            AttributeDisplay::Hidden => 1,
        },
        out,
    );
}

/// `DyeColor.getName`, by `DyeColor.getId`.
pub(crate) const DYE_NAMES: [&str; 16] = [
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

/// `DyeColor.getId`.
fn dye_id(name: &str) -> Option<i32> {
    DYE_NAMES
        .iter()
        .position(|dye| *dye == name)
        .map(|id| id as i32)
}

/// A holder of a registry entry as the network writes it: the entry's place
/// plus one, since zero says the entry follows written out.
fn holder(names: impl Iterator<Item = String>, wanted: &str, out: &mut Vec<u8>) -> Option<()> {
    let place = names.into_iter().position(|name| name == wanted)?;
    write_varint(place as i32 + 1, out);
    Some(())
}

/// A component's type id and value bytes; `None` when the value names an
/// entry of a registry that is not known.
#[expect(clippy::too_many_lines, reason = "one arm per component kind")]
pub fn encode(component: &Component, registries: Option<&Registries>) -> Option<(i32, Vec<u8>)> {
    let mut out = Vec::new();
    let id = match component {
        Component::ItemName(key) => {
            out = translatable(key);
            ids::ITEM_NAME
        }
        Component::Rarity(value) => {
            write_varint(rarity(*value), &mut out);
            ids::RARITY
        }
        Component::AttributeModifiers(modifiers) => {
            write_varint(modifiers.len() as i32, &mut out);
            for modifier in *modifiers {
                attribute_modifier(modifier, &mut out);
            }
            ids::ATTRIBUTE_MODIFIERS
        }
        Component::EnchantmentGlintOverride(value) => {
            out.push(u8::from(*value));
            ids::ENCHANTMENT_GLINT_OVERRIDE
        }
        Component::MapColor(color) => {
            out.extend_from_slice(&color.to_be_bytes());
            ids::MAP_COLOR
        }
        Component::PotionContents(potion) => {
            match potion {
                Some(name) => {
                    let id = POTIONS.iter().position(|potion| potion.name == *name)?;
                    out.push(1);
                    write_varint(id as i32, &mut out);
                }
                None => out.push(0),
            }
            // No colour, no effects, no name.
            out.extend_from_slice(&[0, 0, 0]);
            ids::POTION_CONTENTS
        }
        Component::PotionDurationScale(scale) => {
            out.extend_from_slice(&scale.to_be_bytes());
            ids::POTION_DURATION_SCALE
        }
        Component::Fireworks { flight_duration } => {
            write_varint(*flight_duration, &mut out);
            write_varint(0, &mut out);
            ids::FIREWORKS
        }
        Component::Instrument(name) => {
            let registries = registries?;
            holder(
                registries
                    .instruments
                    .iter()
                    .map(|entry| entry.name.clone()),
                name,
                &mut out,
            )?;
            ids::INSTRUMENT
        }
        Component::JukeboxPlayable(name) => {
            let registries = registries?;
            holder(
                registries
                    .jukebox_songs
                    .iter()
                    .map(|entry| entry.name.clone()),
                name,
                &mut out,
            )?;
            ids::JUKEBOX_PLAYABLE
        }
        Component::BlockState(properties) => {
            write_varint(properties.len() as i32, &mut out);
            for (key, value) in *properties {
                string(&mut out, key);
                string(&mut out, value);
            }
            ids::BLOCK_STATE
        }
        Component::OminousBottleAmplifier(amplifier) => {
            write_varint(*amplifier, &mut out);
            ids::OMINOUS_BOTTLE_AMPLIFIER
        }
        Component::StoredEnchantments(enchantments) => {
            let registries = registries?;
            write_varint(enchantments.len() as i32, &mut out);
            for (name, level) in *enchantments {
                let id = registries
                    .enchantments
                    .iter()
                    .position(|entry| entry.name == *name)?;
                write_varint(id as i32, &mut out);
                write_varint(*level, &mut out);
            }
            ids::STORED_ENCHANTMENTS
        }
        Component::PaintingVariant(name) => {
            let registries = registries?;
            holder(
                registries
                    .painting_variants
                    .iter()
                    .map(|entry| entry.name.clone()),
                name,
                &mut out,
            )?;
            ids::PAINTING_VARIANT
        }
        Component::SuspiciousStewEffects(effects) => {
            write_varint(effects.len() as i32, &mut out);
            for (name, duration) in *effects {
                write_varint(voxel_data::item::effect::effect_id(name)?, &mut out);
                write_varint(*duration, &mut out);
            }
            ids::SUSPICIOUS_STEW_EFFECTS
        }
        Component::BannerPatterns(layers) => {
            let registries = registries?;
            write_varint(layers.len() as i32, &mut out);
            for (pattern, dye) in *layers {
                holder(
                    registries
                        .banner_patterns
                        .iter()
                        .map(|entry| entry.name.clone()),
                    pattern,
                    &mut out,
                )?;
                write_varint(dye_id(dye)?, &mut out);
            }
            ids::BANNER_PATTERNS
        }
        Component::TooltipDisplay(hidden) => {
            // The tooltip is shown; only these components are left out.
            out.push(0);
            write_varint(hidden.len() as i32, &mut out);
            for name in *hidden {
                write_varint(ids::id(name)?, &mut out);
            }
            ids::TOOLTIP_DISPLAY
        }
    };
    Some((id, out))
}

/// An item's default components, in component type order: its name and what
/// it declares. Components the registries cannot resolve are left out.
pub fn defaults(item: &Item, registries: Option<&Registries>) -> Patch {
    let mut added: Vec<(i32, Vec<u8>)> = Vec::new();
    if !item
        .components
        .iter()
        .any(|component| matches!(component, Component::ItemName(_)))
    {
        added.push((ids::ITEM_NAME, translatable(&description_key(item))));
    }
    added.extend(
        item.components
            .iter()
            .filter_map(|component| encode(component, registries)),
    );
    added.sort_by_key(|(id, _)| *id);
    Patch {
        added,
        removed: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::values::Values;

    fn item(name: &str) -> i32 {
        crate::item::id_of(name).expect("an item")
    }

    fn registries() -> Registries {
        let assets = std::path::PathBuf::from(std::env::var("VOXEL_ASSETS").unwrap_or_default());
        Registries::vanilla(&assets.with_file_name("data"))
    }

    #[test]
    fn a_blocks_item_is_named_by_its_block() {
        let stone = voxel_data::item::registry::item("minecraft:stone").expect("stone");
        assert_eq!(description_key(stone), "block.minecraft.stone");
        let stick = voxel_data::item::registry::item("minecraft:stick").expect("stick");
        assert_eq!(description_key(stick), "item.minecraft.stick");
    }

    #[test]
    fn a_default_name_reads_back_as_the_translation() {
        let values = Values::new(item("minecraft:stone"), &[]);
        let name = values.item_name().expect("an item name");
        assert!(matches!(
            name.content,
            crate::text::Content::Translatable { ref key, .. } if key == "block.minecraft.stone"
        ));
    }

    #[test]
    fn default_armour_reads_back_its_attributes() {
        let values = Values::new(item("minecraft:turtle_helmet"), &[]);
        let entries = values.attribute_modifiers();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "minecraft:armor.helmet");
        assert_eq!(entries[0].amount, 2.0);
        assert_eq!(entries[0].slot, 7);
    }

    #[test]
    fn a_default_rarity_is_declared_and_common_is_not() {
        assert_eq!(Values::new(item("minecraft:goat_horn"), &[]).rarity(), 1);
        assert_eq!(Values::new(item("minecraft:stone"), &[]).rarity(), 0);
    }

    #[test]
    fn a_default_holder_needs_the_registries_to_find_its_id() {
        let horn = item("minecraft:goat_horn");
        assert_eq!(Values::new(horn, &[]).holder_id(ids::INSTRUMENT), None);
        let registries = registries();
        if registries.instruments.is_empty() {
            return;
        }
        let values = Values::with_registries(horn, &[], &registries);
        let ponder = registries
            .instruments
            .iter()
            .position(|entry| entry.name == "minecraft:ponder_goat_horn");
        assert_eq!(
            values.holder_id(ids::INSTRUMENT),
            ponder.map(|at| at as i32)
        );
    }

    #[test]
    fn a_potion_has_empty_potion_contents_by_default() {
        let values = Values::new(item("minecraft:potion"), &[]);
        let contents = values.potion_contents().expect("potion contents");
        assert_eq!(contents.potion, None);
    }
}
