//! First-pass classes read off the oracle call graph (the branch that dispatches the operation and
//! the methods it calls, two levels deep), the two most used. Refine an entry by moving it into
//! `exact.rs` with the exact class when the operation is next touched.

pub const INFERRED: &[(&str, &str, &str)] = &[
    ("wire", "login_finished", "network/protocol/login/ClientboundLoginFinishedPacket, network/codec/StreamCodec"),
    (
        "wire",
        "ping",
        "network/protocol/ping/ClientboundPongResponsePacket, network/protocol/ping/ServerboundPingRequestPacket",
    ),
    (
        "wire",
        "select_advancements_tab",
        "network/protocol/game/ClientboundSelectAdvancementsTabPacket, client/sounds/JOrbisAudioStream",
    ),
    (
        "wire",
        "status_request",
        "network/protocol/status/ServerboundStatusRequestPacket, client/sounds/JOrbisAudioStream",
    ),
];
