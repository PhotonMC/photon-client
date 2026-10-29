//! The account a player joins an online-mode server with.

use std::fmt;

use photon_core::binary::Uuid;

/// The session server vanilla uses when nothing says otherwise.
pub const DEFAULT_SESSION_HOST: &str = "https://sessionserver.mojang.com";

/// A session token. It is shown as nothing, so that no log line or error can carry it.
#[derive(Clone, PartialEq, Eq)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn new(token: String) -> Self {
        Self(token)
    }

    /// The token itself, for the one request that has to send it.
    pub(crate) fn reveal(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccessToken(..)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub name: String,
    pub profile_id: Uuid,
    /// Where the join is posted (`Environment.sessionHost`).
    pub session_host: String,
    pub(crate) token: AccessToken,
}

impl Account {
    pub fn new(name: String, profile_id: Uuid, token: AccessToken, session_host: String) -> Self {
        Self {
            name,
            profile_id,
            session_host,
            token,
        }
    }
}

/// A profile id written with or without dashes.
pub fn parse_profile_id(text: &str) -> Option<Uuid> {
    let digits: String = text.chars().filter(|&c| c != '-').collect();
    if digits.len() != 32 || !digits.is_ascii() {
        return None;
    }
    let most = u64::from_str_radix(&digits[..16], 16).ok()?;
    let least = u64::from_str_radix(&digits[16..], 16).ok()?;
    Some(Uuid::new(most, least))
}

/// A profile id as the session server writes it: 32 hex digits, no dashes.
pub(crate) fn undashed(id: Uuid) -> String {
    format!("{:016x}{:016x}", id.most, id.least)
}

/// Whether a plain `http://` host names this machine, by its exact host name.
pub(crate) fn is_this_machine(host: &str) -> bool {
    let Some(rest) = host.strip_prefix("http://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    // Before an `@` comes a user name, and the host is what follows it.
    if authority.contains('@') {
        return false;
    }
    let name = match authority.strip_prefix('[') {
        Some(inner) => inner.split(']').next().map(|ip| format!("[{ip}]")),
        None => authority.split(':').next().map(str::to_owned),
    };
    matches!(name.as_deref(), Some("127.0.0.1" | "localhost" | "[::1]"))
}

/// A string as a JSON literal.
pub(crate) fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_ids_round_trip_with_and_without_dashes() {
        let id = parse_profile_id("0123456789abcdef-fedcba9876543210").unwrap();
        assert_eq!(undashed(id), "0123456789abcdeffedcba9876543210");
        assert_eq!(
            parse_profile_id("01234567-89ab-cdef-fedc-ba9876543210"),
            Some(id)
        );
        assert_eq!(parse_profile_id("short"), None);
    }

    #[test]
    fn the_token_does_not_show() {
        let account = Account::new(
            "Alex".to_owned(),
            Uuid::new(1, 2),
            AccessToken::new("secret-token".to_owned()),
            DEFAULT_SESSION_HOST.to_owned(),
        );
        assert!(!format!("{account:?}").contains("secret-token"));
    }
}
