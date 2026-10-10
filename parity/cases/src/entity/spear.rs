//! Spears in hand (`SpearAnimations`): the arm raises it, sways it and
//! lowers it as the use goes on, for each spear's own timings; the stab of
//! its swing thrusts arm and item; the hit's kick; and what each model
//! state changes (the other hand, the main arm, gliding, swimming).

use serde_json::Value;

use super::pose::{Field, Fields, fields};
use crate::line::float_text;

type Row = fn(&str, bool, &str, Fields) -> Value;

const SPEARS: [(&str, &str); 7] = [
    ("wooden", "minecraft:wooden_spear"),
    ("stone", "minecraft:stone_spear"),
    ("copper", "minecraft:copper_spear"),
    ("iron", "minecraft:iron_spear"),
    ("golden", "minecraft:golden_spear"),
    ("diamond", "minecraft:diamond_spear"),
    ("netherite", "minecraft:netherite_spear"),
];
const IRON: &str = "minecraft:iron_spear";

fn join(sets: &[Fields]) -> Vec<(&'static str, Field)> {
    let mut out: Vec<(&'static str, Field)> = Vec::new();
    for set in sets {
        for (key, value) in *set {
            out.retain(|(name, _)| name != key);
            out.push((key, *value));
        }
    }
    out
}

/// A spear in the right hand, an item in the left.
fn right_spear(spear: &'static str) -> Vec<(&'static str, Field)> {
    fields!(right_pose = "spear", left_pose = "item", holds_right = true, holds_left = true, right_item = spear)
        .to_vec()
}

fn uses(held: Row) -> Vec<Value> {
    let mut rows = Vec::new();
    for (name, spear) in SPEARS {
        for used in [0.5, 9.5, 17.5, 45.5, 95.5, 160.5, 230.5, 300.5] {
            let using = fields!(
                using_item = true,
                ticks_using_item = used,
                head_y_rot = 8.0,
                x_rot = -14.0,
                age_in_ticks = 6.5
            );
            let label = format!("spear-{name}-use-{}", float_text(used));
            rows.push(held("player", false, &label, &join(&[&right_spear(spear), using])));
        }
    }
    rows.extend(both_hands(held));
    rows
}

/// A spear in each hand, for each kind and for either main arm and hand.
fn both_hands(held: Row) -> Vec<Value> {
    let mut rows = Vec::new();
    let kinds = [
        ("player_slim", false),
        ("zombie", false),
        ("zombie", true),
        ("skeleton", false),
        ("piglin", false),
        ("piglin", true),
    ];
    for (kind, baby) in kinds {
        let set = fields!(
            right_pose = "spear",
            left_pose = "spear",
            holds_right = true,
            holds_left = true,
            right_item = IRON,
            left_item = "minecraft:wooden_spear",
            using_item = true,
            ticks_using_item = 33.5,
            head_y_rot = -12.0,
            x_rot = 20.0,
            age_in_ticks = 2.5,
            walk_position = 3.0,
            walk_speed = 0.3
        );
        rows.push(held(kind, baby, "spear-use", set));
    }
    for (main, hand) in [("right", "main"), ("right", "off"), ("left", "main"), ("left", "off")] {
        let set = fields!(
            right_pose = "spear",
            left_pose = "spear",
            holds_right = true,
            holds_left = true,
            right_item = "minecraft:stone_spear",
            left_item = "minecraft:diamond_spear",
            main_arm = main,
            using_item = true,
            use_hand = hand,
            ticks_using_item = 21.5,
            head_y_rot = 14.0,
            x_rot = -18.0,
            age_in_ticks = 7.25
        );
        rows.push(held("player", false, &format!("spear-use-{main}-{hand}"), set));
    }
    rows
}

fn states(held: Row) -> Vec<Value> {
    let iron = right_spear(IRON);
    let rows: [(&str, &[Fields]); 7] = [
        ("spear-held", &[&iron, fields!(head_y_rot = 5.0, x_rot = 10.0)]),
        (
            "spear-held-left",
            &[fields!(
                right_pose = "item",
                left_pose = "spear",
                holds_right = true,
                holds_left = true,
                main_arm = "left",
                left_item = IRON,
                head_y_rot = 5.0,
                x_rot = 10.0
            )],
        ),
        (
            "spear-use-gliding",
            &[fields!(
                right_pose = "spear",
                holds_right = true,
                right_item = IRON,
                using_item = true,
                ticks_using_item = 14.5,
                fall_flying = true,
                x_rot = 30.0
            )],
        ),
        (
            "spear-use-swimming",
            &[
                &iron,
                fields!(
                    using_item = true,
                    ticks_using_item = 14.5,
                    swim_amount = 0.6,
                    walk_position = 3.0,
                    walk_speed = 0.4
                ),
            ],
        ),
        ("spear-held-swimming", &[&iron, fields!(swim_amount = 0.6, walk_position = 15.0, walk_speed = 0.4)]),
        ("spear-use-passenger", &[&iron, fields!(using_item = true, ticks_using_item = 14.5, passenger = true)]),
        ("spear-use-crouching", &[&iron, fields!(using_item = true, ticks_using_item = 40.5, crouching = true)]),
    ];
    let mut out: Vec<Value> = rows.iter().map(|(label, sets)| held("player", false, label, &join(sets))).collect();
    for hit in [0.0, 1.5, 2.5, 3.5, 6.5, 9.5, 12.0] {
        let set = join(&[&iron, fields!(using_item = true, ticks_using_item = 30.5, hit_feedback = hit)]);
        out.push(held("player", false, &format!("spear-hit-feedback-{}", float_text(hit)), &set));
    }
    out
}

fn stabs(held: Row) -> Vec<Value> {
    let mut rows = Vec::new();
    for arm in ["right", "left"] {
        let spear = if arm == "right" {
            fields!(right_pose = "spear", left_pose = "item", right_item = IRON)
        } else {
            fields!(right_pose = "item", left_pose = "spear", left_item = IRON)
        };
        for attack in [0.02, 0.06, 0.12, 0.2, 0.3, 0.45, 0.6, 0.8, 0.95] {
            for (kind, baby) in [("player", false), ("zombie", false), ("piglin", true)] {
                let stab = fields!(
                    holds_right = true,
                    holds_left = true,
                    swing = "stab",
                    attack_arm = arm,
                    main_arm = arm,
                    attack_time = attack,
                    head_y_rot = 6.0,
                    x_rot = -9.0,
                    age_in_ticks = 3.5
                );
                let label = format!("spear-stab-{arm}-{}", float_text(attack));
                rows.push(held(kind, baby, &label, &join(&[spear, stab])));
            }
        }
    }
    rows.extend(stab_states(held));
    rows
}

fn stab_states(held: Row) -> Vec<Value> {
    let mut rows = Vec::new();
    let netherite = right_spear("minecraft:netherite_spear");
    let iron = right_spear(IRON);
    let tail: [(&str, &[Fields]); 4] = [
        (
            "spear-stab-in-use",
            &[
                &netherite,
                fields!(
                    swing = "stab",
                    attack_time = 0.3,
                    using_item = true,
                    ticks_using_item = 12.5,
                    hit_feedback = 4.0
                ),
            ],
        ),
        (
            "spear-stab-offhand-attack",
            &[fields!(
                right_pose = "item",
                left_pose = "spear",
                holds_right = true,
                holds_left = true,
                left_item = IRON,
                swing = "stab",
                attack_arm = "left",
                attack_time = 0.3
            )],
        ),
        ("spear-whack-with-spear-held", &[&iron, fields!(swing = "whack", attack_time = 0.3)]),
        (
            "spear-stab-swimming",
            &[
                &iron,
                fields!(swing = "stab", attack_time = 0.3, swim_amount = 0.6, walk_position = 3.0, walk_speed = 0.4),
            ],
        ),
    ];
    rows.extend(tail.iter().map(|(label, sets)| held("player", false, label, &join(sets))));
    rows
}

/// Every spear row; `held` adds the swing fields real stacks carry.
pub fn rows(held: Row) -> Vec<Value> {
    let mut rows = uses(held);
    rows.extend(states(held));
    rows.extend(stabs(held));
    rows
}
