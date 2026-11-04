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
    let from_environment = from_environment(name)?;
    if from_environment.is_none() && is_offline(env::var("PHOTON_OFFLINE").ok().as_deref()) {
        return Ok(None);
    }
    choose(from_environment, stored)
}

/// `PHOTON_OFFLINE` (anything but `0`) keeps the stored sign-in out of a join: the test tools set
/// it so a client plays as the name they give it, not as whoever is signed in on this machine.
fn is_offline(value: Option<&str>) -> bool {
    value.is_some_and(|value| value != "0")
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

/// How long a resumed sign-in is reused: the resume is several requests to the account services
/// (seconds), and the access token it yields outlives this.
const RESUMED_FOR: std::time::Duration = std::time::Duration::from_secs(30 * 60);

/// The signed-in player, resumed once and kept for a while: a client opened by a link starts the
/// resume as it starts, so the join or sign-in that follows does not wait for the services (the
/// reference joins with the session it already holds). Callers that arrive during a resume wait
/// for it.
type Resumed = (std::time::Instant, Option<Account>);
static KEPT: std::sync::Mutex<Option<Resumed>> = std::sync::Mutex::new(None);

/// A player who has just signed in is the signed-in player: the next join uses this session
/// instead of resuming the stored one.
pub(crate) fn keep(account: Account) {
    let mut kept = KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *kept = Some((std::time::Instant::now(), Some(account)));
}

fn stored() -> Result<Option<Account>, String> {
    let mut kept = KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((at, account)) = kept.as_ref()
        && at.elapsed() < RESUMED_FOR
    {
        return Ok(account.clone());
    }
    let account = resume_stored()?;
    // Only a found sign-in is kept: a failure or no sign-in is asked again at the next join.
    if account.is_some() {
        *kept = Some((std::time::Instant::now(), account.clone()));
    }
    Ok(account)
}

/// The signed-in player, from the refresh token in the credential store.
fn resume_stored() -> Result<Option<Account>, String> {
    let client = signin::ClientId::from_environment();
    let host = session_host();
    let store = crate::keyring_store::os_store();
    let resumed = signin::resume(&signin::Hosts::default(), &client, (&host, store.as_ref()));
    match &resumed {
        Ok(Some(session)) => crate::signin_hint::write(&session.account.name),
        Ok(None) => crate::signin_hint::clear(),
        Err(_) => {}
    }
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
    fn the_offline_variable_is_on_unless_it_is_zero() {
        assert!(is_offline(Some("1")));
        assert!(!is_offline(Some("0")));
        assert!(!is_offline(None));
    }

    #[test]
    fn without_variables_the_signed_in_player_is_used() {
        let chosen = choose(None, || Ok(Some(account("Kept"))));
        assert_eq!(chosen.unwrap().unwrap().name, "Kept");
        assert_eq!(choose(None, || Ok(None)).unwrap(), None);
        assert_eq!(choose(None, || Err("no keyring".to_owned())).unwrap(), None);
    }
}
