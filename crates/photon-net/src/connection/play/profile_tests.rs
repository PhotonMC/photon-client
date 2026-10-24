//! A player entity keeps the profile it was added with (`AbstractClientPlayer.playerInfo`).

use std::net::{TcpListener, TcpStream};

use photon_core::binary::Uuid;
use photon_protocol::play::{InfoAction, ProfileProperty};

use super::lifecycle::spawn_reader;
use super::*;

fn session() -> (PlaySession, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let stream = Transport::from(
        TcpStream::connect(listener.local_addr().expect("address")).expect("connect"),
    );
    let incoming = spawn_reader(stream.try_clone().expect("clone"), None);
    let session = PlaySession::assemble(
        stream,
        None,
        incoming,
        ConnectionState::default(),
        ("Tester", false),
    );
    (session, listener)
}

#[test]
fn a_player_keeps_its_name_and_skin_when_the_tab_list_drops_the_entry() {
    let (mut session, _server) = session();
    let uuid = Uuid::new(3, 4);
    let key = (uuid.most, uuid.least);
    let textures = ProfileProperty {
        name: "textures".to_owned(),
        value: "value".to_owned(),
        signature: None,
    };
    session.apply_player_info(
        key,
        vec![InfoAction::Add {
            name: "Npc".to_owned(),
            properties: vec![textures.clone()],
        }],
    );
    session.remember_entity_profile(key);
    session.social.player_infos.remove(&key);
    assert_eq!(session.player_name(uuid), Some("Npc"));
    assert_eq!(session.player_properties(uuid), [textures]);
}
