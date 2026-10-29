//! The account `photon play` joins an online-mode server with, taken from the environment the
//! launcher gives it (as vanilla's launcher gives `--uuid` and `--accessToken`). Without them the
//! client plays as a name only, which a server that checks accounts refuses.

use std::env;

use photon_net::account::{AccessToken, Account, DEFAULT_SESSION_HOST, parse_profile_id};
use photon_net::signin;

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
    let host = session_host();
    Ok(Some(Account::new(
        name.to_owned(),
        profile_id,
        AccessToken::new(token),
        host,
    )))
}

/// The account a join uses: the development variables first, then the signed-in player kept in
/// the credential store. Nobody signed in is `None`, and the client plays as a name only.
/// The name to join under and the account behind it, if any. A signed-in player joins under
/// their own name.
pub(crate) fn identity() -> Result<(String, Option<Account>), String> {
    let name = env::var("PHOTON_USER").unwrap_or_else(|_| "Photon".to_owned());
    let account = current(&name)?;
    let name = account
        .as_ref()
        .map_or(name, |account| account.name.clone());
    Ok((name, account))
}

fn current(name: &str) -> Result<Option<Account>, String> {
    choose(from_environment(name)?, stored)
}

fn choose(
    from_environment: Option<Account>,
    stored: impl FnOnce() -> Result<Option<Account>, String>,
) -> Result<Option<Account>, String> {
    if let Some(account) = from_environment {
        return Ok(Some(account));
    }
    // A sign-in that cannot be resumed (an outage, no credential store) leaves the player
    // joining as a name only; an online server answers that with its own message.
    Ok(stored().unwrap_or_else(|error| {
        eprintln!("the stored sign-in was not used: {error}");
        None
    }))
}

/// The signed-in player, from the refresh token in the credential store.
fn stored() -> Result<Option<Account>, String> {
    let client = signin::ClientId::from_environment();
    let host = session_host();
    let store = crate::keyring_store::os_store();
    let resumed = signin::resume(&signin::Hosts::default(), &client, (&host, store.as_ref()));
    resumed
        .map(|session| session.map(|session| session.account))
        .map_err(|error| error.to_string())
}

/// Where joins are posted: Mojang's session server unless a test host is named.
pub(crate) fn session_host() -> String {
    env::var("PHOTON_SESSION_HOST").unwrap_or_else(|_| DEFAULT_SESSION_HOST.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(name: &str) -> Account {
        Account::new(
            name.to_owned(),
            parse_profile_id("0123456789abcdeffedcba9876543210").unwrap(),
            AccessToken::new("t".to_owned()),
            DEFAULT_SESSION_HOST.to_owned(),
        )
    }

    #[test]
    fn the_development_variables_win_and_the_store_is_not_asked() {
        let chosen = choose(Some(account("Env")), || panic!("the store was asked"));
        assert_eq!(chosen.unwrap().unwrap().name, "Env");
    }

    #[test]
    fn without_variables_the_signed_in_player_is_used() {
        let chosen = choose(None, || Ok(Some(account("Kept"))));
        assert_eq!(chosen.unwrap().unwrap().name, "Kept");
        assert_eq!(choose(None, || Ok(None)).unwrap(), None);
        assert_eq!(choose(None, || Err("no keyring".to_owned())).unwrap(), None);
    }
}
