//! Particles ticked in a scene.

use serde_json::{Value, json};

use super::scene::*;
use crate::bits::{f32_bits, f64_bits};
use crate::movement::entities::world::floor;
use crate::movement::world::cat;

fn tick_1() -> Vec<Value> {
    vec![
        // Smoke: what the constructor draws, the sprite that follows the age, and the end of its life.
        row("smoke-in-the-air", "smoke", vec![], (0.5, 70.0, 0.5), STILL, (12345, 40)).row(),
        row("smoke-another-seed", "smoke", vec![], (0.5, 70.0, 0.5), STILL, (-987654321, 40)).row(),
        row("smoke-with-a-push", "smoke", vec![], (0.5, 70.0, 0.5), (0.02, 0.05, -0.03), (99, 40)).row(),
        row("smoke-one-sprite", "smoke", vec![], (0.5, 70.0, 0.5), STILL, (5, 30)).sprites(1).row(),
        row("smoke-three-sprites", "smoke", vec![], (0.5, 70.0, 0.5), STILL, (6, 30)).sprites(3).row(),
        // With physics: a ceiling stops its rise (and speeds the sideways drift up), a wall and a floor stop it.
        row("smoke-under-a-ceiling", "smoke", particle_ceiling(), (0.5, 66.0, 0.5), (0.0, 0.1, 0.0), (12345, 40)).row(),
        row(
            "smoke-along-a-wall",
            "smoke",
            cat![particle_wall(), floor()],
            (1.5, 65.0, 0.5),
            (0.1, 0.0, 0.0),
            (321, 40),
        )
        .row(),
        row("smoke-onto-a-floor", "smoke", floor(), (0.5, 64.5, 0.5), (0.05, -0.3, 0.0), (77, 40)).row(),
    ]
}

fn tick_2() -> Vec<Value> {
    vec![
        row("smoke-beside-a-slab", "smoke", particle_slab(), (0.9, 66.2, 0.5), (-0.1, 0.0, 0.0), (8, 30)).row(),
        // Flame: no physics (it goes through blocks), shrinks, glows, and picks its sprite from the engine's random.
        row("flame-in-the-air", "flame", vec![], (0.5, 70.0, 0.5), STILL, (12345, 30)).sprites(1).row(),
        row("flame-another-seed", "flame", vec![], (0.5, 70.0, 0.5), (0.0, 0.02, 0.0), (4242, 30)).sprites(1).row(),
        row("flame-through-a-ceiling", "flame", particle_ceiling(), (0.5, 66.0, 0.5), (0.0, 0.1, 0.0), (12345, 40))
            .sprites(1)
            .row(),
        row("flame-inside-stone", "flame", particle_solid(), (0.5, 64.0, 0.5), STILL, (12345, 30)).sprites(1).row(),
        row("flame-three-sprites", "flame", vec![], (0.5, 70.0, 0.5), STILL, (12345, 10))
            .sprites(3)
            .engine_seed(11)
            .row(),
        row("flame-three-sprites-other-engine", "flame", vec![], (0.5, 70.0, 0.5), STILL, (12345, 10))
            .sprites(3)
            .engine_seed(12)
            .row(),
        row("small-flame", "small_flame", vec![], (0.5, 70.0, 0.5), STILL, (12345, 30)).sprites(1).row(),
    ]
}

fn tick_3() -> Vec<Value> {
    vec![
        // The rest of the smoke family: two and a half times as big, tinted, ash that falls (white ash
        // pushed by the provider's random), a dust plume that slows each tick.
        row("large-smoke", "large_smoke", vec![], (0.5, 70.0, 0.5), (0.0, 0.02, 0.0), (12345, 40)).row(),
        row(
            "large-smoke-under-a-ceiling",
            "large_smoke",
            particle_ceiling(),
            (0.5, 66.0, 0.5),
            (0.0, 0.1, 0.0),
            (77, 40),
        )
        .row(),
        row("white-smoke", "white_smoke", vec![], (0.5, 70.0, 0.5), STILL, (4242, 40)).row(),
        row("ash", "ash", vec![], (0.5, 70.0, 0.5), (0.3, 0.3, 0.3), (12345, 60)).row(),
        row("ash-onto-a-floor", "ash", floor(), (0.5, 64.3, 0.5), STILL, (99, 80)).row(),
        row("white-ash", "white_ash", vec![], (0.5, 70.0, 0.5), STILL, (12345, 60)).engine_seed(13).row(),
        row("white-ash-other-engine", "white_ash", vec![], (0.5, 70.0, 0.5), STILL, (12345, 30)).engine_seed(14).row(),
        row("dust-plume", "dust_plume", vec![], (0.5, 70.0, 0.5), (0.02, 0.0, -0.01), (12345, 30)).row(),
    ]
}

fn tick_4() -> Vec<Value> {
    vec![
        row("dust-plume-onto-a-floor", "dust_plume", floor(), (0.5, 64.2, 0.5), STILL, (8, 30)).row(),
        // Soul flames: they rise with physics (a ceiling stops them), and the sculk soul glows.
        row("soul", "soul", vec![], (0.5, 70.0, 0.5), (0.0, 0.03, 0.0), (12345, 30)).row(),
        row("soul-under-a-ceiling", "soul", particle_ceiling(), (0.5, 67.3, 0.5), (0.0, 0.08, 0.0), (321, 30)).row(),
        row("sculk-soul", "sculk_soul", vec![], (0.5, 70.0, 0.5), (0.0, 0.03, 0.0), (12345, 30)).row(),
        // Campfire smoke wanders up and fades out at its end; the signal fire's lives longer.
        row("campfire-smoke", "campfire_cosy_smoke", vec![], (0.5, 70.0, 0.5), (0.0, 0.07, 0.0), (12345, 140))
            .sprites(3)
            .engine_seed(21)
            .row(),
        row(
            "campfire-smoke-under-a-ceiling",
            "campfire_cosy_smoke",
            particle_ceiling(),
            (0.5, 66.5, 0.5),
            (0.0, 0.07, 0.0),
            (5, 90),
        )
        .sprites(3)
        .row(),
        row("signal-smoke", "campfire_signal_smoke", vec![], (0.5, 70.0, 0.5), (0.0, 0.07, 0.0), (12345, 60))
            .sprites(3)
            .engine_seed(22)
            .row(),
        // A player's cloud grows in like smoke and goes through blocks; a sneeze is green and faint.
        row("cloud", "cloud", vec![], (0.5, 70.0, 0.5), (0.01, 0.05, 0.0), (12345, 40)).row(),
    ]
}

fn tick_5() -> Vec<Value> {
    vec![
        row("cloud-inside-stone", "cloud", particle_solid(), (0.5, 64.0, 0.5), STILL, (77, 30)).row(),
        row("sneeze", "sneeze", vec![], (0.5, 70.0, 0.5), (0.0, 0.0, 0.05), (12345, 30)).row(),
        // Bubbles rise in water and go out of it; the column's rise faster and live longer.
        row(
            "bubble-in-water",
            "bubble",
            cat![particle_pool(), particle_pool_floor()],
            (0.5, 63.0, 0.5),
            (0.0, 0.1, 0.0),
            (12345, 30),
        )
        .row(),
        row("bubble-to-the-surface", "bubble", particle_pool(), (0.5, 66.8, 0.5), (0.0, 0.3, 0.0), (77, 20)).row(),
        row("bubble-in-air", "bubble", vec![], (0.5, 70.0, 0.5), STILL, (5, 5)).row(),
        row(
            "bubble-column-up",
            "bubble_column_up",
            cat![particle_pool(), particle_pool_floor()],
            (0.5, 60.2, 0.5),
            (0.0, 0.0, 0.0),
            (12345, 60),
        )
        .row(),
        row(
            "bubble-column-up-sideways",
            "bubble_column_up",
            particle_pool(),
            (2.5, 63.0, 0.5),
            (0.4, 0.1, 0.0),
            (99, 40),
        )
        .row(),
        row("bubble-pop", "bubble_pop", vec![], (0.5, 70.0, 0.5), (0.01, 0.02, 0.0), (12345, 8)).row(),
    ]
}

fn tick_6() -> Vec<Value> {
    vec![
        // Rain: a speck thrown up that falls and goes on landing, in a fluid or inside a block's shape.
        row("rain-onto-a-floor", "rain", floor(), (0.5, 64.5, 0.5), STILL, (12345, 30)).row(),
        row("rain-into-water", "rain", particle_pool(), (0.5, 67.2, 0.5), STILL, (77, 30)).row(),
        row("rain-onto-a-slab", "rain", particle_slab(), (0.3, 67.0, 0.3), STILL, (8, 30)).row(),
        row("rain-other-seed", "rain", floor(), (0.5, 64.5, 0.5), STILL, (-4242, 30)).row(),
        // A splash falls slower, and a push only sideways throws it so.
        row("splash", "splash", floor(), (0.5, 64.5, 0.5), STILL, (12345, 30)).row(),
        row("splash-sideways", "splash", floor(), (0.5, 64.5, 0.5), (0.1, 0.0, -0.05), (12345, 30)).row(),
        row("splash-pushed-up", "splash", floor(), (0.5, 64.5, 0.5), (0.1, 0.2, 0.0), (12345, 30)).row(),
        // A fishing bobber's wake grows and turns through four pictures.
        row("fishing-wake", "fishing", particle_pool(), (0.5, 67.0, 0.5), (0.02, 0.0, 0.01), (12345, 30))
            .sprites(4)
            .row(),
    ]
}

fn tick_7() -> Vec<Value> {
    vec![
        // A downward current spirals through the water, and goes on the floor or out of it.
        row(
            "current-down",
            "current_down",
            cat![particle_pool(), particle_pool_floor()],
            (0.5, 65.0, 0.5),
            STILL,
            (12345, 100),
        )
        .row(),
        row("current-down-out-of-water", "current_down", particle_pool(), (0.5, 60.1, 0.5), STILL, (7, 30)).row(),
        // Squid ink fades in its second half and sinks in the air; the glow squid's is teal.
        row("squid-ink-in-air", "squid_ink", vec![], (0.5, 70.0, 0.5), (0.05, 0.0, 0.0), (12345, 30)).row(),
        row("squid-ink-in-water", "squid_ink", particle_pool(), (0.5, 63.0, 0.5), (0.0, 0.05, 0.05), (99, 30)).row(),
        row("glow-squid-ink", "glow_squid_ink", particle_pool(), (0.5, 63.0, 0.5), (0.05, 0.0, 0.0), (12345, 30)).row(),
        // Dust: a colour darkened a little at random, a size and a life by the scale (held to 0.01-4);
        // a transition draws between its two colours.
        row("dust-red", "dust", vec![], (0.5, 70.0, 0.5), STILL, (12345, 30))
            .set("color", json!(16711680))
            .set("scale", json!(f32_bits(1.0)))
            .row(),
        row("dust-big-blue", "dust", floor(), (0.5, 64.2, 0.5), (0.0, -0.2, 0.0), (77, 40))
            .set("color", json!(3368703))
            .set("scale", json!(f32_bits(3.5)))
            .row(),
        row("dust-too-small", "dust", vec![], (0.5, 70.0, 0.5), STILL, (5, 10))
            .set("color", json!(65280))
            .set("scale", json!(f32_bits(0.001)))
            .row(),
    ]
}

fn tick_8() -> Vec<Value> {
    vec![
        row("dust-transition", "dust_color_transition", vec![], (0.5, 70.0, 0.5), STILL, (12345, 30))
            .set("color", json!(16746496))
            .set("to_color", json!(35071))
            .set("scale", json!(f32_bits(1.5)))
            .row(),
        // Spells: a sideways push from the class's own random, then rising; the instant effect's colour
        // and power, the mob effect's colour and opacity (eased back each tick), the witch's purple.
        row("spell-infested", "infested", vec![], (0.5, 70.0, 0.5), STILL, (12345, 30))
            .set("spell_seed", json!(31))
            .row(),
        row("spell-instant", "effect", vec![], (0.5, 70.0, 0.5), (0.0, 0.1, 0.0), (12345, 30))
            .set("spell_seed", json!(32))
            .set("color", json!(16755234))
            .set("power", json!(f32_bits(1.5)))
            .row(),
        row("spell-mob-effect", "entity_effect", vec![], (0.5, 70.0, 0.5), (0.2, 0.1, 0.0), (12345, 30))
            .set("spell_seed", json!(33))
            .set("color", json!(2151983871_i64))
            .row(),
        row("spell-witch", "witch", vec![], (0.5, 70.0, 0.5), STILL, (12345, 30))
            .engine_seed(34)
            .set("spell_seed", json!(35))
            .row(),
        // Totem: green or yellow, falling fast and fading out; glows of every kind.
        row("totem", "totem_of_undying", vec![], (0.5, 70.0, 0.5), (0.1, 0.5, -0.1), (12345, 75)).row(),
        row("totem-other-seed", "totem_of_undying", vec![], (0.5, 70.0, 0.5), (0.0, 0.6, 0.0), (2, 20)).row(),
        row("glow", "glow", vec![], (0.5, 70.0, 0.5), (0.0, 0.05, 0.0), (12345, 30)).engine_seed(41).row(),
    ]
}

fn tick_9() -> Vec<Value> {
    vec![
        row("glow-pushed", "glow", vec![], (0.5, 70.0, 0.5), (0.1, 0.05, 0.0), (12345, 30)).engine_seed(42).row(),
        row("electric-spark", "electric_spark", vec![], (0.5, 70.0, 0.5), (0.4, 0.4, 0.0), (12345, 6))
            .engine_seed(43)
            .row(),
        row("scrape", "scrape", vec![], (0.5, 70.0, 0.5), (1.0, 2.0, 0.0), (12345, 45)).engine_seed(44).row(),
        row("wax-on", "wax_on", vec![], (0.5, 70.0, 0.5), (1.0, 2.0, 3.0), (12345, 45)).engine_seed(45).row(),
        row("wax-off", "wax_off", vec![], (0.5, 70.0, 0.5), (1.0, 2.0, 3.0), (12345, 45)).engine_seed(46).row(),
        // Portals spiral in from where they were pushed to, the reverse ones drift out faster; the
        // enchanting table's runes and the nautilus' fall back to where they started, the vault's glow
        // and fade in; the ominous spawn goes straight, blue to white.
        row("portal", "portal", vec![], (0.5, 70.0, 0.5), (0.8, -0.4, 0.3), (12345, 55)).row(),
        row("reverse-portal", "reverse_portal", vec![], (0.5, 70.0, 0.5), (0.05, 0.02, -0.03), (12345, 65)).row(),
        row("enchant", "enchant", vec![], (0.5, 70.0, 0.5), (1.5, 1.0, -0.5), (12345, 45)).row(),
    ]
}

fn tick_10() -> Vec<Value> {
    vec![
        row("nautilus", "nautilus", vec![], (0.5, 70.0, 0.5), (-0.5, 0.5, 0.5), (77, 45)).row(),
        row("vault-connection", "vault_connection", vec![], (0.5, 70.0, 0.5), (2.0, 0.0, 1.0), (12345, 45)).row(),
        row("ominous-spawning", "ominous_spawning", vec![], (0.5, 70.0, 0.5), (1.0, 2.0, 0.5), (12345, 35))
            .set("level_seed", json!(51))
            .row(),
        // Hanging specks: underwater, warped spores, the spore blossom's air (slow to fall, long-lived);
        // town specks drift through blocks: mycelium, a happy villager, a cracked egg, a composter (its life
        // from the level's random) and a dolphin's (faint at random, half the life).
        row("underwater", "underwater", particle_pool(), (0.5, 63.0, 0.5), STILL, (12345, 40)).row(),
        row("warped-spore", "warped_spore", vec![], (0.5, 70.0, 0.5), STILL, (12345, 40)).engine_seed(52).row(),
        row("spore-blossom-air", "spore_blossom_air", floor(), (0.5, 66.0, 0.5), STILL, (12345, 60))
            .engine_seed(53)
            .row(),
        row("mycelium", "mycelium", particle_solid(), (0.5, 64.0, 0.5), (0.2, 0.3, -0.1), (12345, 40)).row(),
        row("happy-villager", "happy_villager", vec![], (0.5, 70.0, 0.5), (0.1, 0.1, 0.1), (12345, 40)).row(),
    ]
}

fn tick_11() -> Vec<Value> {
    vec![
        row("egg-crack", "egg_crack", vec![], (0.5, 70.0, 0.5), (0.0, 0.5, 0.0), (77, 30)).row(),
        row("composter", "composter", vec![], (0.5, 70.0, 0.5), (0.0, 0.2, 0.0), (12345, 10))
            .set("level_seed", json!(54))
            .row(),
        row("dolphin", "dolphin", particle_pool(), (0.5, 63.0, 0.5), (0.3, 0.0, 0.3), (12345, 30))
            .engine_seed(55)
            .row(),
        // Hits: a crit ticks once as it is made (a damage indicator rises and lasts twenty, the enchanted
        // one is bluer); a sweep, an explosion and a sonic boom stand still, full bright, sized by the
        // first push; a poof rises; hearts and notes bob up (a note's colour from its push, under a
        // ceiling that speeds it sideways); gusts stand still; sculk charges turn by their roll and glow.
        row("crit", "crit", vec![], (0.5, 70.0, 0.5), (0.4, 0.2, -0.3), (12345, 12)).engine_seed(61).row(),
        row("damage-indicator", "damage_indicator", vec![], (0.5, 70.0, 0.5), (0.0, 0.1, 0.0), (77, 22)).row(),
        row("enchanted-hit", "enchanted_hit", vec![], (0.5, 70.0, 0.5), (-0.2, 0.3, 0.1), (12345, 12)).row(),
        row("sweep-attack", "sweep_attack", vec![], (0.5, 70.0, 0.5), (0.3, 0.0, 0.0), (12345, 6)).row(),
        row("poof", "poof", vec![], (0.5, 70.0, 0.5), (0.02, 0.05, -0.01), (12345, 50)).row(),
    ]
}

fn tick_12() -> Vec<Value> {
    vec![
        row("explosion", "explosion", vec![], (0.5, 70.0, 0.5), (0.5, 0.0, 0.0), (12345, 12)).row(),
        row("sonic-boom", "sonic_boom", vec![], (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), (12345, 18)).row(),
        row("heart", "heart", vec![], (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), (12345, 18)).engine_seed(62).row(),
        row("angry-villager", "angry_villager", vec![], (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), (77, 18)).row(),
        row("note", "note", particle_ceiling(), (0.5, 67.5, 0.5), (0.375, 0.0, 0.0), (12345, 8)).engine_seed(63).row(),
        row("gust", "gust", vec![], (0.5, 70.0, 0.5), (0.25, 0.0, 0.0), (12345, 17)).row(),
        row("small-gust", "small_gust", vec![], (0.5, 70.0, 0.5), (0.0, 0.0, 0.0), (77, 17)).row(),
        row("sculk-charge", "sculk_charge", vec![], (0.5, 70.0, 0.5), (0.01, 0.02, 0.0), (12345, 22))
            .engine_seed(64)
            .set("roll", json!(f32_bits(0.7)))
            .row(),
    ]
}

fn tick_13() -> Vec<Value> {
    vec![
        // Drops: one hangs (a cooling one reddening) and lets the falling one go when its time is up; one
        // falling lands on the floor (some with a sound, not compared here) or vanishes into water; one landed
        // lies there; honey, nectar, spore blossom, tears (glowing) and dripstone drops at their own pace.
        row("dripping-water", "dripping_water", floor(), (0.5, 66.5, 0.5), STILL, (12345, 44)).engine_seed(66).row(),
        row("dripping-lava", "dripping_lava", floor(), (0.5, 66.5, 0.5), STILL, (77, 44)).row(),
        row("falling-water", "falling_water", floor(), (0.5, 67.5, 0.5), STILL, (12345, 30)).row(),
        row("falling-water-into-pool", "falling_water", particle_pool(), (0.5, 68.0, 0.5), STILL, (12345, 30)).row(),
        row("falling-lava", "falling_lava", floor(), (0.5, 67.5, 0.5), (0.01, 0.0, 0.0), (77, 30)).row(),
        row("landing-lava", "landing_lava", floor(), (0.5, 66.0, 0.5), STILL, (12345, 30)).row(),
        row("dripping-honey", "dripping_honey", floor(), (0.5, 67.0, 0.5), STILL, (12345, 12)).engine_seed(67).row(),
        row("falling-honey", "falling_honey", floor(), (0.5, 67.0, 0.5), STILL, (77, 40)).row(),
    ]
}

fn tick_14() -> Vec<Value> {
    vec![
        row("landing-honey", "landing_honey", floor(), (0.5, 66.0, 0.5), STILL, (12345, 20)).engine_seed(68).row(),
        row("falling-nectar", "falling_nectar", floor(), (0.5, 67.0, 0.5), STILL, (12345, 30)).engine_seed(69).row(),
        row("falling-spore-blossom", "falling_spore_blossom", floor(), (0.5, 67.0, 0.5), STILL, (77, 40)).row(),
        row("dripping-obsidian-tear", "dripping_obsidian_tear", floor(), (0.5, 67.0, 0.5), STILL, (12345, 12)).row(),
        row("falling-obsidian-tear", "falling_obsidian_tear", floor(), (0.5, 67.0, 0.5), STILL, (77, 40)).row(),
        row("landing-obsidian-tear", "landing_obsidian_tear", floor(), (0.5, 66.0, 0.5), STILL, (12345, 20))
            .engine_seed(70)
            .row(),
        row("dripping-dripstone-lava", "dripping_dripstone_lava", floor(), (0.5, 67.0, 0.5), STILL, (12345, 44)).row(),
        row("falling-dripstone-water", "falling_dripstone_water", floor(), (0.5, 67.0, 0.5), STILL, (77, 30)).row(),
    ]
}

fn tick_15() -> Vec<Value> {
    vec![
        // Leaves fall: a cherry petal drifts away on the wind, a pale oak leaf and a tinted one swirl;
        // each is gone on the ground or when a wall stops it.
        row("cherry-leaves", "cherry_leaves", floor(), (0.5, 70.0, 0.5), STILL, (12345, 80)).engine_seed(71).row(),
        row("pale-oak-leaves", "pale_oak_leaves", floor(), (0.5, 70.0, 0.5), STILL, (77, 80)).engine_seed(72).row(),
        row("tinted-leaves", "tinted_leaves", particle_wall(), (1.5, 67.0, 0.5), STILL, (12345, 120))
            .engine_seed(73)
            .set("color", json!(-10380959))
            .row(),
        // Crumbs of an item fall, a quarter of its picture each: one pushed (a tenth of its own spread
        // kept), one not; the sulfur cube's goo of its own picture.
        row("item-crumb", "item_crumb", floor(), (0.5, 67.0, 0.5), (0.05, 0.2, -0.03), (12345, 30)).row(),
        row("item-crumb-still", "item_crumb_still", floor(), (0.5, 67.0, 0.5), (0.05, 0.2, -0.03), (77, 30)).row(),
        row("sulfur-cube-goo", "sulfur_cube_goo", floor(), (0.5, 67.0, 0.5), STILL, (12345, 30)).row(),
        row("end-rod", "end_rod", floor(), (0.5, 66.5, 0.5), (0.003, -0.002, 0.001), (12345, 70)).row(),
        row("sculk-charge-pop", "sculk_charge_pop", vec![], (0.5, 70.0, 0.5), (0.0, 0.03, 0.0), (12345, 12))
            .engine_seed(65)
            .row(),
    ]
}

fn tick_16() -> Vec<Value> {
    vec![
        // A vibration signal closes in on a block position a share of the remaining distance each
        // tick, gone once it arrives; a trail does the same toward a fixed point, tinted at random.
        row("vibration-to-a-block", "vibration", vec![], (0.5, 70.0, 0.5), STILL, (12345, 12))
            .set("destination_x", json!(3))
            .set("destination_y", json!(72))
            .set("destination_z", json!(1))
            .set("arrival_in_ticks", json!(10))
            .row(),
        row("vibration-another-destination", "vibration", vec![], (0.5, 70.0, 0.5), STILL, (77, 8))
            .engine_seed(51)
            .set("destination_x", json!(-2))
            .set("destination_y", json!(68))
            .set("destination_z", json!(4))
            .set("arrival_in_ticks", json!(6))
            .row(),
        row("trail-to-a-point", "trail", vec![], (0.5, 70.0, 0.5), STILL, (12345, 24))
            .set("target_x", json!(f64_bits(2.5)))
            .set("target_y", json!(f64_bits(71.0)))
            .set("target_z", json!(f64_bits(-1.5)))
            .set("color", json!(16746496))
            .set("duration", json!(20))
            .row(),
        row("trail-pushed", "trail", vec![], (0.5, 70.0, 0.5), (0.1, 0.05, -0.08), (77, 10))
            .engine_seed(52)
            .set("target_x", json!(f64_bits(-3.0)))
            .set("target_y", json!(f64_bits(69.0)))
            .set("target_z", json!(f64_bits(0.5)))
            .set("color", json!(3394815))
            .set("duration", json!(6))
            .row(),
        // A firework's spark with nothing but the provider's, a flash of a colour, the dragon's breath thrown
        // by its power, and a trial spawner's puff.
        row("firework-spark", "firework", vec![], (0.5, 70.0, 0.5), (0.0, 0.1, 0.0), (12345, 70)).row(),
        row("firework-spark-another-seed", "firework", vec![], (0.5, 70.0, 0.5), (0.02, -0.05, 0.0), (99, 70)).row(),
        row("flash-orange", "flash", vec![], (0.5, 70.0, 0.5), STILL, (12345, 6)).set("color", json!(16746496)).row(),
        row("flash-another-seed", "flash", vec![], (0.5, 70.0, 0.5), STILL, (7, 6))
            .engine_seed(9)
            .set("color", json!(1056816))
            .row(),
    ]
}

fn tick_17() -> Vec<Value> {
    vec![
        row("dragon-breath", "dragon_breath", vec![], (0.5, 70.0, 0.5), (1.5, 0.2, -0.5), (12345, 60))
            .set("power", json!(f32_bits(2.5)))
            .row(),
        row("dragon-breath-weak", "dragon_breath", vec![], (0.5, 70.0, 0.5), (0.1, 0.05, 0.3), (31, 60))
            .engine_seed(3)
            .set("power", json!(f32_bits(0.25)))
            .row(),
        row(
            "trial-spawner-detection",
            "trial_spawner_detection",
            vec![],
            (0.5, 70.0, 0.5),
            (0.0, 0.05, 0.0),
            (12345, 16),
        )
        .row(),
        row(
            "trial-spawner-detection-ominous",
            "trial_spawner_detection_ominous",
            vec![],
            (0.5, 70.0, 0.5),
            (0.01, 0.0, -0.02),
            (55, 16),
        )
        .row(),
        row(
            "trial-spawner-detection-under-a-ceiling",
            "trial_spawner_detection",
            particle_ceiling(),
            (0.5, 66.0, 0.5),
            (0.0, 0.2, 0.0),
            (77, 16),
        )
        .row(),
        // A shriek sits frozen while its delay counts down, then rises and ages normally
        // (its own lifetime of 30 regardless of the row's tick count).
        row("shriek-with-a-delay", "shriek", vec![], (0.5, 70.0, 0.5), STILL, (12345, 10)).set("delay", json!(5)).row(),
        row("shriek-no-delay", "shriek", vec![], (0.5, 70.0, 0.5), STILL, (77, 8))
            .engine_seed(53)
            .set("delay", json!(0))
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [
        tick_1(),
        tick_2(),
        tick_3(),
        tick_4(),
        tick_5(),
        tick_6(),
        tick_7(),
        tick_8(),
        tick_9(),
        tick_10(),
        tick_11(),
        tick_12(),
        tick_13(),
        tick_14(),
        tick_15(),
        tick_16(),
        tick_17(),
    ]
    .concat()
}
