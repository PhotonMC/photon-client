//! SECTION

use super::*;

fn crafting_rows_1() -> Vec<Clicks> {
    vec![
        clicks(
            "table take the result",
            stacks("0 oak_planks 4, 1 oak_log 3, 2 water_bucket 1"),
            taps("0 0 P, 0 0 P, 20 0 P"),
        )
        .menu("crafting"),
        clicks("table quick move the result", stacks("0 oak_planks 4, 1 oak_log 2, 36 oak_planks 62"), taps("0 0 Q"))
            .menu("crafting"),
        clicks(
            "table grid remainders",
            stacks("0 cake 1, 1 milk_bucket 1, 9 sugar 2, 5 egg 1, 8 honey_bottle 3"),
            taps("0 0 P"),
        )
        .menu("crafting"),
        clicks("table quick move into the grid", stacks("20 stone 30, 40 dirt 3, 3 stone 60"), taps("20 0 Q, 40 0 Q"))
            .menu("crafting"),
        clicks("table quick move out of the grid", stacks("4 stone 5, 9 dirt 7"), taps("4 0 Q, 9 0 Q"))
            .menu("crafting"),
        clicks("table result refuses placing", stacks("0 stick 4"), taps("0 0 P")).carried("stick", 2).menu("crafting"),
        clicks("table number key from the result", stacks("0 oak_planks 4, 1 oak_log 2"), taps("0 3 SW"))
            .menu("crafting"),
        clicks("table double click skips the result", stacks("0 stone 30, 10 stone 10, 45 stone 4"), taps("11 0 PA"))
            .carried("stone", 5)
            .menu("crafting"),
        clicks("table drag over the grid", vec![], taps("-999 0 QC, 1 1 QC, 2 1 QC, 30 1 QC, -999 2 QC"))
            .carried("stone", 9)
            .menu("crafting"),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [crafting_rows_1()].concat()
}
