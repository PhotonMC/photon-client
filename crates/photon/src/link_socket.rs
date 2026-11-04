//! One running client takes the links: a second `photon open-link` hands its link to the first and
//! exits instead of opening a second window. The socket lives in the user's runtime directory
//! (mode 0700); without one, or off Unix, every start is its own client. A link carries a launch
//! code, so nothing received here is logged.

#[cfg(unix)]
mod imp {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::fs::DirBuilderExt;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::time::Duration;

    /// The longest line taken; a real link is under a hundred bytes.
    const LONGEST_LINE: u64 = 512;
    const TALK_TIME: Duration = Duration::from_secs(2);

    /// What the first client said to a handed-over link.
    #[derive(Debug, PartialEq, Eq)]
    pub enum Sent {
        Delivered,
        Refused,
        /// Nobody is listening: this start is the client.
        NoPrimary,
    }

    fn socket_path() -> Option<PathBuf> {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from)?;
        runtime
            .is_absolute()
            .then(|| runtime.join("photon").join("link.sock"))
    }

    /// Takes one line from a connection and answers `ok` when it is a Photon link, `refused`
    /// otherwise. The link becomes the pending one, as if the client had been started with it.
    fn answer(stream: &mut (impl Read + Write)) {
        let mut line = Vec::new();
        let read =
            BufReader::new(Read::take(&mut *stream, LONGEST_LINE)).read_until(b'\n', &mut line);
        let accepted = read.is_ok()
            && line.last() == Some(&b'\n')
            && std::str::from_utf8(&line[..line.len() - 1])
                .is_ok_and(|text| crate::launch_link::remember(text.trim()).is_ok());
        let _ = stream.write_all(if accepted { b"ok\n" } else { b"refused\n" });
    }

    /// Hands `link` to the running client, when there is one.
    pub fn send(link: &str) -> Sent {
        let Some(path) = socket_path() else {
            return Sent::NoPrimary;
        };
        let Ok(mut stream) = UnixStream::connect(&path) else {
            return Sent::NoPrimary;
        };
        let _ = stream.set_read_timeout(Some(TALK_TIME));
        let _ = stream.set_write_timeout(Some(TALK_TIME));
        if stream.write_all(format!("{link}\n").as_bytes()).is_err() {
            return Sent::Refused;
        }
        let mut reply = String::new();
        let _ = BufReader::new(stream).read_line(&mut reply);
        if reply == "ok\n" {
            Sent::Delivered
        } else {
            Sent::Refused
        }
    }

    /// The socket of the running client; it is removed when the client ends normally.
    pub struct Guard(PathBuf);

    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    fn bind(path: &PathBuf) -> Option<UnixListener> {
        let folder = path.parent()?;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(folder)
            .ok()?;
        match UnixListener::bind(path) {
            Ok(listener) => Some(listener),
            // A file is there: another client listens, or a killed one left it behind. Two starts
            // at the same instant can still both miss each other; the later one then opens a
            // second window, which is all that costs.
            Err(_) if UnixStream::connect(path).is_ok() => None,
            Err(_) => {
                std::fs::remove_file(path).ok()?;
                UnixListener::bind(path).ok()
            }
        }
    }

    /// Starts taking links for this client. None when another client does, or no socket can be made.
    pub fn serve() -> Option<Guard> {
        let path = socket_path()?;
        let listener = bind(&path)?;
        let spawned = std::thread::Builder::new()
            .name("Link Socket".to_owned())
            .spawn(move || {
                for mut stream in listener.incoming().flatten() {
                    let _ = stream.set_read_timeout(Some(TALK_TIME));
                    let _ = stream.set_write_timeout(Some(TALK_TIME));
                    answer(&mut stream);
                }
            });
        spawned.ok().map(|_| Guard(path))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn asked(line: &[u8]) -> String {
            let (mut near, mut far) = UnixStream::pair().expect("pair");
            far.write_all(line).expect("write");
            far.shutdown(std::net::Shutdown::Write).expect("shutdown");
            answer(&mut near);
            let mut reply = String::new();
            BufReader::new(far.try_clone().expect("clone"))
                .read_line(&mut reply)
                .expect("reply");
            reply
        }

        #[test]
        fn a_launch_link_is_answered_ok_and_kept() {
            let zeros = "0".repeat(32);
            assert_eq!(
                asked(format!("photon://launch?code={zeros}\n").as_bytes()),
                "ok\n"
            );
            assert!(crate::launch_link::take_pending().is_some());
        }

        #[test]
        fn a_token_link_and_a_long_line_are_refused() {
            let zeros = "0".repeat(32);
            assert_eq!(
                asked(format!("photon://launch?token={zeros}\n").as_bytes()),
                "refused\n"
            );
            assert_eq!(asked(&vec![b'a'; 2000]), "refused\n");
            assert_eq!(asked(b"photon://launch"), "refused\n");
        }

        #[test]
        fn a_slow_writer_cannot_hold_the_reader_past_its_timeout() {
            let (mut near, far) = UnixStream::pair().expect("pair");
            near.set_read_timeout(Some(Duration::from_millis(100)))
                .expect("timeout");
            let start = std::time::Instant::now();
            answer(&mut near);
            assert!(start.elapsed() < Duration::from_secs(2));
            drop(far);
        }

        #[test]
        fn a_stale_socket_file_is_replaced() {
            let folder = std::env::temp_dir().join(format!("photon-sock-{}", std::process::id()));
            std::fs::create_dir_all(&folder).expect("folder");
            let path = folder.join("link.sock");
            drop(UnixListener::bind(&path).expect("first"));
            assert!(path.exists());
            assert!(bind(&path).is_some());
            let _ = std::fs::remove_dir_all(&folder);
        }
    }
}

#[cfg(not(unix))]
mod imp {
    #[derive(Debug, PartialEq, Eq)]
    pub enum Sent {
        Delivered,
        Refused,
        NoPrimary,
    }

    pub struct Guard;

    pub fn send(_link: &str) -> Sent {
        Sent::NoPrimary
    }

    pub fn serve() -> Option<Guard> {
        None
    }
}

pub use imp::{Sent, send, serve};
