//! SECTION

use super::*;

fn furnace_rows_1() -> Vec<Clicks> {
    vec![
        clicks(
            "furnace quick move sorts by use",
            stacks("3 raw_iron 10, 4 coal 20, 5 dirt 7, 30 beef 3, 31 charcoal 5, 6 lava_bucket 1"),
            taps("3 0 Q, 4 0 Q, 5 0 Q, 30 0 Q, 31 0 Q, 6 0 Q"),
        )
        .menu("furnace")
        .furnace(&SMELTS),
        clicks("furnace quick move out", stacks("0 raw_iron 5, 1 coal 3, 2 iron_ingot 7"), taps("0 0 Q, 1 0 Q, 2 0 Q"))
            .menu("furnace")
            .furnace(&SMELTS),
        clicks(
            "furnace fuel slot takes fuel and buckets only",
            stacks("3 coal 10, 4 dirt 5, 5 bucket 3, 6 water_bucket 1"),
            taps("3 0 P, 1 0 P, 3 0 P, 4 0 P, 1 0 P, 4 0 P, 5 0 P, 1 1 P, 5 0 P, 6 0 P, 1 0 P"),
        )
        .menu("furnace")
        .furnace(&SMELTS),
        clicks("furnace ingredient slot takes anything", stacks("3 dirt 10"), taps("3 0 P, 0 0 P, 0 1 P"))
            .menu("furnace")
            .furnace(&SMELTS),
        clicks(
            "furnace result only comes out",
            stacks("2 iron_ingot 7, 3 iron_ingot 4"),
            taps("2 1 P, 3 0 P, 2 0 P, 2 0 P, 10 0 P"),
        )
        .menu("furnace")
        .furnace(&SMELTS),
        clicks(
            "furnace double click gathers the result too",
            stacks("2 iron_ingot 9, 3 iron_ingot 10, 4 iron_ingot 20"),
            taps("12 0 PA"),
        )
        .carried("iron_ingot", 5)
        .menu("furnace")
        .furnace(&SMELTS),
        clicks("blast furnace quick move", stacks("3 raw_iron 10, 4 coal 20, 5 beef 4"), taps("3 0 Q, 4 0 Q, 5 0 Q"))
            .menu("blast_furnace")
            .furnace(&SMELTS),
        clicks(
            "smoker quick move and number key",
            stacks("3 beef 10, 4 stick 20, 2 cooked_beef 4"),
            taps("3 0 Q, 4 0 Q, 2 2 SW, 2 0 TH"),
        )
        .menu("smoker")
        .furnace(&SMELTS),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [furnace_rows_1()].concat()
}
