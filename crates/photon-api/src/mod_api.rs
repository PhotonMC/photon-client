//! The functions a WebAssembly mod can call and the one it must export, declared once.
//!
//! The host in `photon-mods` links its imports from [`IMPORTS`], and `cargo xtask gen` writes
//! `wit/photon-mod.wit` from the same tables, so the interface a guest is built against cannot
//! drift from what the client links. The names are root-level functions of the component, as
//! the first mods were built; a capability decides whether the host defines them for a mod.

/// A value type of the boundary, in the terms of the component model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    S32,
    U32,
    S64,
    F32,
    F64,
    Bool,
    String,
    /// `list<u8>`.
    Bytes,
    /// `option<list<u8>>`.
    OptionBytes,
}

impl Ty {
    /// The type as WIT writes it.
    pub const fn wit(self) -> &'static str {
        match self {
            Self::S32 => "s32",
            Self::U32 => "u32",
            Self::S64 => "s64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Bool => "bool",
            Self::String => "string",
            Self::Bytes => "list<u8>",
            Self::OptionBytes => "option<list<u8>>",
        }
    }

    /// The type as the component text format writes it.
    pub const fn wat(self) -> &'static str {
        match self {
            Self::S32 => "s32",
            Self::U32 => "u32",
            Self::S64 => "s64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Bool => "bool",
            Self::String => "string",
            Self::Bytes => "(list u8)",
            Self::OptionBytes => "(option (list u8))",
        }
    }
}

/// One function the host defines for a mod that holds [`Import::capability`].
#[derive(Debug, Clone, Copy)]
pub struct Import {
    /// The name in the component and in WIT.
    pub name: &'static str,
    /// Parameter names and types, in order.
    pub params: &'static [(&'static str, Ty)],
    /// The result, if any.
    pub result: Option<Ty>,
    /// The capability a manifest must ask for.
    pub capability: &'static str,
    /// One line for the WIT file.
    pub doc: &'static str,
}

/// A function the mod provides and the host calls.
#[derive(Debug, Clone, Copy)]
pub struct Export {
    pub name: &'static str,
    /// The capability under which the host calls it.
    pub capability: &'static str,
    pub doc: &'static str,
}

/// A capability and what it grants, in the words of the WIT file.
#[derive(Debug, Clone, Copy)]
pub struct Capability {
    pub name: &'static str,
    pub doc: &'static str,
}

pub const EVENTS_TICK: &str = "events.tick";
pub const HUD_TEXT: &str = "hud.text";
pub const SOUND_PLAY: &str = "sound.play";
pub const GAME_TIME: &str = "game.time";
pub const PLAYER_STATE: &str = "player.state";
pub const FS_MOD_DATA: &str = "fs.mod_data";

/// Every capability a manifest may ask for today.
pub const CAPABILITIES: &[Capability] = &[
    Capability {
        name: EVENTS_TICK,
        doc: "The host calls the mod's `tick` once per client tick.",
    },
    Capability {
        name: HUD_TEXT,
        doc: "Draw text on the screen.",
    },
    Capability {
        name: SOUND_PLAY,
        doc: "Play a sound.",
    },
    Capability {
        name: GAME_TIME,
        doc: "Read the level's clocks.",
    },
    Capability {
        name: PLAYER_STATE,
        doc: "Read the local player's place, health and look.",
    },
    Capability {
        name: FS_MOD_DATA,
        doc: "A private key-value store; the player must allow it.",
    },
];

/// What the mod must provide.
pub const EXPORTS: &[Export] = &[Export {
    name: "tick",
    capability: EVENTS_TICK,
    doc: "Called once per client tick.",
}];

const fn read(
    name: &'static str,
    result: Ty,
    capability: &'static str,
    doc: &'static str,
) -> Import {
    Import {
        name,
        params: &[],
        result: Some(result),
        capability,
        doc,
    }
}

/// Every function the host can define for a mod.
pub const IMPORTS: &[Import] = &[
    Import {
        name: "hud-text",
        params: &[
            ("x", Ty::S32),
            ("y", Ty::S32),
            ("colour", Ty::U32),
            ("text", Ty::String),
        ],
        result: None,
        capability: HUD_TEXT,
        doc: "Text at (x, y) in GUI units with an 0xRRGGBB colour; at most 256 bytes.",
    },
    Import {
        name: "play-sound",
        params: &[
            ("name", Ty::String),
            ("volume", Ty::F32),
            ("pitch", Ty::F32),
        ],
        result: None,
        capability: SOUND_PLAY,
        doc: "A sound by identifier; volume is clamped to 0..1 and pitch to 0.5..2.",
    },
    read(
        "game-time",
        Ty::S64,
        GAME_TIME,
        "The level's game time in ticks, as the server last set it.",
    ),
    read(
        "day-time",
        Ty::S64,
        GAME_TIME,
        "Ticks of the dimension's own clock (0 without one).",
    ),
    read(
        "player-x",
        Ty::F64,
        PLAYER_STATE,
        "The local player's feet, in blocks.",
    ),
    read(
        "player-y",
        Ty::F64,
        PLAYER_STATE,
        "The local player's feet, in blocks.",
    ),
    read(
        "player-z",
        Ty::F64,
        PLAYER_STATE,
        "The local player's feet, in blocks.",
    ),
    read(
        "player-health",
        Ty::F32,
        PLAYER_STATE,
        "Health in half hearts; 0 before the server has said.",
    ),
    read(
        "player-yaw",
        Ty::F32,
        PLAYER_STATE,
        "Where the player looks, in degrees.",
    ),
    read(
        "player-pitch",
        Ty::F32,
        PLAYER_STATE,
        "Where the player looks, in degrees.",
    ),
    Import {
        name: "data-get",
        params: &[("key", Ty::String)],
        result: Some(Ty::OptionBytes),
        capability: FS_MOD_DATA,
        doc: "The value stored under a key (1 to 64 printable ASCII bytes), if any.",
    },
    Import {
        name: "data-set",
        params: &[("key", Ty::String), ("value", Ty::Bytes)],
        result: Some(Ty::U32),
        capability: FS_MOD_DATA,
        doc: "Stores a value of at most 64 KiB; 0 is success, 1 bad key, 2 value too large, \
              3 store full, 4 too many writes this tick, 5 no store.",
    },
    Import {
        name: "data-delete",
        params: &[("key", Ty::String)],
        result: Some(Ty::Bool),
        capability: FS_MOD_DATA,
        doc: "Removes a key; true if it was there.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_function_names_a_declared_capability() {
        let known = |cap: &str| CAPABILITIES.iter().any(|c| c.name == cap);
        assert!(IMPORTS.iter().all(|i| known(i.capability)));
        assert!(EXPORTS.iter().all(|e| known(e.capability)));
    }

    #[test]
    fn names_are_unique_and_lower_kebab_case() {
        let mut names: Vec<_> = IMPORTS.iter().map(|i| i.name).collect();
        names.extend(EXPORTS.iter().map(|e| e.name));
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count);
        assert!(
            names
                .iter()
                .all(|n| n.bytes().all(|b| b.is_ascii_lowercase() || b == b'-'))
        );
    }
}
