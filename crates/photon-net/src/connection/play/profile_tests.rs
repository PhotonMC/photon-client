//! A player entity keeps the profile it was added with (`AbstractClientPlayer.playerInfo`).

use std::net::{TcpListener, TcpStream};

use photon_core::binary::Uuid;
use photon_core::ids::EntityId;
use photon_protocol::play::{AddEntityObservation, InfoAction, ProfileProperty};

use super::lifecycle::spawn_reader;
use super::*;

fn session() -> (PlaySession, TcpListener) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let stream = Transport::from(
        TcpStream::connect(listener.local_addr().expect("address")).expect("connect"),
    );
    let incoming = spawn_reader(
        stream.try_clone().expect("clone"),
        None,
        VersionRegistries::of(crate::connection::CONNECT_VERSION),
    );
    let session = PlaySession::assemble(
        stream,
        None,
        incoming,
        ConnectionState::default(),
        ("Tester", false),
    );
    (session, listener)
}

fn player(uuid: Uuid) -> AddEntityObservation {
    AddEntityObservation {
        entity_id: EntityId(7),
        uuid,
        entity_type_id: photon_data::entity_type::id("minecraft:player").expect("player"),
        x: 0.0,
        y: 64.0,
        z: 0.0,
        movement: (0.0, 0.0, 0.0),
        x_rot: 0.0,
        y_rot: 0.0,
        y_head_rot: 0.0,
        data: 0,
        applied_to_world: false,
        applied_rotation: None,
    }
}

fn add(name: &str, textures: &ProfileProperty) -> Vec<InfoAction> {
    vec![InfoAction::Add {
        name: name.to_owned(),
        properties: vec![textures.clone()],
    }]
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
    session.on_add_entity(player(uuid)).expect("added");
    session.social.player_infos.remove(&key);
    assert_eq!(session.player_name(uuid), Some("Npc"));
    assert_eq!(session.player_properties(uuid), [textures]);
    session.forget_entity_profiles(&[EntityId(7)]);
    assert_eq!(session.player_name(uuid), None);
}

#[test]
fn a_player_added_before_its_tab_entry_takes_the_entry_when_it_comes() {
    let (mut session, _server) = session();
    let uuid = Uuid::new(3, 4);
    let key = (uuid.most, uuid.least);
    let textures = ProfileProperty {
        name: "textures".to_owned(),
        value: "value".to_owned(),
        signature: None,
    };
    session.on_add_entity(player(uuid)).expect("added");
    session.apply_player_info(key, add("Npc", &textures));
    session.social.player_infos.remove(&key);
    assert_eq!(session.player_name(uuid), Some("Npc"));
    session.apply_player_info(key, add("Other", &textures));
    session.social.player_infos.remove(&key);
    assert_eq!(session.player_name(uuid), Some("Npc"));
}
