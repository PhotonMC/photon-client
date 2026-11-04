//! Playing a clip: fetch, demux and decode on a thread, show each picture at its time, loop.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::VideoError;
use crate::decode::{Decoder, Picture};
use crate::demux::{Packet, demux};
use crate::fetch::fetch;

/// A picture to show.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    /// Counts up with each new picture, so a caller can tell it from the last one it took.
    pub serial: u64,
}

const LOADING: u8 = 0;
const PLAYING: u8 = 1;
const FAILED: u8 = 2;
/// A picture later than this behind its time is dropped, not shown.
const LATE: Duration = Duration::from_millis(120);
const THREADS: i32 = 2;

struct Shared {
    stop: AtomicBool,
    state: AtomicU8,
    newest: Mutex<Option<Frame>>,
}

/// A clip playing on its own thread until this is dropped.
pub struct Player {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl Player {
    /// Starts fetching and playing `url`.
    pub fn start(url: String) -> Self {
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            state: AtomicU8::new(LOADING),
            newest: Mutex::new(None),
        });
        let worker = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("photon-video".to_owned())
            .spawn(move || {
                let result = fetch(&url).and_then(|file| play(&worker, &file));
                if let Err(error) = result {
                    log::warn!("video {url}: {error:?}");
                    worker.state.store(FAILED, Ordering::Relaxed);
                }
            })
            .ok();
        Self { shared, thread }
    }

    /// Whether the clip failed to load or play.
    pub fn failed(&self) -> bool {
        self.shared.state.load(Ordering::Relaxed) == FAILED
    }

    /// Whether a picture has been shown at least once.
    pub fn playing(&self) -> bool {
        self.shared.state.load(Ordering::Relaxed) == PLAYING
    }

    /// The newest picture when it is newer than `after`.
    pub fn take_frame(&self, after: u64) -> Option<Frame> {
        let mut newest = self.shared.newest.lock().ok()?;
        match newest.as_ref() {
            Some(frame) if frame.serial > after => newest.take(),
            _ => None,
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Plays `file` over and over until stopped.
fn play(shared: &Shared, file: &[u8]) -> Result<(), VideoError> {
    let packets = demux(file)?;
    if packets.is_empty() {
        return Err(VideoError::Container("no frames"));
    }
    let mut serial = 0u64;
    while !shared.stop.load(Ordering::Relaxed) {
        let start = Instant::now();
        let mut decoder = Decoder::new(THREADS)?;
        for (index, packet) in packets.iter().enumerate() {
            let mut pictures = decoder.decode(&packet.data)?;
            if index + 1 == packets.len() {
                pictures.extend(decoder.finish()?);
            }
            for (at, picture) in pictures.into_iter().enumerate() {
                let due = pts_of(&packets, index, at);
                if !show(shared, start, due, picture, &mut serial) {
                    return Ok(());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(frame_ms(&packets)));
    }
    Ok(())
}

/// The time of the `at`-th picture a packet produced: a packet normally makes one, in order.
fn pts_of(packets: &[Packet], index: usize, at: usize) -> u64 {
    packets
        .get(index + at)
        .or(packets.last())
        .map_or(0, |packet| packet.pts_ms)
}

/// How long one frame lasts, from the gap between the first two.
fn frame_ms(packets: &[Packet]) -> u64 {
    match packets {
        [first, second, ..] => second.pts_ms.saturating_sub(first.pts_ms).max(1),
        _ => 33,
    }
}

/// Waits until `due`, then makes `picture` the newest; false when asked to stop.
fn show(shared: &Shared, start: Instant, due: u64, picture: Picture, serial: &mut u64) -> bool {
    let time = start + Duration::from_millis(due);
    loop {
        if shared.stop.load(Ordering::Relaxed) {
            return false;
        }
        let now = Instant::now();
        if now >= time {
            if now - time > LATE {
                return true;
            }
            break;
        }
        std::thread::sleep((time - now).min(Duration::from_millis(10)));
    }
    *serial += 1;
    if let Ok(mut newest) = shared.newest.lock() {
        *newest = Some(Frame {
            width: picture.width,
            height: picture.height,
            rgba: picture.rgba,
            serial: *serial,
        });
    }
    shared.state.store(PLAYING, Ordering::Relaxed);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &[u8] = include_bytes!("../tests/data/tiny.webm");

    #[test]
    fn a_clip_plays_in_order_and_stops_when_dropped() {
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            state: AtomicU8::new(LOADING),
            newest: Mutex::new(None),
        });
        let worker = Arc::clone(&shared);
        let thread = std::thread::spawn(move || play(&worker, TINY));
        let started = Instant::now();
        let mut seen = 0u64;
        let mut last = 0u64;
        while seen < 5 && started.elapsed() < Duration::from_secs(10) {
            let taken = shared.newest.lock().unwrap().take();
            if let Some(frame) = taken {
                assert!(frame.serial > last);
                assert_eq!((frame.width, frame.height), (64, 48));
                last = frame.serial;
                seen += 1;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        shared.stop.store(true, Ordering::Relaxed);
        assert!(thread.join().unwrap().is_ok());
        assert!(seen >= 5, "saw {seen} pictures");
        assert_eq!(shared.state.load(Ordering::Relaxed), PLAYING);
    }

    #[test]
    fn frame_length_comes_from_the_first_two_packets() {
        let packet = |pts_ms| Packet {
            data: Vec::new(),
            pts_ms,
        };
        assert_eq!(frame_ms(&[packet(0), packet(100)]), 100);
        assert_eq!(frame_ms(&[packet(0)]), 33);
    }
}
