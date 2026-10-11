//! SECTION

use super::*;

fn inventory_rows_1() -> Vec<Clicks> {
    vec![
        clicks("pick up and put down", stacks("36 stone 64"), taps("36 0 P, 10 0 P")).state(5),
        clicks("halves and single placements", stacks("36 stone 63"), taps("36 1 P, 11 1 P, 12 1 P, 11 0 P, 36 0 P")),
        clicks("swap unlike stacks", stacks("36 stone 12"), taps("36 0 P, 37 0 P")).carried("dirt", 10),
        clicks("merge over the limit", stacks("20 stone 50"), taps("20 0 P, 21 1 P")).carried("stone", 40),
        clicks("sixteen stack limit", stacks("20 ender_pearl 10"), taps("20 0 P")).carried("ender_pearl", 10),
        clicks("unstackable swap", stacks("20 diamond_sword 1"), taps("20 0 P, 21 0 P")).carried("iron_sword", 1),
        clicks("outside the window", vec![], taps("-999 1 P, -999 0 P")).carried("stone", 5),
        clicks("quick move rows", stacks("36 stone 64, 10 dirt 30, 37 dirt 50"), taps("36 0 Q, 10 0 Q, 9 0 Q")),
        clicks(
            "quick move merges widely",
            [slots(9..36, "stone", 60), stacks("36 stone 64, 40 stone 1")].concat(),
            taps("36 0 Q"),
        ),
    ]
}

fn inventory_rows_2() -> Vec<Clicks> {
    vec![
        clicks(
            "quick move armour and shield",
            stacks("20 diamond_helmet 1, 21 iron_boots 1, 22 shield 1, 23 carved_pumpkin 1, 24 white_carpet 3"),
            taps("20 0 Q, 21 0 Q, 22 0 Q, 23 0 Q, 24 0 Q, 5 0 Q"),
        ),
        clicks(
            "armour slots refuse and cap",
            stacks("36 stone 64, 37 diamond_chestplate 1, 38 leather_leggings 1"),
            taps("36 0 P, 6 0 P, 36 0 P, 37 0 P, 6 0 P, 38 0 P, 6 0 P"),
        ),
        clicks(
            "number keys",
            stacks("15 stone 20, 38 dirt 5, 45 torch 3"),
            taps("15 0 SW, 15 2 SW, 16 5 SW, 20 40 SW, 36 40 SW"),
        ),
        clicks(
            "number key into armour",
            stacks("36 diamond_helmet 1, 37 stone 64, 5 iron_helmet 1"),
            taps("5 0 SW, 5 1 SW, 8 1 SW"),
        ),
        clicks("throw one and all", stacks("36 stone 5, 20 dirt 3"), taps("36 0 TH, 36 1 TH, 20 0 TH, 21 1 TH")),
        clicks(
            "double click gathers",
            stacks("10 stone 30, 11 stone 64, 12 stone 10, 40 stone 2, 13 dirt 5"),
            taps("20 0 PA"),
        )
        .carried("stone", 5),
        clicks("double click from the end", stacks("10 stone 30, 44 stone 40, 12 stone 10"), taps("20 1 PA"))
            .carried("stone", 3),
        clicks("left drag splits", vec![], taps("-999 0 QC, 10 1 QC, 11 1 QC, 12 1 QC, 11 1 QC, -999 2 QC"))
            .carried("stone", 64),
        clicks(
            "right drag places one each",
            stacks("11 stone 63"),
            taps("-999 4 QC, 10 5 QC, 11 5 QC, 12 5 QC, -999 6 QC"),
        )
        .carried("stone", 10),
    ]
}

fn inventory_rows_3() -> Vec<Clicks> {
    vec![
        clicks("drag over one slot is a click", vec![], taps("-999 0 QC, 10 1 QC, -999 2 QC")).carried("stone", 7),
        clicks("middle drag refused in survival", vec![], taps("-999 8 QC, 10 9 QC, -999 10 QC, 10 0 P"))
            .carried("stone", 7),
        clicks("broken drag resets", vec![], taps("-999 0 QC, 10 1 QC, 11 0 P")).carried("stone", 7),
        clicks(
            "take the crafting result",
            stacks("0 oak_planks 4, 1 oak_log 3, 2 water_bucket 1"),
            taps("0 0 P, 0 0 P, 20 0 P"),
        ),
        clicks(
            "quick move the crafting result",
            stacks("0 oak_planks 4, 1 oak_log 2, 36 oak_planks 62"),
            taps("0 0 Q"),
        ),
        clicks(
            "crafting grid remainders",
            stacks("0 cake 1, 1 milk_bucket 1, 2 sugar 2, 3 egg 1, 4 honey_bottle 3"),
            taps("0 0 P"),
        ),
        clicks("result refuses placing", stacks("0 stick 4"), taps("0 0 P")).carried("stick", 2),
        clicks("number key from the result", stacks("0 oak_planks 4, 1 oak_log 2"), taps("0 3 SW")),
        clicks("clone needs creative", stacks("10 stone 3"), taps("10 2 CL")),
    ]
}

fn inventory_rows_4() -> Vec<Clicks> {
    vec![
        clicks(
            "right drag over thirty slots grows the changed map",
            vec![],
            [taps("-999 4 QC"), (9..45).map(|i| cl(i, 5, QC)).collect(), taps("-999 6 QC")].concat(),
        )
        .carried("stone", 64),
        clicks(
            "left drag with the zero slot key",
            stacks("0 stick 1"),
            taps("0 0 TH, -999 0 QC, 1 1 QC, 2 1 QC, -999 2 QC"),
        )
        .carried("stone", 9),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [inventory_rows_1(), inventory_rows_2(), inventory_rows_3(), inventory_rows_4()].concat()
}
