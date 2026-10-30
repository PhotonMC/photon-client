//! The service against a loopback host: what it reports, when it downloads, and that it asks
//! nothing without a key.

use std::time::{Duration, Instant};

use photon_api::{UpdateForServer, UpdateState};

use super::host_tests::host;
use super::tests::{folder, manifest, public, test_key, to_hex};
use super::*;

fn config(base: &str, keys: Vec<[u8; 32]>, name: &str, settings: Settings) -> Config {
    Config {
        host: base.to_owned(),
        keys,
        platform: "t".to_owned(),
        current: "1.3.0".to_owned(),
        pending: folder(name).join("pending"),
        settings,
        interval: CHECK_INTERVAL,
    }
}

/// Polls until `done` accepts a state, with every state seen on the way.
fn wait_for(service: &Service, done: impl Fn(&UpdateState) -> bool) -> Vec<UpdateState> {
    let end = Instant::now() + Duration::from_secs(20);
    let mut seen = Vec::new();
    while Instant::now() < end {
        for state in service.poll() {
            let finished = done(&state);
            seen.push(state);
            if finished {
                return seen;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("the service did not get there; saw {seen:?}");
}

/// The files of a release 1.4.0 for platform `t` with a note and protocols, signed by `pair`.
fn files(pair: &ring::signature::Ed25519KeyPair) -> Vec<(&'static str, Vec<u8>)> {
    let text = format!(
        "{}notes=Faster chunks\nprotocols=775,776\n",
        manifest(b"archive")
    );
    let signature = to_hex(pair.sign(text.as_bytes()).as_ref());
    vec![
        ("/photon-t.manifest", text.into_bytes()),
        ("/photon-t.manifest.sig", signature.into_bytes()),
        ("/photon-1.4.0.tar.gz", b"archive".to_vec()),
    ]
}

#[test]
fn settings_are_read_and_a_typo_is_never_taken_for_a_choice() {
    assert_eq!(Settings::parse("").unwrap(), Settings::default());
    let parsed =
        Settings::parse("# mine\nauto_check = false\nauto_download=false\nchannel=stable\n");
    assert_eq!(
        parsed,
        Ok(Settings {
            auto_check: false,
            auto_download: false
        })
    );
    for bad in [
        "auto_check=yes",
        "channel=nightly",
        "autocheck=true",
        "oops",
    ] {
        assert!(Settings::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn without_a_key_nothing_is_asked_and_a_manual_check_says_why() {
    // Nothing listens here: any connection attempt would fail differently.
    let service = Service::start(config(
        "http://127.0.0.1:1",
        Vec::new(),
        "nokey",
        Settings::default(),
    ));
    assert_eq!(service.state(), UpdateState::Idle);
    service.check_now();
    let seen = wait_for(&service, |state| {
        matches!(state, UpdateState::Failed { .. })
    });
    assert!(matches!(
        seen.last(),
        Some(UpdateState::Failed { reason }) if reason.contains("no release key")
    ));
}

#[test]
fn a_newer_release_is_found_downloaded_and_stored_and_answers_for_its_protocols() {
    let pair = test_key();
    let (base, thread) = host(files(&pair), 3);
    let service = Service::start(config(
        &base,
        vec![public(&pair)],
        "auto",
        Settings::default(),
    ));
    let seen = wait_for(&service, |state| matches!(state, UpdateState::Ready { .. }));
    thread.join().unwrap();
    assert_eq!(seen.first(), Some(&UpdateState::Checking));
    assert!(seen.contains(&UpdateState::Available {
        version: "1.4.0".to_owned(),
        notes: "Faster chunks".to_owned(),
        size: 7
    }));
    assert!(seen.contains(&UpdateState::Downloading { done: 7, total: 7 }));
    assert_eq!(
        service.state(),
        UpdateState::Ready {
            version: "1.4.0".to_owned()
        }
    );
    assert_eq!(
        service.update_for_protocol(776),
        UpdateForServer::Staged {
            version: "1.4.0".to_owned()
        }
    );
    assert_eq!(service.update_for_protocol(999), UpdateForServer::None);
}

#[test]
fn with_auto_download_off_the_release_waits_until_it_is_asked_for() {
    let pair = test_key();
    let settings = Settings {
        auto_check: true,
        auto_download: false,
    };
    let (base, thread) = host(files(&pair), 3);
    let service = Service::start(config(&base, vec![public(&pair)], "manual", settings));
    wait_for(&service, |state| {
        matches!(state, UpdateState::Available { .. })
    });
    assert_eq!(
        service.update_for_protocol(775),
        UpdateForServer::Available {
            version: "1.4.0".to_owned()
        }
    );
    service.download();
    wait_for(&service, |state| matches!(state, UpdateState::Ready { .. }));
    thread.join().unwrap();
}

#[test]
fn a_failed_check_is_reported_and_leaves_the_service_usable() {
    let pair = test_key();
    let (base, thread) = host(Vec::new(), 1);
    let service = Service::start(config(
        &base,
        vec![public(&pair)],
        "missing",
        Settings::default(),
    ));
    let seen = wait_for(&service, |state| {
        matches!(state, UpdateState::Failed { .. })
    });
    thread.join().unwrap();
    assert!(matches!(
        seen.last(),
        Some(UpdateState::Failed { reason }) if reason.contains("no release")
    ));
}
