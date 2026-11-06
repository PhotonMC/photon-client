//! Packet numbering. The id constants carry 26.2's numbering; 26.3 added packets and moved
//! some, so the ids on the wire are translated here, in both directions.

use photon_core::version::Version;

use crate::play::{
    CONFIG_CLIENTBOUND_POST_EFFECTS_PACKET_ID, PLAY_CLIENTBOUND_ADD_TRANSIENT_BLOCK_PACKET_ID,
    PLAY_CLIENTBOUND_POST_EFFECTS_PACKET_ID, PLAY_CLIENTBOUND_SWING_ANIMATION_PACKET_ID,
    PLAY_SERVERBOUND_PUNCH_PACKET_ID,
};

/// Where 26.3 put the play packets 26.2 does not have (`add_transient_block`, `post_effects`,
/// `swing_animation`), by wire id, with the constant of each.
const PLAY_CLIENTBOUND_ADDED: [(i32, i32); 3] = [
    (37, PLAY_CLIENTBOUND_ADD_TRANSIENT_BLOCK_PACKET_ID),
    (83, PLAY_CLIENTBOUND_POST_EFFECTS_PACKET_ID),
    (123, PLAY_CLIENTBOUND_SWING_ANIMATION_PACKET_ID),
];

/// The same for the configuration phase (`post_effects`).
const CONFIG_CLIENTBOUND_ADDED: [(i32, i32); 1] = [(10, CONFIG_CLIENTBOUND_POST_EFFECTS_PACKET_ID)];

fn translate_added(wire: i32, added: &[(i32, i32)]) -> i32 {
    if let Some((_, canonical)) = added.iter().find(|(at, _)| *at == wire) {
        return *canonical;
    }
    wire - added.iter().filter(|(at, _)| *at < wire).count() as i32
}

/// A clientbound play packet's constant and payload, split from the body it arrived in.
pub fn split_clientbound_play(
    body: &[u8],
    version: Version,
) -> Result<(i32, &[u8]), crate::play::ConnectionError> {
    let (wire, payload) = crate::play::split_packet(body)?;
    Ok((clientbound_play_id(version, wire), payload))
}

/// The same for a clientbound configuration packet.
pub fn split_clientbound_config(
    body: &[u8],
    version: Version,
) -> Result<(i32, &[u8]), crate::play::ConnectionError> {
    let (wire, payload) = crate::play::split_packet(body)?;
    Ok((clientbound_config_id(version, wire), payload))
}

/// The constant of a clientbound play packet that arrived with this wire id.
pub fn clientbound_play_id(version: Version, wire: i32) -> i32 {
    if version >= Version::V26_3 {
        translate_added(wire, &PLAY_CLIENTBOUND_ADDED)
    } else {
        wire
    }
}

/// The constant of a clientbound configuration packet that arrived with this wire id.
pub fn clientbound_config_id(version: Version, wire: i32) -> i32 {
    if version >= Version::V26_3 {
        translate_added(wire, &CONFIG_CLIENTBOUND_ADDED)
    } else {
        wire
    }
}

/// The wire id of a clientbound play packet given by its constant (the way a server sends it).
pub fn clientbound_play_wire_id(version: Version, id: i32) -> i32 {
    if version >= Version::V26_3 {
        wire_of(id, &PLAY_CLIENTBOUND_ADDED)
    } else {
        id
    }
}

/// The wire id of a clientbound configuration packet given by its constant.
pub fn clientbound_config_wire_id(version: Version, id: i32) -> i32 {
    if version >= Version::V26_3 {
        wire_of(id, &CONFIG_CLIENTBOUND_ADDED)
    } else {
        id
    }
}

fn wire_of(id: i32, added: &[(i32, i32)]) -> i32 {
    if let Some((at, _)) = added.iter().find(|(_, canonical)| *canonical == id) {
        return *at;
    }
    // The packets the version added push the later ones up, in the order they were added.
    let mut wire = id;
    for (at, _) in added {
        if *at <= wire {
            wire += 1;
        }
    }
    wire
}

/// The wire id of a serverbound play packet given by its constant: 26.3 moved the swing packet
/// (now `punch`) up front and the packets in between up by one.
pub fn serverbound_play_wire_id(version: Version, id: i32) -> i32 {
    if version < Version::V26_3 {
        return id;
    }
    match id {
        PLAY_SERVERBOUND_PUNCH_PACKET_ID => 46,
        46..=62 => id + 1,
        _ => id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clientbound_play_ids_of_26_3_map_back_to_26_2s() {
        let v = Version::V26_3;
        assert_eq!(clientbound_play_id(v, 36), 36);
        assert_eq!(
            clientbound_play_id(v, 37),
            PLAY_CLIENTBOUND_ADD_TRANSIENT_BLOCK_PACKET_ID
        );
        // `forget_level_chunk`: 37 in 26.2, 38 in 26.3.
        assert_eq!(clientbound_play_id(v, 38), 37);
        assert_eq!(
            clientbound_play_id(v, 83),
            PLAY_CLIENTBOUND_POST_EFFECTS_PACKET_ID
        );
        assert_eq!(clientbound_play_id(v, 84), 82);
        assert_eq!(
            clientbound_play_id(v, 123),
            PLAY_CLIENTBOUND_SWING_ANIMATION_PACKET_ID
        );
        assert_eq!(clientbound_play_id(v, 143), 140);
        assert_eq!(clientbound_play_id(Version::V26_2, 38), 38);
    }

    #[test]
    fn wire_ids_are_the_inverse_of_the_constants() {
        for id in 0..=143 {
            let wire = clientbound_play_wire_id(Version::V26_3, id);
            assert_eq!(clientbound_play_id(Version::V26_3, wire), id, "play {id}");
        }
        for id in 0..=20 {
            let wire = clientbound_config_wire_id(Version::V26_3, id);
            assert_eq!(
                clientbound_config_id(Version::V26_3, wire),
                id,
                "config {id}"
            );
        }
    }

    #[test]
    fn configuration_and_serverbound_ids_follow_26_3() {
        assert_eq!(
            clientbound_config_id(Version::V26_3, 10),
            CONFIG_CLIENTBOUND_POST_EFFECTS_PACKET_ID
        );
        // `server_links`: 16 in 26.2, 17 in 26.3.
        assert_eq!(clientbound_config_id(Version::V26_3, 17), 16);
        assert_eq!(clientbound_config_id(Version::V26_3, 9), 9);
        assert_eq!(
            serverbound_play_wire_id(Version::V26_3, PLAY_SERVERBOUND_PUNCH_PACKET_ID),
            46
        );
        assert_eq!(serverbound_play_wire_id(Version::V26_3, 46), 47);
        assert_eq!(serverbound_play_wire_id(Version::V26_3, 62), 63);
        assert_eq!(serverbound_play_wire_id(Version::V26_3, 45), 45);
        assert_eq!(serverbound_play_wire_id(Version::V26_2, 62), 62);
    }
}
