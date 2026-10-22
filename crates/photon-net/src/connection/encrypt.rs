//! The login key exchange (`ClientHandshakePacketListenerImpl.handleHello`).

use photon_protocol::crypto;
use photon_protocol::login::{ClientboundHelloPacket, ServerboundKeyPacket};
use photon_protocol::phase::ConnectionProtocol;
use photon_protocol::play::{ConnectionError, LOGIN_KEY_PACKET_ID, LoginFailure};

use std::time::Duration;

use super::join::join_server;
use super::session::send_packet;
use super::transport::Transport;
use crate::account::Account;

/// Answer the server's key request: send a fresh secret and the challenge under the server's
/// public key, then wrap the connection in the stream cipher (the key packet itself goes out in
/// plain, as the server decrypts it with its private key).
///
/// A server that wants the client's session proof (`should_authenticate`) first gets the join
/// posted to the session server; without an account the client refuses before anything is sent.
pub(super) fn answer_hello(
    stream: &mut Transport,
    hello: &ClientboundHelloPacket,
    (account, timeout): (Option<&Account>, Duration),
    compression_threshold: Option<usize>,
) -> Result<(), ConnectionError> {
    let failed = |_| ConnectionError::Codec("login key exchange");
    let secret = crypto::generate_secret().map_err(failed)?;
    let key = ServerboundKeyPacket {
        encrypted_secret: crypto::encrypt_with_public_key(&hello.public_key, &secret)
            .map_err(failed)?,
        encrypted_challenge: crypto::encrypt_with_public_key(&hello.public_key, &hello.challenge)
            .map_err(failed)?,
    };
    if hello.should_authenticate {
        let account = account.ok_or(ConnectionError::LoginFailed(LoginFailure::AccountRequired))?;
        let hash = crypto::server_hash(&hello.server_id, &secret, &hello.public_key);
        join_server(account, &hash, timeout)?;
    }
    send_packet(
        stream,
        ConnectionProtocol::Login,
        LOGIN_KEY_PACKET_ID,
        |output| {
            key.encode(output)
                .map_err(|_| ConnectionError::Codec("login key"))
        },
        compression_threshold,
    )?;
    stream.enable_encryption(&secret);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    use photon_protocol::crypto::SystemRandom;
    use rsa::pkcs8::EncodePublicKey;
    use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

    use super::*;
    use crate::connection::session::read_packet;

    /// A server that wants encryption but no proof of account: it reads the key packet in
    /// plain, takes the secret and the challenge out with its private key, and from then on
    /// talks in the cipher.
    #[test]
    fn the_key_exchange_turns_the_stream_into_the_cipher() {
        let private = RsaPrivateKey::new(&mut SystemRandom, 1024).expect("key");
        let public_key = private
            .to_public_key()
            .to_public_key_der()
            .unwrap()
            .to_vec();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client =
            Transport::from(TcpStream::connect(listener.local_addr().unwrap()).unwrap());
        let mut server = Transport::from(listener.accept().unwrap().0);
        let hello = ClientboundHelloPacket::new(vec![], public_key, vec![1, 2, 3, 4], false);

        let far_end = std::thread::spawn(move || {
            let body = read_packet(&mut server, None).expect("key packet");
            let (id, payload) = photon_protocol::play::split_packet(&body).unwrap();
            assert_eq!(id, LOGIN_KEY_PACKET_ID);
            let key = ServerboundKeyPacket::decode(&mut &payload[..]).unwrap();
            let secret = private
                .decrypt(Pkcs1v15Encrypt, &key.encrypted_secret)
                .unwrap();
            let challenge = private
                .decrypt(Pkcs1v15Encrypt, &key.encrypted_challenge)
                .unwrap();
            assert_eq!(challenge, [1, 2, 3, 4]);
            let secret: [u8; crypto::SECRET_LENGTH] = secret.try_into().unwrap();
            server.enable_encryption(&secret);
            server.write_all(&[2, 0x7f, 0x01]).unwrap();
            let mut answer = [0u8; 3];
            server.read_exact(&mut answer).unwrap();
            assert_eq!(answer, [2, 0x55, 0x66]);
        });

        answer_hello(&mut client, &hello, (None, Duration::from_secs(5)), None).expect("answer");
        let body = read_packet(&mut client, None).expect("encrypted packet");
        assert_eq!(body, [0x7f, 0x01]);
        client.write_all(&[2, 0x55, 0x66]).unwrap();
        far_end.join().unwrap();
    }

    #[test]
    fn a_server_that_checks_accounts_is_refused_before_anything_is_sent() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client =
            Transport::from(TcpStream::connect(listener.local_addr().unwrap()).unwrap());
        let (mut server, _) = listener.accept().unwrap();
        let private = RsaPrivateKey::new(&mut SystemRandom, 1024).expect("key");
        let public_key = private
            .to_public_key()
            .to_public_key_der()
            .unwrap()
            .to_vec();
        let hello = ClientboundHelloPacket::new(vec![], public_key, vec![2], true);
        let result = answer_hello(&mut client, &hello, (None, Duration::from_secs(5)), None);
        assert!(matches!(
            result,
            Err(ConnectionError::LoginFailed(LoginFailure::AccountRequired))
        ));
        drop(client);
        let mut rest = Vec::new();
        server.read_to_end(&mut rest).unwrap();
        assert!(rest.is_empty());
    }
}
