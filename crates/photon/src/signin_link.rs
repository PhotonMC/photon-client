//! A `photon://signin?state=...` link (ADR 0023): the website's browser asks to be signed in. The
//! client asks the player first, then proves the profile to the backend, gets a web code bound to
//! the browser's state and opens the website's own address with it. The address comes from the
//! player's configuration, never from the link.

use std::process::Command;

use photon_net::backend::{BackendError, signin_url};

/// How the attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// The browser was opened with the code.
    Opened,
    NotSignedIn,
    /// The resumed account is not the player the question named.
    Changed,
    /// The website address is missing or not secure.
    NoWebsite,
    Unavailable,
}

/// The translation key of what to tell the player of an outcome.
pub(crate) fn message_key(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::Opened => "photon.signin.opened",
        Outcome::NotSignedIn => "photon.signin.not_signed_in",
        Outcome::Changed => "photon.signin.changed",
        Outcome::NoWebsite => "photon.signin.no_website",
        Outcome::Unavailable => "photon.signin.unavailable",
    }
}

/// The name of the signed-in player, for the question; nothing when nobody is signed in. The
/// name kept from the last sign-in comes at once and the account services are asked meanwhile
/// (so the sign-in that follows finds the session ready); without one, the services are waited on.
pub(crate) fn signed_in_name(answer: impl FnOnce(Option<String>)) {
    match crate::signin_hint::read() {
        Some(name) => {
            answer(Some(name));
            let _ = resumed_name();
        }
        None => answer(resumed_name()),
    }
}

/// The name of the signed-in player from the account services, waited on.
pub(crate) fn resumed_name() -> Option<String> {
    let (_, account) = crate::profile_login::identity().ok()?;
    account.map(|account| account.name)
}

/// Signs in to the backend with the player's own account, gets the web code for `state` and
/// opens the browser. Nothing here is logged: the address carries the code.
pub(crate) fn run(state: &str, shown: Option<&str>) -> Outcome {
    let Some(website) = crate::links_config::website() else {
        return Outcome::NoWebsite;
    };
    // The address is checked before the Mojang join the sign-in costs.
    if signin_url(&website, "x").is_err() {
        return Outcome::NoWebsite;
    }
    let Some((_, Some(account))) = crate::profile_login::identity().ok() else {
        return Outcome::NotSignedIn;
    };
    // The question named a player; only that player is signed in.
    if shown.is_some_and(|shown| account.name != shown) {
        return Outcome::Changed;
    }
    let code = crate::links_config::backend()
        .unwrap_or(Err(BackendError::Unavailable))
        .and_then(|backend| {
            let session = backend.sign_in(&account)?;
            backend.web_code(&session, state)
        });
    let Ok(code) = code else {
        return Outcome::Unavailable;
    };
    match signin_url(&website, &code).map(|url| open(&url)) {
        Ok(true) => Outcome::Opened,
        Ok(false) => Outcome::Unavailable,
        Err(_) => Outcome::NoWebsite,
    }
}

/// Opens an address in the default browser, with no shell. `PHOTON_OPEN_WITH` names another
/// program to hand it to (development only: tests read the address from it).
pub(crate) fn open(url: &str) -> bool {
    let program = std::env::var("PHOTON_OPEN_WITH").unwrap_or_else(|_| default_opener().to_owned());
    Command::new(program)
        .arg(url)
        .status()
        .is_ok_and(|status| status.success())
}

fn default_opener() -> &'static str {
    if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    }
}
