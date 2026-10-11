//! SECTION

use super::*;

fn chest_rows_1() -> Vec<Clicks> {
    vec![
        clicks("chest pick up and put down", stacks("0 stone 64"), taps("0 0 P, 5 0 P, 5 1 P, 30 0 P"))
            .menu("generic_9x3")
            .state(2),
        clicks(
            "chest quick move out and in",
            stacks("0 stone 40, 1 diamond_sword 1, 30 dirt 3, 57 stone 50"),
            taps("0 0 Q, 1 0 Q, 30 0 Q, 57 0 Q"),
        )
        .menu("generic_9x3"),
        clicks(
            "chest quick move merges and fills",
            [slots(0..26, "stone", 64), stacks("26 stone 60, 40 stone 30")].concat(),
            taps("40 0 Q"),
        )
        .menu("generic_9x3"),
        clicks("chest number keys", stacks("3 stone 20, 55 dirt 5"), taps("3 1 SW, 4 1 SW, 3 8 SW, 28 0 SW, 3 40 SW"))
            .menu("generic_9x3"),
        clicks("chest throw", stacks("2 stone 5"), taps("2 0 TH, 2 1 TH")).menu("generic_9x3"),
        clicks(
            "chest left drag over both parts",
            vec![],
            taps("-999 0 QC, 3 1 QC, 4 1 QC, 40 1 QC, 58 1 QC, -999 2 QC"),
        )
        .carried("stone", 64)
        .menu("generic_9x3"),
        clicks("chest right drag", stacks("4 stone 63"), taps("-999 4 QC, 3 5 QC, 4 5 QC, 62 5 QC, -999 6 QC"))
            .carried("stone", 10)
            .menu("generic_9x3"),
        clicks(
            "chest double click gathers across both parts",
            stacks("0 stone 30, 40 stone 64, 41 stone 10, 62 stone 2"),
            taps("5 0 PA, 5 1 PA"),
        )
        .carried("stone", 5)
        .menu("generic_9x3"),
        clicks("clone in a chest", stacks("1 stone 3"), taps("1 2 CL")).menu("generic_9x3"),
    ]
}

fn chest_rows_2() -> Vec<Clicks> {
    vec![
        clicks("one row chest quick move", stacks("0 stone 5, 9 dirt 7"), taps("0 0 Q, 9 0 Q")).menu("generic_9x1"),
        clicks("six row chest quick move", stacks("0 stone 5, 54 dirt 7"), taps("0 0 Q, 54 0 Q")).menu("generic_9x6"),
        clicks("hopper quick move", stacks("0 stone 5, 1 stone 64, 5 dirt 7"), taps("0 0 Q, 5 0 Q, 1 0 Q"))
            .menu("hopper"),
        clicks(
            "dispenser quick move gives up on no change",
            [stacks("0 stone 5, 9 dirt 7, 10 stone 64"), slots(1..9, "cobblestone", 64), stacks("11 diamond_sword 1")]
                .concat(),
            taps("0 0 Q, 9 0 Q, 11 0 Q"),
        )
        .menu("generic_3x3"),
        clicks(
            "shulker box refuses shulker boxes",
            stacks("27 shulker_box 1, 28 red_shulker_box 1, 29 stone 4, 0 dirt 2"),
            taps("27 0 Q, 28 0 P, 1 0 P, 29 0 Q, 0 0 Q"),
        )
        .menu("shulker_box"),
        clicks("shulker box pick up a shulker box", stacks("27 shulker_box 1"), taps("27 0 P, 3 0 P, 40 0 P"))
            .menu("shulker_box"),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [chest_rows_1(), chest_rows_2()].concat()
}
