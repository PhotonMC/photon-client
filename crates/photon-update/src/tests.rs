use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};

use super::*;
use crate::swap;

/// A key made for the test; the real key never exists in the repository.
fn test_key() -> Ed25519KeyPair {
    let document = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
    Ed25519KeyPair::from_pkcs8(document.as_ref()).unwrap()
}

fn public(pair: &Ed25519KeyPair) -> [u8; 32] {
    pair.public_key().as_ref().try_into().unwrap()
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn manifest(archive: &[u8]) -> String {
    let sum = to_hex(Sha256::digest(archive).as_slice());
    format!(
        "version=1.4.0\narchive=photon-1.4.0.tar.gz\nsha256={sum}\nsize={}\n",
        archive.len()
    )
}

#[test]
fn a_manifest_signed_with_the_key_is_accepted() {
    let pair = test_key();
    let text = manifest(b"archive");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let release = verify_manifest(&[public(&pair)], text.as_bytes(), &signature).unwrap();
    assert_eq!(release.version, "1.4.0");
    assert_eq!(release.archive, "photon-1.4.0.tar.gz");
    assert_eq!(check_archive(&release, b"archive"), Ok(()));
    assert_eq!(check_archive(&release, b"other"), Err(Error::WrongArchive));
}

#[test]
fn another_key_or_a_changed_manifest_is_refused() {
    let (pair, other) = (test_key(), test_key());
    let text = manifest(b"archive");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    assert_eq!(
        verify_manifest(&[public(&other)], text.as_bytes(), &signature),
        Err(Error::Untrusted)
    );
    let changed = text.replace("1.4.0", "9.9.9");
    assert_eq!(
        verify_manifest(&[public(&pair)], changed.as_bytes(), &signature),
        Err(Error::Untrusted)
    );
}

#[test]
fn an_unset_key_and_a_malformed_signature_refuse() {
    let text = manifest(b"archive");
    assert_eq!(
        verify_manifest(&[], text.as_bytes(), "00"),
        Err(Error::NoKey)
    );
    let pair = test_key();
    assert_eq!(
        verify_manifest(&[public(&pair)], text.as_bytes(), "zz"),
        Err(Error::BadSignature)
    );
    assert_eq!(RELEASE_KEYS.len(), 0);
}

#[test]
fn a_signed_but_malformed_manifest_is_refused() {
    let pair = test_key();
    for (text, why) in [
        (
            "version=1\nversion=2\narchive=a\nsha256=00\nsize=1\n",
            "repeated field",
        ),
        ("version=1\narchive=a\nsize=1\n", "no sha256"),
        (
            "version=1\narchive=a\nsha256=12\nsize=1\n",
            "sha256 is not 32 bytes of hex",
        ),
        (
            "version=1\narchive=a\nsha256=00\nsize=1\nBad Name=1\n",
            "a malformed field name",
        ),
        (
            "version=1\narchive=a\nsha256=00\nsize=x\n",
            "size is not a number",
        ),
        ("version=1\narchive=a\nsha256=00\n", "no size"),
        (
            "version=1\narchive=a\nsha256=00\nsize=1\nprotocols=1,x\n",
            "protocols are not numbers",
        ),
    ] {
        let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
        // The sha256 of the first rows is not valid hex of 32 bytes, which is checked first.
        let sum = to_hex(Sha256::digest(b"x").as_slice());
        let text = text.replace("sha256=00", &format!("sha256={sum}"));
        let signature = if text.contains(&sum) {
            to_hex(pair.sign(text.as_bytes()).as_ref())
        } else {
            signature
        };
        assert_eq!(
            verify_manifest(&[public(&pair)], text.as_bytes(), &signature),
            Err(Error::BadManifest(why)),
            "{text}"
        );
    }
}

#[test]
fn fields_a_later_release_adds_are_ignored_and_the_known_optional_ones_are_read() {
    let pair = test_key();
    let sum = to_hex(Sha256::digest(b"x").as_slice());
    let text = format!(
        "version=2.0.0\narchive=a.tar.gz\nsha256={sum}\nsize=1\nnotes=Faster chunks\nprotocols=775, 776\nchannel_hint=beta\n"
    );
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let release = verify_manifest(&[public(&pair)], text.as_bytes(), &signature).unwrap();
    assert_eq!(release.notes, "Faster chunks");
    assert_eq!(release.protocols, [775, 776]);
    assert_eq!(release.size, 1);
}

#[test]
fn any_signature_line_under_any_listed_key_is_enough() {
    let (old, new, stranger) = (test_key(), test_key(), test_key());
    let text = manifest(b"archive");
    let by = |pair: &Ed25519KeyPair| to_hex(pair.sign(text.as_bytes()).as_ref());
    let keys = [public(&old), public(&new)];
    // Signed with the new key only, the old key only, and both (the overlap of a rotation).
    for signatures in [
        by(&new),
        by(&old),
        format!("{}\n{}\n", by(&stranger), by(&new)),
    ] {
        assert!(verify_manifest(&keys, text.as_bytes(), &signatures).is_ok());
    }
    assert_eq!(
        verify_manifest(&keys, text.as_bytes(), &by(&stranger)),
        Err(Error::Untrusted)
    );
    assert_eq!(
        verify_manifest(&keys, text.as_bytes(), "not hex\n"),
        Err(Error::BadSignature)
    );
}

#[test]
fn versions_compare_as_numbers() {
    assert!(is_newer("1.2.9", "1.2.10"));
    assert!(!is_newer("1.2.10", "1.2.10"));
    assert!(!is_newer("1.3", "1.2.9"));
    assert!(!is_newer("1.0.0", "next"));
}

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("photon-update-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_staged_release_is_checked_again_when_it_is_read() {
    let pair = test_key();
    let key = public(&pair);
    let text = manifest(b"archive");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let pending = folder("stage").join("pending");
    let staged = stage(
        &pending,
        &[key],
        "1.3.0",
        (text.as_bytes(), &signature),
        b"archive",
    )
    .unwrap();
    assert_eq!(staged.release.version, "1.4.0");
    assert_eq!(self::staged(&pending, &[key]).unwrap(), staged);
    // Someone replaces the stored archive: it is refused.
    std::fs::write(&staged.archive, b"tampered").unwrap();
    assert_eq!(self::staged(&pending, &[key]), Err(Error::WrongArchive));
}

#[test]
fn nothing_is_stored_for_an_old_release_or_a_wrong_archive() {
    let pair = test_key();
    let key = public(&pair);
    let text = manifest(b"archive");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let pending = folder("refuse").join("pending");
    let manifest_and_signature = (text.as_bytes(), signature.as_str());
    assert_eq!(
        stage(
            &pending,
            &[key],
            "1.4.0",
            manifest_and_signature,
            b"archive"
        ),
        Err(Error::NotNewer)
    );
    assert_eq!(
        stage(&pending, &[key], "1.3.0", manifest_and_signature, b"other"),
        Err(Error::WrongArchive)
    );
    assert!(!pending.exists());
}

#[test]
fn an_archive_name_with_a_path_is_refused() {
    let pair = test_key();
    let key = public(&pair);
    let sum = to_hex(Sha256::digest(b"a").as_slice());
    let text = format!("version=2.0.0\narchive=../escape.tar.gz\nsha256={sum}\nsize=1\n");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let pending = folder("name").join("pending");
    assert_eq!(
        stage(
            &pending,
            &[key],
            "1.0.0",
            (text.as_bytes(), &signature),
            b"a"
        ),
        Err(Error::BadName)
    );
    assert!(!pending.exists());
}

/// A host on this machine that serves `files` (path, body) until it has answered `count` requests.
fn host(
    files: Vec<(&'static str, Vec<u8>)>,
    count: usize,
) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let thread = std::thread::spawn(move || {
        for _ in 0..count {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut chunk = [0u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut chunk).unwrap();
                request.extend_from_slice(&chunk[..read]);
            }
            let line = String::from_utf8_lossy(&request);
            let path = line.split_whitespace().nth(1).unwrap().to_owned();
            let body = files
                .iter()
                .find(|(name, _)| *name == path)
                .map(|(_, body)| body);
            let head = if body.is_some() {
                "200 OK"
            } else {
                "404 Not Found"
            };
            let body = body.cloned().unwrap_or_default();
            let reply = format!(
                "HTTP/1.1 {head}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(reply.as_bytes()).unwrap();
            stream.write_all(&body).unwrap();
        }
    });
    (base, thread)
}

#[test]
fn a_newer_release_on_the_host_is_staged_and_a_current_one_is_not() {
    let pair = test_key();
    let key = public(&pair);
    let text = manifest(b"archive");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let files = vec![
        ("/manifest", text.clone().into_bytes()),
        ("/manifest.sig", signature.clone().into_bytes()),
        ("/photon-1.4.0.tar.gz", b"archive".to_vec()),
    ];
    let pending = folder("host").join("pending");
    let (base, thread) = host(files.clone(), 3);
    let staged = check(&base, &[key], "1.3.0", &pending).unwrap().unwrap();
    thread.join().unwrap();
    assert_eq!(staged.release.version, "1.4.0");
    assert_eq!(self::staged(&pending, &[key]).unwrap(), staged);
    // Up to date: only the manifest and its signature are asked for.
    let (base, thread) = host(files, 2);
    assert_eq!(
        check(&base, &[key], "1.4.0", &folder("host2").join("p")),
        Ok(None)
    );
    thread.join().unwrap();
}

#[test]
fn a_host_with_the_wrong_signature_or_plain_http_is_refused() {
    let pair = test_key();
    let key = public(&pair);
    let text = manifest(b"archive");
    let forged = to_hex(test_key().sign(text.as_bytes()).as_ref());
    let pending = folder("forged").join("pending");
    let (base, thread) = host(
        vec![
            ("/manifest", text.into_bytes()),
            ("/manifest.sig", forged.into_bytes()),
        ],
        2,
    );
    assert_eq!(
        check(&base, &[key], "1.3.0", &pending),
        Err(Error::Untrusted)
    );
    thread.join().unwrap();
    assert!(!pending.exists());
    assert!(check("http://example.com/releases", &[key], "1.3.0", &pending).is_err());
}

/// A gzip'd ustar archive of (path, bytes, mode) entries, made without a tar program.
fn archive_of(entries: &[(&str, &[u8], u32)]) -> Vec<u8> {
    use std::io::Write;
    let mut tar = Vec::new();
    for (name, body, mode) in entries {
        let mut header = [0u8; 512];
        header[..name.len()].copy_from_slice(name.as_bytes());
        header[100..108].copy_from_slice(format!("{mode:07o}\0").as_bytes());
        header[124..136].copy_from_slice(format!("{:011o}\0", body.len()).as_bytes());
        header[156] = b'0';
        tar.extend_from_slice(&header);
        tar.extend_from_slice(body);
        tar.resize(tar.len().div_ceil(512) * 512, 0);
    }
    tar.extend_from_slice(&[0u8; 1024]);
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&tar).unwrap();
    encoder.finish().unwrap()
}

/// Stages `archive` as 1.4.0 under a fresh folder with an install holding `old`.
fn staged_install(name: &str, archive: &[u8], key_pair: &Ed25519KeyPair) -> (PathBuf, PathBuf) {
    let root = folder(name);
    let install = root.join("app");
    std::fs::create_dir_all(&install).unwrap();
    std::fs::write(install.join("photon"), b"old").unwrap();
    let sum = to_hex(Sha256::digest(archive).as_slice());
    let text = format!(
        "version=1.4.0\narchive=p.tar.gz\nsha256={sum}\nsize={}\n",
        archive.len()
    );
    let signature = to_hex(key_pair.sign(text.as_bytes()).as_ref());
    let pending = root.join("pending");
    stage(
        &pending,
        &[public(key_pair)],
        "1.3.0",
        (text.as_bytes(), &signature),
        archive,
    )
    .unwrap();
    (install, pending)
}

use std::path::PathBuf;

#[test]
fn a_staged_release_replaces_the_install_and_keeps_the_old_one() {
    let pair = test_key();
    let archive = archive_of(&[
        ("photon-1.4.0/", b"", 0o755),
        ("photon-1.4.0/photon", b"new", 0o755),
        ("photon-1.4.0/README.md", b"read", 0o644),
    ]);
    let (install, pending) = staged_install("swap", &archive, &pair);
    let applied = apply(&install, &pending, &[public(&pair)], ("1.3.0", "photon"))
        .unwrap()
        .unwrap();
    assert_eq!(applied.version, "1.4.0");
    assert_eq!(applied.binary, install.join("photon"));
    assert_eq!(std::fs::read(install.join("photon")).unwrap(), b"new");
    assert_eq!(std::fs::read(install.join("README.md")).unwrap(), b"read");
    let old = install.with_file_name("app.old");
    assert_eq!(std::fs::read(old.join("photon")).unwrap(), b"old");
    assert!(!pending.exists());
    // Nothing pending any more: the next start does nothing.
    assert_eq!(
        apply(&install, &pending, &[public(&pair)], ("1.4.0", "photon")),
        Ok(None)
    );
}

#[test]
fn an_archive_that_escapes_or_lacks_the_binary_leaves_the_install_alone() {
    let pair = test_key();
    for entries in [
        vec![("photon-1.4.0/../../evil", &b"x"[..], 0o644)],
        vec![("photon-1.4.0/README.md", &b"x"[..], 0o644)],
    ] {
        let (install, pending) = staged_install("bad", &archive_of(&entries), &pair);
        assert!(apply(&install, &pending, &[public(&pair)], ("1.3.0", "photon")).is_err());
        assert_eq!(std::fs::read(install.join("photon")).unwrap(), b"old");
        assert!(pending.exists());
        assert!(!install.with_file_name("app.new").exists());
    }
}

#[test]
fn a_release_that_is_not_newer_any_more_is_dropped() {
    let pair = test_key();
    let archive = archive_of(&[("photon-1.4.0/photon", b"new", 0o755)]);
    let (install, pending) = staged_install("stale", &archive, &pair);
    assert_eq!(
        apply(&install, &pending, &[public(&pair)], ("1.4.0", "photon")),
        Ok(None)
    );
    assert!(!pending.exists());
    assert_eq!(std::fs::read(install.join("photon")).unwrap(), b"old");
}

#[test]
fn files_are_swapped_one_by_one_when_the_folder_cannot_be_renamed() {
    let root = folder("files");
    let (install, new) = (root.join("app"), root.join("app.new"));
    std::fs::create_dir_all(&install).unwrap();
    std::fs::create_dir_all(&new).unwrap();
    std::fs::write(install.join("photon"), b"old").unwrap();
    std::fs::write(new.join("photon"), b"new").unwrap();
    std::fs::write(new.join("LICENSE"), b"l").unwrap();
    swap::swap_files(&new, &install).unwrap();
    assert_eq!(std::fs::read(install.join("photon")).unwrap(), b"new");
    assert_eq!(std::fs::read(install.join("photon.old")).unwrap(), b"old");
    assert_eq!(std::fs::read(install.join("LICENSE")).unwrap(), b"l");
}

#[test]
fn a_shared_folder_is_swapped_file_by_file_and_its_other_files_stay() {
    let pair = test_key();
    let archive = archive_of(&[("photon-1.4.0/photon", b"new", 0o755)]);
    let (install, pending) = staged_install("shared", &archive, &pair);
    std::fs::write(install.join("holiday.jpg"), b"mine").unwrap();
    apply(&install, &pending, &[public(&pair)], ("1.3.0", "photon"))
        .unwrap()
        .unwrap();
    assert_eq!(std::fs::read(install.join("photon")).unwrap(), b"new");
    assert_eq!(std::fs::read(install.join("holiday.jpg")).unwrap(), b"mine");
    assert_eq!(std::fs::read(install.join("photon.old")).unwrap(), b"old");
    assert!(!install.with_file_name("app.old").exists());
}

#[test]
fn a_folder_the_updater_did_not_make_is_never_deleted() {
    let pair = test_key();
    let archive = archive_of(&[("photon-1.4.0/photon", b"new", 0o755)]);
    let (install, pending) = staged_install("foreign", &archive, &pair);
    let theirs = install.with_file_name("app.old");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::write(theirs.join("keep"), b"1").unwrap();
    assert!(apply(&install, &pending, &[public(&pair)], ("1.3.0", "photon")).is_err());
    assert_eq!(std::fs::read(theirs.join("keep")).unwrap(), b"1");
    assert_eq!(std::fs::read(install.join("photon")).unwrap(), b"old");
}
