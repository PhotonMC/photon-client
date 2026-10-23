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

    /// The files below `directory` (no trailing slash) in every namespace whose name ends with
    /// `extension`, once each, in id order (`listResources`).
    fn list(&self, directory: &str, extension: &str) -> Vec<Identifier>;

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

    fn list(&self, directory: &str, extension: &str) -> Vec<Identifier> {
        let mut found = Vec::new();
        for namespace in self.namespaces() {
            let base = self.root.join(&namespace);
            let mut pending = vec![(base.join(directory), directory.to_owned())];
            while let Some((folder, path)) = pending.pop() {
                let Ok(entries) = std::fs::read_dir(&folder) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let path = format!("{path}/{name}");
                    match entry.file_type() {
                        Ok(kind) if kind.is_dir() => pending.push((entry.path(), path)),
                        Ok(kind) if kind.is_file() && path.ends_with(extension) => {
                            found.extend(Identifier::parse(&format!("{namespace}:{path}")).ok());
                        }
                        _ => {}
                    }
                }
            }
        }
        found.sort();
        found
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
