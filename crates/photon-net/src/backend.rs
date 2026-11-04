//! Photon's backend (ADR 0022): the server directory and, for a signed-in player, a Photon session
//! and launch codes. It holds no token of the player's: identity is proven by joining a nonce the
//! backend issued, as an online-mode server's join does. Without a backend address the client does
//! none of this.

use std::time::Duration;

use photon_core::binary::Uuid;
use photon_core::json::{JsonObject, JsonValue, parse, parse_object};

use crate::account::{Account, is_this_machine, undashed};
use crate::connection::prove_identity;

/// How long a call to the backend, or the join it asks for, may take.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Why a call to the backend failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    /// The address is not `https`, and is not this machine.
    Insecure,
    /// The backend did not answer, or answered something else than it should.
    Unavailable,
    /// The backend refused: a bad session, an unknown or spent code.
    Refused,
    /// The player's join at Mojang failed.
    Identity,
}

/// What the loading screen, the game page and the home screen show of a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryServer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub description: String,
    pub version: String,
    pub images: Vec<String>,
    pub players: i32,
}

/// What an exchanged launch code names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub address: String,
    /// The profile the code was issued for, as 32 hex digits.
    pub uuid: String,
}

/// A Photon session. It lives in memory only and is proven again on each start.
#[derive(Debug, Clone)]
pub struct Session(String);

pub struct Backend {
    base: String,
    agent: ureq::Agent,
}

impl Backend {
    /// A backend at `base`. Plain `http` is only taken for this machine.
    pub fn new(base: &str) -> Result<Self, BackendError> {
        if !base.starts_with("https://") && !is_this_machine(base) {
            return Err(BackendError::Insecure);
        }
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .http_status_as_error(false)
            .max_redirects(0)
            .build()
            .into();
        Ok(Self {
            base: base.trim_end_matches('/').to_owned(),
            agent,
        })
    }

    /// One call: the answer's body when the status is a success.
    fn call(
        &self,
        path: &str,
        auth: Option<&Session>,
        body: Option<&str>,
    ) -> Result<String, BackendError> {
        let url = format!("{}{path}", self.base);
        let mut response = match (body, auth) {
            (None, _) => self.agent.get(&url).call(),
            (Some(body), auth) => {
                let mut request = self
                    .agent
                    .post(&url)
                    .header("Content-Type", "application/json");
                if let Some(Session(token)) = auth {
                    request = request.header("Authorization", &format!("Bearer {token}"));
                }
                request.send(body.as_bytes())
            }
        }
        .map_err(|_| BackendError::Unavailable)?;
        let status = response.status().as_u16();
        let text = response
            .body_mut()
            .read_to_string()
            .map_err(|_| BackendError::Unavailable)?;
        match status {
            200..=299 => Ok(text),
            400..=499 => Err(BackendError::Refused),
            _ => Err(BackendError::Unavailable),
        }
    }

    /// The directory. It needs no session.
    pub fn servers(&self) -> Result<Vec<DirectoryServer>, BackendError> {
        let text = self.call("/v1/servers", None, None)?;
        let Ok(JsonValue::Array(items)) = parse(&text) else {
            return Err(BackendError::Unavailable);
        };
        Ok(items
            .iter()
            .filter_map(|item| match item {
                JsonValue::Object(object) => directory_server(object),
                _ => None,
            })
            .collect())
    }

    /// Proves the account's identity and gets a session: ask for a challenge, join its nonce at
    /// Mojang, present the secret. The nonce comes from the backend this client asked, never from
    /// a link.
    pub fn sign_in(&self, account: &Account) -> Result<Session, BackendError> {
        let challenge = object(&self.call("/v1/identity/challenge", None, Some(""))?)?;
        let (nonce, secret) = (field(&challenge, "nonce")?, field(&challenge, "secret")?);
        prove_identity(account, &nonce, TIMEOUT).map_err(|_| BackendError::Identity)?;
        let body = format!(
            "{{\"name\":{},\"nonce\":{},\"secret\":{}}}",
            quoted(&account.name),
            quoted(&nonce),
            quoted(&secret)
        );
        let verified = object(&self.call("/v1/identity/verify", None, Some(&body))?)?;
        Ok(Session(field(&verified, "session")?))
    }

    /// A launch code for a server of the directory.
    pub fn launch(&self, session: &Session, server_id: &str) -> Result<String, BackendError> {
        let body = format!("{{\"server\":{}}}", quoted(server_id));
        let answer = object(&self.call("/v1/launch", Some(session), Some(&body))?)?;
        field(&answer, "code")
    }

    /// A web code for the browser that started with `state` (ADR 0023): the player's own session
    /// asks for it, and it is opened in the browser, never the session itself.
    pub fn web_code(&self, session: &Session, state: &str) -> Result<String, BackendError> {
        let body = format!("{{\"state\":{}}}", quoted(state));
        let answer = object(&self.call("/v1/web/code", Some(session), Some(&body))?)?;
        field(&answer, "code")
    }

    /// Spends a launch code: the address to join and the profile the code belongs to.
    pub fn exchange(&self, code: &str) -> Result<Launch, BackendError> {
        let body = format!("{{\"code\":{}}}", quoted(code));
        let answer = object(&self.call("/v1/launch/exchange", None, Some(&body))?)?;
        Ok(Launch {
            address: field(&answer, "address")?,
            uuid: field(&answer, "uuid")?,
        })
    }
}

/// The address of the website's sign-in page for a web code: `https` (or this machine, for
/// development), and the code its only parameter. The base comes from the player's own
/// configuration, never from a link.
pub fn signin_url(website: &str, code: &str) -> Result<String, BackendError> {
    if !website.starts_with("https://") && !is_this_machine(website) {
        return Err(BackendError::Insecure);
    }
    let base = website.trim_end_matches('/');
    Ok(format!("{base}/signin?code={code}"))
}

/// Whether the profile a launch code was issued for is the one signed in. Anything else must be
/// asked about before joining (ADR 0020), never joined silently.
pub fn is_same_profile(launch_uuid: &str, signed_in: Uuid) -> bool {
    launch_uuid.eq_ignore_ascii_case(&undashed(signed_in))
}

fn quoted(text: &str) -> String {
    crate::account::json_string(text)
}

fn object(text: &str) -> Result<JsonObject, BackendError> {
    parse_object(text).ok_or(BackendError::Unavailable)
}

fn field(object: &JsonObject, name: &str) -> Result<String, BackendError> {
    object.text(name).ok_or(BackendError::Unavailable)
}

fn directory_server(object: &JsonObject) -> Option<DirectoryServer> {
    let images = object
        .array("images")
        .unwrap_or_default()
        .iter()
        .filter_map(|image| match image {
            JsonValue::String(units) => String::from_utf16(units).ok(),
            _ => None,
        })
        .collect();
    Some(DirectoryServer {
        id: object.text("id")?,
        name: object.text("name")?,
        address: object.text("address")?,
        description: object.text("description").unwrap_or_default(),
        version: object.text("version").unwrap_or_default(),
        images,
        players: object.i32("players").unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_address_is_taken_only_for_this_machine() {
        assert!(Backend::new("https://photon.example").is_ok());
        assert!(Backend::new("http://127.0.0.1:8080").is_ok());
        assert!(Backend::new("http://localhost:8080/").is_ok());
        assert_eq!(
            Backend::new("http://photon.example").err(),
            Some(BackendError::Insecure)
        );
        assert_eq!(
            Backend::new("ftp://127.0.0.1").err(),
            Some(BackendError::Insecure)
        );
    }

    #[test]
    fn a_directory_entry_is_read_with_its_optional_parts() {
        let text = r#"{"id":"a","name":"A","address":"h:1","images":["u1","u2"],"players":4}"#;
        let server = directory_server(&object(text).expect("object")).expect("entry");
        assert_eq!((server.id.as_str(), server.players), ("a", 4));
        assert_eq!(server.images, ["u1", "u2"]);
        assert_eq!(server.description, "");
        assert!(directory_server(&object(r#"{"id":"a"}"#).expect("object")).is_none());
    }

    #[test]
    fn a_directory_entry_with_videos_still_reads() {
        let text = r#"{"id":"a","name":"A","address":"h:1","videos":["dQw4w9WgXcQ"],"players":1}"#;
        let server = directory_server(&object(text).expect("object")).expect("entry");
        assert_eq!((server.id.as_str(), server.players), ("a", 1));
    }

    #[test]
    fn the_profile_of_a_code_must_be_the_signed_in_one() {
        let signed_in = Uuid::new(0xb280_53e5_d731_4bc5, 0x90cc_1f52_6ca1_fc81);
        assert!(is_same_profile(
            "b28053e5d7314bc590cc1f526ca1fc81",
            signed_in
        ));
        assert!(is_same_profile(
            "B28053E5D7314BC590CC1F526CA1FC81",
            signed_in
        ));
        assert!(!is_same_profile(
            "00000000000000000000000000000001",
            signed_in
        ));
        // The dashed form is not what the backend returns; it does not match silently.
        assert!(!is_same_profile(
            "b28053e5-d731-4bc5-90cc-1f526ca1fc81",
            signed_in
        ));
    }

    #[test]
    fn the_sign_in_address_is_secure_and_carries_only_the_code() {
        assert_eq!(
            signin_url("https://photon.example/", "abc"),
            Ok("https://photon.example/signin?code=abc".to_owned())
        );
        assert_eq!(
            signin_url("http://127.0.0.1:3000", "abc"),
            Ok("http://127.0.0.1:3000/signin?code=abc".to_owned())
        );
        assert_eq!(
            signin_url("http://photon.example", "abc"),
            Err(BackendError::Insecure)
        );
    }
}
