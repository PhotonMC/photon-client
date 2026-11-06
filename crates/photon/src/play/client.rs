//! The window-and-atlas holder between the menu and the games: opening it and choosing the
//! game version the next game is made from.

use super::{Client, Result, bake_blocks, open_window};

impl Client {
    /// Make the game of `version` what the next game is made from: the readers' assets root
    /// becomes that version's and the block atlas is baked again when it came from another
    /// root. Nothing changes for a version whose assets are not on disk.
    pub(crate) fn use_version(&mut self, version: photon_core::version::Version) -> Result<()> {
        if !photon_core::content_root::select_version(version) {
            log::warn!(
                "no assets for {}: playing with the current ones",
                version.name()
            );
            return Ok(());
        }
        let root = photon_core::content_root::assets()
            .map_err(|_| "PHOTON_ASSETS must name the asset folder")?;
        if root != self.baked_root {
            let (stack, renderer) = match super::prefetch::take(&root) {
                Some(baked) => baked,
                None => super::prefetch::bake(&root),
            }?;
            // Every reader sees this version's assets from here on, with the atlas made from them.
            crate::resources::replace(stack);
            (self.block_renderer, self.animated_atlas) =
                super::upload_blocks(renderer, &mut self.backend)?;
            super::prefetch::note_in_use(&root);
            log::info!("game assets are now {} ({root})", version.name());
            self.baked_root = root;
        }
        Ok(())
    }

    /// Open the window and bake the blocks from `PHOTON_ASSETS`.
    pub(crate) fn open((width, height): (i32, i32)) -> Result<Self> {
        let root = photon_core::content_root::assets()
            .map_err(|_| "PHOTON_ASSETS must name the asset folder")?;
        let (surface, mut backend, size) = open_window(width, height)?;
        let (block_renderer, animated_atlas) = bake_blocks(&root, &mut backend)?;
        super::prefetch::note_in_use(&root);
        Ok(Self {
            surface,
            backend,
            size,
            block_renderer,
            animated_atlas,
            baked_root: root,
            tutorial: photon_gui::tutorial::Tutorial::new(
                crate::controls::load_options().tutorial_step,
            ),
            audio: None,
            gui: None,
        })
    }
}
