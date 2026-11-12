//! `fs.mod_data`: a private key-value store for one mod on one server. It lives in memory while
//! the mod runs; what changed is handed to a writer thread at the end of a tick, which replaces
//! the file atomically, so a tick never waits for the disk.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Longest key, in bytes. Keys are printable ASCII.
pub const MAX_KEY: usize = 64;
/// Largest value, in bytes.
pub const MAX_VALUE: usize = 64 << 10;
/// Most a mod keeps in all, counting keys and values.
pub const MAX_TOTAL: usize = 1 << 20;
/// Most writes (sets and deletes) one tick of one mod may make; the rest are refused.
pub const MAX_WRITES_PER_TICK: u32 = 16;

/// Why a write was refused; the number is what the mod's `data-set` returns (0 is success).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataError {
    /// The key is empty, too long or not printable ASCII.
    Key = 1,
    /// The value is over [`MAX_VALUE`].
    Value = 2,
    /// The store would pass [`MAX_TOTAL`].
    Full = 3,
    /// The tick has made [`MAX_WRITES_PER_TICK`] writes already.
    Busy = 4,
    /// The mod has no store.
    Unavailable = 5,
}

#[derive(Default)]
struct Inner {
    entries: BTreeMap<String, Vec<u8>>,
    /// Bytes of keys and values together.
    total: usize,
    dirty: bool,
    writes: u32,
}

/// One mod's store, shared between the mod's imports and the runtime that saves it.
#[derive(Clone, Default)]
pub struct ModData {
    inner: Arc<Mutex<Inner>>,
}

fn valid_key(key: &str) -> bool {
    (1..=MAX_KEY).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_graphic())
}

impl ModData {
    /// The store a file holds; a missing, damaged or over-limit file is an empty store.
    pub fn load(path: &Path) -> Self {
        let data = Self::default();
        match std::fs::read(path) {
            Ok(bytes) => match decode(&bytes) {
                Some(entries) => {
                    let mut inner = data.lock();
                    inner.total = entries.iter().map(|(k, v)| k.len() + v.len()).sum();
                    inner.entries = entries;
                }
                None => log::warn!("mod data {} is damaged and starts empty", path.display()),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => log::warn!("mod data {} not read: {error}", path.display()),
        }
        data
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.lock().entries.get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<(), DataError> {
        if !valid_key(key) {
            return Err(DataError::Key);
        }
        if value.len() > MAX_VALUE {
            return Err(DataError::Value);
        }
        let mut inner = self.lock();
        if inner.writes >= MAX_WRITES_PER_TICK {
            return Err(DataError::Busy);
        }
        let before = inner
            .entries
            .get(key)
            .map_or(0, |old| key.len() + old.len());
        let total = inner.total - before + key.len() + value.len();
        if total > MAX_TOTAL {
            return Err(DataError::Full);
        }
        inner.writes += 1;
        inner.total = total;
        inner.entries.insert(key.to_owned(), value.to_vec());
        inner.dirty = true;
        Ok(())
    }

    /// Removes `key`; whether it was there. A delete counts as a write of the tick, and over the
    /// limit it does nothing.
    pub fn delete(&self, key: &str) -> bool {
        let mut inner = self.lock();
        if inner.writes >= MAX_WRITES_PER_TICK {
            return false;
        }
        inner.writes += 1;
        let Some(old) = inner.entries.remove(key) else {
            return false;
        };
        inner.total -= key.len() + old.len();
        inner.dirty = true;
        true
    }

    /// A new tick begins: the writes it may make count from none.
    pub fn new_tick(&self) {
        self.lock().writes = 0;
    }

    /// The file's bytes if anything changed since the last call.
    pub fn take_changes(&self) -> Option<Vec<u8>> {
        let mut inner = self.lock();
        std::mem::take(&mut inner.dirty).then(|| encode(&inner.entries))
    }
}

/// `key length (1 byte), key, value length (4 bytes, little endian), value`, repeated.
fn encode(entries: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut out = Vec::new();
    for (key, value) in entries {
        out.push(key.len() as u8);
        out.extend_from_slice(key.as_bytes());
        out.extend_from_slice(&(value.len() as u32).to_le_bytes());
        out.extend_from_slice(value);
    }
    out
}

fn decode(mut bytes: &[u8]) -> Option<BTreeMap<String, Vec<u8>>> {
    let mut entries = BTreeMap::new();
    let mut total = 0usize;
    while !bytes.is_empty() {
        let key_len = usize::from(*bytes.first()?);
        let key = std::str::from_utf8(bytes.get(1..1 + key_len)?).ok()?;
        let at = 1 + key_len;
        let value_len = u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize;
        let value = bytes.get(at + 4..at + 4 + value_len)?;
        total += key.len() + value.len();
        if !valid_key(key) || value.len() > MAX_VALUE || total > MAX_TOTAL {
            return None;
        }
        entries.insert(key.to_owned(), value.to_vec());
        bytes = &bytes[at + 4 + value_len..];
    }
    Some(entries)
}

/// Where the store of mod `id` on `server` lives: one folder per server, one file per mod, so
/// an updated mod (another hash, the same id) keeps its data. `server` is the key the app
/// knows the server by; anything but letters, digits, `.` and `-` becomes `_`.
pub fn data_path(game_dir: &Path, server: &str, id: &str) -> PathBuf {
    let folder: String = server
        .chars()
        .take(120)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let folder = if folder.is_empty() {
        "unknown".to_owned()
    } else {
        folder
    };
    game_dir
        .join("mod_data")
        .join(folder)
        .join(format!("{id}.bin"))
}

/// The thread that writes stores to disk. Dropping it writes what is queued first.
pub struct DataWriter {
    queue: Option<Sender<(PathBuf, Vec<u8>)>>,
    thread: Option<JoinHandle<()>>,
}

static TEMP: AtomicU64 = AtomicU64::new(0);

fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let unique = TEMP.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_extension(format!("tmp-{}-{unique}", std::process::id()));
    let result = std::fs::write(&temp, bytes).and_then(|()| std::fs::rename(&temp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

impl DataWriter {
    pub fn new() -> std::io::Result<Self> {
        let (queue, receiver) = channel::<(PathBuf, Vec<u8>)>();
        let thread = std::thread::Builder::new()
            .name("Mod Data".to_owned())
            .spawn(move || {
                while let Ok(first) = receiver.recv() {
                    // Several changes of one file queued up: only the last one matters.
                    let mut latest: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
                    latest.insert(first.0, first.1);
                    while let Ok((path, bytes)) = receiver.try_recv() {
                        latest.insert(path, bytes);
                    }
                    for (path, bytes) in latest {
                        if let Err(error) = write_atomically(&path, &bytes) {
                            log::warn!("mod data {} not saved: {error}", path.display());
                        }
                    }
                }
            })?;
        Ok(Self {
            queue: Some(queue),
            thread: Some(thread),
        })
    }

    /// Queues `bytes` as the new content of `path`; returns at once.
    pub fn submit(&self, path: PathBuf, bytes: Vec<u8>) {
        if let Some(queue) = &self.queue {
            let _ = queue.send((path, bytes));
        }
    }
}

impl Drop for DataWriter {
    fn drop(&mut self) {
        self.queue = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("photon-mod-data-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn keys_values_and_the_whole_are_limited() {
        let data = ModData::default();
        assert_eq!(data.set("", b"x"), Err(DataError::Key));
        assert_eq!(data.set(&"k".repeat(65), b"x"), Err(DataError::Key));
        assert_eq!(data.set("has space", b"x"), Err(DataError::Key));
        assert_eq!(data.set("\u{e9}", b"x"), Err(DataError::Key));
        assert_eq!(
            data.set("k", &vec![0; MAX_VALUE + 1]),
            Err(DataError::Value)
        );
        assert!(data.set(&"k".repeat(64), &vec![0; MAX_VALUE]).is_ok());
        // 16 values of 64 KiB fill the 1 MiB (keys count too): the sixteenth does not fit.
        for n in 0..15 {
            data.new_tick();
            let _ = data.set(&format!("a{n}"), &vec![0; MAX_VALUE]);
        }
        data.new_tick();
        assert_eq!(data.set("z", &vec![0; MAX_VALUE]), Err(DataError::Full));
        // Replacing a value counts the difference only.
        assert!(data.set("a0", b"small").is_ok());
    }

    #[test]
    fn writes_per_tick_are_limited_and_a_new_tick_allows_more() {
        let data = ModData::default();
        for n in 0..MAX_WRITES_PER_TICK {
            assert!(data.set(&format!("k{n}"), b"v").is_ok());
        }
        assert_eq!(data.set("late", b"v"), Err(DataError::Busy));
        assert!(!data.delete("k0"));
        assert_eq!(data.get("k0"), Some(b"v".to_vec()));
        data.new_tick();
        assert!(data.delete("k0"));
        assert_eq!(data.get("k0"), None);
    }

    #[test]
    fn changes_survive_the_writer_and_a_new_load() {
        let dir = scratch("persist");
        let path = data_path(&dir, "play.example.org:25565", "hud");
        let data = ModData::load(&path);
        assert_eq!(data.take_changes(), None);
        data.set("score", b"42").unwrap();
        data.set("name", b"x").unwrap();
        let writer = DataWriter::new().unwrap();
        writer.submit(path.clone(), data.take_changes().unwrap());
        assert_eq!(data.take_changes(), None);
        drop(writer);
        let again = ModData::load(&path);
        assert_eq!(again.get("score"), Some(b"42".to_vec()));
        assert_eq!(again.get("name"), Some(b"x".to_vec()));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_damaged_file_is_an_empty_store_and_servers_and_mods_have_their_own_files() {
        let dir = scratch("damaged");
        let (a, b) = (data_path(&dir, "a:1", "m"), data_path(&dir, "b:1", "m"));
        assert_ne!(a, b);
        assert_ne!(a, data_path(&dir, "a:1", "n"));
        std::fs::create_dir_all(a.parent().unwrap()).unwrap();
        std::fs::write(&a, [200, 1, 2]).unwrap();
        assert_eq!(ModData::load(&a).get("x"), None);
        // A path cannot climb out of its folder through the server's name.
        assert!(data_path(&dir, "../../x", "m").starts_with(dir.join("mod_data")));
        let _ = std::fs::remove_dir_all(dir);
    }
}
