//! The two request shapes of the sign-in chain, with the checks every secret-carrying request
//! needs: the host is reached over TLS (or is this machine), nothing is redirected, and an error
//! status is an answer, not a failure.

use super::{REQUEST_TIMEOUT, SignInError};
use crate::account::is_this_machine;

/// A status and the body that came with it.
pub(super) struct Answer {
    pub(super) status: u16,
    pub(super) body: String,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .http_status_as_error(false)
        .max_redirects(0)
        .build()
        .into()
}

fn checked(host: &str, path: &str) -> Result<String, SignInError> {
    if !host.starts_with("https://") && !is_this_machine(host) {
        return Err(SignInError::InsecureHost);
    }
    Ok(format!("{}{path}", host.trim_end_matches('/')))
}

/// `application/x-www-form-urlencoded` pairs posted to `host` + `path`.
pub(super) fn post_form(
    (host, path): (&str, &str),
    fields: &[(&str, &str)],
) -> Result<Answer, SignInError> {
    let url = checked(host, path)?;
    let body = fields
        .iter()
        .map(|(name, value)| format!("{}={}", percent_encode(name), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    let mut response = agent()
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .send(body.as_bytes())
        .map_err(|_| SignInError::Unavailable)?;
    Ok(Answer {
        status: response.status().as_u16(),
        body: response.body_mut().read_to_string().unwrap_or_default(),
    })
}

/// Everything but the unreserved characters, as percent-escaped bytes.
fn percent_encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    #[test]
    fn form_values_are_percent_encoded() {
        assert_eq!(
            percent_encode("XboxLive.signin offline_access"),
            "XboxLive.signin%20offline_access"
        );
        assert_eq!(
            percent_encode("urn:ietf:params:oauth:grant-type:device_code"),
            "urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code"
        );
        assert_eq!(percent_encode("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn a_secret_is_not_sent_over_plain_http_to_another_host() {
        assert_eq!(
            post_form(("http://example.com", "/x"), &[]).err(),
            Some(SignInError::InsecureHost)
        );
    }

    /// A host on this machine that answers each request in turn, and what it was sent.
    pub(in crate::signin) fn stub_host(
        answers: Vec<String>,
    ) -> (String, std::thread::JoinHandle<Vec<String>>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let seen = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for answer in answers {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = Vec::new();
                let mut chunk = [0u8; 2048];
                // The request ends with its body: read until the declared length is in.
                loop {
                    let count = stream.read(&mut chunk).unwrap();
                    request.extend_from_slice(&chunk[..count]);
                    let text = String::from_utf8_lossy(&request).into_owned();
                    if let Some((head, body)) = text.split_once("\r\n\r\n") {
                        let length = head
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .and_then(|value| value.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if body.len() >= length {
                            break;
                        }
                    }
                    if count == 0 {
                        break;
                    }
                }
                stream.write_all(answer.as_bytes()).unwrap();
                requests.push(String::from_utf8_lossy(&request).into_owned());
            }
            requests
        });
        (host, seen)
    }

    /// A JSON answer with a status line.
    pub(in crate::signin) fn json_answer(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }
}
