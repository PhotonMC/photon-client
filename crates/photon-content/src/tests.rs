//! The pipeline against a server on this machine: no Mojang, no account.

mod server;

use std::io::Write;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;

use crate::cache::Cache;
use crate::endpoints::Endpoints;
use crate::error::ContentError;
use crate::hash::Sha1Hex;
use crate::install::{Installed, Progress, install_with, installed_for};
use crate::pin::Pin;
use server::Stand;

const META: &str = "https://piston-meta.mojang.com";
const DATA: &str = "https://piston-data.mojang.com";

/// A game made up for the test: a jar, two distinct objects (one under two names) and the
/// documents that name them.
struct Fixture {
    jar: Vec<u8>,
    objects: Vec<Vec<u8>>,
    index_names: Vec<(String, usize)>,
}

fn jar_of(members: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, bytes) in members {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

impl Fixture {
    fn new() -> Self {
        Self {
            jar: jar_of(&[
                ("assets/minecraft/lang/en_us.json", b"{}"),
                ("data/minecraft/tags.json", b"[]"),
                ("net/minecraft/Main.class", b"\xca\xfe"),
                ("pack.png", b"png"),
            ]),
            objects: vec![b"one".to_vec(), b"two!".to_vec()],
            index_names: vec![
                ("minecraft/sounds/a.ogg".to_owned(), 0),
                ("minecraft/sounds/b.ogg".to_owned(), 1),
                ("minecraft/sounds/c.ogg".to_owned(), 0),
            ],
        }
    }

    fn index_json(&self) -> String {
        let entries: Vec<String> = self
            .index_names
            .iter()
            .map(|(name, object)| {
                let bytes = &self.objects[*object];
                format!(
                    r#""{name}":{{"hash":"{}","size":{}}}"#,
                    Sha1Hex::of_bytes(bytes).as_str(),
                    bytes.len()
                )
            })
            .collect();
        format!(r#"{{"objects":{{{}}}}}"#, entries.join(","))
    }

    fn pin(&self) -> Pin {
        let sha = Box::leak(
            Sha1Hex::of_bytes(&self.jar)
                .as_str()
                .to_owned()
                .into_boxed_str(),
        );
        Pin {
            id: "26.2",
            client_sha1: sha,
        }
    }

    /// Serve everything, with the version metadata as `version` (made from the fixture if none).
    fn serve(&self, stand: &Stand, version: Option<String>) {
        let index = self.index_json();
        let index_sha = Sha1Hex::of_bytes(index.as_bytes());
        let jar_sha = Sha1Hex::of_bytes(&self.jar);
        let version = version.unwrap_or_else(|| {
            format!(
                r#"{{"id":"26.2","downloads":{{"client":{{"url":"{DATA}/v1/objects/{jar}/client.jar","sha1":"{jar}","size":{}}}}},"assetIndex":{{"id":"32","url":"{META}/v1/packages/{idx}/32.json","sha1":"{idx}","size":{}}}}}"#,
                self.jar.len(),
                index.len(),
                jar = jar_sha.as_str(),
                idx = index_sha.as_str(),
            )
        });
        let version_sha = Sha1Hex::of_bytes(version.as_bytes());
        let manifest = format!(
            r#"{{"versions":[{{"id":"26.2","url":"{META}/v1/packages/{v}/26.2.json","sha1":"{v}"}}]}}"#,
            v = version_sha.as_str()
        );
        stand.serve(
            "/meta/mc/game/version_manifest_v2.json",
            manifest.into_bytes(),
        );
        stand.serve(
            &format!("/meta/v1/packages/{}/26.2.json", version_sha.as_str()),
            version.into_bytes(),
        );
        stand.serve(
            &format!("/meta/v1/packages/{}/32.json", index_sha.as_str()),
            index.into_bytes(),
        );
        stand.serve(
            &format!("/data/v1/objects/{}/client.jar", jar_sha.as_str()),
            self.jar.clone(),
        );
        for bytes in &self.objects {
            let sha = Sha1Hex::of_bytes(bytes);
            stand.serve(
                &format!("/resources/{}/{}", &sha.as_str()[..2], sha.as_str()),
                bytes.clone(),
            );
        }
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("photon-content-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn run(stand: &Stand, fixture: &Fixture, cache: &Cache) -> Result<Installed, ContentError> {
    let endpoints = Endpoints::stand_in(&stand.base);
    install_with(&endpoints, fixture.pin(), cache, &mut |_: Progress| {})
}

fn files_under(root: &Path) -> Vec<(String, u64)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, u64)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else {
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                out.push((name, entry.metadata().unwrap().len()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

#[test]
fn the_pipeline_fills_the_cache_and_a_second_run_fetches_nothing() {
    let (stand, fixture) = (Stand::start(), Fixture::new());
    fixture.serve(&stand, None);
    let cache = Cache::at(scratch("full"));
    let first = run(&stand, &fixture, &cache).unwrap();
    assert_eq!(
        first.downloaded,
        1 + 1 + 1 + 2,
        "version, index, jar and two objects"
    );
    assert_eq!(first.asset_index.as_str(), "32");
    let tree = files_under(first.jar_assets.parent().unwrap());
    let names: Vec<&str> = tree.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "assets/minecraft/lang/en_us.json",
            "data/minecraft/tags.json",
            "pack.png"
        ]
    );
    for object in &fixture.objects {
        let path = cache.object(&Sha1Hex::of_bytes(object));
        assert_eq!(&std::fs::read(path).unwrap(), object);
    }
    assert!(first.root.join("indexes/32.json").is_file());
    let asked = stand.asked().len();
    let second = run(&stand, &fixture, &cache).unwrap();
    assert_eq!(second.downloaded, 0);
    let again = &stand.asked()[asked..];
    assert_eq!(
        again,
        ["/meta/mc/game/version_manifest_v2.json"],
        "only the manifest is asked"
    );
    assert!(
        files_under(cache.root())
            .iter()
            .all(|(name, _)| !name.ends_with(".part"))
    );
}

#[test]
fn an_object_that_is_not_the_promised_one_is_never_kept() {
    let (stand, fixture) = (Stand::start(), Fixture::new());
    fixture.serve(&stand, None);
    let victim = Sha1Hex::of_bytes(&fixture.objects[1]);
    stand.serve(
        &format!("/resources/{}/{}", &victim.as_str()[..2], victim.as_str()),
        b"evil".to_vec(),
    );
    let cache = Cache::at(scratch("mismatch"));
    let error = run(&stand, &fixture, &cache).unwrap_err();
    assert!(matches!(error, ContentError::Mismatch(_)), "{error}");
    assert!(!cache.object(&victim).exists());
    assert!(
        files_under(cache.root())
            .iter()
            .all(|(name, _)| !name.ends_with(".part"))
    );
}

#[test]
fn a_short_body_is_a_mismatch() {
    let (stand, fixture) = (Stand::start(), Fixture::new());
    fixture.serve(&stand, None);
    let victim = Sha1Hex::of_bytes(&fixture.objects[1]);
    stand.serve(
        &format!("/resources/{}/{}", &victim.as_str()[..2], victim.as_str()),
        b"tw".to_vec(),
    );
    let error = run(&stand, &fixture, &Cache::at(scratch("short"))).unwrap_err();
    assert!(matches!(error, ContentError::Mismatch(_)), "{error}");
}

#[test]
fn metadata_cannot_send_the_download_to_another_host() {
    let (stand, fixture) = (Stand::start(), Fixture::new());
    let jar = Sha1Hex::of_bytes(&fixture.jar);
    let version = format!(
        r#"{{"id":"26.2","downloads":{{"client":{{"url":"https://evil.example/client.jar","sha1":"{jar}","size":{}}}}},"assetIndex":{{"id":"32","url":"{META}/x","sha1":"{jar}","size":1}}}}"#,
        fixture.jar.len(),
        jar = jar.as_str()
    );
    fixture.serve(&stand, Some(version));
    let error = run(&stand, &fixture, &Cache::at(scratch("foreign"))).unwrap_err();
    assert!(matches!(error, ContentError::ForeignHost(_)), "{error}");
    assert!(stand.asked().iter().all(|path| !path.contains("evil")));
}

#[test]
fn an_index_with_a_path_for_a_hash_is_refused_before_any_object_is_fetched() {
    let (stand, mut fixture) = (Stand::start(), Fixture::new());
    fixture.index_names.clear();
    let index = r#"{"objects":{"a":{"hash":"../../../etc/passwd","size":3}}}"#;
    let idx = Sha1Hex::of_bytes(index.as_bytes());
    let jar = Sha1Hex::of_bytes(&fixture.jar);
    let version = format!(
        r#"{{"id":"26.2","downloads":{{"client":{{"url":"{DATA}/v1/objects/{jar}/client.jar","sha1":"{jar}","size":{}}}}},"assetIndex":{{"id":"32","url":"{META}/v1/packages/{idx}/32.json","sha1":"{idx}","size":{}}}}}"#,
        fixture.jar.len(),
        index.len(),
        jar = jar.as_str(),
        idx = idx.as_str()
    );
    fixture.serve(&stand, Some(version));
    stand.serve(
        &format!("/meta/v1/packages/{}/32.json", idx.as_str()),
        index.as_bytes().to_vec(),
    );
    let error = run(&stand, &fixture, &Cache::at(scratch("badhash"))).unwrap_err();
    assert!(matches!(error, ContentError::BadHash(_)), "{error}");
    assert!(
        stand
            .asked()
            .iter()
            .all(|path| !path.starts_with("/resources"))
    );
}

#[test]
fn a_jar_that_is_not_the_pinned_one_stops_the_download() {
    let (stand, fixture) = (Stand::start(), Fixture::new());
    fixture.serve(&stand, None);
    let other = Pin {
        id: "26.2",
        client_sha1: "0000000000000000000000000000000000000000",
    };
    let cache = Cache::at(scratch("otherjar"));
    let error = install_with(
        &Endpoints::stand_in(&stand.base),
        other,
        &cache,
        &mut |_| {},
    )
    .unwrap_err();
    assert!(matches!(error, ContentError::Mismatch(_)), "{error}");
}

#[test]
fn an_interrupted_extraction_is_redone_and_a_member_outside_the_folder_is_refused() {
    let (stand, fixture) = (Stand::start(), Fixture::new());
    fixture.serve(&stand, None);
    let cache = Cache::at(scratch("extract"));
    let tree = cache.jar_tree(&Sha1Hex::of_bytes(&fixture.jar));
    let mut leftover = tree.as_os_str().to_owned();
    leftover.push(".tmp");
    std::fs::create_dir_all(Path::new(&leftover).join("assets")).unwrap();
    std::fs::write(Path::new(&leftover).join("assets/half.bin"), b"x").unwrap();
    let done = run(&stand, &fixture, &cache).unwrap();
    assert!(done.jar_assets.join("minecraft/lang/en_us.json").is_file());
    assert!(!Path::new(&leftover).exists());

    let slip = Fixture {
        jar: jar_of(&[("../evil.txt", b"x")]),
        ..Fixture::new()
    };
    let stand = Stand::start();
    slip.serve(&stand, None);
    let cache = Cache::at(scratch("slip"));
    let error = run(&stand, &slip, &cache).unwrap_err();
    assert!(matches!(error, ContentError::UnsafePath(_)), "{error}");
    assert!(!cache.jar_tree(&Sha1Hex::of_bytes(&slip.jar)).exists());
    assert!(!cache.root().join("evil.txt").exists());
}

/// The real client jar of `.data`, served with made-up metadata: the extracted tree is the one
/// the development setup makes.
#[test]
fn the_real_jar_extracts_to_the_development_tree() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.data/minecraft");
    let (jar, dev_tree) = (
        data.join("runtime/client-26.2.jar"),
        data.join("assets/jar"),
    );
    let Ok(bytes) = std::fs::read(&jar) else {
        println!("missing reference: {}", jar.display());
        return;
    };
    let fixture = Fixture {
        jar: bytes,
        objects: Vec::new(),
        index_names: Vec::new(),
    };
    let stand = Stand::start();
    fixture.serve(&stand, None);
    let cache = Cache::at(scratch("real"));
    let done = run(&stand, &fixture, &cache).unwrap();
    let extracted = files_under(done.jar_assets.parent().unwrap());
    assert!(extracted.len() > 1000, "{} files", extracted.len());
    assert_eq!(extracted, files_under(&dev_tree));
    // The jar is the pinned one, so the start-up lookup takes this cache as the installed game.
    let found = crate::installed(&cache).expect("the pinned game is installed");
    assert_eq!(found.jar_assets, done.jar_assets);
    assert_eq!(crate::asset_index_id(cache.root()).unwrap().as_str(), "32");
}

#[test]
fn a_republished_index_of_the_same_size_replaces_the_cached_one() {
    let (stand, fixture) = (Stand::start(), Fixture::new());
    fixture.serve(&stand, None);
    let cache = Cache::at(scratch("republish"));
    let index = fixture.index_json();
    let stale = index.replace("minecraft/sounds/a.ogg", "minecraft/sounds/z.ogg");
    assert_eq!(stale.len(), index.len());
    let path = cache.index(&crate::cache::Id::parse("32").unwrap());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, &stale).unwrap();
    let done = run(&stand, &fixture, &cache).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), index);
    assert_eq!(
        done.downloaded,
        1 + 1 + 1 + 2,
        "the index is fetched again with the rest"
    );
}

#[test]
fn an_installed_cache_is_found_again_and_a_half_made_one_is_not() {
    let (stand, fixture) = (Stand::start(), Fixture::new());
    fixture.serve(&stand, None);
    let cache = Cache::at(scratch("found"));
    assert!(
        installed_for(fixture.pin(), &cache).is_none(),
        "nothing yet"
    );
    let done = run(&stand, &fixture, &cache).unwrap();
    assert_eq!(
        installed_for(fixture.pin(), &cache),
        Some(Installed {
            downloaded: 0,
            ..done
        })
    );
    let other = Pin {
        id: "26.2",
        client_sha1: "0000000000000000000000000000000000000000",
    };
    assert!(
        installed_for(other, &cache).is_none(),
        "a cache of another jar"
    );
    let tree = cache.jar_tree(&Sha1Hex::of_bytes(&fixture.jar));
    std::fs::remove_dir_all(&tree).unwrap();
    assert!(
        installed_for(fixture.pin(), &cache).is_none(),
        "the extraction is what completes it"
    );
}
