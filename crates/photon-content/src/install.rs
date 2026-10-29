//! The pipeline: manifest, version metadata, client jar, asset index, asset objects, then the
//! jar's resources. Every step is skipped when its result is already in the cache.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, mpsc};
use std::time::Duration;

use photon_net::signin::GameOwnership;

use crate::cache::{Cache, Id};
use crate::endpoints::Endpoints;
use crate::error::{ContentError, io_error};
use crate::extract::extract_resources;
use crate::fetch::{Want, agent, fetch, fetch_text};
use crate::hash::Sha1Hex;
use crate::metadata::{AssetObject, VersionInfo, asset_objects, manifest_entry, version_info};
use crate::pin::{PINNED, Pin};

/// The largest metadata document accepted.
const METADATA_LIMIT: u64 = 32 << 20;
const WORKERS: usize = 8;

/// The step the download is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Metadata,
    Jar,
    Assets,
    Extract,
}

/// How far a step has come (`total` is 0 while it is not known).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub stage: Stage,
    pub done: u64,
    pub total: u64,
}

/// What the cache holds after a download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The cache root, which holds `indexes/` and `objects/`.
    pub root: PathBuf,
    /// The jar's `assets` folder.
    pub jar_assets: PathBuf,
    /// The jar's `data` folder.
    pub data: PathBuf,
    pub asset_index: Id,
    /// How many files were fetched (none when the cache was complete).
    pub downloaded: usize,
}

/// Download the pinned game into `cache`. Only an account that owns the game can ask: the proof
/// comes from the sign-in's ownership check.
pub fn install(
    _owner: &GameOwnership,
    cache: &Cache,
    progress: &mut dyn FnMut(Progress),
) -> Result<Installed, ContentError> {
    install_with(&Endpoints::official(), PINNED, cache, progress)
}

pub(crate) fn install_with(
    endpoints: &Endpoints,
    pin: Pin,
    cache: &Cache,
    progress: &mut dyn FnMut(Progress),
) -> Result<Installed, ContentError> {
    let jar_sha = Sha1Hex::parse(pin.client_sha1)?;
    let mut run = Run {
        endpoints,
        agent: agent(endpoints),
        cache,
        downloaded: 0,
    };
    let mut tick = |stage, done, total| progress(Progress { stage, done, total });
    tick(Stage::Metadata, 0, 1);
    let (info, objects) = run.metadata(&Id::parse(pin.id)?, &jar_sha)?;
    tick(Stage::Metadata, 1, 1);

    tick(Stage::Jar, 0, info.client.size);
    let jar_path = cache.object(&jar_sha);
    let want = Want {
        sha1: &jar_sha,
        size: Some(info.client.size),
        limit: 0,
    };
    run.get(&endpoints.allow(&info.client.url)?, &jar_path, &want)?;
    tick(Stage::Jar, info.client.size, info.client.size);

    run.objects(&objects, &mut |done, total| {
        tick(Stage::Assets, done, total);
    })?;

    let tree = cache.jar_tree(&jar_sha);
    if !tree.is_dir() {
        tick(Stage::Extract, 0, 1);
        extract_resources(&jar_path, &tree)?;
    }
    tick(Stage::Extract, 1, 1);
    Ok(Installed {
        root: cache.root().to_path_buf(),
        jar_assets: tree.join("assets"),
        data: tree.join("data"),
        asset_index: info.asset_index_id,
        downloaded: run.downloaded,
    })
}

/// One download in progress: where it goes and how much it has fetched so far.
struct Run<'a> {
    endpoints: &'a Endpoints,
    agent: ureq::Agent,
    cache: &'a Cache,
    downloaded: usize,
}

impl Run<'_> {
    fn get(&mut self, url: &str, dest: &Path, want: &Want<'_>) -> Result<(), ContentError> {
        self.downloaded += usize::from(fetch(self.endpoints, &self.agent, url, dest, want)?);
        Ok(())
    }

    /// The manifest's word on the pinned version, then its metadata and asset index. The jar the
    /// metadata names must be the pinned one, and every URL it gives is vetted before any is
    /// requested.
    fn metadata(
        &mut self,
        id: &Id,
        jar: &Sha1Hex,
    ) -> Result<(VersionInfo, Vec<AssetObject>), ContentError> {
        let manifest = fetch_text(&self.agent, &self.endpoints.manifest_url(), METADATA_LIMIT)?;
        let (version_url, version_sha) = manifest_entry(&manifest, id)?;
        let version_path = self.cache.version(id);
        let want = Want {
            sha1: &version_sha,
            size: None,
            limit: METADATA_LIMIT,
        };
        self.get(&self.endpoints.allow(&version_url)?, &version_path, &want)?;
        let text = read(&version_path)?;
        let info = version_info(&text, id)?;
        if &info.client.sha1 != jar {
            return Err(ContentError::Mismatch(
                "the client jar is not the pinned one".to_owned(),
            ));
        }
        self.endpoints.allow(&info.client.url)?;
        let index_url = self.endpoints.allow(&info.asset_index.url)?;
        let index_path = self.cache.index(&info.asset_index_id);
        let want = Want {
            sha1: &info.asset_index.sha1,
            size: Some(info.asset_index.size),
            limit: 0,
        };
        self.get(&index_url, &index_path, &want)?;
        let objects = asset_objects(&read(&index_path)?)?;
        Ok((info, objects))
    }

    /// The asset objects, each distinct digest once, by a few worker threads.
    fn objects(
        &mut self,
        objects: &[AssetObject],
        progress: &mut dyn FnMut(u64, u64),
    ) -> Result<(), ContentError> {
        let mut seen = std::collections::HashSet::new();
        let wanted: Vec<&AssetObject> = objects.iter().filter(|o| seen.insert(&o.sha1)).collect();
        let total = wanted.len() as u64;
        let pool = Pool {
            run: self,
            wanted: &wanted,
            next: AtomicUsize::new(0),
            failure: Mutex::new(None),
        };
        let (sender, receiver) = mpsc::channel::<bool>();
        let (mut fetched, mut done) = (0usize, 0u64);
        std::thread::scope(|scope| {
            for _ in 0..WORKERS.min(wanted.len()) {
                let sender = sender.clone();
                let pool = &pool;
                scope.spawn(move || pool.work(&sender));
            }
            drop(sender);
            progress(0, total);
            while let Ok(downloaded) = receiver.recv_timeout(Duration::from_secs(3600)) {
                done += 1;
                fetched += usize::from(downloaded);
                progress(done, total);
            }
        });
        let failure = pool.failure.into_inner().ok().flatten();
        self.downloaded += fetched;
        failure.map_or(Ok(()), Err)
    }
}

/// What the worker threads share while the objects are fetched.
struct Pool<'a> {
    run: &'a Run<'a>,
    wanted: &'a [&'a AssetObject],
    next: AtomicUsize,
    failure: Mutex<Option<ContentError>>,
}

impl Pool<'_> {
    /// Fetch objects until none are left or one has failed; each fetch is reported on `sender`.
    fn work(&self, sender: &mpsc::Sender<bool>) {
        while self.failure.lock().is_ok_and(|slot| slot.is_none()) {
            let Some(object) = self.wanted.get(self.next.fetch_add(1, Ordering::Relaxed)) else {
                return;
            };
            let Run {
                endpoints,
                agent,
                cache,
                ..
            } = self.run;
            let want = Want {
                sha1: &object.sha1,
                size: Some(object.size),
                limit: 0,
            };
            let url = endpoints.object_url(&object.sha1);
            match fetch(endpoints, agent, &url, &cache.object(&object.sha1), &want) {
                Ok(downloaded) => {
                    let _ = sender.send(downloaded);
                }
                Err(error) => {
                    if let Ok(mut slot) = self.failure.lock() {
                        slot.get_or_insert(error);
                    }
                }
            }
        }
    }
}

fn read(path: &Path) -> Result<String, ContentError> {
    std::fs::read_to_string(path).map_err(|error| io_error(path, &error))
}
