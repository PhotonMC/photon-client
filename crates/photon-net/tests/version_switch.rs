//! One process joining servers of different versions at one address, run by hand: it waits for
//! each version in `PHOTON_VERSION_SEQUENCE` (`26.2,26.1,26.2`) to answer at `PHOTON_LIVE_ADDRESS`,
//! joins it, and checks what the session numbered with: the version, the player's attributes, the
//! block under the player and the chunks. The servers are swapped by hand between joins.

use std::env;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use photon_core::version::Version;
use photon_game::server_status::ServerStatus;
use photon_net::connection::{PlaySession, query_status_and_ping};

fn var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{name} is not set"))
}

/// The version the server at `address` answers as, once it answers.
fn wait_for(address: SocketAddr, version: Version, within: Duration) {
    let until = Instant::now() + within;
    while Instant::now() < until {
        let protocol = query_status_and_ping(address, Duration::from_secs(2))
            .ok()
            .and_then(|status| ServerStatus::parse(&status.status_response))
            .and_then(|status| status.version)
            .map(|(_, protocol)| protocol);
        if protocol == Some(version.protocol()) {
            return;
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    panic!("no {} server answered within {within:?}", version.name());
}

#[test]
#[ignore = "needs vanilla servers of each version, swapped by hand"]
fn joins_each_version_in_turn_without_a_restart() {
    let address: SocketAddr = var("PHOTON_LIVE_ADDRESS").parse().expect("an address");
    for name in var("PHOTON_VERSION_SEQUENCE").split(',') {
        let version = Version::from_name(name).expect("a supported version");
        wait_for(address, version, Duration::from_secs(300));
        let mut session = PlaySession::connect(
            address,
            "VersionTest",
            Duration::from_secs(20),
            false,
            photon_net::configuration::ClientInformation::default(),
        )
        .expect("joined");
        assert_eq!(session.versions().version(), version);
        for _ in 0..200 {
            if session.is_loaded() {
                break;
            }
            session.wait(Duration::from_millis(100)).expect("play");
            session.tick_level_load().expect("load");
        }
        assert!(session.is_loaded(), "{name}: the level did not load");
        // A few more packets: the attributes and the entity data come after the level.
        for _ in 0..20 {
            session.wait(Duration::from_millis(100)).expect("play");
        }
        let player = session.player().expect("the player");
        let speed = photon_data::item::effect::attribute_id("minecraft:movement_speed")
            .expect("movement_speed");
        let held = player
            .attributes()
            .iter()
            .find(|attribute| attribute.attribute_id() == speed)
            .expect("movement_speed was sent");
        let position = player.position();
        let below = session.world().block_state_or_air(
            position.x.floor() as i32,
            (position.y - 0.5).floor() as i32,
            position.z.floor() as i32,
        );
        let (block, _) =
            photon_data::block::registry::block_for_state(below).expect("a block below");
        println!(
            "version={name} movement_speed={} below={} chunks_loaded=true",
            held.base_value(),
            block.name
        );
        assert!((held.base_value() - 0.1).abs() < 1e-6, "{name}: speed");
        assert!(block.name != "minecraft:air", "{name}: nothing below");
        drop(session);
    }
}
