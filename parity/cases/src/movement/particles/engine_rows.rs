//! The particle engine of a level.

use serde_json::{Value, json};

use super::engine::*;
use super::events::*;
use super::scene::*;
use crate::bits::f32_bits;
use crate::movement::world::{cat, put};

/// The level events that only play a sound.
const SOUND_ONLY_EVENTS: [i64; 37] = [
    1000, 1001, 1002, 1004, 1015, 1016, 1017, 1018, 1019, 1020, 1021, 1022, 1024, 1025, 1026, 1027, 1029, 1030, 1031,
    1032, 1033, 1034, 1035, 1039, 1040, 1041, 1042, 1043, 1044, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052,
];

fn engine_rows_1() -> Vec<Value> {
    vec![
        // What the level lets in: near the camera, far from it, and far but let in by the packet's flag.
        engine_row(
            "level-add-near-and-far",
            vec![
                add(0, "smoke", NEAR),
                add(0, "smoke", FAR),
                add_flagged(0, "flame", FAR, true, false),
                add_pushed(1, "small_flame", NEAR, (0.0, 0.1, 0.0)),
                add(1, "soul_fire_flame", (0.0, 64.0, 31.9)),
                add(1, "copper_fire_flame", (0.0, 64.0, 32.1)),
            ],
        )
        .row(),
        // The option: some are dropped by the level's own random at DECREASED, all at MINIMAL unless
        // shown always.
        engine_row("decreased", (0_i64..14_i64).map(|_| add(0, "smoke", NEAR)).collect::<Vec<_>>())
            .status("DECREASED")
            .seeds(20)
            .row(),
    ]
}

fn engine_rows_2() -> Vec<Value> {
    vec![
        engine_row("minimal", (0_i64..6_i64).map(|_| add(0, "smoke", NEAR)).collect::<Vec<_>>())
            .status("MINIMAL")
            .row(),
        engine_row(
            "minimal-always-shown",
            (0_i64..30_i64).map(|_| add_flagged(0, "smoke", NEAR, false, true)).collect::<Vec<_>>(),
        )
        .status("MINIMAL")
        .seeds(30)
        .row(),
    ]
}

fn engine_rows_3() -> Vec<Value> {
    vec![
        engine_row(
            "decreased-always-shown",
            (0_i64..20_i64).map(|_| add_flagged(0, "flame", NEAR, false, true)).collect::<Vec<_>>(),
        )
        .status("DECREASED")
        .seeds(20)
        .row(),
        // The packet: one particle moving as told, and a count, spread and sped by the listener's random.
        engine_row("packet-without-a-count", vec![packet(0, "smoke", NEAR, (0.5, 0.25, -0.125), 2.0, 0)]).row(),
    ]
}

fn engine_rows_4() -> Vec<Value> {
    vec![
        engine_row("packet-count", vec![packet(0, "flame", (0.0, 64.0, 0.0), (1.0, 0.5, 1.0), 0.05, 20)])
            .seeds(20)
            .describe(20)
            .row(),
        engine_row("packet-count-decreased", vec![packet(1, "smoke", (0.0, 64.0, 0.0), (2.0, 1.0, 2.0), 0.02, 30)])
            .status("DECREASED")
            .seeds(30)
            .row(),
    ]
}

fn engine_rows_5() -> Vec<Value> {
    vec![
        engine_row(
            "packet-far",
            vec![
                packet(0, "smoke", FAR, (1.0, 1.0, 1.0), 0.01, 10),
                packet_override(1, "smoke", FAR, (1.0, 1.0, 1.0), 0.01, 10),
            ],
        )
        .seeds(10)
        .row(),
        // Lava leaves smoke through the level (with its own random and the particles option), and the
        // rest of the rising and ash families come in from the level as the providers make them.
        engine_row("lava-leaves-smoke", vec![add(0, "lava", NEAR), add(0, "lava", (5.5, 64.0, 4.5))])
            .ticks(40)
            .seeds(120)
            .describe(6)
            .row(),
    ]
}

fn engine_rows_6() -> Vec<Value> {
    vec![
        engine_row("lava-decreased", vec![add(0, "lava", NEAR)]).status("DECREASED").ticks(30).seeds(60).row(),
        engine_row("lava-far", vec![add_flagged(0, "lava", FAR, true, false)]).ticks(20).seeds(40).row(),
    ]
}

fn engine_rows_7() -> Vec<Value> {
    vec![
        engine_row(
            "coloured",
            vec![
                with_fields(add(0, "dust", NEAR), json!({"color": 16711680, "scale": f32_bits(1.0)})),
                with_fields(
                    add(0, "dust_color_transition", NEAR),
                    json!({"color": 16711680, "to_color": 255, "scale": f32_bits(2.0)}),
                ),
                with_fields(add(0, "effect", NEAR), json!({"color": 1179426, "power": f32_bits(0.5)})),
                with_fields(add(0, "instant_effect", NEAR), json!({"color": 16777215, "power": f32_bits(1.0)})),
                with_fields(add(1, "entity_effect", NEAR), json!({"color": 1090453759})),
                add(1, "witch", NEAR),
                add(1, "infested", NEAR),
                add(1, "raid_omen", NEAR),
                add(2, "totem_of_undying", NEAR),
                add(2, "glow", NEAR),
                add(2, "electric_spark", NEAR),
                add(2, "scrape", NEAR),
                add(2, "wax_on", NEAR),
                add(2, "wax_off", NEAR),
            ],
        )
        .ticks(10)
        .seeds(14)
        .describe(14)
        .set("spell_seed", json!(77))
        .row(),
        engine_row(
            "rising-and-ash",
            [
                "large_smoke",
                "white_smoke",
                "ash",
                "white_ash",
                "dust_plume",
                "soul",
                "sculk_soul",
                "campfire_cosy_smoke",
                "campfire_signal_smoke",
                "cloud",
                "sneeze",
            ]
            .into_iter()
            .map(|name| add_pushed(0, name, NEAR, (0.0, 0.05, 0.0)))
            .collect::<Vec<_>>(),
        )
        .ticks(12)
        .seeds(11)
        .describe(11)
        .row(),
    ]
}

fn engine_rows_8() -> Vec<Value> {
    vec![
        // The emitters leave explosions and gusts about them through the level; they are a group of their own.
        engine_row(
            "emitters",
            vec![
                add(0, "explosion_emitter", (5.0, 64.0, 5.0)),
                add(0, "gust_emitter_large", (-5.0, 64.0, 5.0)),
                add(1, "gust_emitter_small", (5.0, 64.0, -5.0)),
            ],
        )
        .ticks(12)
        .seeds(100)
        .describe(10)
        .row(),
        // Level events: the sounds alone (their pitch and seed from the level's random, in order), then
        // the ones that puff smoke, a cloud or an explosion about the block, and the dispenser's ten puffs
        // out of each face; one the client does not know plays nothing.
        engine_row(
            "level-event-sounds",
            cat![
                SOUND_ONLY_EVENTS
                    .iter()
                    .enumerate()
                    .map(|(index, &event)| {
                        let index = index as i64;
                        level_event(index % 3, event, (index - 17, 64 + index % 5, 3 - index), 0)
                    })
                    .collect::<Vec<_>>(),
                vec![
                    level_event(0, 1009, (1, 64, 1), 0),
                    level_event(1, 1009, (1, 64, 1), 1),
                    level_event(1, 1009, (1, 64, 1), 2),
                    level_event(2, 1999, (0, 64, 0), 5),
                ],
            ],
        )
        .row(),
    ]
}

fn engine_rows_9() -> Vec<Value> {
    vec![
        // A note block struck plays its instrument: a tuned one pitched by its note with a note rising from it,
        // a mob head's at its own pitch, a custom head's not at all (no head above); any other block nothing.
        engine_row(
            "note-blocks",
            cat![
                note_blocks()
                    .iter()
                    .enumerate()
                    .map(|(index, (pos, _, _))| block_event(index as i64 % 2, *pos, 0, 0))
                    .collect::<Vec<_>>(),
                vec![
                    block_event(2, (0, 64, 0), 0, 0),
                    block_event(2, (0, 64, 0), 0, 0),
                    block_event(2, (9, 64, 0), 0, 0)
                ],
            ],
        )
        .blocks(cat![
            note_blocks().iter().map(|(pos, instrument, note)| note_block(*pos, instrument, *note)).collect::<Vec<_>>(),
            vec![put(9, 64, 0, "minecraft:stone")],
        ])
        .ticks(5)
        .seeds(12)
        .describe(8)
        .row(),
    ]
}

/// Bone meal's blocks: what grows, what spreads, the two that sparkle below, water, what bone meal does
/// nothing to and the air; the count it carries and a floor under half of them.
const BONE_MEAL_BLOCKS: [(&str, i64); 12] = [
    ("minecraft:wheat", 3),
    ("minecraft:oak_sapling", 0),
    ("minecraft:tall_grass", 5),
    ("minecraft:grass_block", 2),
    ("minecraft:crimson_nylium", 3),
    ("minecraft:moss_block", 4),
    ("minecraft:rooted_dirt", 3),
    ("minecraft:mangrove_leaves", 2),
    ("minecraft:water", 3),
    ("minecraft:stone", 3),
    ("minecraft:air", 3),
    ("minecraft:netherrack", 1),
];

fn engine_rows_10() -> Vec<Value> {
    vec![
        // Bone meal: the sound always, the specks by what the block is.
        engine_row(
            "bone-meal",
            BONE_MEAL_BLOCKS
                .iter()
                .enumerate()
                .map(|(index, &(_, count))| level_event(index as i64 % 3, 1505, (index as i64 * 3, 64, 0), count))
                .collect::<Vec<_>>(),
        )
        .blocks(
            BONE_MEAL_BLOCKS
                .iter()
                .enumerate()
                .flat_map(|(index, &(block, _))| {
                    let x = index as i64 * 3;
                    let floor = (index % 2 == 0).then(|| put(x, 63, 0, "minecraft:stone"));
                    floor.into_iter().chain([put(x, 64, 0, block)])
                })
                .collect::<Vec<_>>(),
        )
        .ticks(6)
        .seeds(150)
        .describe(8)
        .row(),
        // A vault closing: twenty flames about its cage's middle, small or (ominous) soul, then its sound.
        engine_row(
            "vault-deactivate",
            vec![
                level_event(0, 3016, (2, 64, 2), 0),
                level_event(1, 3016, (-3, 66, 5), 1),
                // Heard from afar, a sound is delayed by the distance.
                level_event(2, 3016, (40, 64, -25), 0),
            ],
        )
        .ticks(4)
        .seeds(60)
        .describe(8)
        .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    let mut rows = [
        engine_rows_1(),
        engine_rows_2(),
        engine_rows_3(),
        engine_rows_4(),
        engine_rows_5(),
        engine_rows_6(),
        engine_rows_7(),
        engine_rows_8(),
        engine_rows_9(),
        engine_rows_10(),
    ]
    .concat();
    rows.extend(super::engine_rows_late::rows());
    rows.extend(super::piston_rows::rows());
    rows.extend(super::ambient_rows::rows());
    rows.extend(super::sulfur_rows::rows());
    rows.extend(super::ambience_rows::rows());
    rows.extend(super::book_rows::rows());
    rows.extend(super::beacon_rows::rows());
    rows.extend(super::conduit_rows::rows());
    rows.extend(super::vault_rows::rows());
    rows.extend(super::portal_rows::rows());
    rows.extend(super::parrot_rows::rows());
    rows
}
