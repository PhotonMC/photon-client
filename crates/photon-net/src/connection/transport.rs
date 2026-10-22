//! The connection's socket and, once the server asked for it, the AES/CFB8 layer over it.
//!
//! Clones share the cipher state of each direction, so the reader thread, the writer thread and
//! the direct phases all see one stream. The reading side must be one thread at a time and must
//! not be buffered: the cipher runs on the bytes as they come off the socket.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use photon_protocol::crypto::{SECRET_LENGTH, StreamCipher};

type Slot = Arc<Mutex<Option<StreamCipher>>>;

pub struct Transport {
    tcp: TcpStream,
    decrypt: Slot,
    encrypt: Slot,
}

fn locked(slot: &Slot) -> MutexGuard<'_, Option<StreamCipher>> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Transport {
    pub fn try_clone(&self) -> io::Result<Self> {
        Ok(Self {
            tcp: self.tcp.try_clone()?,
            decrypt: Arc::clone(&self.decrypt),
            encrypt: Arc::clone(&self.encrypt),
        })
    }

    /// `Connection.setEncryptionKey`: every byte after this one is encrypted going out and
    /// decrypted coming in, for this stream and its clones.
    pub fn enable_encryption(&self, secret: &[u8; SECRET_LENGTH]) {
        *locked(&self.encrypt) = Some(StreamCipher::encryptor(secret));
        *locked(&self.decrypt) = Some(StreamCipher::decryptor(secret));
    }

    /// Ends the connection for every clone, so a read waiting on it fails at once.
    pub fn shutdown(&self) {
        let _ = self.tcp.shutdown(std::net::Shutdown::Both);
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.tcp.set_read_timeout(timeout)
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.tcp.set_write_timeout(timeout)
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        self.tcp.set_nodelay(nodelay)
    }
}

impl From<TcpStream> for Transport {
    fn from(tcp: TcpStream) -> Self {
        Self {
            tcp,
            decrypt: Slot::default(),
            encrypt: Slot::default(),
        }
    }
}

impl Read for Transport {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.tcp.read(buffer)?;
        if let Some(cipher) = locked(&self.decrypt).as_mut() {
            cipher.apply(&mut buffer[..count]);
        }
        Ok(count)
    }
}

impl Write for Transport {
    /// All of `buffer` is written or the connection is lost: a partly written block would leave
    /// the cipher ahead of the bytes the server has. The cipher and the socket are held together
    /// so that two writers cannot swap the order of their bytes.
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let mut cipher = locked(&self.encrypt);
        match cipher.as_mut() {
            Some(cipher) => {
                let mut sealed = buffer.to_vec();
                cipher.apply(&mut sealed);
                self.tcp.write_all(&sealed)?;
            }
            None => self.tcp.write_all(buffer)?,
        }
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tcp.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn both_directions_use_the_cipher_after_it_is_enabled() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client =
            Transport::from(TcpStream::connect(listener.local_addr().unwrap()).unwrap());
        let mut server = listener.accept().unwrap().0;
        let secret = [7u8; SECRET_LENGTH];

        client.write_all(b"plain").unwrap();
        let mut seen = [0u8; 5];
        server.read_exact(&mut seen).unwrap();
        assert_eq!(&seen, b"plain");

        client.enable_encryption(&secret);
        let mut copy = client.try_clone().unwrap();
        copy.write_all(b"secret").unwrap();
        client.write_all(b"!").unwrap();
        let mut seen = [0u8; 7];
        server.read_exact(&mut seen).unwrap();
        assert_ne!(&seen, b"secret!");
        StreamCipher::decryptor(&secret).apply(&mut seen);
        assert_eq!(&seen, b"secret!");

        let mut reply = *b"reply";
        StreamCipher::encryptor(&secret).apply(&mut reply);
        server.write_all(&reply).unwrap();
        let mut heard = [0u8; 5];
        client.read_exact(&mut heard).unwrap();
        assert_eq!(&heard, b"reply");
    }
}
