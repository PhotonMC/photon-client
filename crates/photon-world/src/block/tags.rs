//! Block tags as the server sends them (`ClientboundUpdateTagsPacket`):
//! what `BlockState.is(tag)` reads. Until a server sends any, every tag is
//! empty, as the reference's are.

use std::collections::{BTreeMap, BTreeSet};

use photon_core::tags::RegistryTags;

/// Each block tag, by id without the leading `#`, as block registry indices.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockTags(BTreeMap<String, BTreeSet<usize>>);

impl BlockTags {
    /// The block registry's tags from a tag packet; `None` when the packet
    /// does not update the block registry.
    pub fn from_registries(registries: &[RegistryTags]) -> Option<Self> {
        let blocks = registries
            .iter()
            .find(|registry| registry.registry.as_string() == "minecraft:block")?;
        Some(Self(
            blocks
                .tags
                .iter()
                .map(|tag| {
                    let members = tag
                        .values
                        .iter()
                        .filter_map(|value| usize::try_from(*value).ok())
                        .collect();
                    (tag.id.as_string(), members)
                })
                .collect(),
        ))
    }

    /// `BlockState.is(tag)` for a block, by registry index.
    pub fn contains(&self, block: usize, tag: &str) -> bool {
        self.0
            .get(tag)
            .is_some_and(|blocks| blocks.contains(&block))
    }
}

/// Each item tag, by id without the leading `#`, as item registry ids:
/// what `ItemStack.is(tag)` reads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemTags(BTreeMap<String, BTreeSet<i32>>);

impl ItemTags {
    /// The item registry's tags from a tag packet; `None` when the packet
    /// does not update the item registry.
    pub fn from_registries(registries: &[RegistryTags]) -> Option<Self> {
        Self::of_registry(registries, "minecraft:item")
    }

    /// The tags of another registry (`minecraft:banner_pattern`), as registry
    /// ids, read the same way.
    pub fn of_registry(registries: &[RegistryTags], name: &str) -> Option<Self> {
        let items = registries
            .iter()
            .find(|registry| registry.registry.as_string() == name)?;
        Some(Self(
            items
                .tags
                .iter()
                .map(|tag| (tag.id.as_string(), tag.values.iter().copied().collect()))
                .collect(),
        ))
    }

    /// The items of a tag, by registry id.
    pub fn members(&self, tag: &str) -> impl Iterator<Item = i32> + '_ {
        self.0.get(tag).into_iter().flatten().copied()
    }

    /// `ItemStack.is(tag)` for an item, by registry id.
    pub fn contains(&self, item: i32, tag: &str) -> bool {
        self.0.get(tag).is_some_and(|items| items.contains(&item))
    }

    /// Each tag and its items, by tag id.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &BTreeSet<i32>)> {
        self.0.iter().map(|(tag, items)| (tag.as_str(), items))
    }
}
