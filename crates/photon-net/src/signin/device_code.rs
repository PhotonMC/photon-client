//! The OAuth 2.0 device authorization grant against the Microsoft identity platform: the player
//! opens a page, types a short code, and the client collects the tokens meanwhile.

use std::time::Duration;

use photon_core::json;

use super::http::{Answer, post_form};
use super::{ClientId, Hosts, SignInError};
use crate::account::AccessToken;

/// What the sign-in is allowed to do: Xbox Live sign-in, and a refresh token to stay signed in.
const SCOPE: &str = "XboxLive.signin offline_access";
const DEVICE_CODE_PATH: &str = "/consumers/oauth2/v2.0/devicecode";
const TOKEN_PATH: &str = "/consumers/oauth2/v2.0/token";
const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

/// A code the player types at the page, and what the client needs to collect the result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCode {
    /// What the player types.
    pub user_code: String,
    /// Where the player types it.
    pub verification_uri: String,
    /// How long the code lives.
    pub expires_in: Duration,
    /// The shortest time between two polls.
    pub interval: Duration,
    device_code: AccessToken,
}

/// The tokens a finished sign-in or a refresh gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicrosoftTokens {
    pub(crate) access: AccessToken,
    pub(crate) refresh: AccessToken,
    pub expires_in: Duration,
}

impl MicrosoftTokens {
    /// The refresh token, for the one place that stores it.
    pub fn refresh_token(&self) -> &AccessToken {
        &self.refresh
    }
}

/// The text the service words an error with, else its error name.
fn reason(body: &str, status: u16) -> String {
    let parsed = json::parse_object(body);
    parsed
        .as_ref()
        .and_then(|object| {
            object
                .text("error_description")
                .or_else(|| object.text("error"))
        })
        .unwrap_or_else(|| format!("Status: {status}"))
}

fn failure(answer: &Answer) -> SignInError {
    if answer.status >= 500 {
        return SignInError::Unavailable;
    }
    let error = json::parse_object(&answer.body).and_then(|object| object.text("error"));
    match error.as_deref() {
        Some("authorization_declined") => SignInError::Declined,
        Some("expired_token" | "bad_verification_code") => SignInError::Expired,
        // A refresh token that no longer works: the player signs in again.
        Some("invalid_grant") => SignInError::Expired,
        _ => SignInError::Rejected(reason(&answer.body, answer.status)),
    }
}

fn seconds(object: &json::JsonObject, key: &str, default: u64) -> Duration {
    let value = object.i32(key).and_then(|value| u64::try_from(value).ok());
    Duration::from_secs(value.unwrap_or(default))
}

/// Ask for a code to show the player.
pub fn request_device_code(hosts: &Hosts, client: &ClientId) -> Result<DeviceCode, SignInError> {
    let answer = post_form(
        (&hosts.login, DEVICE_CODE_PATH),
        &[("client_id", client.as_str()), ("scope", SCOPE)],
    )?;
    if answer.status != 200 {
        return Err(failure(&answer));
    }
    let object = json::parse_object(&answer.body).ok_or(SignInError::Unavailable)?;
    let (Some(user_code), Some(verification_uri), Some(device_code)) = (
        object.text("user_code"),
        object.text("verification_uri"),
        object.text("device_code"),
    ) else {
        return Err(SignInError::Rejected(
            "an incomplete device code".to_owned(),
        ));
    };
    Ok(DeviceCode {
        user_code,
        verification_uri,
        expires_in: seconds(&object, "expires_in", 900),
        interval: seconds(&object, "interval", 5),
        device_code: AccessToken::new(device_code),
    })
}

/// The tokens of an answer that carries them.
fn tokens(answer: &Answer) -> Result<MicrosoftTokens, SignInError> {
    let object = json::parse_object(&answer.body).ok_or(SignInError::Unavailable)?;
    let (Some(access), Some(refresh)) = (object.text("access_token"), object.text("refresh_token"))
    else {
        return Err(SignInError::Rejected("an answer without tokens".to_owned()));
    };
    Ok(MicrosoftTokens {
        access: AccessToken::new(access),
        refresh: AccessToken::new(refresh),
        expires_in: seconds(&object, "expires_in", 3600),
    })
}

/// Poll until the player has finished at the page, the code runs out or the player declines.
/// `wait` is how the caller sleeps, so a test does not.
pub fn poll_for_token(
    hosts: &Hosts,
    client: &ClientId,
    code: &DeviceCode,
    wait: &mut dyn FnMut(Duration),
) -> Result<MicrosoftTokens, SignInError> {
    let mut interval = code.interval.max(Duration::from_secs(1));
    let mut waited = Duration::ZERO;
    while waited < code.expires_in {
        wait(interval);
        waited += interval;
        let answer = post_form(
            (&hosts.login, TOKEN_PATH),
            &[
                ("grant_type", DEVICE_GRANT),
                ("client_id", client.as_str()),
                ("device_code", code.device_code.reveal()),
            ],
        )?;
        if answer.status == 200 {
            return tokens(&answer);
        }
        let error = json::parse_object(&answer.body).and_then(|object| object.text("error"));
        match error.as_deref() {
            Some("authorization_pending") => {}
            // The service asks for fewer polls: five seconds more, as the standard says.
            Some("slow_down") => interval += Duration::from_secs(5),
            _ => return Err(failure(&answer)),
        }
    }
    Err(SignInError::Expired)
}

/// Exchange a refresh token for new tokens.
pub fn refresh(
    hosts: &Hosts,
    client: &ClientId,
    refresh_token: &AccessToken,
) -> Result<MicrosoftTokens, SignInError> {
    let answer = post_form(
        (&hosts.login, TOKEN_PATH),
        &[
            ("grant_type", "refresh_token"),
            ("client_id", client.as_str()),
            ("refresh_token", refresh_token.reveal()),
            ("scope", SCOPE),
        ],
    )?;
    if answer.status != 200 {
        return Err(failure(&answer));
    }
    tokens(&answer)
}

#[cfg(test)]
mod tests {
    use super::super::http::tests::{json_answer, stub_host};
    use super::*;

    fn client() -> ClientId {
        ClientId::new("client-1".to_owned())
    }

    fn hosts(login: String) -> Hosts {
        Hosts {
            login,
            ..Hosts::default()
        }
    }

    const CODE_BODY: &str = r#"{"device_code":"DEV","user_code":"ABCD1234","verification_uri":"https://www.microsoft.com/link","expires_in":900,"interval":5}"#;
    const TOKEN_BODY: &str =
        r#"{"access_token":"ACC","refresh_token":"REF","expires_in":3599,"token_type":"Bearer"}"#;

    fn code() -> DeviceCode {
        DeviceCode {
            user_code: "ABCD1234".to_owned(),
            verification_uri: "https://www.microsoft.com/link".to_owned(),
            expires_in: Duration::from_secs(900),
            interval: Duration::from_secs(5),
            device_code: AccessToken::new("DEV".to_owned()),
        }
    }

    #[test]
    fn a_code_is_requested_with_the_client_and_the_scope() {
        let (host, seen) = stub_host(vec![json_answer("200 OK", CODE_BODY)]);
        let code = request_device_code(&hosts(host), &client()).expect("a code");
        assert_eq!(code.user_code, "ABCD1234");
        assert_eq!(code.verification_uri, "https://www.microsoft.com/link");
        assert_eq!(code.interval, Duration::from_secs(5));
        assert_eq!(code.expires_in, Duration::from_secs(900));
        let request = seen.join().unwrap().remove(0);
        assert!(request.starts_with("POST /consumers/oauth2/v2.0/devicecode "));
        assert!(request.contains("client_id=client-1&scope=XboxLive.signin%20offline_access"));
    }

    #[test]
    fn polling_waits_through_pending_and_slow_down_then_gives_the_tokens() {
        let (host, seen) = stub_host(vec![
            json_answer("400 Bad Request", r#"{"error":"authorization_pending"}"#),
            json_answer("400 Bad Request", r#"{"error":"slow_down"}"#),
            json_answer("200 OK", TOKEN_BODY),
        ]);
        let mut waits = Vec::new();
        let tokens = poll_for_token(&hosts(host.clone()), &client(), &code(), &mut |time| {
            waits.push(time.as_secs());
        })
        .expect("tokens");
        assert_eq!(waits, vec![5, 5, 10]);
        assert_eq!(tokens.access.reveal(), "ACC");
        assert_eq!(tokens.refresh.reveal(), "REF");
        assert_eq!(tokens.expires_in, Duration::from_secs(3599));
        let requests = seen.join().unwrap();
        assert!(requests[0].contains(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code&client_id=client-1&device_code=DEV"
        ));
    }

    #[test]
    fn a_declined_or_expired_code_ends_the_polling() {
        for (error, expected) in [
            ("authorization_declined", SignInError::Declined),
            ("expired_token", SignInError::Expired),
        ] {
            let body = format!(r#"{{"error":"{error}"}}"#);
            let (host, _) = stub_host(vec![json_answer("400 Bad Request", &body)]);
            let result = poll_for_token(&hosts(host.clone()), &client(), &code(), &mut |_| {});
            assert_eq!(result.err(), Some(expected));
        }
    }

    #[test]
    fn polling_stops_when_the_code_has_run_out() {
        let mut short = code();
        short.expires_in = Duration::from_secs(10);
        let (host, _) = stub_host(vec![
            json_answer("400 Bad Request", r#"{"error":"authorization_pending"}"#),
            json_answer("400 Bad Request", r#"{"error":"authorization_pending"}"#),
        ]);
        let result = poll_for_token(&hosts(host), &client(), &short, &mut |_| {});
        assert_eq!(result.err(), Some(SignInError::Expired));
    }

    #[test]
    fn a_refresh_token_is_exchanged_and_a_dead_one_means_signing_in_again() {
        let (host, seen) = stub_host(vec![
            json_answer("200 OK", TOKEN_BODY),
            json_answer(
                "400 Bad Request",
                r#"{"error":"invalid_grant","error_description":"AADSTS70000 expired"}"#,
            ),
        ]);
        let old = AccessToken::new("OLD".to_owned());
        let tokens = refresh(&hosts(host.clone()), &client(), &old).expect("tokens");
        assert_eq!(tokens.refresh.reveal(), "REF");
        assert_eq!(
            refresh(&hosts(host), &client(), &old).err(),
            Some(SignInError::Expired)
        );
        let requests = seen.join().unwrap();
        assert!(
            requests[0].contains("grant_type=refresh_token&client_id=client-1&refresh_token=OLD")
        );
    }

    #[test]
    fn other_refusals_keep_the_services_own_words() {
        let (host, _) = stub_host(vec![json_answer(
            "400 Bad Request",
            r#"{"error":"invalid_client","error_description":"unknown client"}"#,
        )]);
        assert_eq!(
            request_device_code(&hosts(host), &client()).err(),
            Some(SignInError::Rejected("unknown client".to_owned()))
        );
    }

    #[test]
    fn a_server_error_is_unavailable_and_tokens_are_not_printed() {
        let (host, _) = stub_host(vec![json_answer("503 Service Unavailable", "")]);
        assert_eq!(
            request_device_code(&hosts(host), &client()).err(),
            Some(SignInError::Unavailable)
        );
        let tokens = MicrosoftTokens {
            access: AccessToken::new("ACC".to_owned()),
            refresh: AccessToken::new("REF".to_owned()),
            expires_in: Duration::ZERO,
        };
        let shown = format!("{tokens:?}");
        assert!(!shown.contains("ACC") && !shown.contains("REF"));
    }
}
