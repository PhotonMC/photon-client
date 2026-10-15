//! `cargo xtask setup`: the pinned game from Mojang's servers into `.data/minecraft/`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io;
use std::path::Path;

use rayon::prelude::*;
use serde::Deserialize;

use super::args::{Spec, parse};
use super::decompile;
use super::download::{fetch, fetch_json, sha1_of};
use super::layout::{assets, runtime};
use crate::parity::pins::Pins;

const VERSION_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES: &str = "https://resources.download.minecraft.net";

#[derive(Deserialize)]
struct Manifest {
    versions: Vec<ManifestEntry>,
}

#[derive(Deserialize)]
struct ManifestEntry {
    id: String,
    url: String,
    sha1: Option<String>,
}

#[derive(Deserialize, Clone)]
struct Artifact {
    url: String,
    sha1: String,
    #[serde(default)]
    path: String,
}

#[derive(Deserialize)]
struct Downloads {
    client: Artifact,
    server: Artifact,
}

#[derive(Deserialize)]
struct LibraryDownloads {
    artifact: Option<Artifact>,
}

#[derive(Deserialize)]
struct Library {
    downloads: Option<LibraryDownloads>,
}

#[derive(Deserialize)]
struct AssetIndexInfo {
    id: String,
    url: String,
    sha1: String,
}

#[derive(Deserialize)]
struct Version {
    downloads: Downloads,
    libraries: Vec<Library>,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndexInfo,
}

#[derive(Deserialize)]
struct AssetObject {
    hash: String,
}

#[derive(Deserialize)]
struct AssetIndex {
    objects: BTreeMap<String, AssetObject>,
}

fn pool(threads: usize) -> Result<rayon::ThreadPool, String> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|error| error.to_string())
}

fn say(message: &str) {
    println!("{message}");
}

/// The version file of the pinned release.
fn fetch_version(pins: &Pins) -> Result<Version, String> {
    say(&format!("Minecraft {}: version file", pins.version));
    let manifest: Manifest = fetch_json(VERSION_MANIFEST)?;
    let entry = manifest
        .versions
        .into_iter()
        .find(|entry| entry.id == pins.version)
        .ok_or(format!(
            "version {} is not in Mojang's manifest",
            pins.version
        ))?;
    let path = runtime().join(format!("{}.json", pins.version));
    fetch(&entry.url, &path, entry.sha1.as_deref())?;
    let text = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

fn fetch_jars(pins: &Pins, version: &Version) -> Result<(), String> {
    say("client and server jars");
    let client = runtime().join(format!("client-{}.jar", pins.version));
    fetch(
        &version.downloads.client.url,
        &client,
        Some(&version.downloads.client.sha1),
    )?;
    let named = client
        .file_name()
        .is_some_and(|name| name.to_string_lossy() == pins.client_file);
    if !named || sha1_of(&client)? != pins.client_sha1 {
        return Err("the client jar does not match tests/behavior/reference.json".to_owned());
    }
    let server = runtime().join(format!("server-{}.jar", pins.version));
    fetch(
        &version.downloads.server.url,
        &server,
        Some(&version.downloads.server.sha1),
    )?;
    Ok(())
}

/// Every library the version lists, natives for every platform included, as
/// the parity pins expect them by file name.
fn fetch_libraries(version: &Version) -> Result<(), String> {
    say("libraries");
    let mut libraries: Vec<Artifact> = version
        .libraries
        .iter()
        .filter_map(|library| library.downloads.as_ref()?.artifact.clone())
        .collect();
    pool(8)?.install(|| {
        libraries.par_iter().try_for_each(|artifact| {
            let name = Path::new(&artifact.path)
                .file_name()
                .ok_or("library without a path")?;
            fetch(
                &artifact.url,
                &runtime().join("libs").join(name),
                Some(&artifact.sha1),
            )
            .map(drop)
        })
    })?;
    libraries.sort_by(|a, b| a.url.cmp(&b.url));
    let urls: String = libraries
        .iter()
        .map(|artifact| format!("{}\n", artifact.url))
        .collect();
    let path = runtime().join("lib-urls.txt");
    fs::write(&path, urls).map_err(|error| format!("{}: {error}", path.display()))
}

/// What the client reads from the jar: textures, models, shaders, data.
fn extract_jar(pins: &Pins) -> Result<(), String> {
    say("jar resources -> .data/minecraft/assets/jar");
    let jar = runtime().join(format!("client-{}.jar", pins.version));
    let file = File::open(&jar).map_err(|error| format!("{}: {error}", jar.display()))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| format!("{}: {error}", jar.display()))?;
    let target = assets().join("jar");
    for index in 0..archive.len() {
        let mut member = archive.by_index(index).map_err(|error| error.to_string())?;
        let Some(relative) = member.enclosed_name() else {
            continue;
        };
        if member.is_dir()
            || relative
                .extension()
                .is_some_and(|extension| extension == "class")
        {
            continue;
        }
        let destination = target.join(relative);
        if fs::metadata(&destination).is_ok_and(|meta| meta.len() == member.size()) {
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        let mut out = File::create(&destination)
            .map_err(|error| format!("{}: {error}", destination.display()))?;
        io::copy(&mut member, &mut out)
            .map_err(|error| format!("{}: {error}", destination.display()))?;
    }
    Ok(())
}

/// Sounds and languages, content-addressed; returns the asset index.
fn fetch_objects(version: &Version) -> Result<AssetIndex, String> {
    let info = &version.asset_index;
    let path = assets().join("indexes").join(format!("{}.json", info.id));
    fetch(&info.url, &path, Some(&info.sha1))?;
    let text = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let index: AssetIndex =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    let hashes: BTreeSet<&str> = index
        .objects
        .values()
        .map(|object| object.hash.as_str())
        .collect();
    say(&format!(
        "asset objects: {} (sounds and languages)",
        hashes.len()
    ));
    let fetched: Vec<bool> = pool(16)?.install(|| {
        hashes
            .par_iter()
            .map(|digest| {
                let url = format!("{RESOURCES}/{}/{digest}", &digest[..2]);
                fetch(
                    &url,
                    &assets().join("objects").join(&digest[..2]).join(digest),
                    Some(digest),
                )
            })
            .collect::<Result<_, _>>()
    })?;
    say(&format!(
        "  downloaded {}, the rest were already there",
        fetched.iter().filter(|new| **new).count()
    ));
    Ok(index)
}

#[cfg(unix)]
fn link(source: &Path, destination: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(source, destination)
}

#[cfg(windows)]
fn link(source: &Path, destination: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(source, destination)
}

/// `assets/named/<logical path>`: links (copies where links are refused)
/// into the hash-named object store, for browsing.
fn build_named(index: &AssetIndex) -> Result<(), String> {
    say("named asset tree");
    for (logical, object) in &index.objects {
        let source = assets()
            .join("objects")
            .join(&object.hash[..2])
            .join(&object.hash);
        let destination = assets().join("named").join(logical);
        if destination.exists() {
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        }
        if link(&source, &destination).is_err() {
            fs::copy(&source, &destination)
                .map_err(|error| format!("{}: {error}", destination.display()))?;
        }
    }
    Ok(())
}

const SPEC: Spec = Spec {
    flags: &["--skip-objects", "--named", "--decompile"],
    options: &[("--java", 1)],
};

pub fn setup(args: &[String]) -> Result<(), String> {
    let parsed = parse(args, &SPEC)?;
    let pins = Pins::load()?;
    fs::create_dir_all(runtime()).map_err(|error| error.to_string())?;
    let version = fetch_version(&pins)?;
    fetch_jars(&pins, &version)?;
    fetch_libraries(&version)?;
    extract_jar(&pins)?;
    if !parsed.flag("--skip-objects") {
        let index = fetch_objects(&version)?;
        if parsed.flag("--named") {
            build_named(&index)?;
        }
    }
    if parsed.flag("--decompile") {
        decompile::run(&pins, parsed.value("--java"))?;
    }
    say("done");
    Ok(())
}
