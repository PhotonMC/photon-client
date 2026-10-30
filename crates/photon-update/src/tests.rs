use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};

use super::*;

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
    format!("version=1.4.0\narchive=photon-1.4.0.tar.gz\nsha256={sum}\n")
}

#[test]
fn a_manifest_signed_with_the_key_is_accepted() {
    let pair = test_key();
    let text = manifest(b"archive");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let release = verify_manifest(Some(&public(&pair)), text.as_bytes(), &signature).unwrap();
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
        verify_manifest(Some(&public(&other)), text.as_bytes(), &signature),
        Err(Error::Untrusted)
    );
    let changed = text.replace("1.4.0", "9.9.9");
    assert_eq!(
        verify_manifest(Some(&public(&pair)), changed.as_bytes(), &signature),
        Err(Error::Untrusted)
    );
}

#[test]
fn an_unset_key_and_a_malformed_signature_refuse() {
    let text = manifest(b"archive");
    assert_eq!(
        verify_manifest(None, text.as_bytes(), "00"),
        Err(Error::NoKey)
    );
    let pair = test_key();
    assert_eq!(
        verify_manifest(Some(&public(&pair)), text.as_bytes(), "zz"),
        Err(Error::BadSignature)
    );
    assert_eq!(RELEASE_KEY, None);
}

#[test]
fn a_signed_but_malformed_manifest_is_refused() {
    let pair = test_key();
    for (text, why) in [
        (
            "version=1\nversion=2\narchive=a\nsha256=00\n",
            "repeated field",
        ),
        ("version=1\narchive=a\n", "no sha256"),
        (
            "version=1\narchive=a\nsha256=12\n",
            "sha256 is not 32 bytes of hex",
        ),
        (
            "version=1\narchive=a\nsha256=00\nextra=1\n",
            "unknown field",
        ),
    ] {
        let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
        assert_eq!(
            verify_manifest(Some(&public(&pair)), text.as_bytes(), &signature),
            Err(Error::BadManifest(why)),
            "{text}"
        );
    }
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
        Some(&key),
        "1.3.0",
        (text.as_bytes(), &signature),
        b"archive",
    )
    .unwrap();
    assert_eq!(staged.release.version, "1.4.0");
    assert_eq!(self::staged(&pending, Some(&key)).unwrap(), staged);
    // Someone replaces the stored archive: it is refused.
    std::fs::write(&staged.archive, b"tampered").unwrap();
    assert_eq!(self::staged(&pending, Some(&key)), Err(Error::WrongArchive));
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
            Some(&key),
            "1.4.0",
            manifest_and_signature,
            b"archive"
        ),
        Err(Error::NotNewer)
    );
    assert_eq!(
        stage(
            &pending,
            Some(&key),
            "1.3.0",
            manifest_and_signature,
            b"other"
        ),
        Err(Error::WrongArchive)
    );
    assert!(!pending.exists());
}

#[test]
fn an_archive_name_with_a_path_is_refused() {
    let pair = test_key();
    let key = public(&pair);
    let sum = to_hex(Sha256::digest(b"a").as_slice());
    let text = format!("version=2.0.0\narchive=../escape.tar.gz\nsha256={sum}\n");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let pending = folder("name").join("pending");
    assert_eq!(
        stage(
            &pending,
            Some(&key),
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
    let staged = check(&base, Some(&key), "1.3.0", &pending)
        .unwrap()
        .unwrap();
    thread.join().unwrap();
    assert_eq!(staged.release.version, "1.4.0");
    assert_eq!(self::staged(&pending, Some(&key)).unwrap(), staged);
    // Up to date: only the manifest and its signature are asked for.
    let (base, thread) = host(files, 2);
    assert_eq!(
        check(&base, Some(&key), "1.4.0", &folder("host2").join("p")),
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
        check(&base, Some(&key), "1.3.0", &pending),
        Err(Error::Untrusted)
    );
    thread.join().unwrap();
    assert!(!pending.exists());
    assert!(check("http://example.com/releases", Some(&key), "1.3.0", &pending).is_err());
}
