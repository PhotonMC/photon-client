//! The release host: files served by a loopback server, redirects, sizes and refusals.

use super::tests::{folder, manifest, public, test_key, to_hex};
use super::*;

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
            // A body written as `->/path` is a redirect to that path, like the release host's.
            let reply = if let Some(target) = body.strip_prefix(b"->") {
                let target = String::from_utf8_lossy(target);
                format!(
                    "HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
            } else {
                format!(
                    "HTTP/1.1 {head}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
            };
            let body = if body.starts_with(b"->") {
                Vec::new()
            } else {
                body
            };
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
        ("/photon-t.manifest", text.clone().into_bytes()),
        ("/photon-t.manifest.sig", signature.clone().into_bytes()),
        ("/photon-1.4.0.tar.gz", b"archive".to_vec()),
    ];
    let pending = folder("host").join("pending");
    let (base, thread) = host(files.clone(), 3);
    let staged = check(&base, &[key], "t", ("1.3.0", &pending))
        .unwrap()
        .unwrap();
    thread.join().unwrap();
    assert_eq!(staged.release.version, "1.4.0");
    assert_eq!(self::staged(&pending, &[key]).unwrap(), staged);
    // Up to date: only the manifest and its signature are asked for.
    let (base, thread) = host(files, 2);
    assert_eq!(
        check(&base, &[key], "t", ("1.4.0", &folder("host2").join("p"))),
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
            ("/photon-t.manifest", text.into_bytes()),
            ("/photon-t.manifest.sig", forged.into_bytes()),
        ],
        2,
    );
    assert_eq!(
        check(&base, &[key], "t", ("1.3.0", &pending)),
        Err(Error::Untrusted)
    );
    thread.join().unwrap();
    assert!(!pending.exists());
    assert!(
        check(
            "http://example.com/releases",
            &[key],
            "t",
            ("1.3.0", &pending)
        )
        .is_err()
    );
}

#[test]
fn redirects_are_followed_and_progress_and_missing_files_are_reported() {
    let pair = test_key();
    let key = public(&pair);
    let text = manifest(b"archive");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let files = vec![
        ("/photon-t.manifest", b"->/assets/1".to_vec()),
        ("/assets/1", text.clone().into_bytes()),
        ("/photon-t.manifest.sig", signature.into_bytes()),
        ("/photon-1.4.0.tar.gz", b"->/assets/2".to_vec()),
        ("/assets/2", b"archive".to_vec()),
    ];
    let pending = folder("redirect").join("pending");
    let (base, thread) = host(files, 5);
    let offered = offer(&base, &[key], "t", "1.3.0").unwrap().unwrap();
    assert_eq!(
        (offered.release.version.as_str(), offered.release.size),
        ("1.4.0", 7)
    );
    let mut seen = Vec::new();
    let staged = download(
        &base,
        &[key],
        &offered,
        ("1.3.0", &pending),
        &mut |done, total| {
            seen.push((done, total));
        },
    )
    .unwrap();
    thread.join().unwrap();
    assert_eq!(seen.last(), Some(&(7, 7)));
    assert_eq!(self::staged(&pending, &[key]).unwrap(), staged);
    // A host without the platform's files has no release for it.
    let (base, thread) = host(vec![], 1);
    assert_eq!(offer(&base, &[key], "t", "1.3.0"), Err(Error::NotFound));
    thread.join().unwrap();
}

#[test]
fn an_archive_longer_than_the_signed_size_is_refused() {
    let pair = test_key();
    let key = public(&pair);
    let text = manifest(b"archive");
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    let files = vec![
        ("/photon-t.manifest", text.into_bytes()),
        ("/photon-t.manifest.sig", signature.into_bytes()),
        ("/photon-1.4.0.tar.gz", b"archive plus more".to_vec()),
    ];
    let pending = folder("long").join("pending");
    let (base, thread) = host(files, 3);
    assert_eq!(
        check(&base, &[key], "t", ("1.3.0", &pending)),
        Err(Error::WrongArchive)
    );
    thread.join().unwrap();
    assert!(!pending.exists());
}
