//! SECTION

use super::*;

fn mounts_rows_1() -> Vec<Clicks> {
    vec![
        mounted("a tame horse takes a saddle and horse armour", mount("horse"), gear(), wear()),
        mounted("an untamed horse takes a saddle all the same", mount("horse").untamed(), gear(), wear()),
        mounted("an untamed donkey takes armour but no saddle", mount("donkey").untamed(), gear(), wear()),
        mounted("a donkey foal takes armour but no saddle", mount("donkey").baby(), gear(), wear()),
        mounted(
            "a llama takes a carpet and no saddle or horse armour",
            mount("llama").columns(2).untamed(),
            stacks("35 saddle 1, 36 iron_horse_armor 1, 37 white_carpet 1"),
            taps("35 0 P, 0 0 P, 36 0 P, 1 0 P, 37 0 P, 1 0 P"),
        ),
        mounted("a camel takes a saddle", mount("camel"), gear(), wear()),
        mounted(
            "a tame nautilus takes a saddle and nautilus armour",
            mount("nautilus"),
            stacks("29 saddle 1, 30 diamond_nautilus_armor 1, 31 iron_horse_armor 1"),
            taps("29 0 P, 0 0 P, 30 0 P, 1 0 P, 31 0 P, 1 0 P"),
        ),
        mounted(
            "an untamed nautilus takes nothing",
            mount("nautilus").untamed(),
            stacks("29 saddle 1, 30 diamond_nautilus_armor 1"),
            taps("29 0 P, 0 0 P, 30 0 P, 1 0 P"),
        ),
        mounted(
            "shift-click puts gear on and the rest in the chest",
            mount("donkey").columns(5),
            stacks("38 saddle 1, 39 iron_horse_armor 1, 40 stone 20, 41 saddle 1"),
            taps("38 0 Q, 39 0 Q, 40 0 Q, 41 0 Q"),
        ),
    ]
}

fn mounts_rows_2() -> Vec<Clicks> {
    vec![
        mounted(
            "shift-click from the chest and the saddle slot goes to the inventory",
            mount("mule").columns(3),
            stacks("0 saddle 1, 1 iron_horse_armor 1, 2 stone 5, 8 dirt 7"),
            taps("0 0 Q, 1 0 Q, 2 0 Q, 8 0 Q"),
        ),
        mounted(
            "shift-click moves between the inventory rows of a mount with no chest",
            mount("horse"),
            stacks("5 stone 3, 32 dirt 4"),
            taps("5 0 Q, 32 0 Q"),
        ),
        mounted(
            "horse armour that cannot be taken off stays on",
            mount("horse"),
            vec![enchanted(1, "iron_horse_armor", &[(BINDING_CURSE, 1)])],
            taps("1 0 P, 1 0 Q"),
        ),
        mounted(
            "horse armour that cannot be taken off comes off in creative",
            mount("horse"),
            vec![enchanted(1, "iron_horse_armor", &[(BINDING_CURSE, 1)])],
            taps("1 0 P"),
        )
        .mode("creative"),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [mounts_rows_1(), mounts_rows_2()].concat()
}
