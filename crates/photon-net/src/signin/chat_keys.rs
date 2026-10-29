//! The key pair the player signs chat with: fetched from the Minecraft services with the
//! account token, kept in memory only and fetched again when it is due a refresh.

use std::fmt;

use photon_core::json;
use photon_protocol::crypto::key_pair_matches;

use super::http::post_bearer;
use super::{Hosts, SignInError};
use crate::account::AccessToken;

/// A profile key pair with the service's signature over its public half.
#[derive(Clone, PartialEq, Eq)]
pub struct ProfileKeys {
    /// The private key, PKCS#8 DER. Never written anywhere.
    private_key: Vec<u8>,
    /// The public key, X.509 DER.
    pub public_key: Vec<u8>,
    /// The service's signature over the public key and its expiry (the V2 signature).
    pub key_signature: Vec<u8>,
    /// When the public key stops being accepted, in milliseconds since the epoch.
    pub expires_at_ms: i64,
    /// When the pair should be fetched again, in milliseconds since the epoch.
    pub refreshed_after_ms: i64,
}

impl fmt::Debug for ProfileKeys {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProfileKeys")
            .field("expires_at_ms", &self.expires_at_ms)
            .field("refreshed_after_ms", &self.refreshed_after_ms)
            .finish_non_exhaustive()
    }
}

impl ProfileKeys {
    /// A pair made in a test.
    #[cfg(test)]
    pub(crate) fn for_test(private_key: Vec<u8>, public_key: Vec<u8>) -> Self {
        Self {
            private_key,
            public_key,
            key_signature: vec![1],
            expires_at_ms: i64::MAX,
            refreshed_after_ms: i64::MAX,
        }
    }

    /// The private key, for the one place that signs with it.
    pub fn private_key_der(&self) -> &[u8] {
        &self.private_key
    }

    /// `ProfileKeyPair.dueRefresh`: whether it is time to fetch a new pair.
    pub fn due_refresh(&self, now_ms: i64) -> bool {
        self.refreshed_after_ms < now_ms
    }

    /// `ProfilePublicKey.hasExpired`: whether the server would turn the public key down.
    pub fn expired(&self, now_ms: i64) -> bool {
        self.expires_at_ms < now_ms
    }
}

/// `POST /player/certificates`: a new key pair for the account the token belongs to.
pub fn fetch_profile_keys(hosts: &Hosts, token: &AccessToken) -> Result<ProfileKeys, SignInError> {
    let answer = post_bearer((&hosts.services, "/player/certificates"), token.reveal())?;
    if answer.status >= 500 {
        return Err(SignInError::Unavailable);
    }
    if answer.status != 200 {
        return Err(SignInError::Rejected(format!("Status: {}", answer.status)));
    }
    parse(&answer.body)
}

fn rejected(what: &str) -> SignInError {
    SignInError::Rejected(format!("a key pair answer with {what}"))
}

fn parse(body: &str) -> Result<ProfileKeys, SignInError> {
    let object = json::parse_object(body).ok_or_else(|| rejected("no object"))?;
    let pair = object
        .object("keyPair")
        .ok_or_else(|| rejected("no key pair"))?;
    let private_key = pem_body(
        &pair
            .text("privateKey")
            .ok_or_else(|| rejected("no private key"))?,
    )?;
    let public_key = pem_body(
        &pair
            .text("publicKey")
            .ok_or_else(|| rejected("no public key"))?,
    )?;
    let key_signature = decode_base64(
        &object
            .text("publicKeySignatureV2")
            .ok_or_else(|| rejected("no signature"))?,
    )
    .ok_or_else(|| rejected("a bad signature"))?;
    let time = |key: &str| {
        object
            .text(key)
            .and_then(|text| parse_instant_ms(&text))
            .ok_or_else(|| rejected("a bad time"))
    };
    let (expires_at_ms, refreshed_after_ms) = (time("expiresAt")?, time("refreshedAfter")?);
    if !key_pair_matches(&private_key, &public_key) {
        return Err(rejected("a private key that is not the public key's"));
    }
    Ok(ProfileKeys {
        private_key,
        public_key,
        key_signature,
        expires_at_ms,
        refreshed_after_ms,
    })
}

/// The bytes between the `-----BEGIN` and `-----END` lines of a PEM block.
fn pem_body(pem: &str) -> Result<Vec<u8>, SignInError> {
    let body: String = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();
    decode_base64(&body).ok_or_else(|| rejected("a bad key"))
}

/// The value of a base64 digit.
fn base64_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Standard base64 with padding; whitespace is ignored (the MIME form the services use).
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let (mut bits, mut count, mut padding) = (0u32, 0u32, 0u32);
    for byte in text.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            padding += 1;
            continue;
        }
        let value = base64_digit(byte)?;
        if padding > 0 {
            return None;
        }
        bits = bits << 6 | u32::from(value);
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
            bits &= (1 << count) - 1;
        }
    }
    (padding <= 2 && bits == 0).then_some(out)
}

/// `Instant.parse` for the ISO 8601 form the services send (`2026-11-01T10:20:30.123456Z`), as
/// milliseconds since the epoch.
fn parse_instant_ms(text: &str) -> Option<i64> {
    let text = text.strip_suffix('Z')?;
    let (date, time) = text.split_once('T')?;
    let mut date = date.splitn(3, '-');
    let (year, month, day) = (
        date.next()?.parse::<i64>().ok()?,
        date.next()?.parse::<i64>().ok()?,
        date.next()?.parse::<i64>().ok()?,
    );
    let (clock, fraction) = time.split_once('.').unwrap_or((time, ""));
    let mut clock = clock.splitn(3, ':');
    let (hour, minute, second) = (
        clock.next()?.parse::<i64>().ok()?,
        clock.next()?.parse::<i64>().ok()?,
        clock.next().unwrap_or("0").parse::<i64>().ok()?,
    );
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return None;
    }
    let millis: i64 = format!("{fraction:0<3}")[..3].parse().ok()?;
    // Days from the civil date (Howard Hinnant's algorithm).
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some(((days * 24 + hour) * 60 + minute) * 60_000 + second * 1000 + millis)
}

#[cfg(test)]
mod tests {
    use super::super::http::tests::{json_answer, stub_host};
    use super::*;
    use photon_protocol::crypto::SystemRandom;
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};

    fn pem(label: &str, der: &[u8]) -> String {
        let mut text = format!("-----BEGIN {label}-----\\n");
        let encoded = encode_base64(der);
        for chunk in encoded.as_bytes().chunks(76) {
            text.push_str(std::str::from_utf8(chunk).unwrap());
            text.push_str("\\n");
        }
        text.push_str(&format!("-----END {label}-----\\n"));
        text
    }

    fn encode_base64(bytes: &[u8]) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let group = chunk.iter().enumerate().fold(0u32, |sum, (place, byte)| {
                sum | u32::from(*byte) << (16 - 8 * place)
            });
            for place in 0..4 {
                if place <= chunk.len() {
                    out.push(char::from(TABLE[(group >> (18 - 6 * place) & 63) as usize]));
                } else {
                    out.push('=');
                }
            }
        }
        out
    }

    fn answer_for(private: &rsa::RsaPrivateKey, public: &rsa::RsaPrivateKey) -> String {
        let private_der = private.to_pkcs8_der().unwrap();
        let public_der = public.to_public_key().to_public_key_der().unwrap();
        format!(
            r#"{{"keyPair":{{"privateKey":"{}","publicKey":"{}"}},"publicKeySignature":"AAAA","publicKeySignatureV2":"c2ln","expiresAt":"2026-11-01T10:20:30.5Z","refreshedAfter":"2026-10-30T10:20:30.123456Z"}}"#,
            pem("RSA PRIVATE KEY", private_der.as_bytes()).replace('\n', "\\n"),
            pem("RSA PUBLIC KEY", public_der.as_bytes()).replace('\n', "\\n"),
        )
    }

    fn hosts(host: String) -> Hosts {
        Hosts {
            services: host,
            ..Hosts::default()
        }
    }

    #[test]
    fn instants_and_base64_are_read() {
        assert_eq!(parse_instant_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_instant_ms("2000-03-01T00:00:01.5Z"),
            Some(951_868_801_500)
        );
        assert_eq!(parse_instant_ms("2000-03-01T00:00:00"), None);
        assert_eq!(decode_base64("c2ln").as_deref(), Some(&b"sig"[..]));
        assert_eq!(decode_base64("c2ln\n").as_deref(), Some(&b"sig"[..]));
        assert_eq!(decode_base64("c2lnZA==").as_deref(), Some(&b"sigd"[..]));
        assert_eq!(decode_base64("c2l*"), None);
    }

    #[test]
    fn a_key_pair_is_fetched_with_the_token_and_kept_without_printing_the_key() {
        let key = rsa::RsaPrivateKey::new(&mut SystemRandom, 1024).unwrap();
        let (host, seen) = stub_host(vec![json_answer("200 OK", &answer_for(&key, &key))]);
        let keys = fetch_profile_keys(&hosts(host), &AccessToken::new("MCTOKEN".to_owned()))
            .expect("keys");
        let requests = seen.join().unwrap();
        assert!(requests[0].starts_with("POST /player/certificates"));
        assert!(
            requests[0]
                .to_lowercase()
                .contains("authorization: bearer mctoken")
        );
        assert_eq!(keys.key_signature, b"sig");
        assert_eq!(keys.expires_at_ms, 1_793_528_430_500);
        assert!(!keys.due_refresh(keys.refreshed_after_ms));
        assert!(keys.due_refresh(keys.refreshed_after_ms + 1));
        assert!(keys.expired(keys.expires_at_ms + 1));
        let shown = format!("{keys:?}");
        assert!(!shown.contains("private_key") && !shown.contains("public_key"));
    }

    #[test]
    fn a_private_key_of_another_pair_is_refused() {
        let one = rsa::RsaPrivateKey::new(&mut SystemRandom, 1024).unwrap();
        let other = rsa::RsaPrivateKey::new(&mut SystemRandom, 1024).unwrap();
        let (host, _) = stub_host(vec![json_answer("200 OK", &answer_for(&one, &other))]);
        let error = fetch_profile_keys(&hosts(host), &AccessToken::new("T".to_owned())).err();
        assert!(matches!(error, Some(SignInError::Rejected(_))));
    }

    #[test]
    fn a_refusal_and_a_server_error_are_told_apart() {
        let (host, _) = stub_host(vec![json_answer("401 Unauthorized", "")]);
        let token = AccessToken::new("T".to_owned());
        assert!(matches!(
            fetch_profile_keys(&hosts(host), &token).err(),
            Some(SignInError::Rejected(_))
        ));
        let (host, _) = stub_host(vec![json_answer("503 Service Unavailable", "")]);
        assert_eq!(
            fetch_profile_keys(&hosts(host), &token).err(),
            Some(SignInError::Unavailable)
        );
    }
}
