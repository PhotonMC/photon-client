//! SECTION

use super::*;

fn stacks_rows_1() -> Vec<Clicks> {
    vec![
        clicks(
            "a named sword is hashed with its name",
            vec![with_patch(st(20, "diamond_sword", 1), patch(&[(CUSTOM_NAME, nbt_text("Excalibur"))]))],
            taps("20 0 P, 21 0 P"),
        ),
        clicks("named stacks of one name merge", vec![with_patch(st(10, "stone", 5), rock())], taps("10 0 P"))
            .carried("stone", 3)
            .carried_with(rock()),
        clicks(
            "an enchanted pickaxe swaps with a damaged sword",
            vec![with_patch(st(20, "iron_sword", 1), patch(&[(DAMAGE, vi(37))]))],
            taps("20 0 P"),
        )
        .carried("iron_pickaxe", 1)
        .carried_with(patch(&[(ENCHANTMENTS, enchantments(&[(EFFICIENCY, 3), (UNBREAKING, 2)]))])),
        clicks(
            "a stack with a removed component",
            vec![with_patch(st(36, "diamond_sword", 1), patch_of(&[], &[DAMAGE]))],
            taps("36 0 P, 9 0 P"),
        ),
        clicks(
            "a carried bundle takes a stack in and puts it out",
            stacks("20 stone 64"),
            taps("20 0 P, 21 1 P, 22 1 P"),
        )
        .carried("bundle", 1),
        clicks(
            "a carried bundle takes what fits and then fails",
            stacks("20 stone 64, 21 dirt 5"),
            taps("20 0 P, 21 0 P"),
        )
        .carried("bundle", 1)
        .carried_with(bundle(&[stack("ender_pearl", 10)])),
        clicks(
            "a stack carried onto a bundle merges and comes back out",
            vec![with_patch(st(20, "bundle", 1), bundle(&[stack("dirt", 5), stack("ender_pearl", 2)]))],
            taps("20 0 P, 20 1 P, 20 1 P, 20 0 P"),
        )
        .carried("dirt", 30),
        clicks(
            "a sword and a bundle go into a bundle",
            vec![st(20, "iron_sword", 1), with_patch(st(21, "bundle", 1), bundle(&[stack("stone", 3)]))],
            taps("20 0 P, 21 0 P, 22 1 P"),
        )
        .carried("bundle", 1),
        clicks("a shulker box does not go into a bundle", stacks("20 bundle 1"), taps("20 0 P, 21 1 P"))
            .carried("shulker_box", 1),
    ]
}

fn stacks_rows_2() -> Vec<Clicks> {
    vec![
        clicks(
            "a secondary click on an empty bundle takes nothing",
            stacks("20 stone 1, 21 bundle 1"),
            taps("21 1 P, 20 1 P, -999 0 P"),
        ),
        clicks("cursed armour stays on in survival", vec![cursed(5), st(36, "stone", 1)], taps("5 0 P, 5 0 SW, 5 0 Q")),
        clicks("cursed armour comes off in creative", vec![cursed(5)], taps("5 0 P")).mode("creative"),
        clicks(
            "the wheel selects a bundle's item and a secondary click takes it",
            vec![holding_three()],
            taps("20 1 S, 20 -1 S, 20 -1 S, 20 1 P, 20 1 P"),
        ),
        clicks(
            "leaving a bundle selects none and says so each time",
            vec![holding_three(), st(21, "bundle", 1)],
            taps("20 1 S, 20 0 U, 20 0 U, 21 1 S, 21 0 U, 22 0 U"),
        ),
        clicks(
            "a stack put into a bundle keeps the selected index",
            vec![holding_three()],
            taps("20 -1 S, 20 0 P, 20 1 P"),
        )
        .carried("ender_pearl", 4),
        clicks(
            "a primary click with nothing carried selects none",
            vec![holding_three()],
            taps("20 1 S, 20 0 P, 21 0 P, 20 1 P"),
        ),
        clicks(
            "thirteen stacks show eleven and the wheel wraps there",
            vec![with_patch(st(20, "bundle", 1), bundle(&vec![stack("stone", 1); 13]))],
            taps("20 1 S, 20 -1 S, 20 -1 S, 20 1 P"),
        ),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [stacks_rows_1(), stacks_rows_2()].concat()
}
