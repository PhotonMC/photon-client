//! A server on this machine that answers from a table of paths, and notes what was asked.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

pub(super) type Routes = Arc<Mutex<HashMap<String, Vec<u8>>>>;

pub(super) struct Stand {
    pub(super) base: String,
    pub(super) routes: Routes,
    pub(super) hits: Arc<Mutex<Vec<String>>>,
}

impl Stand {
    pub(super) fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let routes: Routes = Arc::default();
        let hits: Arc<Mutex<Vec<String>>> = Arc::default();
        let (table, log) = (Arc::clone(&routes), Arc::clone(&hits));
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (table, log) = (Arc::clone(&table), Arc::clone(&log));
                std::thread::spawn(move || answer(stream, &table, &log));
            }
        });
        Self { base, routes, hits }
    }

    pub(super) fn serve(&self, path: &str, body: Vec<u8>) {
        self.routes.lock().unwrap().insert(path.to_owned(), body);
    }

    /// The paths asked for so far.
    pub(super) fn asked(&self) -> Vec<String> {
        self.hits.lock().unwrap().clone()
    }
}

fn answer(stream: std::net::TcpStream, table: &Routes, log: &Arc<Mutex<Vec<String>>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let path = line.split(' ').nth(1).unwrap_or("").to_owned();
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).unwrap_or(0) <= 2 {
            break;
        }
    }
    log.lock().unwrap().push(path.clone());
    let body = table.lock().unwrap().get(&path).cloned();
    let mut out = stream;
    let _ = match body {
        Some(body) => {
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            out.write_all(head.as_bytes())
                .and_then(|()| out.write_all(&body))
        }
        None => out
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),
    };
}
