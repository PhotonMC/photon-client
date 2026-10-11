//! Entity animation scenarios: aquatic.

use crate::movement::entities::prelude::*;

fn swimmers_1() -> Vec<Value> {
    vec![
        // A turtle is on land when it is out of the water and on the ground; it carries an egg
        // and lays it as the data says. A dolphin moves while its motion covers ground.
        fish("turtle-in-water", "turtle", 10, vec![on_ground(2, false)]).tag("turtle").row(),
        fish("turtle-on-land", "turtle", 10, vec![on_ground(2, true)]).water("none").tag("turtle").row(),
        fish("turtle-on-the-ground-in-water", "turtle", 10, vec![on_ground(2, true)])
            .water("shallow")
            .y(63.5)
            .tag("turtle")
            .row(),
        fish("turtle-in-the-air", "turtle", 10, vec![on_ground(2, false)]).water("none").tag("turtle").row(),
    ]
}

fn swimmers_2() -> Vec<Value> {
    vec![
        fish(
            "turtle-with-an-egg",
            "turtle",
            12,
            vec![on_ground(1, true), json!({"tick": 3, "type": "turtle_data", "has_egg": true})],
        )
        .water("none")
        .tag("turtle")
        .row(),
        fish(
            "turtle-lays-its-egg",
            "turtle",
            12,
            vec![on_ground(1, true), json!({"tick": 3, "type": "turtle_data", "has_egg": true, "laying_egg": true})],
        )
        .water("none")
        .tag("turtle")
        .row(),
        fish("dolphin-at-rest", "dolphin", 10, vec![]).tag("dolphin").row(),
        fish("dolphin-swims", "dolphin", 30, vec![motion(3, 0.1, 0.0, 0.05)]).tag("dolphin").row(),
    ]
}

fn swimmers_3() -> Vec<Value> {
    vec![
        fish("dolphin-glides-to-a-stop", "dolphin", 60, vec![motion(3, 0.02, 0.0, 0.0)]).tag("dolphin").row(),
        fish("dolphin-moves-only-up", "dolphin", 20, vec![motion(3, 0.0, 0.2, 0.0)]).tag("dolphin").row(),
        // An axolotl's eased factors and animation states follow where it is (water, ground), whether
        // it walks (its position changes) and whether it plays dead; a baby runs animation states
        // instead of factors. A dolphin's baby flag also changes its box.
        fish("axolotl-in-water", "axolotl", 30, vec![]).tag("axolotl").row(),
        fish(
            "axolotl-swims",
            "axolotl",
            40,
            vec![
                json!({"tick": 3, "type": "move", "xa": 600, "ya": 0, "za": 300}),
                json!({"tick": 9, "type": "move", "xa": 600, "ya": 0, "za": 300}),
                json!({"tick": 15, "type": "move", "xa": 0, "ya": 0, "za": 0}),
            ],
        )
        .tag("axolotl")
        .row(),
    ]
}

fn swimmers_4() -> Vec<Value> {
    vec![
        fish("axolotl-on-land", "axolotl", 40, vec![on_ground(2, true)]).water("none").tag("axolotl").row(),
        fish(
            "axolotl-crawls",
            "axolotl",
            40,
            vec![
                on_ground(2, true),
                json!({"tick": 4, "type": "move", "xa": 500, "ya": 0, "za": 0, "on_ground": true}),
                json!({"tick": 10, "type": "move", "xa": 500, "ya": 0, "za": 0, "on_ground": true}),
            ],
        )
        .water("none")
        .tag("axolotl")
        .row(),
        fish("axolotl-in-the-air", "axolotl", 30, vec![on_ground(2, false)]).water("none").tag("axolotl").row(),
        fish(
            "axolotl-plays-dead",
            "axolotl",
            40,
            vec![json!({"tick": 5, "type": "axolotl_data", "playing_dead": true})],
        )
        .tag("axolotl")
        .row(),
    ]
}

fn swimmers_5() -> Vec<Value> {
    vec![
        fish(
            "axolotl-gets-up",
            "axolotl",
            50,
            vec![
                json!({"tick": 5, "type": "axolotl_data", "playing_dead": true}),
                json!({"tick": 25, "type": "axolotl_data", "playing_dead": false}),
            ],
        )
        .tag("axolotl")
        .row(),
        fish("axolotl-variants", "axolotl", 10, vec![json!({"tick": 2, "type": "axolotl_data", "variant": 4})])
            .tag("axolotl")
            .row(),
        fish(
            "axolotl-variant-past-the-last",
            "axolotl",
            10,
            vec![json!({"tick": 2, "type": "axolotl_data", "variant": 9})],
        )
        .tag("axolotl")
        .row(),
        fish("baby-axolotl-in-water", "axolotl", 30, vec![json!({"tick": 1, "type": "baby_data", "baby": true})])
            .tag("axolotl")
            .row(),
    ]
}

fn swimmers_6() -> Vec<Value> {
    vec![
        fish(
            "baby-axolotl-swims",
            "axolotl",
            40,
            vec![
                json!({"tick": 1, "type": "baby_data", "baby": true}),
                json!({"tick": 3, "type": "move", "xa": 600, "ya": 0, "za": 300}),
                json!({"tick": 9, "type": "move", "xa": 600, "ya": 0, "za": 300}),
                json!({"tick": 15, "type": "move", "xa": 0, "ya": 0, "za": 0}),
            ],
        )
        .tag("axolotl")
        .row(),
        fish(
            "baby-axolotl-on-land",
            "axolotl",
            30,
            vec![json!({"tick": 1, "type": "baby_data", "baby": true}), on_ground(2, true)],
        )
        .water("none")
        .tag("axolotl")
        .row(),
        fish(
            "baby-axolotl-crawls",
            "axolotl",
            40,
            vec![
                json!({"tick": 1, "type": "baby_data", "baby": true}),
                on_ground(2, true),
                json!({"tick": 4, "type": "move", "xa": 500, "ya": 0, "za": 0, "on_ground": true}),
                json!({"tick": 10, "type": "move", "xa": 500, "ya": 0, "za": 0, "on_ground": true}),
            ],
        )
        .water("none")
        .tag("axolotl")
        .row(),
        fish(
            "baby-axolotl-on-the-bottom",
            "axolotl",
            30,
            vec![json!({"tick": 1, "type": "baby_data", "baby": true}), on_ground(2, true)],
        )
        .water("shallow")
        .y(63.5)
        .tag("axolotl")
        .row(),
    ]
}

fn swimmers_7() -> Vec<Value> {
    vec![
        fish(
            "baby-axolotl-plays-dead",
            "axolotl",
            40,
            vec![
                json!({"tick": 1, "type": "baby_data", "baby": true}),
                json!({"tick": 5, "type": "axolotl_data", "playing_dead": true}),
            ],
        )
        .tag("axolotl")
        .row(),
        fish("baby-dolphin-box", "dolphin", 10, vec![json!({"tick": 1, "type": "baby_data", "baby": true})])
            .tag("dolphin")
            .row(),
        // A frog's pose starts and stops the animation states (a pose sent again starts its state
        // again); floating without walking runs the idle state; its stride follows a distance
        // moved twenty-five times over, and none while it jumps. A tadpole is in water or not.
        fish("frog-floats", "frog", 30, vec![]).tag("frog").row(),
        fish(
            "frog-floats-then-swims",
            "frog",
            50,
            vec![
                json!({"tick": 10, "type": "move", "xa": 600, "ya": 0, "za": 300}),
                json!({"tick": 16, "type": "move", "xa": 600, "ya": 0, "za": 300}),
                json!({"tick": 22, "type": "move", "xa": 0, "ya": 0, "za": 0}),
            ],
        )
        .tag("frog")
        .row(),
    ]
}

fn swimmers_8() -> Vec<Value> {
    vec![
        fish("frog-on-land", "frog", 20, vec![on_ground(2, true)]).water("none").tag("frog").row(),
        fish(
            "frog-walks",
            "frog",
            40,
            vec![
                on_ground(2, true),
                json!({"tick": 4, "type": "move", "xa": 200, "ya": 0, "za": 0, "on_ground": true}),
                json!({"tick": 10, "type": "move", "xa": 200, "ya": 0, "za": 0, "on_ground": true}),
            ],
        )
        .water("none")
        .tag("frog")
        .row(),
        fish(
            "frog-jumps",
            "frog",
            40,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "LONG_JUMPING"}),
                json!({"tick": 20, "type": "frog_data", "pose": "STANDING"}),
            ],
        )
        .water("none")
        .tag("frog")
        .row(),
        fish(
            "frog-jump-restarts",
            "frog",
            30,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "LONG_JUMPING"}),
                json!({"tick": 11, "type": "frog_data", "pose": "LONG_JUMPING"}),
            ],
        )
        .water("none")
        .tag("frog")
        .row(),
    ]
}

fn swimmers_9() -> Vec<Value> {
    vec![
        fish(
            "frog-jumps-while-moving",
            "frog",
            40,
            vec![
                on_ground(2, true),
                json!({"tick": 4, "type": "move", "xa": 200, "ya": 0, "za": 0, "on_ground": true}),
                json!({"tick": 8, "type": "frog_data", "pose": "LONG_JUMPING"}),
                json!({"tick": 10, "type": "move", "xa": 200, "ya": 0, "za": 0, "on_ground": true}),
            ],
        )
        .water("none")
        .tag("frog")
        .row(),
        fish(
            "frog-croaks",
            "frog",
            40,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "CROAKING"}),
                json!({"tick": 25, "type": "frog_data", "pose": "STANDING"}),
            ],
        )
        .tag("frog")
        .row(),
        fish(
            "frog-uses-its-tongue",
            "frog",
            30,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "USING_TONGUE"}),
                json!({"tick": 12, "type": "frog_data", "pose": "STANDING"}),
            ],
        )
        .water("none")
        .tag("frog")
        .row(),
        fish(
            "frog-changes-pose",
            "frog",
            40,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "CROAKING"}),
                json!({"tick": 9, "type": "frog_data", "pose": "USING_TONGUE"}),
            ],
        )
        .water("none")
        .tag("frog")
        .row(),
    ]
}

fn swimmers_10() -> Vec<Value> {
    vec![
        fish(
            "frog-in-the-pool-jumps",
            "frog",
            30,
            vec![json!({"tick": 5, "type": "frog_data", "pose": "LONG_JUMPING"})],
        )
        .tag("frog")
        .row(),
        // A guardian's tail sways at a speed that follows whether it moves (and in water or not),
        // its spikes breathe (out of water drawn at random, from a source fixed here), and its beam
        // counts up to the attack duration (shorter for the elder), starting over whenever the target
        // is sent again.
        fish("guardian-glides", "guardian", 40, vec![guardian_random(0.37, 11)]).tag("guardian").row(),
        fish(
            "guardian-swims-and-stops",
            "guardian",
            60,
            vec![
                guardian_random(0.37, 11),
                json!({"tick": 5, "type": "guardian_data", "moving": true}),
                json!({"tick": 30, "type": "guardian_data", "moving": false}),
            ],
        )
        .tag("guardian")
        .row(),
        fish("guardian-out-of-water", "guardian", 30, vec![guardian_random(0.82, 23)])
            .water("none")
            .tag("guardian")
            .row(),
    ]
}

fn swimmers_11() -> Vec<Value> {
    vec![
        fish(
            "guardian-moves-out-of-water",
            "guardian",
            30,
            vec![guardian_random(0.82, 23), json!({"tick": 4, "type": "guardian_data", "moving": true})],
        )
        .water("none")
        .tag("guardian")
        .row(),
        fish(
            "guardian-takes-aim",
            "guardian",
            100,
            vec![
                guardian_random(0.37, 11),
                json!({"tick": 5, "type": "guardian_data", "target": 999}),
                json!({"tick": 30, "type": "guardian_data", "target": 999}),
            ],
        )
        .tag("guardian")
        .row(),
        fish(
            "guardian-stops-aiming",
            "guardian",
            40,
            vec![
                guardian_random(0.37, 11),
                json!({"tick": 5, "type": "guardian_data", "target": 999}),
                json!({"tick": 20, "type": "guardian_data", "target": 0}),
            ],
        )
        .tag("guardian")
        .row(),
        fish(
            "elder-guardian-takes-aim",
            "elder_guardian",
            100,
            vec![guardian_random(0.37, 11), json!({"tick": 5, "type": "guardian_data", "target": 999})],
        )
        .tag("guardian")
        .row(),
    ]
}

fn swimmers_12() -> Vec<Value> {
    vec![
        fish(
            "elder-guardian-glides",
            "elder_guardian",
            30,
            vec![guardian_random(0.5, 3), json!({"tick": 5, "type": "guardian_data", "moving": true})],
        )
        .tag("guardian")
        .row(),
        // With a target the world holds (a pig here, which stays where it is) the beam aims: the
        // head and pitch turn to it, at most 90 degrees a tick, the body's yaw follows the head,
        // and the bubbles along the beam take their draws from the same source. A target right
        // above leaves the head where it is; one behind meets the limit of a tick's turn.
        fish(
            "guardian-looks-at-its-target",
            "guardian",
            70,
            vec![guardian_random(0.37, 11), json!({"tick": 3, "type": "guardian_data", "target": 41})],
        )
        .tag("guardian")
        .beside("pig", 11.5, 64.0, 5.5)
        .row(),
        fish(
            "guardian-looks-at-its-target-while-moving",
            "guardian",
            50,
            vec![
                guardian_random(0.37, 11),
                json!({"tick": 2, "type": "guardian_data", "moving": true}),
                json!({"tick": 3, "type": "guardian_data", "target": 41}),
            ],
        )
        .tag("guardian")
        .beside("pig", 5.5, 63.0, 11.5)
        .row(),
        fish(
            "guardian-looks-up-at-its-target",
            "guardian",
            40,
            vec![guardian_random(0.37, 11), json!({"tick": 3, "type": "guardian_data", "target": 41})],
        )
        .tag("guardian")
        .beside("cow", 8.5, 66.0, 8.5)
        .row(),
    ]
}

fn swimmers_13() -> Vec<Value> {
    vec![
        fish(
            "guardian-turns-away-to-its-target",
            "guardian",
            40,
            vec![guardian_random(0.37, 11), json!({"tick": 3, "type": "guardian_data", "target": 41})],
        )
        .tag("guardian")
        .beside("pig", 8.5, 64.0, 4.5)
        .row(),
        fish(
            "elder-guardian-looks-at-its-target",
            "elder_guardian",
            80,
            vec![guardian_random(0.37, 11), json!({"tick": 3, "type": "guardian_data", "target": 41})],
        )
        .tag("guardian")
        .beside("pig", 5.5, 64.0, 5.5)
        .row(),
        fish(
            "guardian-keeps-aiming-when-sent-again",
            "guardian",
            60,
            vec![
                guardian_random(0.37, 11),
                json!({"tick": 3, "type": "guardian_data", "target": 41}),
                json!({"tick": 25, "type": "guardian_data", "target": 41}),
                json!({"tick": 40, "type": "guardian_data", "target": 0}),
            ],
        )
        .tag("guardian")
        .beside("pig", 11.5, 64.0, 5.5)
        .row(),
        // Out of water a guardian that rests on the ground flops: a push, a turn and a leap, drawn
        // in that order; with a target its yaw is then the head's.
        fish(
            "guardian-flops-on-land",
            "guardian",
            30,
            vec![
                guardian_random(0.82, 23),
                json!({"tick": 3, "type": "move", "xa": 0, "ya": 0, "za": 0}),
                json!({"tick": 12, "type": "move", "xa": 0, "ya": 0, "za": 0}),
            ],
        )
        .water("none")
        .tag("guardian")
        .row(),
    ]
}

fn swimmers_14() -> Vec<Value> {
    vec![
        fish(
            "guardian-flops-on-land-aiming",
            "guardian",
            30,
            vec![
                guardian_random(0.82, 23),
                json!({"tick": 2, "type": "guardian_data", "target": 41}),
                json!({"tick": 5, "type": "move", "xa": 0, "ya": 0, "za": 0}),
            ],
        )
        .water("none")
        .tag("guardian")
        .beside("pig", 11.5, 64.0, 5.5)
        .row(),
        fish("tadpole-in-water", "tadpole", 10, vec![]).tag("frog").row(),
        fish("tadpole-out-of-water", "tadpole", 10, vec![]).water("none").tag("frog").row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [
        swimmers_1(),
        swimmers_2(),
        swimmers_3(),
        swimmers_4(),
        swimmers_5(),
        swimmers_6(),
        swimmers_7(),
        swimmers_8(),
        swimmers_9(),
        swimmers_10(),
        swimmers_11(),
        swimmers_12(),
        swimmers_13(),
        swimmers_14(),
    ]
    .concat()
}
