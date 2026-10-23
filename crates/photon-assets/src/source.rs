//! Where the files of a pack are kept: a folder or a zip archive. Paths are relative to the
//! pack's root, written with `/` (`assets/minecraft/lang/en_us.json`).

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The files of one pack.
pub trait Source: Send + Sync {
    /// The bytes of the file at `path`, if there is one.
    fn read(&self, path: &str) -> Option<Vec<u8>>;

    /// Whether there is a file at `path`.
    fn exists(&self, path: &str) -> bool {
        self.read(path).is_some()
    }

    /// Every file below `directory` (no trailing slash; empty for the root), as paths from the
    /// root, in no particular order.
    fn files_below(&self, directory: &str) -> Vec<String>;

    /// The names of the folders directly inside `directory`.
    fn folders_in(&self, directory: &str) -> Vec<String>;
}

/// A pack that is a folder on disk.
pub struct DirectorySource {
    root: PathBuf,
}

impl DirectorySource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The file `path` names, unless it climbs out of the root.
    fn resolve(&self, path: &str) -> Option<PathBuf> {
        let clean =
            !path.starts_with('/') && path.split('/').all(|part| part != ".." && part != ".");
        clean.then(|| self.root.join(path))
    }
}

fn collect(folder: &Path, prefix: &str, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => collect(&entry.path(), &path, out),
            Ok(kind) if kind.is_file() => out.push(path),
            _ => {}
        }
    }
}

impl Source for DirectorySource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let path = self.resolve(path)?;
        path.is_file().then(|| std::fs::read(path).ok()).flatten()
    }

    fn exists(&self, path: &str) -> bool {
        self.resolve(path).is_some_and(|path| path.is_file())
    }

    fn files_below(&self, directory: &str) -> Vec<String> {
        let mut files = Vec::new();
        if let Some(folder) = self.resolve(directory) {
            collect(&folder, directory, &mut files);
        }
        files
    }

    fn folders_in(&self, directory: &str) -> Vec<String> {
        let Some(folder) = self.resolve(directory) else {
            return Vec::new();
        };
        let Ok(entries) = std::fs::read_dir(folder) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect()
    }
}

/// A pack that is a zip archive (`FilePackResources`).
pub struct ZipSource {
    archive: Mutex<zip::ZipArchive<File>>,
    names: Vec<String>,
    present: HashSet<String>,
}

impl ZipSource {
    pub fn open(path: &Path) -> Result<Self, zip::result::ZipError> {
        let archive = zip::ZipArchive::new(File::open(path)?)?;
        let names: Vec<String> = archive
            .file_names()
            .flatten()
            .filter(|name| !name.ends_with('/'))
            .map(|name| name.into_owned())
            .collect();
        Ok(Self {
            archive: Mutex::new(archive),
            present: names.iter().cloned().collect(),
            names,
        })
    }
}

impl Source for ZipSource {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let mut archive = self.archive.lock().ok()?;
        let mut file = archive.by_name(path).ok()?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).ok()?;
        Some(bytes)
    }

    fn exists(&self, path: &str) -> bool {
        self.present.contains(path)
    }

    fn files_below(&self, directory: &str) -> Vec<String> {
        let prefix = if directory.is_empty() {
            String::new()
        } else {
            format!("{directory}/")
        };
        let below = |name: &&String| name.starts_with(&prefix);
        self.names.iter().filter(below).cloned().collect()
    }

    fn folders_in(&self, directory: &str) -> Vec<String> {
        let mut folders: Vec<String> = self
            .files_below(directory)
            .iter()
            .filter_map(|name| {
                let inside = name.get(directory.len()..)?.trim_start_matches('/');
                inside.split_once('/').map(|(folder, _)| folder.to_owned())
            })
            .collect();
        folders.sort();
        folders.dedup();
        folders
    }
}
