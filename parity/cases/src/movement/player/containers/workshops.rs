//! SECTION

use super::*;

fn workshops_rows_1() -> Vec<Clicks> {
    vec![
        clicks("grindstone takes damageable and enchanted only", stacks("30 diamond_sword 1, 31 stone 4, 32 iron_pickaxe 1"), taps("30 0 P, 0 0 P, 31 0 P, 1 0 P, 1 1 P, 32 0 P, 1 0 P")).menu("grindstone"),
        clicks("grindstone quick move", stacks("5 diamond_sword 1, 6 iron_pickaxe 1, 7 golden_axe 1, 8 stone 3"), taps("5 0 Q, 6 0 Q, 7 0 Q, 8 0 Q, 0 0 Q")).menu("grindstone"),
        clicks("grindstone take the result", stacks("0 diamond_sword 1, 1 diamond_sword 1, 2 diamond_sword 1"), taps("2 0 P, 20 0 P")).menu("grindstone"),
        clicks("grindstone quick move the result", stacks("0 iron_sword 1, 2 iron_sword 1"), taps("2 0 Q")).menu("grindstone"),
        clicks("cartography slots", stacks("30 paper 8, 31 map 1, 32 glass_pane 2, 33 stone 5"), taps("30 0 P, 1 0 P, 31 0 P, 0 0 P, 0 0 P, 32 0 P, 1 0 P, 33 0 P, 1 0 P")).menu("cartography_table"),
        clicks("cartography quick move", stacks("5 paper 8, 6 map 1, 7 stone 3, 31 glass_pane 4"), taps("5 0 Q, 6 0 Q, 7 0 Q, 31 0 Q")).menu("cartography_table"),
        clicks("cartography take the result", stacks("0 filled_map 2, 1 paper 3, 2 filled_map 2"), taps("2 0 P, 20 0 P")).menu("cartography_table"),
        clicks("smithing slots by set", stacks("30 netherite_upgrade_smithing_template 1, 31 diamond_sword 1, 32 netherite_ingot 2, 33 stone 3, 34 diamond_sword 1"), taps("30 0 P, 0 0 P, 31 0 P, 1 0 P, 32 0 P, 2 0 P, 33 0 P, 2 0 P, 34 0 P, 0 0 P")).menu("smithing").extra(smith_sets()),
        clicks("smithing quick move", stacks("5 netherite_upgrade_smithing_template 1, 6 diamond_sword 1, 7 netherite_ingot 2, 8 stone 3, 9 diamond_chestplate 1"), taps("5 0 Q, 6 0 Q, 7 0 Q, 8 0 Q, 9 0 Q, 0 0 Q")).menu("smithing").extra(smith_sets()),
    ]
}

fn workshops_rows_2() -> Vec<Clicks> {
    vec![
        clicks(
            "smithing take the result",
            stacks(
                "0 netherite_upgrade_smithing_template 1, 1 diamond_sword 1, 2 netherite_ingot 3, 3 netherite_sword 1",
            ),
            taps("3 0 P, 20 0 P"),
        )
        .menu("smithing")
        .extra(smith_sets()),
        clicks(
            "smithing result leaves by quick move",
            stacks("0 netherite_upgrade_smithing_template 1, 1 diamond_sword 1, 3 netherite_sword 1"),
            taps("3 0 Q"),
        )
        .menu("smithing")
        .extra(smith_sets()),
        clicks(
            "stonecutter quick move",
            stacks("2 stone 10, 3 dirt 4, 30 cobblestone 8, 1 stone_slab 3"),
            taps("2 0 Q, 3 0 Q, 30 0 Q, 1 0 Q"),
        )
        .menu("stonecutter")
        .extra(json!({"stonecutter_inputs": ["stone", "cobblestone"]})),
        clicks("stonecutter take the result", stacks("0 stone 5, 1 stone_slab 2"), taps("1 0 P, 20 0 P, 0 0 P"))
            .menu("stonecutter")
            .extra(json!({"stonecutter_inputs": ["stone"]})),
        clicks(
            "stonecutter double click skips the result",
            stacks("1 stone_slab 2, 10 stone_slab 5, 30 stone_slab 9"),
            taps("12 0 PA"),
        )
        .carried("stone_slab", 3)
        .menu("stonecutter")
        .extra(json!({"stonecutter_inputs": ["stone"]})),
        clicks(
            "loom slots by tag and component",
            stacks("31 white_banner 1, 32 red_dye 4, 33 creeper_banner_pattern 1, 34 stone 2, 35 sugar 2"),
            taps("31 0 P, 0 0 P, 32 0 P, 0 0 P, 1 0 P, 34 0 P, 1 0 P, 35 0 P, 2 0 P"),
        )
        .menu("loom"),
        clicks(
            "loom pattern with the banner",
            stacks("31 white_banner 1, 33 creeper_banner_pattern 1, 34 flower_banner_pattern 1"),
            taps("31 0 P, 0 0 P, 33 0 P, 2 0 P, 34 0 P, 2 0 P"),
        )
        .menu("loom"),
        clicks(
            "loom quick move",
            stacks("5 white_banner 1, 6 blue_dye 3, 7 flower_banner_pattern 1, 8 stone 3, 9 black_banner 1"),
            taps("5 0 Q, 8 0 Q, 9 0 Q, 3 0 Q, 6 0 Q"),
        )
        .menu("loom"),
        clicks("loom pattern quick move", stacks("7 flower_banner_pattern 1, 8 stone 3"), taps("7 0 Q, 8 0 Q"))
            .menu("loom"),
    ]
}

fn workshops_rows_3() -> Vec<Clicks> {
    vec![
        clicks(
            "loom take the result",
            stacks("0 white_banner 2, 1 red_dye 1, 3 white_banner 1"),
            taps("3 0 P, 20 0 P"),
        )
        .menu("loom"),
        clicks(
            "loom result keeps splitting",
            stacks("0 white_banner 1, 1 red_dye 4, 3 white_banner 3"),
            taps("3 1 P, 20 0 P, 3 0 P"),
        )
        .menu("loom"),
        clicks(
            "anvil takes anything in its inputs",
            stacks("30 stone 4, 31 diamond_sword 1"),
            taps("30 0 P, 0 0 P, 31 0 P, 1 0 P"),
        )
        .menu("anvil"),
        clicks("anvil result needs the levels", stacks("0 iron_sword 1, 2 iron_sword 1"), taps("2 0 P, 20 0 P, 2 0 P"))
            .menu("anvil")
            .extra(json!({"data": [5], "xp_level": 3})),
        clicks(
            "anvil result with the levels",
            stacks("0 iron_sword 1, 1 iron_ingot 2, 2 iron_sword 1"),
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .extra(json!({"data": [5], "xp_level": 12})),
        clicks(
            "anvil result costs nothing in creative",
            stacks("0 iron_sword 1, 2 iron_sword 1"),
            taps("2 0 P, 20 0 P"),
        )
        .menu("anvil")
        .mode("creative")
        .extra(json!({"data": [39]})),
        clicks(
            "anvil quick move",
            stacks("0 iron_sword 1, 2 iron_sword 1, 10 stone 2, 30 diamond_sword 1"),
            taps("0 0 Q, 2 0 Q, 10 0 Q, 30 0 Q"),
        )
        .menu("anvil")
        .extra(json!({"data": [1], "xp_level": 30})),
        clicks(
            "enchanting table item slot holds one",
            stacks("30 diamond_sword 3, 31 lapis_lazuli 64, 32 stone 9"),
            taps("30 0 P, 0 0 P, 0 0 P, 31 0 P, 1 0 P, 32 0 P, 1 0 P"),
        )
        .menu("enchantment"),
        clicks(
            "enchanting table quick move",
            stacks("5 diamond_sword 4, 6 lapis_lazuli 20, 7 stone 3, 0 iron_axe 1"),
            taps("5 0 Q, 6 0 Q, 7 0 Q, 0 0 Q, 1 0 Q, 5 0 Q, 5 0 Q"),
        )
        .menu("enchantment"),
    ]
}

fn workshops_rows_4() -> Vec<Clicks> {
    vec![
        clicks("enchanting table number keys", stacks("0 iron_sword 1, 1 lapis_lazuli 10"), taps("0 2 SW, 1 3 SW")).menu("enchantment"),
        clicks("brewing stand slots take potions, one ingredient kind and fuel", stacks("30 potion 3, 31 glass_bottle 5, 32 blaze_powder 20, 33 nether_wart 8, 34 stone 4, 35 water_bucket 1"), taps("30 0 P, 0 0 P, 1 0 P, 31 0 P, 2 0 P, 31 0 P, 32 0 P, 4 0 P, 32 0 P, 3 0 P, 4 0 P, 33 1 P, 3 0 P, 34 0 P, 3 0 P, 4 0 P, 35 0 P, 1 0 P")).menu("brewing_stand"),
        clicks("brewing stand quick move", stacks("5 blaze_powder 4, 6 nether_wart 6, 7 potion 1, 8 splash_potion 1, 9 glass_bottle 2, 10 stone 3, 36 glass_bottle 1, 0 potion 1, 3 nether_wart 2, 4 blaze_powder 5"), taps("5 0 Q, 5 0 Q, 6 0 Q, 7 0 Q, 8 0 Q, 9 0 Q, 10 0 Q, 36 0 Q, 0 0 Q, 3 0 Q, 4 0 Q")).menu("brewing_stand"),
        clicks("beacon payment slot holds one payment item", stacks("5 iron_ingot 3, 6 emerald 1, 7 stone 2, 30 diamond 4"), taps("5 0 P, 0 0 P, 0 0 P, 6 0 P, 0 0 P, 7 0 P, 0 0 P, 30 1 P, 0 0 P, 0 0 P")).menu("beacon"),
        clicks("beacon quick move", stacks("5 iron_ingot 3, 6 emerald 1, 7 stone 2, 30 diamond 4, 8 netherite_ingot 1, 0 gold_ingot 1"), taps("5 0 Q, 6 0 Q, 7 0 Q, 30 0 Q, 8 0 Q, 0 0 Q, 6 0 Q, 8 0 Q")).menu("beacon"),
        clicks("crafter disabled slot takes nothing", stacks("20 stone 10, 21 stick 3, 2 stone 2"), taps("20 0 P, 1 0 P, 0 0 P, 1 1 P, 21 0 P, 1 0 P, 4 0 P, 2 0 P")).menu("crafter_3x3").extra(json!({"data": [0, 1, 0, 0, 1, 0, 0, 0, 0, 0]})),
        clicks("crafter quick move", stacks("20 stone 10, 21 stick 3, 2 stone 2, 3 dirt 5, 44 stone 8"), taps("20 0 Q, 21 0 Q, 2 0 Q, 3 0 Q, 44 0 Q")).menu("crafter_3x3").extra(json!({"data": [1, 0, 0, 1, 0, 0, 0, 0, 0, 0]})),
        clicks("crafter result slot gives nothing", stacks("45 stick 4, 20 stone 3, 21 stick 5"), taps("45 0 P, 45 1 P, 45 0 Q, 20 0 P, 45 0 P, 21 0 PA, 45 0 TH, 0 2 SW, 45 2 SW")).menu("crafter_3x3"),
        clicks("merchant payment makes the result and taking it pays the price", stacks("3 emerald 5, 4 wheat 40"), taps("3 0 P, 0 0 P, 2 0 P, 10 0 P, 0 0 P, 4 0 P, 1 0 P, 0 0 P")).menu("merchant").extra(json!({"offers": merchant_offers()})),
    ]
}

fn workshops_rows_5() -> Vec<Clicks> {
    vec![
        clicks(
            "merchant demand raises the price",
            stacks("0 wheat 30, 1 paper 2, 10 emerald 3"),
            taps("2 0 P, 11 0 P, 0 0 P, 10 0 P, 0 0 P"),
        )
        .menu("merchant")
        .extra(json!({"offers": merchant_offers()})),
        clicks(
            "merchant takes two payments and an out of stock offer gives nothing",
            stacks("0 emerald 7, 1 paper 5, 5 diamond 2"),
            taps("2 0 P, 12 0 P, 5 0 P, 0 0 P, 0 0 P, 2 0 P"),
        )
        .menu("merchant")
        .extra(json!({"offers": merchant_offers()})),
        clicks(
            "merchant quick move",
            stacks("0 emerald 3, 1 paper 2, 4 emerald 7, 33 wheat 40, 10 bread 12"),
            taps("2 0 Q, 4 0 Q, 33 0 Q, 10 0 Q, 0 0 Q, 1 0 Q"),
        )
        .menu("merchant")
        .extra(json!({"offers": merchant_offers()})),
        clicks(
            "merchant chosen trade moves its payments in",
            stacks("10 emerald 5, 11 wheat 40, 30 wheat 30"),
            taps("2 0 Q, 0 0 P"),
        )
        .menu("merchant")
        .extra(json!({"offers": merchant_offers(), "select": 1})),
        clicks(
            "merchant chosen trade gives the payments back first",
            stacks("0 emerald 3, 1 paper 2, 11 wheat 25, 12 emerald 9"),
            taps("2 0 Q"),
        )
        .menu("merchant")
        .extra(json!({"offers": merchant_offers(), "select": 1})),
        clicks(
            "merchant double click gathers from no slot",
            stacks("3 emerald 5, 4 emerald 5, 0 emerald 3"),
            taps("3 0 P, 4 0 PA"),
        )
        .menu("merchant")
        .extra(json!({"offers": merchant_offers()})),
        clicks(
            "brewing stand number keys",
            stacks("0 potion 1, 4 blaze_powder 6, 30 glass_bottle 1, 31 stone 1"),
            taps("0 2 SW, 4 3 SW, 2 5 SW, 3 6 SW"),
        )
        .menu("brewing_stand"),
    ]
}

pub fn rows() -> Vec<Clicks> {
    [workshops_rows_1(), workshops_rows_2(), workshops_rows_3(), workshops_rows_4(), workshops_rows_5()].concat()
}
