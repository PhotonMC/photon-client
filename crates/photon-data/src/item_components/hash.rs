//! The hashes a click packet carries for a stack's added components
//! (`HashedPatchMap`): each value encoded by its persistent codec into
//! `HashOps` over CRC32C, as `ClientPacketListener.decoratedHashOpsGenerator`
//! does.
//!
//! `HashOps` hashes a value from its parts: a tag byte, then the value's
//! bytes, or the four bytes of each part's own hash for a list or a map.
//! A map's entries are sorted by the key's hash, then the value's, as
//! unsigned numbers; Guava's hasher writes numbers little-endian.
//!
//! Only the components ported here are hashed; the rest give `None`.

use photon_core::nbt::{FieldValue, read_any_nbt};

use super::{Cursor, ids};

const fn crc32c_table() -> [u32; 256] {
    let mut table = [0; 256];
    let mut index = 0;
    while index < 256 {
        let mut crc = index as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 == 0 {
                crc >> 1
            } else {
                (crc >> 1) ^ 0x82F6_3B78
            };
            bit += 1;
        }
        table[index] = crc;
        index += 1;
    }
    table
}

static CRC32C: [u32; 256] = crc32c_table();

/// `Hashing.crc32c()` over the bytes.
pub fn crc32c(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(!0_u32, |crc, &byte| {
        CRC32C[((crc ^ u32::from(byte)) & 0xFF) as usize] ^ (crc >> 8)
    })
}

fn tagged(tag: u8, payload: &[u8]) -> u32 {
    let mut bytes = Vec::with_capacity(payload.len() + 1);
    bytes.push(tag);
    bytes.extend_from_slice(payload);
    crc32c(&bytes)
}

/// `HashOps.createByte`.
pub fn byte(value: i8) -> u32 {
    tagged(6, &value.to_le_bytes())
}

/// `HashOps.createInt`.
pub fn int(value: i32) -> u32 {
    tagged(8, &value.to_le_bytes())
}

/// `HashOps.createBoolean`.
pub fn boolean(value: bool) -> u32 {
    tagged(13, &[u8::from(value)])
}

/// `HashOps.createString`: the length in UTF-16 units, then the units.
pub fn string(value: &str) -> u32 {
    let units: Vec<u16> = value.encode_utf16().collect();
    let mut bytes = vec![12];
    bytes.extend_from_slice(&(units.len() as i32).to_le_bytes());
    for unit in units {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    crc32c(&bytes)
}

/// `HashOps.createList`: the parts' hashes in order.
pub fn list(parts: impl IntoIterator<Item = u32>) -> u32 {
    let mut bytes = vec![4];
    for part in parts {
        bytes.extend_from_slice(&part.to_le_bytes());
    }
    bytes.push(5);
    crc32c(&bytes)
}

/// `HashOps.createMap`: the entries sorted by key hash, then value hash.
pub fn map(mut entries: Vec<(u32, u32)>) -> u32 {
    entries.sort_unstable();
    let mut bytes = vec![2];
    for (key, value) in entries {
        bytes.extend_from_slice(&key.to_le_bytes());
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.push(3);
    crc32c(&bytes)
}

/// A tag as `NbtOps.convertTo` hands it to `HashOps`. With `text`, the
/// keys a text component's codec reads as booleans are hashed as booleans
/// (the network tag carries them as bytes).
pub fn tag(value: &FieldValue, text: bool) -> u32 {
    fn array<T: Copy>(start: u8, values: &[T], bytes: impl Fn(T) -> Vec<u8>) -> u32 {
        let mut out = vec![start];
        for &value in values {
            out.extend(bytes(value));
        }
        out.push(start + 1);
        crc32c(&out)
    }
    match value {
        FieldValue::Byte(value) => byte(*value),
        FieldValue::Short(value) => tagged(7, &value.to_le_bytes()),
        FieldValue::Int(value) => int(*value),
        FieldValue::Long(value) => tagged(9, &value.to_le_bytes()),
        FieldValue::Float(value) => tagged(10, &value.to_bits().to_le_bytes()),
        FieldValue::Double(value) => tagged(11, &value.to_bits().to_le_bytes()),
        FieldValue::Str(value) => string(value),
        FieldValue::ByteArray(values) => array(14, values, |value| vec![value as u8]),
        FieldValue::IntArray(values) => array(16, values, |value| value.to_le_bytes().to_vec()),
        FieldValue::LongArray(values) => array(18, values, |value| value.to_le_bytes().to_vec()),
        FieldValue::List(values) => list(values.iter().map(|value| tag(value, text))),
        FieldValue::Compound(fields) => map(fields
            .iter()
            .map(|field| {
                let value = match (
                    &field.value,
                    text && TEXT_BOOLEANS.contains(&field.key.as_str()),
                ) {
                    (FieldValue::Byte(value), true) => boolean(*value != 0),
                    (value, _) => tag(value, text),
                };
                (string(&field.key), value)
            })
            .collect()),
    }
}

/// The fields `Style` and `NbtContents` read with `Codec.BOOL`.
const TEXT_BOOLEANS: [&str; 7] = [
    "bold",
    "italic",
    "underlined",
    "strikethrough",
    "obfuscated",
    "interpret",
    "plain",
];

/// `Rarity`, by its network id.
const RARITIES: [&str; 4] = ["common", "uncommon", "rare", "epic"];

/// The hash `ClientPacketListener.decoratedHashOpsGenerator` gives an added
/// component, from its value as the wire carried it; `enchantments` names
/// the enchantment registry's entries by network id. `None` for a component
/// not ported here, or a value that does not read.
pub fn component_hash(type_id: i32, value: &[u8], enchantments: &[String]) -> Option<i32> {
    let mut cursor = Cursor { data: value };
    let text = |cursor: &mut Cursor| -> Option<u32> {
        let tag_value = read_any_nbt(&mut cursor.data).ok()??;
        Some(tag(&tag_value, true))
    };
    let hash = match type_id {
        ids::CUSTOM_DATA => tag(&read_any_nbt(&mut cursor.data).ok()??, false),
        ids::MAX_STACK_SIZE | ids::MAX_DAMAGE | ids::DAMAGE | ids::REPAIR_COST => {
            int(cursor.varint().ok()?)
        }
        // `Unit.CODEC` writes an empty map.
        ids::UNBREAKABLE => map(Vec::new()),
        ids::CUSTOM_NAME | ids::ITEM_NAME => text(&mut cursor)?,
        ids::LORE => {
            let lines = cursor.count().ok()?;
            let mut hashes = Vec::with_capacity(lines);
            for _ in 0..lines {
                hashes.push(text(&mut cursor)?);
            }
            list(hashes)
        }
        ids::RARITY => string(RARITIES.get(cursor.count().ok()?)?),
        ids::ENCHANTMENTS | ids::STORED_ENCHANTMENTS => {
            let count = cursor.count().ok()?;
            let mut entries = Vec::with_capacity(count);
            for _ in 0..count {
                let name = enchantments.get(cursor.count().ok()?)?;
                entries.push((string(name), int(cursor.varint().ok()?)));
            }
            map(entries)
        }
        ids::ENCHANTMENT_GLINT_OVERRIDE => boolean(cursor.boolean().ok()?),
        ids::BUNDLE_CONTENTS => {
            let count = cursor.count().ok()?;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(item_template(&mut cursor, enchantments)?);
            }
            list(items)
        }
        ids::DYED_COLOR => {
            let bytes: [u8; 4] = cursor.data.get(..4)?.try_into().ok()?;
            int(i32::from_be_bytes(bytes))
        }
        _ => return None,
    };
    Some(hash as i32)
}

/// `ItemStackTemplate.CODEC` of a template as its stream codec wrote it:
/// the item's id, the count unless it is one, the patch unless it is empty.
fn item_template(cursor: &mut Cursor, enchantments: &[String]) -> Option<u32> {
    let item = usize::try_from(cursor.varint().ok()?).ok()?;
    let count = cursor.varint().ok()?;
    let patch = cursor.patch().ok()?;
    let mut entries = vec![(
        string("id"),
        string(crate::item::registry::ITEMS.get(item)?.name),
    )];
    if count != 1 {
        entries.push((string("count"), int(count)));
    }
    if !patch.added.is_empty() || !patch.removed.is_empty() {
        entries.push((string("components"), patch_map(&patch, enchantments)?));
    }
    Some(map(entries))
}

/// `DataComponentPatch.CODEC`: each added component under its id, each
/// removed one under `!` and its id with the unit (an empty map).
fn patch_map(patch: &super::Patch, enchantments: &[String]) -> Option<u32> {
    let name = |id: i32| {
        crate::item::component_types::NAMES
            .get(usize::try_from(id).ok()?)
            .copied()
    };
    let mut entries = Vec::new();
    for (id, value) in &patch.added {
        entries.push((
            string(name(*id)?),
            component_hash(*id, value, enchantments)? as u32,
        ));
    }
    for id in &patch.removed {
        entries.push((string(&format!("!{}", name(*id)?)), map(Vec::new())));
    }
    Some(map(entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32c_matches_the_standard_check_value() {
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);
    }
}
