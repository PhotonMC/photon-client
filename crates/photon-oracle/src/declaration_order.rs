//! The orders in which the reference lists what it meets while it walks its block registry:
//! classes, interfaces, state properties and face shapes. The client's tables number them by the
//! newest version's registry; a version that orders its blocks differently meets them in
//! another order.

use photon_data::block::classes::{BlockClass, BlockInterface};
use photon_data::block::properties::FIELDS;
use photon_data::block::registry::{Block, Property};
use photon_data::item::classes::ItemClass;

/// The classes of `blocks` in the order the walk first meets them: a block's own class, then the
/// classes above it.
pub(crate) fn classes<'a>(blocks: impl Iterator<Item = &'a Block>) -> Vec<BlockClass> {
    let mut seen: Vec<BlockClass> = Vec::new();
    for block in blocks {
        let mut above = Some(crate::case_ids::reference_block_class(block));
        while let Some(class) = above {
            if crate::case_ids::reference_has_block_class(class) && !seen.contains(&class) {
                seen.push(class);
            }
            above = class.parent();
        }
    }
    seen
}

/// The item classes of `items` in the order the walk first meets them: an item's own class, then
/// the classes above it up to `Item`.
pub(crate) fn item_classes<'a>(
    items: impl Iterator<Item = &'a photon_data::item::definition::Item>,
) -> Vec<ItemClass> {
    let mut seen: Vec<ItemClass> = Vec::new();
    for item in items {
        let mut at = Some(crate::case_ids::reference_class(item.class));
        while let Some(class) = at {
            if crate::case_ids::reference_has_class(class) && !seen.contains(&class) {
                seen.push(class);
            }
            at = class.parent().map(crate::case_ids::reference_class);
        }
    }
    seen
}

/// The state property fields of `blocks`, each at the walk's first meeting with it.
pub(crate) fn properties<'a>(
    blocks: impl Iterator<Item = &'a Block>,
) -> Vec<&'static (&'static str, &'static Property)> {
    let mut seen: Vec<&'static (&'static str, &'static Property)> = Vec::new();
    for block in blocks {
        for property in block.properties {
            let listed = FIELDS
                .iter()
                .find(|(_, own)| std::ptr::eq(*own, *property))
                .expect("every property is listed");
            if !seen.iter().any(|have| std::ptr::eq(*have, listed)) {
                seen.push(listed);
            }
        }
    }
    seen
}

/// The interfaces of `interfaces` (one list per class, in class order) and of the interfaces
/// they extend: each is listed where it is first met, followed by what it extends.
pub(crate) fn interfaces(per_class: &[Vec<BlockInterface>]) -> Vec<BlockInterface> {
    fn collect(interface: BlockInterface, seen: &mut Vec<BlockInterface>) {
        if seen.contains(&interface) {
            return;
        }
        seen.push(interface);
        for parent in interface.parents() {
            collect(*parent, seen);
        }
    }
    let mut seen = Vec::new();
    for interface in per_class.iter().flatten() {
        collect(*interface, &mut seen);
    }
    seen
}

/// Renumbers face shapes by first meeting: nothing and the whole face keep their numbers, a
/// partial face is numbered from 2 in the order the walk reaches it. `old[new]` is the client's
/// number of the face the walk numbers `new`.
#[derive(Default)]
pub(crate) struct FaceNumbers {
    old: Vec<u16>,
}

impl FaceNumbers {
    pub(crate) fn new() -> Self {
        Self { old: vec![0, 1] }
    }

    /// The walk's number for the client's face number `face`.
    pub(crate) fn number(&mut self, face: u16) -> u16 {
        let place = self.old.iter().position(|have| *have == face);
        u16::try_from(place.unwrap_or_else(|| {
            self.old.push(face);
            self.old.len() - 1
        }))
        .expect("fewer than 65536 faces")
    }

    /// How many faces the walk met, nothing and the whole face included.
    pub(crate) fn count(&self) -> usize {
        self.old.len()
    }

    /// A table over the faces in the walk's numbering, from one over the client's.
    pub(crate) fn table(&self, entry: &dyn Fn(u16, u16) -> bool) -> Vec<String> {
        self.old
            .iter()
            .map(|row| {
                self.old
                    .iter()
                    .map(|column| if entry(*row, *column) { '1' } else { '0' })
                    .collect()
            })
            .collect()
    }
}
