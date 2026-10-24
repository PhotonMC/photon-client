//! The fonts as the Java oracle loads them.

use photon_core::identifier::Identifier;
use photon_core::resources::{FolderResources, Resources};
use photon_render::gui::font::Font;

/// The packs the fonts are read from: the asset index's own files (the unifont definition and its
/// hex archives) above the jar's, as the game's stack has them.
pub(crate) struct FontResources(Vec<FolderResources>);

impl FontResources {
    /// The jar's assets under `root`, with the index's files beside them in `named`.
    pub(crate) fn new(root: &std::path::Path) -> Self {
        let named = root.join("../../named");
        Self(vec![
            FolderResources::new(named),
            FolderResources::new(root),
        ])
    }
}

impl Resources for FontResources {
    fn read(&self, id: &Identifier) -> Option<Vec<u8>> {
        self.0.iter().find_map(|pack| pack.read(id))
    }

    fn read_stack(&self, id: &Identifier) -> Vec<Vec<u8>> {
        self.0
            .iter()
            .rev()
            .flat_map(|pack| pack.read_stack(id))
            .collect()
    }

    fn list(&self, directory: &str, extension: &str) -> Vec<Identifier> {
        let mut found: Vec<Identifier> = self
            .0
            .iter()
            .flat_map(|pack| pack.list(directory, extension))
            .collect();
        found.sort();
        found.dedup();
        found
    }

    fn namespaces(&self) -> Vec<String> {
        let mut names: Vec<String> = self.0.iter().flat_map(|pack| pack.namespaces()).collect();
        names.sort();
        names.dedup();
        names
    }
}

/// The default font as the Java oracle loads it, the unifont included.
pub(crate) fn load_font(root: &std::path::Path) -> Option<Font> {
    Font::load_from(&FontResources::new(root))
}

/// The alternate font, the same way.
pub(crate) fn load_alt(root: &std::path::Path) -> Option<Font> {
    Font::load_alt_from(&FontResources::new(root))
}
