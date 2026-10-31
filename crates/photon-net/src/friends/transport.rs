//! How a request reaches the friends service: a plain trait, and the HTTPS transport that
//! carries the account token.

use std::time::Duration;

use crate::account::{AccessToken, Account, is_this_machine};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
}

/// One request to the services host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: Method,
    pub path: &'static str,
    pub body: Option<String>,
    /// The `ETag` of the answer the caller holds, sent as `If-None-Match`.
    pub etag: Option<String>,
}

/// The answer to a request, whatever its status.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub etag: Option<String>,
    pub retry_after: Option<Duration>,
}

/// The service could not be reached or its answer could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportError;

pub trait Transport {
    fn send(&self, request: &Request) -> Result<Reply, TransportError>;
}

/// `MinecraftClient.parseRetryAfter`: whole seconds above zero, anything else is no hint.
pub(super) fn parse_retry_after(value: Option<&str>) -> Option<Duration> {
    let seconds: u64 = value?.trim().parse().ok()?;
    (seconds > 0).then(|| Duration::from_secs(seconds))
}

/// The services host over TLS (or this machine, for a development stub), with the account token
/// as the bearer. Nothing is redirected, so the token goes to the host it was meant for.
pub struct HttpTransport {
    host: String,
    token: AccessToken,
}

impl HttpTransport {
    /// The reference's connect and read timeout.
    const TIMEOUT: Duration = Duration::from_secs(5);

    pub fn new(host: String, token: AccessToken) -> Self {
        Self { host, token }
    }

    /// The services of an account's token, at `host` (`Hosts::services`).
    pub fn for_account(account: &Account, host: &str) -> Self {
        Self::new(host.to_owned(), account.token.clone())
    }
}

impl Transport for HttpTransport {
    fn send(&self, request: &Request) -> Result<Reply, TransportError> {
        if !self.host.starts_with("https://") && !is_this_machine(&self.host) {
            return Err(TransportError);
        }
        let url = format!("{}{}", self.host.trim_end_matches('/'), request.path);
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Self::TIMEOUT))
            .http_status_as_error(false)
            .max_redirects(0)
            .build()
            .into();
        let bearer = format!("Bearer {}", self.token.reveal());
        let mut builder = ureq::http::Request::builder()
            .method(match request.method {
                Method::Get => "GET",
                Method::Post => "POST",
                Method::Put => "PUT",
            })
            .uri(&url)
            .header("Authorization", &bearer);
        if let Some(etag) = &request.etag {
            builder = builder.header("If-None-Match", etag);
        }
        let body = request.body.clone().unwrap_or_default();
        if request.method != Method::Get {
            builder = builder.header("Content-Type", "application/json; charset=utf-8");
        }
        let built = builder.body(body).map_err(|_| TransportError)?;
        let mut response = agent.run(built).map_err(|_| TransportError)?;
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        };
        Ok(Reply {
            status: response.status().as_u16(),
            etag: header("ETag"),
            retry_after: parse_retry_after(header("Retry-After").as_deref()),
            body: response.body_mut().read_to_string().unwrap_or_default(),
        })
    }
}
