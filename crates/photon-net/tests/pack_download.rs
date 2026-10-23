//! Fetching server resource packs from a local web server.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use photon_core::binary::Uuid;
use photon_net::server_packs::{DownloadRequest, HttpFetcher, PackFetcher, PackUser};
use sha1::{Digest, Sha1};

/// What each request carried: its path and headers.
type Seen = Mutex<Vec<(String, Vec<(String, String)>)>>;

const BODY: &[u8] = b"not really a zip, but bytes all the same";
const ID: Uuid = Uuid::new(0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210);

fn sha1_hex(bytes: &[u8]) -> String {
    Sha1::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// A web server on a port of its own: what each request carried, by path.
struct Web {
    address: String,
    requests: Arc<Seen>,
}

impl Web {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = format!("http://{}", listener.local_addr().expect("address"));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&requests);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                answer(stream, &seen);
            }
        });
        Self { address, requests }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.address)
    }

    fn count(&self) -> usize {
        self.requests.lock().expect("requests").len()
    }

    fn header(&self, index: usize, name: &str) -> Option<String> {
        let requests = self.requests.lock().expect("requests");
        requests[index]
            .1
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.clone())
    }
}

fn answer(stream: TcpStream, seen: &Seen) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut first = String::new();
    reader.read_line(&mut first).expect("request line");
    let path = first.split_whitespace().nth(1).unwrap_or("/").to_owned();
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("header");
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.push((key.to_owned(), value.trim().to_owned()));
        }
    }
    seen.lock().expect("requests").push((path.clone(), headers));
    let mut stream = stream;
    let response = match path.as_str() {
        "/pack.zip" => {
            let mut out = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                BODY.len()
            )
            .into_bytes();
            out.extend_from_slice(BODY);
            out
        }
        "/moved" => b"HTTP/1.1 302 Found\r\nLocation: /pack.zip\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
        "/huge" => b"HTTP/1.1 200 OK\r\nContent-Length: 262144001\r\nConnection: close\r\n\r\n".to_vec(),
        _ => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
    };
    let _ = stream.write_all(&response);
    let _ = stream.flush();
    let mut rest = Vec::new();
    let _ = reader.read_to_end(&mut rest);
}

fn cache() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "photon-pack-test-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn user() -> PackUser {
    PackUser {
        name: "Tester".to_owned(),
        id: Uuid::new(0, 1),
    }
}

fn request(url: String, hash: Option<String>) -> DownloadRequest {
    DownloadRequest {
        id: ID,
        ticket: 1,
        url,
        hash,
    }
}

#[test]
fn a_pack_is_saved_under_its_hash_and_the_server_is_told_who_asks() {
    let web = Web::start();
    let cache = cache();
    let fetcher = HttpFetcher::new(cache.clone()).expect("fetcher");
    let hash = sha1_hex(BODY);
    let path = fetcher
        .fetch(&request(web.url("/pack.zip"), Some(hash.clone())), &user())
        .expect("downloaded");
    assert_eq!(
        path,
        cache
            .join("01234567-89ab-cdef-fedc-ba9876543210")
            .join(&hash)
    );
    assert_eq!(std::fs::read(&path).expect("file"), BODY);
    assert_eq!(
        web.header(0, "X-Minecraft-Username").as_deref(),
        Some("Tester")
    );
    assert_eq!(
        web.header(0, "X-Minecraft-UUID").as_deref(),
        Some("00000000000000000000000000000001")
    );
    assert_eq!(
        web.header(0, "X-Minecraft-Version").as_deref(),
        Some("26.2")
    );
    assert_eq!(
        web.header(0, "X-Minecraft-Version-ID").as_deref(),
        Some("26.2")
    );
    assert_eq!(
        web.header(0, "X-Minecraft-Pack-Format").as_deref(),
        Some("88.0")
    );
    assert_eq!(
        web.header(0, "User-Agent").as_deref(),
        Some("Minecraft Java/26.2")
    );
}

#[test]
fn a_pack_without_a_hash_is_named_for_the_hash_it_has() {
    let web = Web::start();
    let fetcher = HttpFetcher::new(cache()).expect("fetcher");
    let path = fetcher
        .fetch(&request(web.url("/pack.zip"), None), &user())
        .expect("downloaded");
    assert_eq!(
        path.file_name().expect("name").to_string_lossy(),
        sha1_hex(BODY)
    );
    assert_eq!(std::fs::read(path).expect("file"), BODY);
}

#[test]
fn a_pack_with_the_wrong_hash_is_refused_and_not_kept() {
    let web = Web::start();
    let cache = cache();
    let fetcher = HttpFetcher::new(cache.clone()).expect("fetcher");
    let wrong = "0".repeat(40);
    let result = fetcher.fetch(&request(web.url("/pack.zip"), Some(wrong.clone())), &user());
    assert!(result.expect_err("mismatch").contains("did not match"));
    let folder = cache.join(ID.to_string_java());
    assert!(!folder.join(wrong).exists());
}

#[test]
fn a_pack_kept_from_before_is_not_fetched_again() {
    let web = Web::start();
    let fetcher = HttpFetcher::new(cache()).expect("fetcher");
    let asked = request(web.url("/pack.zip"), Some(sha1_hex(BODY)));
    fetcher.fetch(&asked, &user()).expect("first");
    fetcher.fetch(&asked, &user()).expect("second");
    assert_eq!(web.count(), 1);
}

#[test]
fn a_redirect_is_followed() {
    let web = Web::start();
    let fetcher = HttpFetcher::new(cache()).expect("fetcher");
    let path = fetcher
        .fetch(&request(web.url("/moved"), Some(sha1_hex(BODY))), &user())
        .expect("downloaded");
    assert_eq!(std::fs::read(path).expect("file"), BODY);
}

#[test]
fn a_pack_that_is_too_big_or_missing_fails() {
    let web = Web::start();
    let fetcher = HttpFetcher::new(cache()).expect("fetcher");
    let huge = fetcher.fetch(&request(web.url("/huge"), None), &user());
    assert!(huge.expect_err("too big").contains("bigger than maximum"));
    let missing = fetcher.fetch(&request(web.url("/missing"), None), &user());
    assert!(missing.is_err());
}
