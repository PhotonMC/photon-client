//! The components of a stack as a block entity's tag holds them (`ItemStack.CODEC`), put into the
//! patch the network form has. Only what the renderers of a shown item read is decoded: the
//! potion and the custom colour of `potion_contents`, and the `enchantments` (the glint).

use photon_core::nbt::{Field, FieldValue};
use photon_core::wire::write_varint;

use super::{Patch, ids};

fn field<'a>(fields: &'a [Field], key: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|own| own.key == key)
        .map(|own| &own.value)
}

/// The patch a `components` compound makes. `enchantments` are the names of the enchantment
/// registry in the order the server synchronised it.
pub fn patch_from_tag(components: &[Field], enchantments: &[String]) -> Patch {
    let mut added = Vec::new();
    if let Some(value) = field(components, "minecraft:potion_contents") {
        added.extend(potion_contents(value).map(|bytes| (ids::POTION_CONTENTS, bytes)));
    }
    if let Some(FieldValue::Compound(levels)) = field(components, "minecraft:enchantments") {
        added.push((ids::ENCHANTMENTS, item_enchantments(levels, enchantments)));
    }
    Patch {
        added,
        removed: Vec::new(),
    }
}

/// `PotionContents.CODEC`: a potion alone, or the record of a potion and a colour.
fn potion_contents(value: &FieldValue) -> Option<Vec<u8>> {
    let (potion, color) = match value {
        FieldValue::Str(name) => (Some(name.as_str()), None),
        FieldValue::Compound(record) => (
            match field(record, "potion") {
                Some(FieldValue::Str(name)) => Some(name.as_str()),
                _ => None,
            },
            match field(record, "custom_color") {
                Some(FieldValue::Int(color)) => Some(*color),
                _ => None,
            },
        ),
        _ => return None,
    };
    let mut out = Vec::new();
    let index = potion.and_then(|name| {
        crate::item::potions::POTIONS
            .iter()
            .position(|potion| potion.name == name)
    });
    out.push(u8::from(index.is_some()));
    if let Some(index) = index {
        write_varint(index as i32, &mut out);
    }
    out.push(u8::from(color.is_some()));
    if let Some(color) = color {
        out.extend(color.to_be_bytes());
    }
    // No custom effects and no custom name.
    write_varint(0, &mut out);
    out.push(0);
    Some(out)
}

/// `ItemEnchantments.CODEC`: the level of each enchantment by name. A name the registry does not
/// know cannot come from a server, so it is left out.
fn item_enchantments(levels: &[Field], registry: &[String]) -> Vec<u8> {
    let known: Vec<(i32, i32)> = levels
        .iter()
        .filter_map(|entry| {
            let id = registry.iter().position(|name| *name == entry.key)?;
            let level = photon_core::nbt::numeric_nbt_value(&entry.value)?;
            ((1..=255).contains(&level)).then_some((id as i32, level))
        })
        .collect();
    let mut out = Vec::new();
    write_varint(known.len() as i32, &mut out);
    for (id, level) in known {
        write_varint(id, &mut out);
        write_varint(level, &mut out);
    }
    out
}
