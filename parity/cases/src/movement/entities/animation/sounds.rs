//! Entity animation scenarios: the sounds that follow an entity (`MinecartSoundInstance`,
//! `BeeSoundInstance`, `GuardianAttackSoundInstance`, `SnifferSoundInstance`), which the row
//! ticks after the entity does and reads back whole.

use crate::movement::entities::prelude::*;

fn sound(label: &str, entity: &str, ticks: i64, events: Vec<Value>) -> Living {
    living(format!("sound-{label}"), entity, ticks, events).extras(&["sound"]).player_x(30.5)
}

fn rail_under_the_cart() -> Value {
    json!([put(0, 63, 0, "minecraft:rail")])
}

fn minecarts() -> Vec<Value> {
    vec![
        // A minecart rolls louder as it goes faster, and is silent standing still or too slowly.
        // Its sound keeps its pitch at one (its own pitch field hides the one the engine reads).
        sound("minecart-still", "minecart", 6, vec![]).row(),
        sound("minecart-rolls", "minecart", 10, vec![motion(2, 0.25, 0.0, 0.0)]).row(),
        sound("minecart-too-slow", "minecart", 6, vec![motion(2, 0.005, 0.0, 0.0)]).row(),
        sound("minecart-fast", "minecart", 10, vec![motion(2, 0.8, 0.0, 0.6)]).row(),
        sound(
            "minecart-slows-down",
            "minecart",
            14,
            vec![motion(2, 0.4, 0.0, 0.0), motion(6, 0.1, 0.0, 0.0), motion(10, 0.0, 0.0, 0.0)],
        )
        .row(),
        sound("minecart-going-up", "minecart", 8, vec![motion(2, 0.0, 0.4, 0.0)]).row(),
        // The new behaviour is silent off its rails, and sounds on them.
        sound("minecart-new-off-rails", "minecart", 8, vec![motion(2, 0.25, 0.0, 0.0)])
            .set("chunk_radius", json!(1))
            .set("minecart_improvements", json!(true))
            .row(),
        sound("minecart-new-on-rails", "minecart", 8, vec![motion(2, 0.25, 0.0, 0.0)])
            .set("chunk_radius", json!(1))
            .set("minecart_improvements", json!(true))
            .set("blocks", rail_under_the_cart())
            .row(),
        sound("chest-minecart-rolls", "chest_minecart", 8, vec![motion(2, 0.3, 0.0, 0.0)]).row(),
    ]
}

fn bees() -> Vec<Value> {
    vec![
        // A bee buzzes by its speed, higher when it is a baby, and hands the buzz to the other
        // sound when it gets angry or calms down.
        sound("bee-hovers", "bee", 6, vec![]).row(),
        sound("bee-flies", "bee", 10, vec![motion(2, 0.2, 0.0, 0.1)]).row(),
        sound("bee-flies-fast", "bee", 10, vec![motion(2, 0.9, 0.0, 0.0)]).row(),
        sound(
            "bee-baby-flies",
            "bee",
            10,
            vec![json!({"tick": 1, "type": "baby_data", "baby": true}), motion(2, 0.2, 0.0, 0.1)],
        )
        .row(),
        sound(
            "bee-gets-angry",
            "bee",
            12,
            vec![motion(1, 0.2, 0.0, 0.0), json!({"tick": 4, "type": "bee_data", "anger_end": 500})],
        )
        .row(),
        sound(
            "bee-calms-down",
            "bee",
            14,
            vec![
                motion(1, 0.2, 0.0, 0.0),
                json!({"tick": 1, "type": "bee_data", "anger_end": 7}),
                json!({"tick": 6, "type": "game_time", "time": 9}),
            ],
        )
        .set("aggressive_start", json!(true))
        .row(),
        sound("bee-aggressive-start", "bee", 8, vec![motion(1, 0.2, 0.0, 0.0)])
            .set("aggressive_start", json!(true))
            .row(),
    ]
}

fn guardians() -> Vec<Value> {
    let aimed = |label: &str, entity: &str, ticks: i64, events: Vec<Value>| {
        let mut all = vec![guardian_random(0.37, 11)];
        all.extend(events);
        fish(&format!("sound-{label}"), entity, ticks, all).extras(&["sound"])
    };
    vec![
        // The beam hums louder and higher as it charges, and goes quiet when the target is lost.
        aimed("guardian-idle", "guardian", 6, vec![]).row(),
        aimed("guardian-charges", "guardian", 40, vec![json!({"tick": 5, "type": "guardian_data", "target": 999})])
            .row(),
        aimed(
            "guardian-loses-its-target",
            "guardian",
            50,
            vec![
                json!({"tick": 5, "type": "guardian_data", "target": 999}),
                json!({"tick": 30, "type": "guardian_data", "target": 0}),
            ],
        )
        .row(),
        aimed(
            "elder-guardian-charges",
            "elder_guardian",
            40,
            vec![json!({"tick": 5, "type": "guardian_data", "target": 999})],
        )
        .row(),
    ]
}

fn sniffers() -> Vec<Value> {
    vec![
        // A sniffer digs loudly while it digs or searches, and not in its other states.
        // The sound begins when the server says the sniffer digs (event 63) and ends with the
        // digging.
        sound("sniffer-idle", "sniffer", 6, vec![]).set("sound_from", json!(2)).row(),
        sound("sniffer-digs", "sniffer", 20, vec![json!({"tick": 3, "type": "sniffer_data", "state": 5})])
            .set("sound_from", json!(3))
            .row(),
        sound("sniffer-searches", "sniffer", 20, vec![json!({"tick": 3, "type": "sniffer_data", "state": 4})])
            .set("sound_from", json!(3))
            .row(),
        sound(
            "sniffer-stops-digging",
            "sniffer",
            24,
            vec![
                json!({"tick": 3, "type": "sniffer_data", "state": 5}),
                json!({"tick": 12, "type": "sniffer_data", "state": 6}),
            ],
        )
        .set("sound_from", json!(3))
        .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [minecarts(), bees(), guardians(), sniffers()].concat()
}
