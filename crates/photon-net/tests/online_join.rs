//! A join against a server that checks accounts, run by hand: it needs a server and a session
//! host to talk to, named by the environment (`PHOTON_LIVE_ADDRESS`, `PHOTON_LIVE_NAME`,
//! `PHOTON_UUID`, `PHOTON_ACCESS_TOKEN`, `PHOTON_SESSION_HOST`).

use std::env;
use std::time::Duration;

use photon_net::account::{AccessToken, Account, parse_profile_id};
use photon_net::connection::PlaySession;

fn var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{name} is not set"))
}

#[test]
#[ignore = "needs a live online-mode server and a session host"]
fn joins_a_server_that_checks_accounts() {
    let name = var("PHOTON_LIVE_NAME");
    let account = Account::new(
        name,
        parse_profile_id(&var("PHOTON_UUID")).expect("a profile id"),
        AccessToken::new(var("PHOTON_ACCESS_TOKEN")),
        var("PHOTON_SESSION_HOST"),
    );
    let address = var("PHOTON_LIVE_ADDRESS").parse().expect("an address");
    let information = photon_net::configuration::ClientInformation::default();
    let mut session = PlaySession::connect_signed_in(
        address,
        account,
        Duration::from_secs(20),
        false,
        information,
    )
    .expect("joined");
    for _ in 0..100 {
        if session.is_loaded() {
            break;
        }
        session.wait(Duration::from_millis(100)).expect("play");
    }
    assert!(session.is_loaded(), "the level did not load");
}
