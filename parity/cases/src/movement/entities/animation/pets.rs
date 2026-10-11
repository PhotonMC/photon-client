//! Entity animation scenarios: animals.

use crate::movement::entities::prelude::*;

fn pets_1() -> Vec<Value> {
    vec![
        living("wolf-wild", "wolf", 6, vec![]).extras(&["wolf"]).row(),
        living("wolf-tame-sits", "wolf", 6, vec![json!({"tick": 1, "type": "wolf_data", "flags": 5})]).extras(&["wolf"]).row(),
        living("wolf-tame-hurt", "wolf", 8, vec![json!({"tick": 0, "type": "attributes", "attributes": [{"name": "minecraft:max_health", "base": f64_bits(40.0), "modifiers": []}]}), json!({"tick": 1, "type": "wolf_data", "flags": 4, "health": f32_bits(13.5)})]).extras(&["wolf"]).row(),
        living("wolf-angers", "wolf", 12, vec![json!({"tick": 0, "type": "game_time", "time": 1000}), json!({"tick": 2, "type": "wolf_data", "anger_end": 1006}), json!({"tick": 6, "type": "game_time", "time": 1007})]).extras(&["wolf"]).row(),
        living("wolf-takes-an-interest", "wolf", 30, vec![json!({"tick": 2, "type": "wolf_data", "interested": true}), json!({"tick": 12, "type": "wolf_data", "interested": false})]).extras(&["wolf"]).row(),
        // The shake past 0.4 throws splash particles, which a headless client cannot add; the
        // rest of its run is a unit test of the state.
        living("wolf-shakes", "wolf", 10, vec![json!({"tick": 3, "type": "entity_event", "event": 8})]).extras(&["wolf"]).row(),
    ]
}

fn pets_2() -> Vec<Value> {
    vec![
        living(
            "wolf-shake-cancelled",
            "wolf",
            20,
            vec![
                json!({"tick": 3, "type": "entity_event", "event": 8}),
                json!({"tick": 8, "type": "entity_event", "event": 56}),
            ],
        )
        .extras(&["wolf"])
        .row(),
        // A strider with AI works out for itself, from the blocks and the lava around it, whether it
        // suffocates (`Strider.tick`); the client does not carry that, so these have no AI and go by
        // the data the server sends.
        living("strider-warm", "strider", 4, vec![json!({"tick": 0, "type": "mob_flags", "flags": 1})])
            .extras(&["strider"])
            .row(),
        living(
            "strider-suffocates",
            "strider",
            6,
            vec![
                json!({"tick": 0, "type": "mob_flags", "flags": 1}),
                json!({"tick": 2, "type": "strider_data", "suffocating": true}),
            ],
        )
        .extras(&["strider"])
        .row(),
        living("ravager-idle", "ravager", 4, vec![]).extras(&["ravager"]).row(),
        // The stun's effect throws a cloud one tick in six, which a headless client cannot add (and
        // the reference's draws for it are not repeatable): the stun and the roar after it are a
        // unit test of the state.
        living("ravager-swings", "ravager", 16, vec![json!({"tick": 2, "type": "entity_event", "event": 4})])
            .extras(&["ravager"])
            .row(),
        living(
            "ravager-swings-twice",
            "ravager",
            24,
            vec![
                json!({"tick": 2, "type": "entity_event", "event": 4}),
                json!({"tick": 8, "type": "entity_event", "event": 4}),
            ],
        )
        .extras(&["ravager"])
        .row(),
    ]
}

fn pets_3() -> Vec<Value> {
    vec![
        // The stun's effect throws a cloud one tick in six, which a headless client cannot add: the
        // seed is one whose forty draws all miss. The roar's burst (event 69) likewise.
        living("panda-idle", "panda", 4, vec![]).extras(&["panda"]).row(),
        living("panda-unhappy-counts-down", "panda", 12, vec![json!({"tick": 2, "type": "panda_data", "unhappy": 6})])
            .extras(&["panda"])
            .row(),
        // A headless client cannot add the sneeze's, nor the eating's (from the fifth tick), particles:
        // the end of the sneeze and a long meal are unit tests of the state.
        living("panda-sneezes", "panda", 22, vec![json!({"tick": 2, "type": "panda_data", "flags": 2})])
            .extras(&["panda"])
            .row(),
        living("panda-rolls", "panda", 40, vec![json!({"tick": 1, "type": "panda_data", "flags": 4})])
            .extras(&["panda"])
            .row(),
        living(
            "panda-sits-then-lies-on-its-back",
            "panda",
            24,
            vec![
                json!({"tick": 1, "type": "panda_data", "flags": 8}),
                json!({"tick": 9, "type": "panda_data", "flags": 16}),
            ],
        )
        .extras(&["panda"])
        .row(),
        living(
            "panda-sits-with-bamboo-and-starts-to-eat",
            "panda",
            8,
            vec![
                json!({"tick": 0, "type": "seed", "seed": 6}),
                json!({"tick": 0, "type": "equipment", "item": "minecraft:bamboo"}),
                json!({"tick": 0, "type": "panda_data", "flags": 8}),
            ],
        )
        .extras(&["panda"])
        .row(),
    ]
}

fn pets_4() -> Vec<Value> {
    vec![
        living(
            "panda-eats-until-the-item-is-gone",
            "panda",
            6,
            vec![
                json!({"tick": 0, "type": "seed", "seed": 11}),
                json!({"tick": 0, "type": "equipment", "item": "minecraft:bamboo"}),
                json!({"tick": 0, "type": "panda_data", "flags": 8, "eat": 1}),
                json!({"tick": 3, "type": "equipment", "item": null}),
            ],
        )
        .extras(&["panda"])
        .row(),
        living(
            "worried-panda-sits-in-thunder-and-gets-up-after",
            "panda",
            30,
            vec![
                json!({"tick": 0, "type": "panda_data", "main_gene": 2}),
                json!({"tick": 2, "type": "weather", "rain": f32_bits(1.0), "thunder": f32_bits(1.0)}),
                json!({"tick": 16, "type": "weather", "rain": f32_bits(0.0), "thunder": f32_bits(0.0)}),
            ],
        )
        .extras(&["panda"])
        .row(),
        living(
            "brown-panda-needs-both-genes",
            "panda",
            4,
            vec![json!({"tick": 0, "type": "panda_data", "main_gene": 4, "hidden_gene": 1})],
        )
        .extras(&["panda"])
        .row(),
        living("fox-idle", "fox", 4, vec![]).extras(&["fox"]).row(),
        living("fox-snow-sleeps", "fox", 6, vec![json!({"tick": 1, "type": "fox_data", "variant": 1, "flags": 32})])
            .extras(&["fox"])
            .row(),
        living(
            "fox-crouches",
            "fox",
            20,
            vec![
                json!({"tick": 2, "type": "fox_data", "flags": 4}),
                json!({"tick": 14, "type": "fox_data", "flags": 0}),
            ],
        )
        .extras(&["fox"])
        .row(),
    ]
}

fn pets_5() -> Vec<Value> {
    vec![
        living(
            "fox-takes-an-interest",
            "fox",
            24,
            vec![
                json!({"tick": 2, "type": "fox_data", "flags": 8}),
                json!({"tick": 12, "type": "fox_data", "flags": 1}),
            ],
        )
        .extras(&["fox"])
        .row(),
        living(
            "fox-pounces-and-faceplants",
            "fox",
            8,
            vec![
                json!({"tick": 1, "type": "fox_data", "flags": 16}),
                json!({"tick": 4, "type": "fox_data", "flags": 64}),
            ],
        )
        .extras(&["fox"])
        .row(),
        living("hoglin-idle", "hoglin", 4, vec![]).extras(&["hoglin"]).row(),
        living("hoglin-swings", "hoglin", 20, vec![json!({"tick": 3, "type": "entity_event", "event": 4})])
            .extras(&["hoglin"])
            .row(),
        living(
            "zoglin-swings-twice",
            "zoglin",
            30,
            vec![
                json!({"tick": 2, "type": "entity_event", "event": 4}),
                json!({"tick": 8, "type": "entity_event", "event": 4}),
            ],
        )
        .extras(&["hoglin"])
        .row(),
        living("polar-bear-idle", "polar_bear", 4, vec![]).extras(&["polar_bear"]).row(),
    ]
}

fn pets_6() -> Vec<Value> {
    vec![
        living(
            "polar-bear-stands-and-sits",
            "polar_bear",
            24,
            vec![
                json!({"tick": 2, "type": "polar_bear_data", "standing": true}),
                json!({"tick": 14, "type": "polar_bear_data", "standing": false}),
            ],
        )
        .extras(&["polar_bear"])
        .row(),
        living("cat-wild", "cat", 6, vec![]).extras(&["cat"]).row(),
        living("cat-tame-sits", "cat", 6, vec![json!({"tick": 1, "type": "cat_data", "flags": 5, "collar": 3})])
            .extras(&["cat"])
            .row(),
        living("cat-crouches", "cat", 6, vec![json!({"tick": 1, "type": "cat_data", "pose": "CROUCHING"})])
            .extras(&["cat"])
            .row(),
        // The amounts climb by their own steps, the tail's slower than the body's, and fall faster.
        living(
            "cat-lies-down-and-gets-up",
            "cat",
            24,
            vec![
                json!({"tick": 2, "type": "cat_data", "lying": true}),
                json!({"tick": 12, "type": "cat_data", "lying": false}),
            ],
        )
        .extras(&["cat"])
        .row(),
        living(
            "cat-relaxes",
            "cat",
            20,
            vec![
                json!({"tick": 1, "type": "cat_data", "relaxed": true}),
                json!({"tick": 9, "type": "cat_data", "relaxed": false}),
            ],
        )
        .extras(&["cat"])
        .row(),
    ]
}

fn pets_7() -> Vec<Value> {
    vec![
        living(
            "cat-lies-down-mid-relax",
            "cat",
            14,
            vec![
                json!({"tick": 1, "type": "cat_data", "relaxed": true}),
                json!({"tick": 4, "type": "cat_data", "lying": true, "relaxed": false}),
            ],
        )
        .extras(&["cat"])
        .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [pets_1(), pets_2(), pets_3(), pets_4(), pets_5(), pets_6(), pets_7()].concat()
}
