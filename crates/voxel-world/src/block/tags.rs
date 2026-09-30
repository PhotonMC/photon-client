//! Block tags as the server sends them (`ClientboundUpdateTagsPacket`):
//! what `BlockState.is(tag)` reads. Until a server sends any, every tag is
//! empty, as the reference's are.

use std::collections::{BTreeMap, BTreeSet};

use voxel_core::tags::RegistryTags;

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
