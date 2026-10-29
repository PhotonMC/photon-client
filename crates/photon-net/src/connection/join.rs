//! `MinecraftSessionService.joinServer`: the client tells the session server which server hash
//! it is about to log in with, so that the server can ask the session server who it is.

use std::time::Duration;

use photon_core::json;
use photon_protocol::play::{ConnectionError, LoginFailure};

use crate::account::{Account, is_this_machine, undashed};

/// Post the join. Everything that goes wrong comes back as the failure vanilla words it.
pub(super) fn join_server(
    account: &Account,
    server_hash: &str,
    timeout: Duration,
) -> Result<(), ConnectionError> {
    let url = join_url(&account.session_host)?;
    let body = format!(
        "{{\"accessToken\":{},\"selectedProfile\":\"{}\",\"serverId\":{}}}",
        json_string(account.token.reveal()),
        undashed(account.profile_id),
        json_string(server_hash),
    );
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        // The token is in the body: it is not sent anywhere the host did not name.
        .max_redirects(0)
        .build()
        .into();
    let mut response = agent
        .post(&url)
        .header("Content-Type", "application/json; charset=utf-8")
        .send(body.as_bytes())
        .map_err(|_| ConnectionError::LoginFailed(LoginFailure::ServersUnavailable))?;
    let status = response.status().as_u16();
    if (200..300).contains(&status) {
        return Ok(());
    }
    let text = response.body_mut().read_to_string().unwrap_or_default();
    Err(ConnectionError::LoginFailed(failure_for(status, &text)))
}

/// `MinecraftClientHttpException.toAuthenticationException`: the error the body names comes
/// first, then the status.
fn failure_for(status: u16, body: &str) -> LoginFailure {
    let parsed = json::parse_object(body);
    let error = parsed.as_ref().and_then(|object| object.text("error"));
    let message = parsed
        .as_ref()
        .and_then(|object| object.text("errorMessage"))
        .filter(|message| !message.is_empty())
        .or_else(|| error.clone().filter(|error| !error.is_empty()))
        .unwrap_or_else(|| format!("Status: {status}"));
    match error.as_deref() {
        Some("ForbiddenOperationException") => LoginFailure::InvalidSession,
        Some("multiplayer.access.banned" | "FORCED_USERNAME_CHANGE") => LoginFailure::UserBanned,
        Some("InsufficientPrivilegesException") => LoginFailure::InsufficientPrivileges,
        _ if status == 401 => LoginFailure::InvalidSession,
        _ if status >= 500 => LoginFailure::ServersUnavailable,
        _ => LoginFailure::Other(message),
    }
}

/// The join address for a session host. The token travels in the request, so a host that is not
/// reached over TLS is only taken on this machine.
fn join_url(host: &str) -> Result<String, ConnectionError> {
    if !host.starts_with("https://") && !is_this_machine(host) {
        return Err(ConnectionError::InvalidArgument("session host needs https"));
    }
    Ok(format!(
        "{}/session/minecraft/join",
        host.trim_end_matches('/')
    ))
}

fn json_string(text: &str) -> String {
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
    fn answers_map_to_the_failures_vanilla_words() {
        let error = |name: &str| format!("{{\"error\":\"{name}\"}}");
        assert_eq!(
            failure_for(403, &error("ForbiddenOperationException")),
            LoginFailure::InvalidSession
        );
        assert_eq!(
            failure_for(403, &error("multiplayer.access.banned")),
            LoginFailure::UserBanned
        );
        assert_eq!(
            failure_for(403, &error("InsufficientPrivilegesException")),
            LoginFailure::InsufficientPrivileges
        );
        assert_eq!(failure_for(401, ""), LoginFailure::InvalidSession);
        assert_eq!(failure_for(503, ""), LoginFailure::ServersUnavailable);
        assert_eq!(
            failure_for(400, "{\"error\":\"Oops\",\"errorMessage\":\"Bad thing\"}"),
            LoginFailure::Other("Bad thing".to_owned())
        );
        assert_eq!(
            failure_for(404, ""),
            LoginFailure::Other("Status: 404".to_owned())
        );
    }

    #[test]
    fn plain_http_is_only_for_this_machine() {
        assert!(join_url("https://sessionserver.mojang.com").is_ok());
        assert!(join_url("http://127.0.0.1:9000/").is_ok());
        assert!(join_url("http://[::1]:9000").is_ok());
        assert!(join_url("http://localhost/x").is_ok());
        assert!(join_url("http://example.com").is_err());
        assert!(join_url("http://localhost.example.com").is_err());
        assert!(join_url("http://127.0.0.1.nip.io").is_err());
        assert!(join_url("http://127.0.0.1@example.com").is_err());
        assert!(join_url("http://localhost:1@example.com").is_err());
        assert!(join_url("http://[::1]@example.com").is_err());
    }

    #[test]
    fn strings_are_escaped() {
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000a\"");
    }

    /// A session host on this machine that answers one request, and what it was sent.
    fn one_answer(answer: &'static str) -> (String, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let seen = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut chunk = [0u8; 1024];
            while !String::from_utf8_lossy(&request).contains("serverId") {
                let count = stream.read(&mut chunk).unwrap();
                request.extend_from_slice(&chunk[..count]);
            }
            stream.write_all(answer.as_bytes()).unwrap();
            String::from_utf8_lossy(&request).into_owned()
        });
        (host, seen)
    }

    fn account_at(host: String) -> Account {
        use crate::account::AccessToken;
        Account::new(
            "Alex".to_owned(),
            photon_core::binary::Uuid::new(1, 2),
            AccessToken::new("tok".to_owned()),
            host,
        )
    }

    #[test]
    fn a_join_posts_the_token_the_profile_and_the_hash() {
        let (host, seen) = one_answer("HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n");
        join_server(&account_at(host), "-1a2b", Duration::from_secs(5)).expect("joined");
        let request = seen.join().unwrap();
        assert!(request.starts_with("POST /session/minecraft/join "));
        assert!(request.contains("\"accessToken\":\"tok\""));
        assert!(request.contains("\"selectedProfile\":\"00000000000000010000000000000002\""));
        assert!(request.contains("\"serverId\":\"-1a2b\""));
    }

    #[test]
    fn a_refused_join_is_the_failure_the_answer_names() {
        let body = "{\"error\":\"ForbiddenOperationException\"}";
        let answer = Box::leak(
            format!(
                "HTTP/1.1 403 Forbidden\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .into_boxed_str(),
        );
        let (host, seen) = one_answer(answer);
        let result = join_server(&account_at(host), "x", Duration::from_secs(5));
        seen.join().unwrap();
        assert!(matches!(
            result,
            Err(ConnectionError::LoginFailed(LoginFailure::InvalidSession))
        ));
    }
}
