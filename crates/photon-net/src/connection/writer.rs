//! The play session's socket writes, on a thread of their own: a packet is framed where it is
//! sent and written in the order it was queued, so the game's thread never waits on the network.

use std::io::{self, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use photon_protocol::play::ConnectionError;

use super::transport::Transport;

enum Job {
    Packet(Vec<u8>),
    /// Answered once every packet before it is written.
    Barrier(SyncSender<()>),
}

/// Why the writer stopped writing: the first failure, kept so every later send reports it.
type Failure = Arc<Mutex<Option<(io::ErrorKind, String)>>>;

pub(super) struct PacketWriter {
    jobs: Option<Sender<Job>>,
    failure: Failure,
    stream: Transport,
    thread: Option<JoinHandle<()>>,
    /// `Connection.sentPackets`: the packets queued since the count was last taken.
    sent: AtomicU32,
}

impl PacketWriter {
    pub(super) fn start(stream: Transport) -> io::Result<Self> {
        let (jobs, queue) = mpsc::channel();
        let failure = Failure::default();
        let mut sink = stream.try_clone()?;
        let failed = Arc::clone(&failure);
        let thread = std::thread::Builder::new()
            .name("Packet Writer".to_owned())
            .spawn(move || write_until_closed(&queue, &mut sink, &failed))?;
        Ok(Self {
            jobs: Some(jobs),
            failure,
            stream,
            thread: Some(thread),
            sent: AtomicU32::new(0),
        })
    }

    /// Queue a framed packet. A write that failed before is reported here.
    pub(super) fn send(&self, framed: Vec<u8>) -> Result<(), ConnectionError> {
        self.check()?;
        self.sent.fetch_add(1, Ordering::Relaxed);
        self.queue(Job::Packet(framed))
    }

    /// The packets sent since this was last asked, which starts the count again.
    pub(super) fn take_sent(&self) -> u32 {
        self.sent.swap(0, Ordering::Relaxed)
    }

    /// Wait until everything queued is written, for what must reach the server before the
    /// connection is used directly.
    pub(super) fn flush(&self) -> Result<(), ConnectionError> {
        let (answer, written) = mpsc::sync_channel(1);
        self.queue(Job::Barrier(answer))?;
        written
            .recv()
            .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))?;
        self.check()
    }

    /// The connection itself, for a phase that is run on it directly.
    pub(super) fn stream(&self) -> io::Result<Transport> {
        self.stream.try_clone()
    }

    fn queue(&self, job: Job) -> Result<(), ConnectionError> {
        let jobs = self
            .jobs
            .as_ref()
            .ok_or(ConnectionError::ServerDisconnected)?;
        jobs.send(job)
            .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe).into())
    }

    fn check(&self) -> Result<(), ConnectionError> {
        let failure = self
            .failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match failure.as_ref() {
            Some((kind, message)) => Err(io::Error::new(*kind, message.clone()).into()),
            None => Ok(()),
        }
    }
}

impl Drop for PacketWriter {
    /// What was queued is written before the connection goes.
    fn drop(&mut self) {
        self.jobs = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn write_until_closed(queue: &mpsc::Receiver<Job>, sink: &mut Transport, failure: &Failure) {
    while let Ok(job) = queue.recv() {
        match job {
            Job::Packet(framed) => {
                let mut failure = failure
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if failure.is_none()
                    && let Err(error) = sink.write_all(&framed).and_then(|()| sink.flush())
                {
                    *failure = Some((error.kind(), error.to_string()));
                }
            }
            Job::Barrier(answer) => {
                let _ = answer.send(());
            }
        }
    }
}
