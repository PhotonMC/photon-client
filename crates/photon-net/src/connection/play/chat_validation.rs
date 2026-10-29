//! Checking what other players say (`SignedMessageValidator`): a player who told the server its
//! chat session has each message checked against the session's public key and chain; one who did
//! not has its messages taken unsigned, or turned away where the server enforces secure chat.

use std::collections::BTreeMap;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use photon_core::binary::Uuid;
use photon_protocol::crypto::{verify_sha1_rsa, verify_sha256_rsa};
use photon_protocol::play::{
    MessageBody, MessageLink, PlayerChatPacket, RemoteChatSession, signed_message_payload,
};

use crate::skins::fetch_certificate_keys;

/// `ProfilePublicKey.EXPIRY_GRACE_PERIOD`: a key is still taken for 8 hours after it expired.
const EXPIRY_GRACE_MS: i64 = 8 * 60 * 60 * 1000;
/// How long a message waits for the services' keys when they have not come yet.
const KEYS_WAIT: Duration = Duration::from_secs(5);

/// The keys that sign the players' profile keys (`ServicesKeySet`, `PROFILE_KEY`), fetched from
/// the services in the background once an account is signed in.
#[derive(Default)]
pub(super) enum CertificateKeys {
    #[default]
    NotRequested,
    Pending(Receiver<Option<Vec<Vec<u8>>>>),
    /// The keys; none where the services could not be asked.
    Ready(Option<Vec<Vec<u8>>>),
}

impl CertificateKeys {
    pub(super) fn request(&mut self) {
        if !matches!(self, Self::NotRequested) {
            return;
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("certificate-keys".to_owned())
            .spawn(move || {
                let _ = sender.send(fetch_certificate_keys(Duration::from_secs(15)));
            });
        if spawned.is_ok() {
            *self = Self::Pending(receiver);
        }
    }

    /// The keys, waiting for them if they are on their way. An empty set is no keys
    /// (`canValidateProfileKeys`).
    fn resolve(&mut self) -> Option<&[Vec<u8>]> {
        if let Self::Pending(receiver) = self {
            let keys = receiver.recv_timeout(KEYS_WAIT).ok().flatten();
            *self = Self::Ready(keys);
        }
        match self {
            Self::Ready(Some(keys)) if !keys.is_empty() => Some(keys),
            _ => None,
        }
    }
}

/// How a message stands after the check.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    /// Shown; `signed` where the signature held up (an unsigned message loses it).
    Accepted { signed: bool },
    /// Not shown as a message (`handleChatMessageError`).
    Rejected,
}

/// What the client keeps of each player's chat: the session the server announced and, once a
/// message needs it, the validator made from it.
#[derive(Default)]
pub(super) struct Senders {
    senders: BTreeMap<(u64, u64), Sender>,
    /// `serverEnforcesSecureChat`.
    pub(super) enforces_secure_chat: bool,
    pub(super) keys: CertificateKeys,
}

#[derive(Default)]
struct Sender {
    session: Option<RemoteChatSession>,
    validator: Option<Validator>,
}

enum Validator {
    AcceptUnsigned,
    RejectAll,
    Keyed(Keyed),
}

struct Keyed {
    public_key: Vec<u8>,
    session: Uuid,
    expires_at_ms: i64,
    chain_valid: bool,
    /// The index, session and signature of the last message taken.
    last: Option<(i32, Vec<u8>)>,
}

impl Senders {
    /// `INITIALIZE_CHAT`: the player's session, or none; the validator is made again.
    pub(super) fn set_session(&mut self, id: (u64, u64), session: Option<RemoteChatSession>) {
        self.senders.insert(
            id,
            Sender {
                session,
                validator: None,
            },
        );
    }

    pub(super) fn remove(&mut self, id: &(u64, u64)) {
        self.senders.remove(id);
    }

    /// `PlayerInfo.getMessageValidator().updateAndValidate`.
    pub(super) fn validate(
        &mut self,
        packet: &PlayerChatPacket,
        last_seen: &[Vec<u8>],
        now_ms: i64,
    ) -> Verdict {
        let sender = Uuid::new(packet.sender.0, packet.sender.1);
        let enforces = self.enforces_secure_chat;
        let entry = self.senders.entry(packet.sender).or_default();
        if entry.validator.is_none() {
            let keys = self.keys.resolve();
            entry.validator = Some(make_validator(
                entry.session.as_ref(),
                keys,
                sender,
                enforces,
            ));
        }
        let message = Message {
            link: MessageLink {
                sender,
                session: entry
                    .session
                    .as_ref()
                    .map_or(Uuid::new(0, 0), |s| s.session_id),
                index: packet.index,
            },
            body: MessageBody {
                content: &packet.signed_text,
                epoch_second: packet.timestamp_millis.div_euclid(1000),
                salt: packet.salt,
                last_seen,
            },
            signature: packet.signature.as_deref(),
        };
        match entry.validator.as_mut() {
            Some(Validator::Keyed(keyed)) => keyed.update_and_validate(&message, now_ms),
            Some(Validator::RejectAll) => Verdict::Rejected,
            _ => Verdict::Accepted { signed: false },
        }
    }
}

struct Message<'a> {
    link: MessageLink,
    body: MessageBody<'a>,
    signature: Option<&'a [u8]>,
}

/// `initializeChatSession` and `fallbackMessageValidator`: the session counts if the services
/// signed its key for this player; otherwise the player is taken as having none.
fn make_validator(
    session: Option<&RemoteChatSession>,
    keys: Option<&[Vec<u8>]>,
    sender: Uuid,
    enforces: bool,
) -> Validator {
    let enforces = enforces && keys.is_some();
    let fallback = || {
        if enforces {
            Validator::RejectAll
        } else {
            Validator::AcceptUnsigned
        }
    };
    let (Some(session), Some(keys)) = (session, keys) else {
        return fallback();
    };
    let mut payload = Vec::with_capacity(24 + session.public_key.len());
    photon_core::binary::write_uuid(sender, &mut payload);
    payload.extend_from_slice(&session.expires_at_ms.to_be_bytes());
    payload.extend_from_slice(&session.public_key);
    if !keys
        .iter()
        .any(|key| verify_sha1_rsa(key, &payload, &session.key_signature))
    {
        return fallback();
    }
    Validator::Keyed(Keyed {
        public_key: session.public_key.clone(),
        session: session.session_id,
        expires_at_ms: session.expires_at_ms,
        chain_valid: true,
        last: None,
    })
}

impl Keyed {
    /// `SignedMessageValidator.KeyBased.updateAndValidate`: once a message fails, so does every
    /// one after it.
    fn update_and_validate(&mut self, message: &Message<'_>, now_ms: i64) -> Verdict {
        self.chain_valid = self.chain_valid && self.validate(message, now_ms);
        if !self.chain_valid {
            return Verdict::Rejected;
        }
        let signature = message.signature.unwrap_or_default().to_vec();
        self.last = Some((message.link.index, signature));
        Verdict::Accepted { signed: true }
    }

    fn validate(&self, message: &Message<'_>, now_ms: i64) -> bool {
        if self.expires_at_ms.saturating_add(EXPIRY_GRACE_MS) < now_ms {
            return false;
        }
        let Some(signature) = message.signature else {
            return false;
        };
        let payload = signed_message_payload(message.link, message.body);
        if !verify_sha256_rsa(&self.public_key, &payload, signature) {
            return false;
        }
        // `validateChain`: the same message again passes; a later one has to have a higher index.
        match &self.last {
            Some((index, last)) if last == signature && *index == message.link.index => true,
            Some((index, _)) => message.link.index > *index && message.link.session == self.session,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use rsa::pkcs8::EncodePublicKey;
    use rsa::{Pkcs1v15Sign, RsaPrivateKey};
    use sha1::Digest as _;

    use super::*;

    const SENDER: (u64, u64) = (1, 2);

    fn key() -> RsaPrivateKey {
        RsaPrivateKey::new(&mut photon_protocol::crypto::SystemRandom, 1024).expect("key")
    }

    fn der(key: &RsaPrivateKey) -> Vec<u8> {
        key.to_public_key()
            .to_public_key_der()
            .expect("der")
            .as_bytes()
            .to_vec()
    }

    /// A services key and a player key it signed, with the session announced.
    fn world() -> (RsaPrivateKey, Senders) {
        let (services, player) = (key(), key());
        let public_key = der(&player);
        let expires_at_ms = i64::MAX / 2;
        let mut payload = Vec::new();
        photon_core::binary::write_uuid(Uuid::new(SENDER.0, SENDER.1), &mut payload);
        payload.extend_from_slice(&expires_at_ms.to_be_bytes());
        payload.extend_from_slice(&public_key);
        // The services sign with SHA-1; the digest info header is added by hand.
        let mut info = vec![
            0x30, 0x21, 0x30, 0x09, 0x06, 0x05, 0x2b, 0x0e, 0x03, 0x02, 0x1a, 0x05, 0x00, 0x04,
            0x14,
        ];
        info.extend(sha1::Sha1::digest(&payload));
        let key_signature = services
            .sign(Pkcs1v15Sign::new_unprefixed(), &info)
            .expect("signed");
        let mut senders = Senders {
            keys: CertificateKeys::Ready(Some(vec![der(&services)])),
            ..Senders::default()
        };
        senders.set_session(
            SENDER,
            Some(RemoteChatSession {
                session_id: Uuid::new(3, 4),
                expires_at_ms,
                public_key,
                key_signature,
            }),
        );
        (player, senders)
    }

    fn packet(player: &RsaPrivateKey, index: i32, text: &str) -> PlayerChatPacket {
        let link = MessageLink {
            sender: Uuid::new(SENDER.0, SENDER.1),
            session: Uuid::new(3, 4),
            index,
        };
        let body = MessageBody {
            content: text,
            epoch_second: 5,
            salt: 9,
            last_seen: &[],
        };
        let signature = photon_protocol::crypto::sign_sha256_rsa(
            player.to_pkcs8_der_for_test().as_slice(),
            &[&signed_message_payload(link, body)],
        )
        .expect("signed");
        let mut packet = unsigned_packet(index, text);
        packet.signature = Some(signature);
        packet
    }

    fn unsigned_packet(index: i32, text: &str) -> PlayerChatPacket {
        PlayerChatPacket {
            global_index: 0,
            sender: SENDER,
            index,
            signature: None,
            signed_text: text.to_owned(),
            timestamp_millis: 5500,
            salt: 9,
            last_seen: Vec::new(),
            unsigned: None,
            filter: photon_protocol::play::FilterMask::PassThrough,
            bound: photon_protocol::play::ChatBound {
                holder: photon_protocol::play::ChatHolder::Registry(0),
                name: photon_core::text::Component::literal("n"),
                target: None,
            },
        }
    }

    trait PkcsForTest {
        fn to_pkcs8_der_for_test(&self) -> Vec<u8>;
    }

    impl PkcsForTest for RsaPrivateKey {
        fn to_pkcs8_der_for_test(&self) -> Vec<u8> {
            use rsa::pkcs8::EncodePrivateKey;
            self.to_pkcs8_der().expect("der").as_bytes().to_vec()
        }
    }

    #[test]
    fn a_message_signed_in_the_announced_session_is_accepted_in_order() {
        let (player, mut senders) = world();
        for index in 0..3 {
            let message = packet(&player, index, "hi");
            assert_eq!(
                senders.validate(&message, &[], 1000),
                Verdict::Accepted { signed: true }
            );
        }
    }

    #[test]
    fn one_bad_message_breaks_the_chain_for_the_rest() {
        let (player, mut senders) = world();
        assert_eq!(
            senders.validate(&packet(&player, 0, "hi"), &[], 1000),
            Verdict::Accepted { signed: true }
        );
        let mut forged = packet(&player, 1, "hi");
        forged.signed_text = "ho".to_owned();
        assert_eq!(senders.validate(&forged, &[], 1000), Verdict::Rejected);
        assert_eq!(
            senders.validate(&packet(&player, 2, "hi"), &[], 1000),
            Verdict::Rejected
        );
    }

    #[test]
    fn an_old_index_and_a_missing_signature_are_turned_away() {
        let (player, mut senders) = world();
        senders.validate(&packet(&player, 4, "hi"), &[], 1000);
        assert_eq!(
            senders.validate(&packet(&player, 3, "hi"), &[], 1000),
            Verdict::Rejected
        );
        let (_, mut senders) = world();
        assert_eq!(
            senders.validate(&unsigned_packet(0, "hi"), &[], 1000),
            Verdict::Rejected
        );
    }

    #[test]
    fn an_expired_key_is_turned_away_after_its_grace() {
        let (player, mut senders) = world();
        let late = i64::MAX / 2 + EXPIRY_GRACE_MS + 1;
        assert_eq!(
            senders.validate(&packet(&player, 0, "hi"), &[], late),
            Verdict::Rejected
        );
    }

    #[test]
    fn a_player_without_a_session_is_unsigned_or_rejected_where_chat_is_enforced() {
        let mut senders = Senders {
            keys: CertificateKeys::Ready(Some(vec![vec![1]])),
            ..Senders::default()
        };
        assert_eq!(
            senders.validate(&unsigned_packet(0, "hi"), &[], 0),
            Verdict::Accepted { signed: false }
        );
        let mut senders = Senders {
            keys: CertificateKeys::Ready(Some(vec![vec![1]])),
            enforces_secure_chat: true,
            ..Senders::default()
        };
        assert_eq!(
            senders.validate(&unsigned_packet(0, "hi"), &[], 0),
            Verdict::Rejected
        );
        // Without the services' keys nothing can be checked, so nothing is enforced.
        let mut senders = Senders {
            keys: CertificateKeys::Ready(None),
            enforces_secure_chat: true,
            ..Senders::default()
        };
        assert_eq!(
            senders.validate(&unsigned_packet(0, "hi"), &[], 0),
            Verdict::Accepted { signed: false }
        );
    }

    #[test]
    fn a_session_the_services_did_not_sign_counts_for_nothing() {
        let (_, mut senders) = world();
        senders.keys = CertificateKeys::Ready(Some(vec![der(&key())]));
        assert_eq!(
            senders.validate(&unsigned_packet(0, "hi"), &[], 0),
            Verdict::Accepted { signed: false }
        );
    }
}
