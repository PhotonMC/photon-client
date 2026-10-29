//! From a Microsoft token to a Minecraft session: Xbox Live user token, XSTS token, the
//! Minecraft services login, the ownership check and the profile.

use std::time::Duration;

use photon_core::json::{self, JsonObject, JsonValue};

use super::http::{Answer, get_bearer, post_json};
use super::{Hosts, MicrosoftTokens, SignInError};
use crate::account::{AccessToken, Account, json_string, parse_profile_id};

/// The relying party the Minecraft services accept an XSTS token for.
const MINECRAFT_PARTY: &str = "rp://api.minecraftservices.com/";

/// What a finished sign-in gives: the account to join servers with, and how long its token lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinecraftSession {
    pub account: Account,
    pub expires_in: Duration,
    /// That the account owns the game: what the game's content may be fetched on.
    pub ownership: GameOwnership,
}

/// Proof that the entitlements of a signed-in account name the game. Only the ownership check
/// builds it, so whatever takes one has been given a verified account; it holds no secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOwnership(());

impl GameOwnership {
    /// A proof for tests of what is gated on it; the shipped build has no way to make one.
    #[cfg(any(test, feature = "test-support"))]
    pub fn for_tests() -> Self {
        Self(())
    }
}

/// The object a value holds.
fn as_object(value: &JsonValue) -> Option<&JsonObject> {
    match value {
        JsonValue::Object(object) => Some(object),
        _ => None,
    }
}

/// The player-facing words for the codes (`XErr`) Xbox Live answers a refusal with.
fn xbox_reason(code: i64) -> String {
    match code {
        2_148_916_233 => {
            "this Microsoft account has no Xbox account; sign in at xbox.com to create one"
                .to_owned()
        }
        2_148_916_235 => "Xbox Live is not available in this country or region".to_owned(),
        2_148_916_236 | 2_148_916_237 => {
            "the account needs adult verification on the Xbox page".to_owned()
        }
        2_148_916_238 => {
            "this is a child account; an adult has to add it to a family first".to_owned()
        }
        other => format!("error {other}"),
    }
}

/// A refusal of the Xbox services: its `XErr` code in words, else the status.
fn xbox_failure(answer: &Answer) -> SignInError {
    if answer.status >= 500 {
        return SignInError::Unavailable;
    }
    let code = json::parse_object(&answer.body).and_then(|object| object.f64("XErr"));
    match code {
        Some(code) => SignInError::Xbox(xbox_reason(code as i64)),
        None => SignInError::Rejected(format!("Status: {}", answer.status)),
    }
}

/// The token and the user hash of an Xbox token answer.
fn token_and_hash(answer: &Answer) -> Result<(String, String), SignInError> {
    let object = json::parse_object(&answer.body).ok_or(SignInError::Unavailable)?;
    let hash = object
        .object("DisplayClaims")
        .and_then(|claims| claims.array("xui"))
        .and_then(|users| users.first())
        .and_then(as_object)
        .and_then(|user| user.text("uhs"));
    match (object.text("Token"), hash) {
        (Some(token), Some(hash)) => Ok((token, hash)),
        _ => Err(SignInError::Rejected(
            "an Xbox answer without a token".to_owned(),
        )),
    }
}

/// `user.auth.xboxlive.com`: the Microsoft access token for an Xbox Live user token.
fn xbox_user_token(hosts: &Hosts, microsoft: &MicrosoftTokens) -> Result<String, SignInError> {
    let body = format!(
        "{{\"Properties\":{{\"AuthMethod\":\"RPS\",\"SiteName\":\"user.auth.xboxlive.com\",\"RpsTicket\":{}}},\"RelyingParty\":\"http://auth.xboxlive.com\",\"TokenType\":\"JWT\"}}",
        json_string(&format!(
            "{}{}",
            microsoft.ticket_prefix,
            microsoft.access.reveal()
        )),
    );
    let answer = post_json((&hosts.xbox_user, "/user/authenticate"), &body)?;
    if answer.status != 200 {
        return Err(xbox_failure(&answer));
    }
    Ok(token_and_hash(&answer)?.0)
}

/// `xsts.auth.xboxlive.com`: the XSTS token and user hash for the Minecraft services.
fn xsts_token(hosts: &Hosts, user_token: &str) -> Result<(String, String), SignInError> {
    let body = format!(
        "{{\"Properties\":{{\"SandboxId\":\"RETAIL\",\"UserTokens\":[{}]}},\"RelyingParty\":{},\"TokenType\":\"JWT\"}}",
        json_string(user_token),
        json_string(MINECRAFT_PARTY),
    );
    let answer = post_json((&hosts.xsts, "/xsts/authorize"), &body)?;
    if answer.status != 200 {
        return Err(xbox_failure(&answer));
    }
    token_and_hash(&answer)
}

/// `login_with_xbox`: the Minecraft services token for an XSTS token.
fn services_token(
    hosts: &Hosts,
    (xsts, hash): (&str, &str),
) -> Result<(AccessToken, Duration), SignInError> {
    let body = format!(
        "{{\"identityToken\":{}}}",
        json_string(&format!("XBL3.0 x={hash};{xsts}")),
    );
    let answer = post_json((&hosts.services, "/authentication/login_with_xbox"), &body)?;
    if answer.status >= 500 {
        return Err(SignInError::Unavailable);
    }
    let object = json::parse_object(&answer.body).filter(|_| answer.status == 200);
    let token = object
        .as_ref()
        .and_then(|object| object.text("access_token"));
    let Some(token) = token else {
        return Err(SignInError::Rejected(format!("Status: {}", answer.status)));
    };
    let seconds = object.as_ref().and_then(|object| object.i32("expires_in"));
    let expires = seconds.and_then(|seconds| u64::try_from(seconds).ok());
    Ok((
        AccessToken::new(token),
        Duration::from_secs(expires.unwrap_or(86_400)),
    ))
}

/// Whether the entitlements name the game (the store product or the game itself).
fn owns_game(object: &JsonObject) -> bool {
    object.array("items").is_some_and(|items| {
        items.iter().filter_map(as_object).any(|item| {
            matches!(
                item.text("name").as_deref(),
                Some("product_minecraft" | "game_minecraft")
            )
        })
    })
}

fn check_ownership(hosts: &Hosts, token: &AccessToken) -> Result<GameOwnership, SignInError> {
    let answer = get_bearer((&hosts.services, "/entitlements/mcstore"), token.reveal())?;
    if answer.status >= 500 {
        return Err(SignInError::Unavailable);
    }
    let owned = json::parse_object(&answer.body).is_some_and(|object| owns_game(&object));
    if answer.status == 200 && owned {
        Ok(GameOwnership(()))
    } else {
        Err(SignInError::NotOwned)
    }
}

/// The profile (player name and id) of the account.
fn fetch_profile(
    hosts: &Hosts,
    token: &AccessToken,
) -> Result<(String, photon_core::binary::Uuid), SignInError> {
    let answer = get_bearer((&hosts.services, "/minecraft/profile"), token.reveal())?;
    if answer.status == 404 {
        return Err(SignInError::NoProfile);
    }
    if answer.status >= 500 {
        return Err(SignInError::Unavailable);
    }
    let object = json::parse_object(&answer.body).filter(|_| answer.status == 200);
    let name = object.as_ref().and_then(|object| object.text("name"));
    let id = object
        .as_ref()
        .and_then(|object| object.text("id"))
        .and_then(|id| parse_profile_id(&id));
    match (name, id) {
        (Some(name), Some(id)) => Ok((name, id)),
        _ => Err(SignInError::Rejected(format!("Status: {}", answer.status))),
    }
}

/// The whole chain: Xbox Live, XSTS, the Minecraft services login, the ownership check and the
/// profile. `session_host` is where the account will post its server joins.
pub fn minecraft_session(
    hosts: &Hosts,
    microsoft: &MicrosoftTokens,
    session_host: &str,
) -> Result<MinecraftSession, SignInError> {
    let user = xbox_user_token(hosts, microsoft)?;
    let (xsts, hash) = xsts_token(hosts, &user)?;
    let (token, expires_in) = services_token(hosts, (&xsts, &hash))?;
    let ownership = check_ownership(hosts, &token)?;
    let (name, profile_id) = fetch_profile(hosts, &token)?;
    Ok(MinecraftSession {
        account: Account::new(name, profile_id, token, session_host.to_owned()),
        expires_in,
        ownership,
    })
}

#[cfg(test)]
mod tests {
    use super::super::http::tests::{json_answer, stub_host};
    use super::*;

    const XBOX_BODY: &str = r#"{"IssueInstant":"x","NotAfter":"y","Token":"XTOKEN","DisplayClaims":{"xui":[{"uhs":"HASH"}]}}"#;
    const XSTS_BODY: &str = r#"{"Token":"STSTOKEN","DisplayClaims":{"xui":[{"uhs":"HASH2"}]}}"#;
    const LOGIN_BODY: &str =
        r#"{"username":"u","access_token":"MCTOKEN","token_type":"Bearer","expires_in":86400}"#;
    const OWNED_BODY: &str = r#"{"items":[{"name":"product_minecraft","signature":"s"},{"name":"game_minecraft","signature":"s"}],"signature":"s"}"#;
    const PROFILE_BODY: &str =
        r#"{"id":"0123456789abcdeffedcba9876543210","name":"Alex","skins":[],"capes":[]}"#;

    fn microsoft() -> MicrosoftTokens {
        MicrosoftTokens {
            access: AccessToken::new("MSACCESS".to_owned()),
            ticket_prefix: "d=",
            refresh: AccessToken::new("MSREFRESH".to_owned()),
            expires_in: Duration::from_secs(3600),
        }
    }

    /// Every service of the chain at one stub host.
    fn hosts(host: &str) -> Hosts {
        Hosts {
            login: host.to_owned(),
            live: host.to_owned(),
            xbox_user: host.to_owned(),
            xsts: host.to_owned(),
            services: host.to_owned(),
        }
    }

    #[test]
    fn a_full_chain_gives_the_account_and_sends_each_token_on() {
        let (host, seen) = stub_host(vec![
            json_answer("200 OK", XBOX_BODY),
            json_answer("200 OK", XSTS_BODY),
            json_answer("200 OK", LOGIN_BODY),
            json_answer("200 OK", OWNED_BODY),
            json_answer("200 OK", PROFILE_BODY),
        ]);
        let session = minecraft_session(&hosts(&host), &microsoft(), "https://sessions.example")
            .expect("a session");
        assert_eq!(session.account.name, "Alex");
        assert_eq!(
            session.account.profile_id,
            parse_profile_id("0123456789abcdeffedcba9876543210").unwrap()
        );
        assert_eq!(session.account.session_host, "https://sessions.example");
        assert_eq!(session.account.token.reveal(), "MCTOKEN");
        assert_eq!(session.expires_in, Duration::from_secs(86_400));
        let requests = seen.join().unwrap();
        assert!(requests[0].starts_with("POST /user/authenticate "));
        assert!(requests[0].contains("\"RpsTicket\":\"d=MSACCESS\""));
        assert!(requests[1].starts_with("POST /xsts/authorize "));
        assert!(requests[1].contains("\"UserTokens\":[\"XTOKEN\"]"));
        assert!(requests[1].contains("\"RelyingParty\":\"rp://api.minecraftservices.com/\""));
        assert!(requests[2].starts_with("POST /authentication/login_with_xbox "));
        assert!(requests[2].contains("\"identityToken\":\"XBL3.0 x=HASH2;STSTOKEN\""));
        assert!(requests[3].starts_with("GET /entitlements/mcstore "));
        assert!(
            requests[3]
                .to_ascii_lowercase()
                .contains("authorization: bearer mctoken")
        );
        assert!(requests[4].starts_with("GET /minecraft/profile "));
    }

    #[test]
    fn an_xsts_refusal_is_worded_for_the_player() {
        let cases = [
            (2_148_916_233_i64, "no Xbox account"),
            (2_148_916_235, "not available in this country"),
            (2_148_916_238, "child account"),
            (1, "error 1"),
        ];
        for (code, words) in cases {
            let body = format!(r#"{{"Identity":"0","XErr":{code},"Message":"","Redirect":"x"}}"#);
            let (host, _) = stub_host(vec![
                json_answer("200 OK", XBOX_BODY),
                json_answer("401 Unauthorized", &body),
            ]);
            let error = minecraft_session(&hosts(&host), &microsoft(), "https://s.example")
                .expect_err("refused");
            match error {
                SignInError::Xbox(reason) => assert!(reason.contains(words), "{reason}"),
                other => panic!("{other:?}"),
            }
        }
    }

    #[test]
    fn an_account_without_the_game_or_a_profile_is_told_so() {
        let (host, _) = stub_host(vec![
            json_answer("200 OK", XBOX_BODY),
            json_answer("200 OK", XSTS_BODY),
            json_answer("200 OK", LOGIN_BODY),
            json_answer("200 OK", r#"{"items":[],"signature":"s"}"#),
        ]);
        let error = minecraft_session(&hosts(&host), &microsoft(), "https://s.example");
        assert_eq!(error.err(), Some(SignInError::NotOwned));
        let (host, _) = stub_host(vec![
            json_answer("200 OK", XBOX_BODY),
            json_answer("200 OK", XSTS_BODY),
            json_answer("200 OK", LOGIN_BODY),
            json_answer("200 OK", OWNED_BODY),
            json_answer(
                "404 Not Found",
                r#"{"path":"/minecraft/profile","errorType":"NOT_FOUND"}"#,
            ),
        ]);
        let error = minecraft_session(&hosts(&host), &microsoft(), "https://s.example");
        assert_eq!(error.err(), Some(SignInError::NoProfile));
    }

    #[test]
    fn a_server_error_anywhere_in_the_chain_is_unavailable() {
        let (host, _) = stub_host(vec![json_answer("503 Service Unavailable", "")]);
        let error = minecraft_session(&hosts(&host), &microsoft(), "https://s.example");
        assert_eq!(error.err(), Some(SignInError::Unavailable));
    }

    #[test]
    fn the_session_does_not_print_its_tokens() {
        let (host, _) = stub_host(vec![
            json_answer("200 OK", XBOX_BODY),
            json_answer("200 OK", XSTS_BODY),
            json_answer("200 OK", LOGIN_BODY),
            json_answer("200 OK", OWNED_BODY),
            json_answer("200 OK", PROFILE_BODY),
        ]);
        let session = minecraft_session(&hosts(&host), &microsoft(), "https://s.example").unwrap();
        assert!(!format!("{session:?}").contains("MCTOKEN"));
    }
}
