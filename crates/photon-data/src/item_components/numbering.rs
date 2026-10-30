//! The ids of a version inside a component patch: read into the canonical ids a stack keeps,
//! and written back as the server numbers them.

use super::*;
use crate::version_registries::{IdMap, VersionRegistries};

/// How a version numbers the registries a component value names ids of, and which way a
/// translation goes: into the canonical ids a stack keeps, or out to the ones the server reads.
pub struct Numbering<'a> {
    registries: &'a VersionRegistries,
    outgoing: bool,
}

impl<'a> Numbering<'a> {
    /// The ids a server sent, to canonical ones.
    pub fn incoming(registries: &'a VersionRegistries) -> Self {
        Self {
            registries,
            outgoing: false,
        }
    }

    /// The canonical ids a stack keeps, to the server's.
    pub fn outgoing(registries: &'a VersionRegistries) -> Self {
        Self {
            registries,
            outgoing: true,
        }
    }

    fn map_of(&self, site: Site) -> &IdMap {
        match site {
            Site::ComponentType => &self.registries.component_types,
            Site::Attribute => &self.registries.attributes,
            Site::SoundEvent => &self.registries.sound_events,
            Site::Item => &self.registries.items,
            Site::Block => &self.registries.blocks,
            Site::EntityType => &self.registries.entity_types,
            Site::BlockEntityType => &self.registries.block_entity_types,
        }
    }

    pub fn map(&self, site: Site, id: i32) -> i32 {
        let map = self.map_of(site);
        if self.outgoing {
            map.wire_or_unknown(id)
        } else {
            map.canonical_or_unknown(id)
        }
    }

    /// Whether no id this reads changes, so the bytes can stay as they are.
    pub fn is_identity(&self) -> bool {
        [
            Site::ComponentType,
            Site::Attribute,
            Site::SoundEvent,
            Site::Item,
            Site::Block,
            Site::EntityType,
            Site::BlockEntityType,
        ]
        .iter()
        .all(|site| self.map_of(*site).is_identity())
    }
}

/// A patch as a server numbers its registries, read into canonical ids and values.
pub fn read_wire_patch(input: &mut &[u8], registries: &VersionRegistries) -> R<Patch> {
    let numbering = Numbering::incoming(registries);
    if numbering.is_identity() {
        return read_patch(input);
    }
    let map = |site, id| numbering.map(site, id);
    let data: &[u8] = input;
    let mut cursor = Cursor::rewriting(data, &map);
    let patch = cursor.patch()?;
    let left = cursor.data.len();
    *input = &data[data.len() - left..];
    Ok(patch)
}

impl Patch {
    /// [`Self::delimited`] with every id numbered as `numbering` has it; `None` when a value
    /// is not one this reads.
    pub fn delimited_for(&self, numbering: &Numbering<'_>) -> Option<Vec<u8>> {
        if numbering.is_identity() {
            return Some(self.delimited());
        }
        let map = |site, id| numbering.map(site, id);
        let mut wire = Patch {
            added: Vec::new(),
            removed: Vec::new(),
        };
        for (id, value) in &self.added {
            let mut cursor = Cursor::rewriting(value, &map);
            cursor.component(*id).ok()?;
            if !cursor.data.is_empty() {
                return None;
            }
            wire.added
                .push((numbering.map(Site::ComponentType, *id), cursor.finish()));
        }
        for id in &self.removed {
            wire.removed.push(numbering.map(Site::ComponentType, *id));
        }
        Some(wire.write(true))
    }
}
