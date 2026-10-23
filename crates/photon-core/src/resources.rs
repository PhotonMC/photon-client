//! Where the game's assets are read from: the namespaces of a stack of resource packs, whose
//! files are named by the identifier of their path below the namespace's folder
//! (`minecraft:lang/en_us.json`).

use std::path::PathBuf;

use crate::identifier::Identifier;

/// The files of the resource packs in use (`ResourceManager`).
pub trait Resources: Send + Sync {
    /// The topmost version of a file (`getResource`).
    fn read(&self, id: &Identifier) -> Option<Vec<u8>>;

    /// Every version of a file, the lowest pack first (`getResourceStack`).
    fn read_stack(&self, id: &Identifier) -> Vec<Vec<u8>>;

    /// The namespaces some pack has files for (`getNamespaces`), in name order.
    fn namespaces(&self) -> Vec<String>;
}

/// A folder that holds the namespaces, as the only pack.
pub struct FolderResources {
    root: PathBuf,
}

impl FolderResources {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl Resources for FolderResources {
    fn read(&self, id: &Identifier) -> Option<Vec<u8>> {
        let relative = id.path().replace('/', std::path::MAIN_SEPARATOR_STR);
        std::fs::read(self.root.join(id.namespace()).join(relative)).ok()
    }

    fn read_stack(&self, id: &Identifier) -> Vec<Vec<u8>> {
        self.read(id).into_iter().collect()
    }

    fn namespaces(&self) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return Vec::new();
        };
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}
