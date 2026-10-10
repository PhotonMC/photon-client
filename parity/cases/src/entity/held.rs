//! What a humanoid holds, as the reference's `ItemInHandLayer` places it on
//! the posed model, with the arm poses the held items ask for, the head
//! items and skulls, a spyglass at the eye, and spears.

use serde_json::Value;

use super::pose::{Field, Fields, entity, fields, row};
use crate::line::float_text;

fn defaults() -> Vec<(&'static str, Field)> {
    fields!(
        ticks_using_item = 0.0,
        max_crossbow_charge = 0.0,
        using_item = false,
        use_hand = "main",
        passenger = false,
        fall_flying = false,
        speed_value = 1.0,
        right_pose = "empty",
        left_pose = "empty",
        main_arm = "right",
        holds_right = false,
        holds_left = false,
        head_y_rot = 0.0,
        x_rot = 0.0,
        walk_position = 0.0,
        walk_speed = 0.0,
        age_in_ticks = 0.0,
        attack_time = 0.0,
        swim_amount = 0.0,
        crouching = false,
        aggressive = false,
        holding_bow = false
    )
    .to_vec()
}

/// The fields a row with real stacks in its hands adds.
const SWING: Fields =
    &[("swing", Field::Text("whack")), ("attack_arm", Field::Text("right")), ("hit_feedback", Field::Float(0.0))];

fn held(kind: &str, baby: bool, label: &str, set: Fields) -> Value {
    let label = format!("{kind}{}-{label}", if baby { "-baby" } else { "" });
    let mut row = row(&label, &defaults(), set);
    row["kind"] = kind.into();
    row["baby"] = baby.into();
    row
}

/// A row with real stacks: `SWING`'s fields unless `set` names them.
fn stacked(kind: &str, baby: bool, label: &str, set: Fields) -> Value {
    let missing: Vec<(&str, Field)> =
        SWING.iter().filter(|(key, _)| !set.iter().any(|(name, _)| name == key)).copied().collect();
    held(kind, baby, label, &[set, &missing].concat())
}

fn player_basics(kind: &str) -> Vec<Value> {
    let both = fields!(right_pose = "item", left_pose = "item", holds_right = true, holds_left = true);
    let right = fields!(right_pose = "item", holds_right = true);
    let rows: [(&str, &[Fields]); 11] = [
        ("empty-hands", &[]),
        ("item-right", &[right, fields!(age_in_ticks = 3.0)]),
        ("item-left-hand", &[fields!(left_pose = "item", holds_left = true, age_in_ticks = 3.0)]),
        ("both", &[both, fields!(walk_position = 7.3, walk_speed = 0.83, age_in_ticks = 40.0)]),
        ("looking", &[right, fields!(head_y_rot = -35.5, x_rot = 48.0)]),
        ("crouching", &[both, fields!(crouching = true)]),
        ("swing", &[right, fields!(attack_time = 0.4, age_in_ticks = 9.0)]),
        ("swimming", &[right, fields!(swim_amount = 0.6, age_in_ticks = 11.0, walk_position = 2.0, walk_speed = 0.5)]),
        ("left-handed", &[both, fields!(main_arm = "left", age_in_ticks = 17.5, attack_time = 0.3)]),
        (
            "bow-drawn-left-handed",
            &[both, fields!(right_pose = "bow_and_arrow", main_arm = "left", head_y_rot = 12.0, x_rot = -8.0)],
        ),
        ("offhand-two-handed", &[both, fields!(left_pose = "bow_and_arrow")]),
    ];
    rows.iter().map(|(label, sets)| held(kind, false, label, &overlay(sets))).collect()
}

/// Later fields win over earlier ones.
fn overlay(sets: &[Fields]) -> Vec<(&'static str, Field)> {
    let mut out: Vec<(&'static str, Field)> = Vec::new();
    for set in sets {
        for (key, value) in *set {
            out.retain(|(name, _)| name != key);
            out.push((key, *value));
        }
    }
    out
}

/// The poses of the arm holding `pose` on `arm`, the other holding an item.
fn sides(arm: &str, pose: &'static str) -> Vec<(&'static str, Field)> {
    let (right, left) = if arm == "right" { (pose, "item") } else { ("item", pose) };
    fields!(right_pose = right, left_pose = left, holds_right = true, holds_left = true).to_vec()
}

fn arm_poses(kind: &str) -> Vec<Value> {
    let mut rows = Vec::new();
    for pose in ["block", "throw_trident", "crossbow_hold", "spyglass", "toot_horn", "brush"] {
        for arm in ["right", "left"] {
            let looks = [
                ("level", 0.0, 0.0, false),
                ("up", -35.5, -48.0, false),
                ("down", 40.0, 70.0, true),
                ("wide", 95.0, 12.0, false),
            ];
            for (look, yaw, pitch, crouch) in looks {
                let set = overlay(&[
                    &sides(arm, pose),
                    fields!(head_y_rot = yaw, x_rot = pitch, crouching = crouch, age_in_ticks = 5.5),
                ]);
                rows.push(held(kind, false, &format!("{pose}-{arm}-{look}"), &set));
            }
        }
    }
    for arm in ["right", "left"] {
        for used in [0.0, 3.0, 12.5, 25.0, 40.0] {
            let charge = fields!(ticks_using_item = used, max_crossbow_charge = 25.0, head_y_rot = 10.0, x_rot = -20.0);
            let set = overlay(&[&sides(arm, "crossbow_charge"), charge]);
            rows.push(held(kind, false, &format!("crossbow-charge-{arm}-{}", float_text(used)), &set));
        }
        let both = fields!(
            right_pose = "crossbow_hold",
            left_pose = "crossbow_hold",
            holds_right = true,
            holds_left = true,
            head_y_rot = -15.0,
            x_rot = 25.0
        );
        rows.push(held(kind, false, &format!("crossbow-hold-both-{arm}"), both));
    }
    rows
}

/// Whether the hand used is the right one, for a main arm and a hand.
fn right_hand(main: &str, hand: &str) -> bool {
    (main == "right") == (hand == "main")
}

const HANDS: [(&str, &str); 4] = [("right", "main"), ("right", "off"), ("left", "main"), ("left", "off")];

fn in_use(kind: &str) -> Vec<Value> {
    let mut rows = Vec::new();
    // An item in use: the arm of the hand using it is posed first, for
    // either main arm.
    for pose in ["bow_and_arrow", "block", "spyglass", "toot_horn", "brush", "crossbow_charge", "throw_trident"] {
        for (main, hand) in HANDS {
            let arm = if right_hand(main, hand) { "right" } else { "left" };
            let using = fields!(
                main_arm = main,
                using_item = true,
                use_hand = hand,
                ticks_using_item = 9.0,
                max_crossbow_charge = 25.0,
                head_y_rot = 14.0,
                x_rot = -18.0,
                age_in_ticks = 7.25
            );
            let set = overlay(&[&sides(arm, pose), using]);
            rows.push(held(kind, false, &format!("using-{pose}-{main}-{hand}"), &set));
        }
    }
    rows
}

fn swimming(kind: &str) -> Vec<Value> {
    let bow = sides("right", "bow_and_arrow");
    let rows: [(&str, &[Fields]); 3] = [
        (
            "using-swimming",
            &[
                &bow,
                fields!(
                    using_item = true,
                    swim_amount = 0.6,
                    walk_position = 3.0,
                    walk_speed = 0.4,
                    age_in_ticks = 2.5,
                    head_y_rot = 5.0,
                    x_rot = 10.0
                ),
            ],
        ),
        (
            "not-using-swimming",
            &[
                &bow,
                fields!(
                    swim_amount = 0.6,
                    walk_position = 3.0,
                    walk_speed = 0.4,
                    age_in_ticks = 2.5,
                    head_y_rot = 5.0,
                    x_rot = 10.0
                ),
            ],
        ),
        (
            "spyglass-swimming",
            &[
                &sides("right", "spyglass"),
                fields!(
                    using_item = true,
                    swim_amount = 0.4,
                    walk_position = 15.0,
                    walk_speed = 0.4,
                    age_in_ticks = 2.5
                ),
            ],
        ),
    ];
    rows.iter().map(|(label, sets)| held(kind, false, label, &overlay(sets))).collect()
}

fn movement(kind: &str) -> Vec<Value> {
    let bow = sides("right", "bow_and_arrow");
    let both = sides("right", "item");
    let right = fields!(right_pose = "item", holds_right = true);
    let rows: [(&str, &[Fields]); 8] = [
        ("passenger", &[&both, fields!(passenger = true, walk_position = 2.0, walk_speed = 0.3, age_in_ticks = 11.0)]),
        ("passenger-bow", &[&bow, fields!(passenger = true, using_item = true, head_y_rot = -10.0, x_rot = 15.0)]),
        ("passenger-crouching", &[right, fields!(passenger = true, crouching = true)]),
        ("gliding", &[&both, fields!(fall_flying = true, head_y_rot = 12.0, x_rot = 33.0)]),
        (
            "gliding-fast",
            &[
                &both,
                fields!(fall_flying = true, walk_position = 4.0, walk_speed = 0.5, speed_value = 3.5, x_rot = -12.0),
            ],
        ),
        ("slowed-limbs", &[&both, fields!(walk_position = 4.0, walk_speed = 0.5, speed_value = 3.5, x_rot = -12.0)]),
        (
            "gliding-walking",
            &[&both, fields!(fall_flying = true, walk_position = 4.0, walk_speed = 0.5, x_rot = -12.0)],
        ),
        (
            "gliding-swim",
            &[
                right,
                fields!(fall_flying = true, swim_amount = 0.5, walk_position = 3.0, walk_speed = 0.4, x_rot = 20.0),
            ],
        ),
    ];
    rows.iter().map(|(label, sets)| held(kind, false, label, &overlay(sets))).collect()
}

fn mobs() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in ["zombie", "piglin"] {
        for baby in [false, true] {
            rows.push(held(kind, baby, "standing", fields!(holds_right = true, holds_left = true)));
            let walking = fields!(holds_right = true, walk_position = 7.3, walk_speed = 0.83, age_in_ticks = 40.0);
            rows.push(held(kind, baby, "walking", walking));
            rows.push(held(kind, baby, "swing", fields!(holds_left = true, attack_time = 0.5, age_in_ticks = 9.0)));
        }
    }
    for baby in [false, true] {
        let aggressive = fields!(holds_right = true, aggressive = true, age_in_ticks = 17.5);
        rows.push(held("zombie", baby, "aggressive", aggressive));
        rows.push(held(
            "zombie",
            baby,
            "left-handed",
            fields!(holds_right = true, holds_left = true, main_arm = "left"),
        ));
    }
    rows.extend(skeletons());
    rows
}

fn skeletons() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend([
        held("skeleton", false, "standing", fields!(holds_right = true, holding_bow = true)),
        held(
            "skeleton",
            false,
            "bow-drawn",
            fields!(
                right_pose = "bow_and_arrow",
                holds_right = true,
                holds_left = true,
                aggressive = true,
                holding_bow = true,
                age_in_ticks = 17.5,
                attack_time = 0.4
            ),
        ),
        held(
            "skeleton",
            false,
            "bow-drawn-left-handed",
            fields!(
                left_pose = "bow_and_arrow",
                holds_right = true,
                holds_left = true,
                main_arm = "left",
                aggressive = true,
                holding_bow = true,
                head_y_rot = -35.5,
                x_rot = 48.0,
                age_in_ticks = 3.25
            ),
        ),
        held(
            "skeleton",
            false,
            "sword-aggressive",
            fields!(holds_right = true, aggressive = true, age_in_ticks = 17.5, attack_time = 0.55),
        ),
    ]);
    rows
}

/// Each renderer's head transforms: y offset, skull y and scale.
const HEAD_KINDS: [(&str, [f64; 3]); 5] = [
    ("player", [0.0, 0.0, 1.0]),
    ("player_slim", [0.0, 0.0, 1.0]),
    ("zombie", [0.0, 0.0, 1.0]),
    ("skeleton", [0.0, 0.0, 1.0]),
    ("piglin", [0.0, 0.0, 1.0019531]),
];
const VILLAGER_HEAD: [f64; 3] = [-0.1171875, -0.07421875, 1.0];

fn head(transforms: [f64; 3]) -> Vec<(&'static str, Field)> {
    let [offset, skull_y, scale] = transforms;
    fields!(head_y_offset = offset, head_skull_y = skull_y, head_scale = scale).to_vec()
}

fn babies(kind: &str) -> &'static [bool] {
    if matches!(kind, "zombie" | "piglin") { &[false, true] } else { &[false] }
}

/// The item on the head (`CustomHeadLayer`), with each renderer's
/// transforms, and skulls: each kind, the walk's swing of a piglin's ears
/// and a dragon's jaw, the head turned, sneaking.
fn heads() -> (Vec<Value>, Vec<Value>) {
    let mut items = Vec::new();
    let mut skulls = Vec::new();
    let kinds = ["skeleton", "wither_skeleton", "player", "zombie", "creeper", "piglin", "dragon"];
    for (kind, transforms) in HEAD_KINDS {
        for baby in babies(kind) {
            let head = head(transforms);
            items.push(held(kind, *baby, "head-item", &head));
            let looking = overlay(&[&head, fields!(head_y_rot = -35.5, x_rot = 48.0)]);
            items.push(held(kind, *baby, "head-item-looking", &looking));
            let crouching = overlay(&[&head, fields!(crouching = true, age_in_ticks = 3.0)]);
            items.push(held(kind, *baby, "head-item-crouching", &crouching));
            for skull in kinds {
                let set = overlay(&[&head, fields!(skull = skull, skull_animation = 3.7)]);
                skulls.push(held(kind, *baby, &format!("skull-{skull}"), &set));
            }
            let piglin = fields!(skull = "piglin", skull_animation = 12.25, head_y_rot = -35.5, x_rot = 48.0);
            skulls.push(held(kind, *baby, "skull-piglin-walking", &overlay(&[&head, piglin])));
            let dragon = fields!(skull = "dragon", skull_animation = 21.5, crouching = true, age_in_ticks = 3.0);
            skulls.push(held(kind, *baby, "skull-dragon-walking", &overlay(&[&head, dragon])));
        }
    }
    let walking = fields!(walk_position = 7.3, walk_speed = 0.83);
    items.push(held("zombie", false, "head-item-villager-transforms", &overlay(&[&head(VILLAGER_HEAD), walking])));
    (items, skulls)
}

/// A spyglass in use is held to the eye (a player's only), from either
/// hand, looking about, and when it is not.
fn eyes() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in ["player", "player_slim"] {
        for (main, hand) in [("right", "main"), ("right", "off"), ("left", "main"), ("left", "off")] {
            for (label, yaw, pitch) in
                [("level", 0.0, 0.0), ("up", -35.5, -60.0), ("down", 20.0, 80.0), ("far-down", 5.0, 120.0)]
            {
                let arm = if right_hand(main, hand) { "right" } else { "left" };
                let using = fields!(
                    main_arm = main,
                    using_item = true,
                    use_hand = hand,
                    eye_item = true,
                    head_y_rot = yaw,
                    x_rot = pitch,
                    age_in_ticks = 4.5
                );
                let set = overlay(&[&sides(arm, "spyglass"), using]);
                rows.push(held(kind, false, &format!("eye-{main}-{hand}-{label}"), &set));
            }
        }
        let glass = sides("right", "spyglass");
        let rest: [(&str, Fields); 4] = [
            ("eye-swinging", fields!(using_item = true, eye_item = true, attack_time = 0.3, age_in_ticks = 4.5)),
            ("eye-not-in-use", fields!(eye_item = true)),
            (
                "eye-crouching",
                fields!(using_item = true, eye_item = true, crouching = true, head_y_rot = 10.0, x_rot = -20.0),
            ),
            ("eye-no-eye-item", fields!(using_item = true, head_y_rot = 10.0, x_rot = -20.0)),
        ];
        rows.extend(rest.iter().map(|(label, set)| held(kind, false, label, &overlay(&[&glass, set]))));
    }
    rows
}

pub fn held_item() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in ["player", "player_slim"] {
        rows.extend(player_basics(kind));
        rows.extend(arm_poses(kind));
        rows.extend(in_use(kind));
        rows.extend(swimming(kind));
        rows.extend(movement(kind));
    }
    rows.extend(mobs());
    let (items, skulls) = heads();
    rows.extend(items);
    rows.extend(skulls);
    rows.extend(eyes());
    let villager_skull = fields!(skull = "zombie", skull_animation = 0.0, walk_position = 7.3, walk_speed = 0.83);
    rows.push(held("zombie", false, "skull-villager-transforms", &overlay(&[&head(VILLAGER_HEAD), villager_skull])));
    rows.extend(super::spear::rows(stacked));
    vec![entity("held-item-pose", "held_item_pose", rows)]
}
