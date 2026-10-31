//! `HotbarManager` and `Hotbar`: the nine toolbars a creative player saves and loads, kept in
//! `hotbar.nbt` of the game directory.
//!
//! The file is the reference's: an uncompressed compound with an empty root name, a `DataVersion`
//! and one list of nine stacks under each of the keys `"0"` to `"8"`. A stack is the compound of
//! the optional item codec: empty for no stack, otherwise `id` and `count`, and `components`
//! when the stack carries any.
//!
//! Not ported: stacks with components. They are written as an empty stack (what the reference
//! stores when it cannot encode one) and read as one, and the file has no data fixer, so a file
//! of an older data version is read as it is.

use std::path::{Path, PathBuf};

use photon_core::ids::ItemId;
use photon_core::nbt::{self, Field, FieldValue};
use photon_data::item::registry::AIR_ITEM;
use photon_world::ClientItemStackState as Stack;

use crate::options::DATA_VERSION;

/// `HotbarManager.NUM_HOTBAR_GROUPS`.
pub const HOTBARS: usize = 9;
/// `Inventory.getSelectionSize`: the stacks in one saved hotbar.
pub const SLOTS: usize = 9;

/// The stack that is no stack: the optional codec's empty compound.
fn empty_entry() -> FieldValue {
    FieldValue::Compound(Vec::new())
}

fn field<'a>(fields: &'a [Field], key: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|field| field.key == key)
        .map(|field| &field.value)
}

/// `ItemStack.OPTIONAL_CODEC` encoding: `None` when the stack is empty or carries components
/// this cannot write yet.
pub fn encode_entry(stack: Option<&Stack>) -> FieldValue {
    let Some(stack) = stack.filter(|stack| stack.item_count() > 0) else {
        return empty_entry();
    };
    let name = usize::try_from(stack.item_registry_id().0)
        .ok()
        .and_then(|id| photon_data::item::registry::ITEMS.get(id));
    match name {
        Some(item) if stack.patch_bytes().is_empty() => FieldValue::Compound(vec![
            Field {
                key: "id".to_owned(),
                value: FieldValue::Str(item.name.to_owned()),
            },
            Field {
                key: "count".to_owned(),
                value: FieldValue::Int(stack.item_count()),
            },
        ]),
        _ => empty_entry(),
    }
}

/// `ItemStack.OPTIONAL_CODEC` parsing: the stack, or `None` for the empty compound and for an
/// entry that does not parse (the reference logs it and takes the empty stack).
pub fn decode_entry(entry: &FieldValue) -> Option<Stack> {
    let FieldValue::Compound(fields) = entry else {
        return None;
    };
    let FieldValue::Str(name) = field(fields, "id")? else {
        return None;
    };
    let count = match field(fields, "count") {
        None => 1,
        Some(value) => nbt::numeric_nbt_value(value)?,
    };
    let has_components = match field(fields, "components") {
        None => false,
        Some(FieldValue::Compound(components)) => !components.is_empty(),
        Some(_) => return None,
    };
    if !(1..=99).contains(&count) || has_components {
        return None;
    }
    let index = photon_data::item::registry::index(name)?;
    let id = ItemId(i32::try_from(index).ok()?);
    // The item codec leaves out air, which is no stack.
    (id != AIR_ITEM).then(|| Stack::new(count, id))
}

/// `Hotbar`: nine stacks, kept as the file's own entries so what is not understood stays as read.
#[derive(Debug, Clone, PartialEq)]
pub struct Hotbar {
    items: Vec<FieldValue>,
}

impl Default for Hotbar {
    fn default() -> Self {
        Self {
            items: vec![empty_entry(); SLOTS],
        }
    }
}

impl Hotbar {
    /// `Hotbar.CODEC`: a list of exactly nine entries.
    fn from_entry(entry: &FieldValue) -> Option<Self> {
        match entry {
            FieldValue::List(items) if items.len() == SLOTS => Some(Self {
                items: items.clone(),
            }),
            _ => None,
        }
    }

    /// `Hotbar.load`: the stacks, an empty stack where an entry does not parse.
    pub fn load(&self) -> Vec<Option<Stack>> {
        self.items.iter().map(decode_entry).collect()
    }

    /// `Hotbar.storeFrom`: the first nine slots of the inventory.
    pub fn store_from(&mut self, inventory: &[Option<Stack>]) {
        self.items = (0..SLOTS)
            .map(|slot| encode_entry(inventory.get(slot).and_then(Option::as_ref)))
            .collect();
    }

    /// `Hotbar.isEmpty`: every entry is the empty stack.
    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|item| *item == empty_entry())
    }
}

/// `HotbarManager`: the nine hotbars, read from the file on first use.
#[derive(Debug)]
pub struct HotbarManager {
    file: PathBuf,
    hotbars: Vec<Hotbar>,
    loaded: bool,
}

impl HotbarManager {
    pub fn new(game_dir: &Path) -> Self {
        Self {
            file: game_dir.join("hotbar.nbt"),
            hotbars: vec![Hotbar::default(); HOTBARS],
            loaded: false,
        }
    }

    /// `HotbarManager.get`.
    pub fn get(&mut self, id: usize) -> &Hotbar {
        self.ensure_loaded();
        &self.hotbars[id]
    }

    /// `Hotbar.storeFrom` on one hotbar, then `HotbarManager.save`.
    pub fn store(&mut self, id: usize, inventory: &[Option<Stack>]) {
        self.ensure_loaded();
        self.hotbars[id].store_from(inventory);
        self.save();
    }

    fn ensure_loaded(&mut self) {
        if !self.loaded {
            self.load();
            self.loaded = true;
        }
    }

    /// `HotbarManager.load`: a missing or unreadable file leaves the hotbars empty, and a hotbar
    /// that is not nine entries stays empty.
    fn load(&mut self) {
        let Ok(bytes) = std::fs::read(&self.file) else {
            return;
        };
        let Some(fields) = parse_file(&bytes) else {
            return;
        };
        for (id, hotbar) in self.hotbars.iter_mut().enumerate() {
            *hotbar = field(&fields, &id.to_string())
                .and_then(Hotbar::from_entry)
                .unwrap_or_default();
        }
    }

    /// `HotbarManager.save`: a failure is not reported to the player.
    fn save(&self) {
        let _ = std::fs::write(&self.file, encode_file(&self.hotbars));
    }
}

/// `NbtIo.write(tag, path)`: the root tag type, an empty name, then the compound.
fn encode_file(hotbars: &[Hotbar]) -> Vec<u8> {
    let mut fields = vec![Field {
        key: "DataVersion".to_owned(),
        value: FieldValue::Int(DATA_VERSION),
    }];
    for (id, hotbar) in hotbars.iter().enumerate() {
        fields.push(Field {
            key: id.to_string(),
            value: FieldValue::List(hotbar.items.clone()),
        });
    }
    let mut network = Vec::new();
    nbt::write_nbt(Some(&fields), &mut network);
    let mut file = vec![network[0], 0, 0];
    file.extend_from_slice(&network[1..]);
    file
}

/// `NbtIo.read(path)`: the compound behind the root tag's name.
fn parse_file(bytes: &[u8]) -> Option<Vec<Field>> {
    let (&root, rest) = bytes.split_first()?;
    let name_len = usize::from(u16::from_be_bytes([*rest.first()?, *rest.get(1)?]));
    let payload = rest.get(2 + name_len..)?;
    let mut network = vec![root];
    network.extend_from_slice(payload);
    nbt::read_nbt(&mut network.as_slice()).ok().flatten()
}

#[cfg(test)]
mod tests;
