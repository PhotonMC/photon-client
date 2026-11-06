//! The blocks of a game version baked ahead on a worker thread, so the join that switches to it
//! does not wait for them: started when the player picks the version or begins a join.

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use photon_assets::ResourceStack;
use photon_core::version::Version;
use photon_render::block::world_mesh::BlockRenderer;

/// A stack of the version's assets and the blocks baked from it.
pub(crate) type Baked = (Arc<ResourceStack>, BlockRenderer);

/// A bake under way: the assets folder it reads, and its thread.
type Running = (String, JoinHandle<Result<Baked, String>>);

static SLOT: Mutex<Option<Running>> = Mutex::new(None);

/// The assets folder the atlas in use was baked from, which no bake ahead is worth for.
static IN_USE: Mutex<String> = Mutex::new(String::new());

/// Note the folder the atlas in use came from.
pub(crate) fn note_in_use(root: &str) {
    if let Ok(mut in_use) = IN_USE.lock() {
        root.clone_into(&mut in_use);
    }
}

/// Bake the blocks of the assets folder `root`, as the thread that does it and as a join that
/// found nothing ready does.
pub(crate) fn bake(root: &str) -> Result<Baked, String> {
    let stack = crate::resources::stack_for(std::path::Path::new(root));
    let renderer = crate::resources::preparing(Arc::clone(&stack), || {
        super::bake_renderer(root).map_err(|error| error.to_string())
    })?;
    Ok((stack, renderer))
}

/// Start baking the blocks of `version`, unless that is already going on or its assets are not
/// on disk.
pub(crate) fn start(version: Version) {
    let Some(root) = photon_core::content_root::assets_for(version) else {
        return;
    };
    let root = root.to_string_lossy().into_owned();
    if IN_USE.lock().is_ok_and(|in_use| *in_use == root) {
        return;
    }
    let Ok(mut slot) = SLOT.lock() else {
        return;
    };
    if slot.as_ref().is_some_and(|(baking, _)| *baking == root) {
        return;
    }
    let thread_root = root.clone();
    *slot = Some((root, std::thread::spawn(move || bake(&thread_root))));
}

/// What was baked ahead for `root`, waiting for the thread when it is still running; none when
/// nothing was started for that folder.
pub(crate) fn take(root: &str) -> Option<Result<Baked, String>> {
    let (baking, handle) = SLOT.lock().ok()?.take_if(|(baking, _)| baking == root)?;
    debug_assert_eq!(baking, root);
    Some(
        handle
            .join()
            .unwrap_or_else(|_| Err("the bake thread panicked".to_owned())),
    )
}
