//! Block tags as the server sends them (`ClientboundUpdateTagsPacket`):
//! what `BlockState.is(tag)` reads. Until a server sends any, every tag is
//! empty, as the reference's are.

use std::collections::{BTreeMap, BTreeSet};

use photon_core::tags::RegistryTags;
use photon_data::version_registries::IdMap;

/// Each block tag, by id without the leading `#`, as block registry indices.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockTags(BTreeMap<String, BTreeSet<usize>>);

impl BlockTags {
    /// The block registry's tags from a tag packet; `None` when the packet
    /// does not update the block registry.
    ///
    /// The members arrive as the connected version's block ids; `numbering` turns them into the
    /// canonical indices the client uses.
    pub fn from_registries(registries: &[RegistryTags], numbering: &IdMap) -> Option<Self> {
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
                        .map(|wire| numbering.canonical_or_unknown(wire as i32) as usize)
                        .collect();
                    (tag.id.as_string(), members)
                })
                .collect(),
        ))
    }

    /// The blocks of a tag, by registry index.
    pub fn members(&self, tag: &str) -> impl Iterator<Item = usize> + '_ {
        self.0.get(tag).into_iter().flatten().copied()
    }

    /// The tags a block is in (`TypedInstance.tags`), by id without the leading `#`.
    pub fn of_block(&self, block: usize) -> impl Iterator<Item = &str> + '_ {
        self.0
            .iter()
            .filter(move |(_, blocks)| blocks.contains(&block))
            .map(|(tag, _)| tag.as_str())
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
    pub fn from_registries(registries: &[RegistryTags], items: &IdMap) -> Option<Self> {
        Self::of_registry(registries, "minecraft:item", Some(items))
    }

    /// The tags of another registry (`minecraft:entity_type`), as registry ids, read the same
    /// way. A registry the client numbers itself gives its map, which turns the connected
    /// version's ids into canonical ones; one it does not (the fluids) keeps the ids as sent.
    pub fn of_registry(
        registries: &[RegistryTags],
        name: &str,
        numbering: Option<&IdMap>,
    ) -> Option<Self> {
        let items = registries
            .iter()
            .find(|registry| registry.registry.as_string() == name)?;
        Some(Self(
            items
                .tags
                .iter()
                .map(|tag| {
                    let members = tag
                        .values
                        .iter()
                        .map(|wire| numbering.map_or(*wire, |map| map.canonical_or_unknown(*wire)))
                        .collect();
                    (tag.id.as_string(), members)
                })
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

/// The tags of a registry whose order matters, as the server listed their members: the loom
/// offers a tag's patterns in that order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrderedTags(BTreeMap<String, Vec<i32>>);

impl OrderedTags {
    /// Tags from their ids and members.
    pub fn new(tags: impl IntoIterator<Item = (String, Vec<i32>)>) -> Self {
        Self(tags.into_iter().collect())
    }

    /// The tags of one registry from a tag packet; `None` when it does not update that registry.
    pub fn of_registry(registries: &[RegistryTags], name: &str) -> Option<Self> {
        let entries = registries
            .iter()
            .find(|registry| registry.registry.as_string() == name)?;
        Some(Self(
            entries
                .tags
                .iter()
                .map(|tag| (tag.id.as_string(), tag.values.clone()))
                .collect(),
        ))
    }

    /// The members of a tag by registry id, in the order the server listed them.
    pub fn members(&self, tag: &str) -> impl Iterator<Item = i32> + '_ {
        self.0.get(tag).into_iter().flatten().copied()
    }
}

#[cfg(test)]
mod ordered_tests {
    use super::*;

    #[test]
    fn a_tag_keeps_the_order_the_server_listed() {
        let tags = OrderedTags::new([("minecraft:t".to_owned(), vec![5, 1, 3])]);
        assert_eq!(tags.members("minecraft:t").collect::<Vec<_>>(), [5, 1, 3]);
    }
}
