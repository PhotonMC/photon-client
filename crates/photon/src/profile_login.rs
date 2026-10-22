//! The account `photon play` joins an online-mode server with, taken from the environment the
//! launcher gives it (as vanilla's launcher gives `--uuid` and `--accessToken`). Without them the
//! client plays as a name only, which a server that checks accounts refuses.

use std::env;

use photon_net::account::{AccessToken, Account, DEFAULT_SESSION_HOST, parse_profile_id};

/// `PHOTON_UUID` and `PHOTON_ACCESS_TOKEN` together make an account; one of them alone is a
/// mistake worth saying so. `PHOTON_SESSION_HOST` points the join elsewhere (a test host).
pub(crate) fn from_environment(name: &str) -> Result<Option<Account>, String> {
    let (id, token) = (env::var("PHOTON_UUID"), env::var("PHOTON_ACCESS_TOKEN"));
    let (id, token) = match (id, token) {
        (Ok(id), Ok(token)) => (id, token),
        (Err(_), Err(_)) => return Ok(None),
        _ => return Err("PHOTON_UUID and PHOTON_ACCESS_TOKEN go together".to_owned()),
    };
    let profile_id =
        parse_profile_id(&id).ok_or_else(|| "PHOTON_UUID is not a profile id".to_owned())?;
    let host = env::var("PHOTON_SESSION_HOST").unwrap_or_else(|_| DEFAULT_SESSION_HOST.to_owned());
    Ok(Some(Account::new(
        name.to_owned(),
        profile_id,
        AccessToken::new(token),
        host,
    )))
}
