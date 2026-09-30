//! Block tags as the reference binds them: the registry's holders carry the
//! tags the server last sent, and `BlockState.is(tag)` reads them from
//! anywhere. Until a server sends any, every tag is empty.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, LazyLock, RwLock};

/// Each tag, by id without the leading `#`, as block registry indices.
pub type BlockTags = BTreeMap<String, BTreeSet<usize>>;

fn bound() -> &'static RwLock<Arc<BlockTags>> {
    static BOUND: LazyLock<RwLock<Arc<BlockTags>>> = LazyLock::new(RwLock::default);
    &BOUND
}

/// `TagLoader`'s result bound to the registry, replacing what was there.
pub fn bind(tags: BlockTags) {
    *bound().write().expect("the tag binding is never poisoned") = Arc::new(tags);
}

/// The tags bound now.
pub fn current() -> Arc<BlockTags> {
    Arc::clone(&bound().read().expect("the tag binding is never poisoned"))
}

/// `BlockState.is(tag)` for a block, by registry index.
pub fn is(block: usize, tag: &str) -> bool {
    current()
        .get(tag)
        .is_some_and(|blocks| blocks.contains(&block))
}
