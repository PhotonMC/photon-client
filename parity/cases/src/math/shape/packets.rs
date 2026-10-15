//! Cases for `shape/player-abilities`, `shape/player-game-mode`.

use serde_json::{Value, json};

use crate::case::case;

fn clientbound_state() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "shape/player-abilities/clientbound-state",
        "math",
        "shape",
        json!({
            "kind": "player_abilities_packet",
            "inputs": [
                {"label": "defaults", "hex": "003d4ccccd3dcccccd"},
                {"label": "invulnerable-only", "hex": "013d4ccccd3dcccccd"},
                {"label": "flying-and-can-fly", "hex": "063e4ccccd3d4ccccd"},
                {"label": "instabuild-only", "hex": "083d4ccccd3dcccccd"},
                {"label": "all-flags-and-unknown-bit", "hex": "ff3e4ccccd3e99999a"},
                {"label": "raw-speed-bits", "hex": "02800000007fc00001"},
            ],
        }),
    ));
    out.push(case(
        "shape/player-game-mode/change-game-mode",
        "math",
        "shape",
        json!({
            "kind": "player_game_mode_event",
            "inputs": [
                {"label": "survival", "hex": "0300000000"},
                {"label": "creative", "hex": "033f800000"},
                {
                    "label": "creative-preserves-existing-flying",
                    "hex": "033f800000",
                    "initial_flying": true,
                    "initial_flying_speed": "3f000000",
                    "initial_walking_speed": "3e800000",
                },
                {"label": "adventure", "hex": "0340000000"},
                {"label": "spectator", "hex": "0340400000"},
                {"label": "rounds-to-creative", "hex": "033fbeb852"},
                {"label": "exact-half-rounds-up", "hex": "033fc00000"},
                {"label": "high-id-falls-back-to-survival", "hex": "0340666666"},
                {"label": "negative-id-falls-back-to-survival", "hex": "03bf028f5c"},
            ],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [clientbound_state()].concat()
}
