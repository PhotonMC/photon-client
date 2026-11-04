//! The mods the client has fetched, kept by their hash. A file that no longer matches its name
//! is treated as missing, so a damaged or edited cache cannot run.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::MAX_MOD_BYTES;

/// Most files the cache keeps; the least recently written go first.
const MAX_FILES: usize = 128;

#[derive(Debug)]
pub enum CacheError {
    /// The bytes are not the file the hash names, or are too many.
    Mismatch,
    Io(std::io::Error),
}

pub struct ModCache {
    dir: PathBuf,
}

/// SHA-256 of `bytes` as 64 lower-case hex digits.
pub fn hash_of(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl ModCache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn path(&self, hash: &str) -> PathBuf {
        self.dir.join(format!("{hash}.wasm"))
    }

    /// The file with this hash, if the cache has it and it is still intact.
    pub fn get(&self, hash: &str) -> Option<Vec<u8>> {
        let path = self.path(hash);
        let bytes = std::fs::read(&path).ok()?;
        if bytes.len() as u64 <= MAX_MOD_BYTES && hash_of(&bytes) == hash {
            return Some(bytes);
        }
        let _ = std::fs::remove_file(&path);
        None
    }

    /// Keeps `bytes` under `hash`, after checking it is what the hash says.
    pub fn put(&self, hash: &str, bytes: &[u8]) -> Result<(), CacheError> {
        if bytes.len() as u64 > MAX_MOD_BYTES || hash_of(bytes) != hash {
            return Err(CacheError::Mismatch);
        }
        std::fs::create_dir_all(&self.dir).map_err(CacheError::Io)?;
        let temporary = self.dir.join(format!("{hash}.part"));
        std::fs::write(&temporary, bytes).map_err(CacheError::Io)?;
        std::fs::rename(&temporary, self.path(hash)).map_err(CacheError::Io)?;
        self.prune(MAX_FILES);
        Ok(())
    }

    /// Removes the oldest files beyond `keep`.
    fn prune(&self, keep: usize) {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return;
        };
        let mut files: Vec<(std::time::SystemTime, PathBuf)> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| is_cached(path))
            .filter_map(|path| Some((path.metadata().ok()?.modified().ok()?, path)))
            .collect();
        files.sort();
        let excess = files.len().saturating_sub(keep);
        for (_, path) in files.into_iter().take(excess) {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn is_cached(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "wasm")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("photon-mods-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_stored_file_comes_back_and_a_wrong_one_is_refused() {
        let dir = scratch("put");
        let cache = ModCache::new(&dir);
        let hash = hash_of(b"module");
        assert!(cache.get(&hash).is_none());
        assert!(matches!(
            cache.put(&hash, b"other"),
            Err(CacheError::Mismatch)
        ));
        cache.put(&hash, b"module").expect("put");
        assert_eq!(cache.get(&hash).as_deref(), Some(&b"module"[..]));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_file_changed_on_disk_is_a_miss_and_is_removed() {
        let dir = scratch("tamper");
        let cache = ModCache::new(&dir);
        let hash = hash_of(b"module");
        cache.put(&hash, b"module").expect("put");
        std::fs::write(dir.join(format!("{hash}.wasm")), b"evil").expect("write");
        assert!(cache.get(&hash).is_none());
        assert!(!dir.join(format!("{hash}.wasm")).exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_oldest_files_go_past_the_limit() {
        let dir = scratch("prune");
        let cache = ModCache::new(&dir);
        for n in 0..5u8 {
            let bytes = [n];
            cache.put(&hash_of(&bytes), &bytes).expect("put");
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        cache.prune(2);
        let kept = std::fs::read_dir(&dir).expect("dir").count();
        assert_eq!(kept, 2);
        assert!(cache.get(&hash_of(&[4])).is_some());
        assert!(cache.get(&hash_of(&[0])).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }
}
