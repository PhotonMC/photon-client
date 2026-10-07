"""The `math` family: the `shape/packets` cases."""

from cases.math.common import *
from cases.math.shared import *


def cases():
    yield {"case_id": "shape/player-abilities/clientbound-state", "family": "math",
           "operation": "shape", "kind": "player_abilities_packet", "inputs": [
               {"label": "defaults", "hex": "003d4ccccd3dcccccd"},
               {"label": "invulnerable-only", "hex": "013d4ccccd3dcccccd"},
               {"label": "flying-and-can-fly", "hex": "063e4ccccd3d4ccccd"},
               {"label": "instabuild-only", "hex": "083d4ccccd3dcccccd"},
               {"label": "all-flags-and-unknown-bit", "hex": "ff3e4ccccd3e99999a"},
               {"label": "raw-speed-bits", "hex": "02800000007fc00001"},
           ]}

    yield {"case_id": "shape/player-game-mode/change-game-mode", "family": "math",
           "operation": "shape", "kind": "player_game_mode_event", "inputs": [
               {"label": "survival", "hex": "0300000000"},
               {"label": "creative", "hex": "033f800000"},
               {"label": "creative-preserves-existing-flying", "hex": "033f800000",
                "initial_flying": True, "initial_flying_speed": "3f000000",
                "initial_walking_speed": "3e800000"},
               {"label": "adventure", "hex": "0340000000"},
               {"label": "spectator", "hex": "0340400000"},
               {"label": "rounds-to-creative", "hex": "033fbeb852"},
               {"label": "exact-half-rounds-up", "hex": "033fc00000"},
               {"label": "high-id-falls-back-to-survival", "hex": "0340666666"},
               {"label": "negative-id-falls-back-to-survival", "hex": "03bf028f5c"},
           ]}
