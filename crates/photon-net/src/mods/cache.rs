//! The mods the client has fetched, kept by their hash. A file that no longer matches its name
//! is treated as missing, so a damaged or edited cache cannot run.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};

use super::MAX_MOD_BYTES;

/// Most files the cache keeps; the least recently used go first.
const MAX_FILES: usize = 128;
/// A half-written file older than this was left by a client that died; it is removed.
const STALE_PART: Duration = Duration::from_secs(24 * 60 * 60);
/// Tells apart the temporary files of writes in one process.
static PARTS: AtomicU64 = AtomicU64::new(0);

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
    /// The cache in `dir`. Opening it removes temporary files a dead client left behind.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let cache = Self { dir: dir.into() };
        cache.sweep(STALE_PART);
        cache
    }

    /// Removes `.part` files not touched for `age`.
    fn sweep(&self, age: Duration) {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return;
        };
        for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
            let old = path
                .metadata()
                .and_then(|meta| meta.modified())
                .ok()
                .and_then(|time| time.elapsed().ok())
                .is_some_and(|elapsed| elapsed >= age);
            if path.extension().is_some_and(|ext| ext == "part") && old {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    fn path(&self, hash: &str) -> PathBuf {
        self.dir.join(format!("{hash}.wasm"))
    }

    /// The file with this hash, if the cache has it and it is still intact.
    pub fn get(&self, hash: &str) -> Option<Vec<u8>> {
        let path = self.path(hash);
        let bytes = std::fs::read(&path).ok()?;
        if bytes.len() as u64 <= MAX_MOD_BYTES && hash_of(&bytes) == hash {
            // A read is a use: pruning removes what was used least recently, not written first.
            let _ = std::fs::File::options()
                .write(true)
                .open(&path)
                .and_then(|file| file.set_modified(SystemTime::now()));
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
        // Two clients on one game directory, or two threads, never write the same file.
        let unique = PARTS.fetch_add(1, Ordering::Relaxed);
        let temporary = self
            .dir
            .join(format!("{hash}.{}.{unique}.part", std::process::id()));
        std::fs::write(&temporary, bytes).map_err(CacheError::Io)?;
        if let Err(error) = std::fs::rename(&temporary, self.path(hash)) {
            let _ = std::fs::remove_file(&temporary);
            return Err(CacheError::Io(error));
        }
        self.prune(MAX_FILES);
        Ok(())
    }

    /// Removes the least recently used files beyond `keep`.
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

    #[test]
    fn a_read_counts_as_a_use_for_pruning() {
        let dir = scratch("lru");
        let cache = ModCache::new(&dir);
        for n in 0..3u8 {
            cache.put(&hash_of(&[n]), &[n]).expect("put");
            std::thread::sleep(Duration::from_millis(15));
        }
        // The first file written is read last, so it is the most recently used.
        assert!(cache.get(&hash_of(&[0])).is_some());
        cache.prune(2);
        assert!(cache.get(&hash_of(&[0])).is_some());
        assert!(cache.get(&hash_of(&[2])).is_some());
        assert!(cache.get(&hash_of(&[1])).is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn writers_sharing_a_directory_do_not_collide() {
        let dir = scratch("race");
        let bytes = vec![9u8; 64 << 10];
        let hash = hash_of(&bytes);
        let writers: Vec<_> = (0..8)
            .map(|_| {
                let (cache, bytes, hash) = (ModCache::new(&dir), bytes.clone(), hash.clone());
                std::thread::spawn(move || cache.put(&hash, &bytes))
            })
            .collect();
        for writer in writers {
            writer.join().expect("thread").expect("put");
        }
        assert_eq!(ModCache::new(&dir).get(&hash), Some(bytes));
        let names: Vec<_> = std::fs::read_dir(&dir).expect("dir").collect();
        assert_eq!(names.len(), 1, "no temporary file is left");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn opening_removes_only_stale_temporary_files() {
        let dir = scratch("stale");
        std::fs::create_dir_all(&dir).expect("dir");
        let (old, fresh) = (dir.join("a.1.0.part"), dir.join("b.2.0.part"));
        let kept = dir.join(format!("{}.wasm", hash_of(b"x")));
        for path in [&old, &fresh, &kept] {
            std::fs::write(path, b"x").expect("write");
        }
        let two_days_ago = SystemTime::now() - Duration::from_secs(2 * 24 * 60 * 60);
        std::fs::File::options()
            .write(true)
            .open(&old)
            .and_then(|file| file.set_modified(two_days_ago))
            .expect("age");
        // Even an old cached file is not a temporary one.
        std::fs::File::options()
            .write(true)
            .open(&kept)
            .and_then(|file| file.set_modified(two_days_ago))
            .expect("age");
        let _cache = ModCache::new(&dir);
        assert!(!old.exists());
        assert!(fresh.exists() && kept.exists());
        let _ = std::fs::remove_dir_all(dir);
    }
}
