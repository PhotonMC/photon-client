//! Mobs posed by keyframe animations: the warden, the creaking, the
//! armadillo, the sniffer, and the wither's heads.

use serde_json::Value;

use super::pose::{Fields, entity, fields, prefixed};

/// The warden's five models posed by the reference's own
/// `WardenModel.setupAnim`: the look, the walk (slow, fast, past the speed
/// it stops growing at), the tendrils' pulse, each of the six animations
/// alone (early, late and run out), and some together. The layers differ
/// only in which parts they keep, so all but the body run a shorter list.
pub fn warden() -> Vec<Value> {
    let defaults =
        fields!(age_in_ticks = 37.5, y_rot = 0.0, x_rot = 0.0, walk_position = 0.0, walk_speed = 0.0, tendril = 0.0);
    let common: [(&str, Fields); 3] = [
        ("nothing", fields!()),
        (
            "look-walk-tendril",
            fields!(
                age_in_ticks = 91.25,
                y_rot = 23.5,
                x_rot = -14.0,
                walk_position = 6.3,
                walk_speed = 0.12,
                tendril = 0.6
            ),
        ),
        ("emerge", fields!(age_in_ticks = 60.0, emerge = 20)),
    ];
    let body: [(&str, Fields); 18] = [
        ("look", fields!(y_rot = -31.0, x_rot = 17.5)),
        ("slow-walk", fields!(walk_position = 2.7, walk_speed = 0.04)),
        ("fast-walk", fields!(walk_position = 11.9, walk_speed = 0.31)),
        ("fastest-walk", fields!(age_in_ticks = 12.0, walk_position = 30.1, walk_speed = 1.0)),
        ("tendril", fields!(age_in_ticks = 12.5, tendril = 1.0)),
        ("attack", fields!(age_in_ticks = 45.0, attack = 40)),
        ("attack-over", fields!(age_in_ticks = 120.0, attack = 40)),
        ("sonic-boom", fields!(age_in_ticks = 70.0, sonic_boom = 50)),
        ("sonic-boom-late", fields!(age_in_ticks = 105.0, sonic_boom = 50)),
        ("dig", fields!(age_in_ticks = 64.0, digging = 40)),
        ("dig-late", fields!(age_in_ticks = 140.0, digging = 40)),
        ("emerge-late", fields!(age_in_ticks = 150.0, emerge = 20)),
        ("roar", fields!(age_in_ticks = 55.0, roar = 40)),
        ("roar-late", fields!(age_in_ticks = 100.0, roar = 40)),
        ("sniff", fields!(age_in_ticks = 50.0, sniff = 40)),
        ("sniff-late", fields!(age_in_ticks = 80.0, sniff = 40)),
        ("roar-then-attack", fields!(age_in_ticks = 70.0, y_rot = 10.0, x_rot = 5.0, roar = 40, attack = 65)),
        ("walking-roar", fields!(age_in_ticks = 55.0, walk_position = 8.1, walk_speed = 0.2, roar = 40)),
    ];
    let mut rows = Vec::new();
    for layer in ["main", "bioluminescent", "pulsating_spots", "tendrils", "heart"] {
        let (prefix, model) = (format!("warden-{layer}"), format!("minecraft:warden#{layer}"));
        rows.extend(prefixed(&prefix, &model, defaults, &common));
        if layer == "main" {
            rows.extend(prefixed(&prefix, &model, defaults, &body));
        }
    }
    vec![entity("warden-model", "airborne_model", rows)]
}

/// The creaking's two models (the body, the eyes) posed by the reference's
/// own `CreakingModel.setupAnim`: the look, the walk (shown only while it
/// can move, so with it frozen as well), each of the three animations alone
/// (early, late and run out), and some together.
pub fn creaking() -> Vec<Value> {
    let defaults =
        fields!(age_in_ticks = 37.5, y_rot = 0.0, x_rot = 0.0, walk_position = 0.0, walk_speed = 0.0, can_move = true);
    let common: [(&str, Fields); 3] = [
        ("nothing", fields!()),
        (
            "look-walk",
            fields!(age_in_ticks = 91.25, y_rot = 23.5, x_rot = -14.0, walk_position = 6.3, walk_speed = 0.62),
        ),
        ("attack", fields!(age_in_ticks = 45.0, attack = 40)),
    ];
    let body: [(&str, Fields); 12] = [
        ("look", fields!(y_rot = -31.0, x_rot = 17.5)),
        ("slow-walk", fields!(walk_position = 2.7, walk_speed = 0.1)),
        ("fast-walk", fields!(walk_position = 11.9, walk_speed = 0.9)),
        ("fastest-walk", fields!(walk_position = 30.1, walk_speed = 3.0)),
        ("frozen-walk", fields!(walk_position = 11.9, walk_speed = 0.9, can_move = false)),
        ("attack-over", fields!(age_in_ticks = 120.0, attack = 40)),
        ("invulnerable", fields!(age_in_ticks = 62.0, invulnerable = 60)),
        ("invulnerable-over", fields!(age_in_ticks = 90.0, invulnerable = 60)),
        ("death", fields!(age_in_ticks = 70.0, death = 50)),
        ("death-late", fields!(age_in_ticks = 150.0, death = 50)),
        ("walking-attack", fields!(age_in_ticks = 55.0, walk_position = 8.1, walk_speed = 0.5, attack = 50)),
        ("frozen-death", fields!(age_in_ticks = 80.0, can_move = false, death = 50, attack = 70)),
    ];
    let mut rows = Vec::new();
    for layer in ["main", "eyes"] {
        let (prefix, model) = (format!("creaking-{layer}"), format!("minecraft:creaking#{layer}"));
        rows.extend(prefixed(&prefix, &model, defaults, &common));
        if layer == "main" {
            rows.extend(prefixed(&prefix, &model, defaults, &body));
        }
    }
    vec![entity("creaking-model", "airborne_model", rows)]
}

/// The armadillo's two models (adult, baby) posed by the reference's own
/// `ArmadilloModel.setupAnim`: the look inside its limits, the walk (and
/// not while it hides in its shell), the ball and the body it hides behind,
/// and each of the three animations alone and with others.
pub fn armadillo() -> Vec<Value> {
    let defaults =
        fields!(age_in_ticks = 37.5, y_rot = 0.0, x_rot = 0.0, walk_position = 0.0, walk_speed = 0.0, hiding = false);
    let poses: [(&str, Fields); 14] = [
        ("nothing", fields!()),
        ("look", fields!(y_rot = 20.0, x_rot = -10.0)),
        ("look-past-limits", fields!(y_rot = -70.0, x_rot = 60.0)),
        ("look-past-other-limits", fields!(y_rot = 70.0, x_rot = -60.0)),
        ("walk", fields!(walk_position = 6.3, walk_speed = 0.2)),
        ("fast-walk", fields!(walk_position = 11.9, walk_speed = 0.9)),
        ("hiding", fields!(y_rot = 20.0, x_rot = -10.0, walk_position = 6.3, walk_speed = 0.2, hiding = true)),
        ("roll-up", fields!(age_in_ticks = 40.0, roll_up = 36, hiding = true)),
        ("roll-up-over", fields!(age_in_ticks = 70.0, roll_up = 36, hiding = true)),
        ("peek", fields!(age_in_ticks = 80.0, peek = 50, hiding = true)),
        ("peek-late", fields!(age_in_ticks = 160.0, peek = 50, hiding = true)),
        ("roll-out", fields!(age_in_ticks = 75.0, roll_out = 60)),
        ("roll-out-late", fields!(age_in_ticks = 100.0, roll_out = 60)),
        ("rolling-out-walking", fields!(age_in_ticks = 70.0, walk_position = 8.1, walk_speed = 0.5, roll_out = 60)),
    ];
    let rows = ["armadillo", "armadillo_baby"]
        .iter()
        .flat_map(|layer| prefixed(&format!("armadillo-{layer}"), &format!("minecraft:{layer}#main"), defaults, &poses))
        .collect();
    vec![entity("armadillo-model", "airborne_model", rows)]
}

/// The sniffer's two models (adult, snifflet) posed by the reference's own
/// `SnifferModel.setupAnim`: the look, the walk and the searching walk, and
/// each of the five animations alone (early, late and run out) and with the
/// walk.
pub fn sniffer() -> Vec<Value> {
    let defaults = fields!(
        age_in_ticks = 37.5,
        y_rot = 0.0,
        x_rot = 0.0,
        walk_position = 0.0,
        walk_speed = 0.0,
        searching = false
    );
    let poses: [(&str, Fields); 16] = [
        ("nothing", fields!()),
        ("look", fields!(y_rot = 20.0, x_rot = -10.0)),
        ("walk", fields!(walk_position = 6.3, walk_speed = 0.02)),
        ("fast-walk", fields!(walk_position = 11.9, walk_speed = 0.3)),
        ("searching", fields!(walk_position = 6.3, walk_speed = 0.02, searching = true)),
        ("searching-still", fields!(age_in_ticks = 44.0, searching = true)),
        ("dig", fields!(age_in_ticks = 60.0, digging = 40)),
        ("dig-late", fields!(age_in_ticks = 240.0, digging = 40)),
        ("sniff", fields!(age_in_ticks = 52.0, sniffing = 40)),
        ("sniff-over", fields!(age_in_ticks = 100.0, sniffing = 40)),
        ("rise", fields!(age_in_ticks = 70.0, rising = 40)),
        ("rise-over", fields!(age_in_ticks = 140.0, rising = 40)),
        ("happy", fields!(age_in_ticks = 70.0, happy = 40)),
        ("scenting", fields!(age_in_ticks = 70.0, scenting = 40)),
        ("scenting-late", fields!(age_in_ticks = 260.0, scenting = 40)),
        ("digging-walking", fields!(age_in_ticks = 70.0, walk_position = 8.1, walk_speed = 0.05, digging = 40)),
    ];
    let rows = ["sniffer", "sniffer_baby"]
        .iter()
        .flat_map(|layer| prefixed(&format!("sniffer-{layer}"), &format!("minecraft:{layer}#main"), defaults, &poses))
        .collect();
    vec![entity("sniffer-model", "airborne_model", rows)]
}

/// The wither's two models (the body, the armor) posed by the reference's
/// own `WitherBossModel.setupAnim`: the middle head's look, the side heads'
/// turns relative to the body, and the ribcage's and the tail's sway over
/// the age.
pub fn wither() -> Vec<Value> {
    let defaults = fields!(
        age_in_ticks = 37.5,
        y_rot = 0.0,
        x_rot = 0.0,
        body_rot = 0.0,
        x_head_0 = 0.0,
        y_head_0 = 0.0,
        x_head_1 = 0.0,
        y_head_1 = 0.0
    );
    let poses: [(&str, Fields); 5] = [
        ("nothing", fields!()),
        ("later", fields!(age_in_ticks = 201.75)),
        ("look", fields!(y_rot = 23.5, x_rot = -14.0)),
        ("heads", fields!(body_rot = 40.0, x_head_0 = 12.0, y_head_0 = 70.0, x_head_1 = -25.0, y_head_1 = 10.0)),
        (
            "everything",
            fields!(
                age_in_ticks = 91.25,
                y_rot = -31.0,
                x_rot = 17.5,
                body_rot = -120.0,
                x_head_0 = 30.0,
                y_head_0 = -100.0,
                x_head_1 = -5.0,
                y_head_1 = 45.5
            ),
        ),
    ];
    let rows = ["main", "armor"]
        .iter()
        .flat_map(|layer| prefixed(&format!("wither-{layer}"), &format!("minecraft:wither#{layer}"), defaults, &poses))
        .collect();
    vec![entity("wither-model", "airborne_model", rows)]
}
