//! Keeping the download cache small (`DownloadCacheCleaner`): past a number of files the oldest
//! go, taking from every folder (one per pack id) in turn so that none is emptied while another
//! keeps many.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// How many cached files `DownloadQueue` keeps.
pub const MAX_KEPT_PACKS: usize = 20;

/// The cached files in the folders of `cache_dir`, with when each was last used.
fn cached_files(cache_dir: &Path) -> std::io::Result<Vec<(PathBuf, SystemTime)>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(cache_dir)? {
        let folder = entry?.path();
        if !folder.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&folder)? {
            let path = entry?.path();
            let metadata = path.metadata()?;
            if metadata.is_file() {
                files.push((path, metadata.modified()?));
            }
        }
    }
    Ok(files)
}

/// Which of `files` to delete to leave `keep`: the files are ranked within their folder from the
/// newest, and the highest ranks (the oldest of the fullest folders) go first.
fn files_to_remove(mut files: Vec<(PathBuf, SystemTime)>, keep: usize) -> Vec<PathBuf> {
    let Some(count) = files.len().checked_sub(keep).filter(|count| *count > 0) else {
        return Vec::new();
    };
    files.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));
    let mut seen: Vec<(PathBuf, usize)> = Vec::new();
    let mut ranked = Vec::with_capacity(files.len());
    for (path, _) in files {
        let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let rank = if let Some((_, taken)) = seen.iter_mut().find(|(folder, _)| *folder == parent) {
            std::mem::replace(taken, *taken + 1)
        } else {
            seen.push((parent, 1));
            0
        };
        ranked.push((path, rank));
    }
    ranked.reverse();
    ranked.sort_by_key(|(_, rank)| std::cmp::Reverse(*rank));
    ranked
        .into_iter()
        .take(count)
        .map(|(path, _)| path)
        .collect()
}

/// `DownloadCacheCleaner.vacuumCacheDir`: leave at most `keep` files; a folder left empty goes.
pub fn vacuum(cache_dir: &Path, keep: usize) {
    let files = match cached_files(cache_dir) {
        Ok(files) => files,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            log::error!(
                "failed to vacuum cache dir {}: {error}",
                cache_dir.display()
            );
            return;
        }
    };
    let removed = files_to_remove(files, keep);
    for path in &removed {
        if let Err(error) = std::fs::remove_file(path) {
            log::warn!("failed to delete cache file {}: {error}", path.display());
        }
    }
    for folder in removed.iter().filter_map(|path| path.parent()) {
        // A folder that still holds files is kept (the removal fails).
        let _ = std::fs::remove_dir(folder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn at(path: &str, age: u64) -> (PathBuf, SystemTime) {
        (
            PathBuf::from(path),
            SystemTime::UNIX_EPOCH + Duration::from_secs(1_000 - age),
        )
    }

    #[test]
    fn nothing_goes_while_the_files_fit() {
        let files = vec![at("a/1", 1), at("a/2", 2)];
        assert_eq!(files_to_remove(files, 2), Vec::<PathBuf>::new());
    }

    #[test]
    fn the_oldest_of_the_fullest_folder_goes_first() {
        // Folder a holds three files, folder b one older than all of a's: b's file is the
        // newest of its folder, so it outlasts a's older ones.
        let files = vec![
            at("a/new", 1),
            at("a/mid", 2),
            at("a/old", 3),
            at("b/only", 9),
        ];
        assert_eq!(
            files_to_remove(files, 2),
            vec![PathBuf::from("a/old"), PathBuf::from("a/mid")]
        );
    }
}
