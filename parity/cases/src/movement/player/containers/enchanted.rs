//! SECTION

use super::*;

fn enchanted_rows_1() -> Vec<Clicks> {
    vec![
        clicks(
            "grindstone strips all but curses",
            vec![enchanted(30, "diamond_sword", &[(SHARPNESS, 3), (VANISHING_CURSE, 1)])],
            taps("30 0 P, 0 0 P, 2 0 P, 20 0 P"),
        )
        .menu("grindstone"),
        clicks(
            "grindstone merges two worn pickaxes",
            vec![worn(0, "iron_pickaxe", 100), worn(30, "iron_pickaxe", 200)],
            taps("30 0 P, 1 0 P, 2 0 Q"),
        )
        .menu("grindstone"),
        clicks(
            "grindstone merges enchantments and keeps curses",
            vec![
                enchanted(0, "diamond_sword", &[(BINDING_CURSE, 1), (SHARPNESS, 2)]),
                enchanted(30, "diamond_sword", &[(VANISHING_CURSE, 1), (SHARPNESS, 5), (UNBREAKING, 3)]),
            ],
            taps("30 0 P, 1 0 P"),
        )
        .menu("grindstone"),
        clicks(
            "grindstone turns a book of no curse into a book",
            vec![book(30, &[(SHARPNESS, 2)]), book(31, &[(VANISHING_CURSE, 1), (MENDING, 1)])],
            taps("30 0 P, 0 0 P, 2 0 Q, 31 0 P, 1 0 P, 1 0 Q"),
        )
        .menu("grindstone"),
        clicks(
            "grindstone refuses two books and two items of a kind",
            vec![
                book(0, &[(SHARPNESS, 2)]),
                book(30, &[(UNBREAKING, 1)]),
                enchanted(31, "iron_sword", &[(SHARPNESS, 1)]),
            ],
            taps("30 0 P, 1 0 P, 1 0 P, 31 0 P, 0 0 SW"),
        )
        .menu("grindstone"),
        clicks(
            "damaged stacks do not merge on a quick move",
            vec![
                with_patch(st(9, "stick", 1), damaged_stick()),
                with_patch(st(36, "stick", 1), damaged_stick()),
                with_patch(st(10, "stick", 1), max_four()),
                with_patch(st(37, "stick", 1), max_four()),
            ],
            taps("9 0 Q, 10 0 Q"),
        ),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [enchanted_rows_1()].concat()
}
