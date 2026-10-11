//! SECTION

use super::*;

fn anvil_rows_1() -> Vec<Clicks> {
    vec![
        clicks(
            "anvil repairs with its material",
            vec![worn(0, "iron_sword", 100), st(1, "iron_ingot", 5)],
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"xp_level": 12})),
        clicks(
            "anvil repair needs the levels",
            vec![worn(0, "iron_sword", 100), st(1, "iron_ingot", 5)],
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"xp_level": 1})),
        clicks(
            "anvil merges two worn tools",
            vec![worn(0, "iron_pickaxe", 100), worn(1, "iron_pickaxe", 150)],
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"xp_level": 12})),
        clicks(
            "anvil puts a book's enchantments on an item",
            vec![enchanted(0, "diamond_sword", &[(UNBREAKING, 2)]), book(1, &[(UNBREAKING, 2), (SHARPNESS, 3)])],
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"xp_level": 30})),
        clicks(
            "anvil refuses a book of enchantments the item does not take",
            vec![st(0, "iron_pickaxe", 1), book(1, &[(SHARPNESS, 3)])],
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"xp_level": 30})),
        clicks(
            "anvil charges for the enchantments that exclude each other",
            vec![enchanted(0, "diamond_sword", &[(SHARPNESS, 3)]), book(1, &[(SMITE, 2), (UNBREAKING, 1)])],
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"xp_level": 30})),
        clicks("anvil refuses a result of forty levels", prior_work_stacks(), taps("2 0 P, 20 0 P"))
            .menu("anvil")
            .extra(json!({"xp_level": 40})),
        clicks("anvil gives a result of forty levels in creative", prior_work_stacks(), taps("2 0 P, 20 0 P"))
            .menu("anvil")
            .mode("creative"),
        clicks(
            "anvil renames and keeps the additional item",
            stacks("0 iron_sword 1, 1 stone 3"),
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"xp_level": 5, "item_name": "Sting"})),
    ]
}

fn anvil_rows_2() -> Vec<Clicks> {
    vec![
        clicks(
            "anvil renames while it repairs",
            vec![worn(0, "iron_sword", 100), st(1, "iron_ingot", 5)],
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"xp_level": 12, "item_name": "Sting"})),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [anvil_rows_1(), anvil_rows_2()].concat()
}
