//! The chat session (`LocalChatSession`): the profile key pair is fetched in the background once
//! the server says it is online, and the server is told of the session when the pair arrives.

use std::sync::mpsc::{Receiver, TryRecvError, channel};

use photon_core::binary::Uuid;

use super::*;
use crate::account::{AccessToken, Account};
use crate::signin::{Hosts, ProfileKeys, SignInError, fetch_profile_keys};
use photon_protocol::play::{MessageBody, MessageLink, signed_message_payload};

/// What the connection knows of its chat session.
#[derive(Default)]
pub(super) struct ChatSession {
    /// The key pair being fetched (`keyPairFuture`).
    pending: Option<Receiver<Result<ProfileKeys, SignInError>>>,
    /// The pair of the session the server was told of, and the session's id.
    current: Option<(ProfileKeys, Uuid)>,
    /// The index of the next signed message (`SignedMessageChain.nextLink`); none once the chain
    /// ran out.
    next_index: Option<i32>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |time| time.as_millis() as i64)
}

/// A random version 4 UUID, the session id (`UUID.randomUUID`).
fn random_uuid() -> Option<Uuid> {
    let bytes = photon_protocol::crypto::generate_secret().ok()?;
    let (most, least) = bytes.split_at(8);
    let most = u64::from_be_bytes(most.try_into().ok()?);
    let least = u64::from_be_bytes(least.try_into().ok()?);
    Some(Uuid::new(
        (most & !0xF000) | 0x4000,
        (least & !(0xC000 << 48)) | (0x8000 << 48),
    ))
}

impl ChatSession {
    /// `handleLogin`: the old session is gone, and an online server starts a fetch.
    pub(super) fn reset(&mut self, fetch: Option<(Hosts, AccessToken)>) {
        *self = Self::default();
        if let Some((hosts, token)) = fetch {
            self.start_fetch(hosts, token);
        }
    }

    fn start_fetch(&mut self, hosts: Hosts, token: AccessToken) {
        let (sender, receiver) = channel();
        let spawned = std::thread::Builder::new()
            .name("profile-keys".to_owned())
            .spawn(move || {
                let _ = sender.send(fetch_profile_keys(&hosts, &token));
            });
        if spawned.is_ok() {
            self.pending = Some(receiver);
        }
    }

    /// `tick`: a pair that is due a refresh is fetched again; a pair that arrived is a new
    /// session unless it is the one in use. The new session's id and public key are returned.
    pub(super) fn tick(&mut self, account: &Account) -> Option<Update> {
        if self.pending.is_none()
            && self
                .current
                .as_ref()
                .is_some_and(|(keys, _)| keys.due_refresh(now_ms()))
        {
            self.start_fetch(Hosts::from_environment(), account.token.clone());
        }
        let arrived = match self.pending.as_ref()?.try_recv() {
            Ok(answer) => answer,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Err(SignInError::Unavailable),
        };
        self.pending = None;
        // A failed fetch leaves the player sending unsigned chat (`Optional.empty`).
        let keys = match arrived {
            Ok(keys) => keys,
            Err(error) => {
                log::warn!("secure chat: the profile keys were not fetched: {error:?}");
                return None;
            }
        };
        if self.current.as_ref().is_some_and(|(held, _)| *held == keys) {
            return None;
        }
        let id = random_uuid()?;
        let update = Update {
            id,
            expires_at_ms: keys.expires_at_ms,
            public_key: keys.public_key.clone(),
            key_signature: keys.key_signature.clone(),
        };
        self.current = Some((keys, id));
        self.next_index = Some(0);
        Some(update)
    }
}

impl ChatSession {
    /// `SignedMessageChain.Encoder.pack`: the signature of a message, or none while there is no
    /// session (the message goes unsigned) or the chain has run out.
    pub(super) fn sign(
        &mut self,
        sender: Uuid,
        content: &str,
        (millis, salt): (i64, i64),
        seen: &[Vec<u8>],
    ) -> Option<Vec<u8>> {
        let (keys, session) = self.current.as_ref()?;
        let index = self.next_index?;
        let link = MessageLink {
            sender,
            session: *session,
            index,
        };
        let body = MessageBody {
            content,
            epoch_second: millis.div_euclid(1000),
            salt,
            last_seen: seen,
        };
        let payload = signed_message_payload(link, body);
        let signature =
            photon_protocol::crypto::sign_sha256_rsa(keys.private_key_der(), &[&payload]);
        self.next_index = index.checked_add(1);
        signature.ok()
    }
}

/// The data of `ServerboundChatSessionUpdatePacket`.
pub(super) struct Update {
    id: Uuid,
    expires_at_ms: i64,
    public_key: Vec<u8>,
    key_signature: Vec<u8>,
}

impl Update {
    pub(super) fn encode(&self, output: &mut Vec<u8>) {
        let key = (self.expires_at_ms, &*self.public_key, &*self.key_signature);
        photon_protocol::play::encode_chat_session_update(self.id, key, output);
    }
}

impl PlaySession {
    /// The services to fetch a key pair from for the account, if the connection has one.
    fn key_fetch(&self) -> Option<(Hosts, AccessToken)> {
        let account = self.state.account.as_ref()?;
        Some((Hosts::from_environment(), account.token.clone()))
    }

    /// `handleLogin`: the chat session is reset, and an online server gets a key pair fetched.
    pub(super) fn reset_chat_session(&mut self, online: bool) {
        let fetch = if online { self.key_fetch() } else { None };
        if fetch.is_some() {
            self.social.senders.keys.request();
        }
        self.social.chat_session.reset(fetch);
    }

    /// `tick`: fetch results become a chat session the server is told of.
    pub(super) fn tick_chat_session(&mut self) -> Result<(), ConnectionError> {
        let Some(account) = self.state.account.as_ref() else {
            return Ok(());
        };
        let Some(update) = self.social.chat_session.tick(account) else {
            return Ok(());
        };
        self.send(PLAY_SERVERBOUND_CHAT_SESSION_UPDATE_PACKET_ID, |output| {
            update.encode(output);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_session_id_is_a_version_4_uuid() {
        let id = random_uuid().expect("randomness");
        assert_eq!((id.most >> 12) & 0xF, 4);
        assert_eq!(id.least >> 62, 2);
    }

    #[test]
    fn signed_messages_follow_the_chain_and_verify_under_the_public_key() {
        use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
        use sha2::Digest as _;
        let private =
            rsa::RsaPrivateKey::new(&mut photon_protocol::crypto::SystemRandom, 1024).expect("key");
        let public = private.to_public_key();
        let keys = ProfileKeys::for_test(
            private.to_pkcs8_der().expect("der").as_bytes().to_vec(),
            public.to_public_key_der().expect("der").as_bytes().to_vec(),
        );
        let mut session = ChatSession::default();
        let (sender, id) = (Uuid::new(1, 2), Uuid::new(3, 4));
        assert_eq!(session.sign(sender, "hi", (5000, 9), &[]), None);
        session.current = Some((keys, id));
        session.next_index = Some(0);
        for index in 0..2 {
            let signature = session.sign(sender, "hi", (5999, 9), &[]).expect("signed");
            let link = MessageLink {
                sender,
                session: id,
                index,
            };
            let body = MessageBody {
                content: "hi",
                epoch_second: 5,
                salt: 9,
                last_seen: &[],
            };
            let digest = sha2::Sha256::digest(signed_message_payload(link, body));
            let scheme = rsa::Pkcs1v15Sign::new::<sha2::Sha256>();
            assert!(public.verify(scheme, &digest, &signature).is_ok());
        }
        session.next_index = Some(i32::MAX);
        assert!(session.sign(sender, "hi", (0, 0), &[]).is_some());
        assert_eq!(session.sign(sender, "hi", (0, 0), &[]), None);
    }
}
