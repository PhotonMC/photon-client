//! Entity animation scenarios: bosses.

use crate::movement::entities::prelude::*;

fn bosses_1() -> Vec<Value> {
    vec![
        // A warden's pose starts its roar, sniff, emerge and dig animations; its attack event stops
        // the roar and starts the attack, the sonic boom event its boom, the tendril event its
        // pulse; its heart beats every forty ticks, faster as its anger synced to the client grows.
        living("warden-calm", "warden", 100, vec![]).extras(&["warden"]).row(),
        living(
            "warden-poses",
            "warden",
            60,
            vec![
                json!({"tick": 4, "type": "frog_data", "pose": "EMERGING"}),
                json!({"tick": 20, "type": "frog_data", "pose": "DIGGING"}),
                json!({"tick": 30, "type": "frog_data", "pose": "ROARING"}),
                json!({"tick": 45, "type": "frog_data", "pose": "SNIFFING"}),
            ],
        )
        .extras(&["warden"])
        .row(),
        living(
            "warden-roars-then-attacks",
            "warden",
            50,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "ROARING"}),
                json!({"tick": 20, "type": "entity_event", "event": 4}),
            ],
        )
        .extras(&["warden"])
        .row(),
        living("warden-booms", "warden", 40, vec![json!({"tick": 6, "type": "entity_event", "event": 62})])
            .extras(&["warden"])
            .row(),
        living("warden-tendrils", "warden", 20, vec![json!({"tick": 6, "type": "entity_event", "event": 61})])
            .extras(&["warden"])
            .row(),
        living("warden-agitated", "warden", 90, vec![json!({"tick": 1, "type": "warden_data", "anger": 40})])
            .extras(&["warden"])
            .row(),
    ]
}

fn bosses_2() -> Vec<Value> {
    vec![
        living("warden-angry", "warden", 40, vec![json!({"tick": 1, "type": "warden_data", "anger": 80})])
            .extras(&["warden"])
            .row(),
        living("warden-furious", "warden", 40, vec![json!({"tick": 1, "type": "warden_data", "anger": 150})])
            .extras(&["warden"])
            .row(),
        // A creaking's attack event runs its attack for fifteen ticks and its sway event its
        // invulnerable animation for eight; a torn-down one (heart bound) starts its death animation
        // and counts death time; its stride is quicker and capped at three. The player stands off, as the
        // reference's own player here has no connection to look pushers up in.
        living("creaking-idle", "creaking", 30, vec![]).extras(&["creaking"]).player_x(30.5).row(),
        living("creaking-attacks", "creaking", 40, vec![json!({"tick": 5, "type": "entity_event", "event": 4})])
            .extras(&["creaking"])
            .player_x(30.5)
            .row(),
        living(
            "creaking-attacks-twice",
            "creaking",
            50,
            vec![
                json!({"tick": 5, "type": "entity_event", "event": 4}),
                json!({"tick": 12, "type": "entity_event", "event": 4}),
            ],
        )
        .extras(&["creaking"])
        .player_x(30.5)
        .row(),
        living("creaking-sways", "creaking", 30, vec![json!({"tick": 5, "type": "entity_event", "event": 66})])
            .extras(&["creaking"])
            .player_x(30.5)
            .row(),
    ]
}

fn bosses_3() -> Vec<Value> {
    vec![
        living(
            "creaking-walks",
            "creaking",
            20,
            (1_i64..15).map(|t| json!({"tick": t, "type": "move", "xa": 800, "ya": 0, "za": 0})).collect::<Vec<_>>(),
        )
        .extras(&["creaking"])
        .player_x(30.5)
        .row(),
        living(
            "creaking-sprints",
            "creaking",
            20,
            (1_i64..15).map(|t| json!({"tick": t, "type": "move", "xa": 4000, "ya": 0, "za": 0})).collect::<Vec<_>>(),
        )
        .extras(&["creaking"])
        .player_x(30.5)
        .row(),
        living(
            "creaking-frozen",
            "creaking",
            20,
            vec![json!({"tick": 2, "type": "creaking_data", "can_move": false, "active": true})],
        )
        .extras(&["creaking"])
        .player_x(30.5)
        .row(),
        living(
            "creaking-tears-down",
            "creaking",
            60,
            vec![
                json!({"tick": 2, "type": "creaking_data", "home": true}),
                json!({"tick": 5, "type": "creaking_data", "tearing_down": true}),
                json!({"tick": 6, "type": "death"}),
            ],
        )
        .extras(&["creaking"])
        .player_x(30.5)
        .row(),
        living("creaking-dies-unbound", "creaking", 40, vec![json!({"tick": 3, "type": "death"})])
            .extras(&["creaking"])
            .player_x(30.5)
            .row(),
        // An armadillo's synced state starts its roll up (rolling), peek (scared, from the
        // middle of it when the state has just come) and roll out (unrolling) animations and
        // decides when it hides in its shell; the peek event restarts the peek; a scared one's head
        // is held to its body.
        living("armadillo-idle", "armadillo", 20, vec![]).extras(&["armadillo"]).row(),
    ]
}

fn bosses_4() -> Vec<Value> {
    vec![
        living("armadillo-rolls", "armadillo", 30, vec![json!({"tick": 4, "type": "armadillo_data", "state": 1})])
            .extras(&["armadillo"])
            .row(),
        living(
            "armadillo-rolls-and-is-scared",
            "armadillo",
            80,
            vec![
                json!({"tick": 4, "type": "armadillo_data", "state": 1}),
                json!({"tick": 14, "type": "armadillo_data", "state": 2}),
            ],
        )
        .extras(&["armadillo"])
        .row(),
        living(
            "armadillo-peeks-again",
            "armadillo",
            80,
            vec![
                json!({"tick": 4, "type": "armadillo_data", "state": 2}),
                json!({"tick": 30, "type": "entity_event", "event": 64}),
            ],
        )
        .extras(&["armadillo"])
        .row(),
        living(
            "armadillo-unrolls",
            "armadillo",
            60,
            vec![
                json!({"tick": 4, "type": "armadillo_data", "state": 2}),
                json!({"tick": 20, "type": "armadillo_data", "state": 3}),
                json!({"tick": 55, "type": "armadillo_data", "state": 0}),
            ],
        )
        .extras(&["armadillo"])
        .row(),
        living(
            "armadillo-state-resent",
            "armadillo",
            40,
            vec![
                json!({"tick": 4, "type": "armadillo_data", "state": 2}),
                json!({"tick": 20, "type": "armadillo_data", "state": 2}),
            ],
        )
        .extras(&["armadillo"])
        .row(),
        // A sniffer's synced state stops every animation and starts its own (happy, scenting,
        // sniffing, digging, rising; searching has none) even when it is sent again.
        living("sniffer-idle", "sniffer", 20, vec![]).extras(&["sniffer"]).row(),
    ]
}

fn bosses_5() -> Vec<Value> {
    vec![
        living(
            "sniffer-states",
            "sniffer",
            70,
            vec![
                json!({"tick": 4, "type": "sniffer_data", "state": 1}),
                json!({"tick": 12, "type": "sniffer_data", "state": 2}),
                json!({"tick": 20, "type": "sniffer_data", "state": 3}),
                json!({"tick": 28, "type": "sniffer_data", "state": 4}),
                json!({"tick": 36, "type": "sniffer_data", "state": 5}),
                json!({"tick": 44, "type": "sniffer_data", "state": 6}),
                json!({"tick": 52, "type": "sniffer_data", "state": 0}),
            ],
        )
        .extras(&["sniffer"])
        .row(),
        living("sniffer-digs", "sniffer", 40, vec![json!({"tick": 4, "type": "sniffer_data", "state": 5})])
            .extras(&["sniffer"])
            .row(),
        living(
            "sniffer-state-resent",
            "sniffer",
            40,
            vec![
                json!({"tick": 4, "type": "sniffer_data", "state": 5}),
                json!({"tick": 20, "type": "sniffer_data", "state": 5}),
            ],
        )
        .extras(&["sniffer"])
        .row(),
        living("sniffer-searches", "sniffer", 30, vec![json!({"tick": 4, "type": "sniffer_data", "state": 4})])
            .extras(&["sniffer"])
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [bosses_1(), bosses_2(), bosses_3(), bosses_4(), bosses_5()].concat()
}
