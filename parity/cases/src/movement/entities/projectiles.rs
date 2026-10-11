//! Projectiles the server moves in coarse steps: the client carries them on between its packets.

use crate::movement::entities::prelude::*;

const HURTING: [&str; 6] = [
    "minecraft:fireball",
    "minecraft:small_fireball",
    "minecraft:dragon_fireball",
    "minecraft:wither_skull",
    "minecraft:wind_charge",
    "minecraft:breeze_wind_charge",
];

/// The blocks around: a floor and a deep pool.
fn lake() -> Vec<Value> {
    cat![floor(), pool("minecraft:water", 4, -6, 6)]
}

fn open_air() -> Vec<Value> {
    HURTING
        .iter()
        .map(|kind| {
            let name = kind.split(':').nth(1).unwrap_or(kind);
            proj(&format!("{name}-open-air"), kind, floor(), (0.5, 66.0, 0.5), (0.25, 0.0625, -0.125), 30)
                .y_rot(-90.0)
                .x_rot(0.0)
                .row()
        })
        .collect()
}

fn projectile_1() -> Vec<Value> {
    vec![
        proj("wither-skull-blue", "minecraft:wither_skull", floor(), (0.5, 66.0, 0.5), (0.25, 0.0625, -0.125), 30)
            .dangerous(true)
            .row(),
        // Turns: straight up and down, backwards across the yaw seam, and a standstill.
        proj("fireball-up", "minecraft:fireball", floor(), (0.5, 66.0, 0.5), (0.0, 0.5, 0.0), 20).row(),
        proj("fireball-down", "minecraft:fireball", floor(), (0.5, 80.0, 0.5), (0.0, -0.5, 0.0), 20).row(),
        proj("fireball-across-the-seam", "minecraft:fireball", floor(), (0.5, 66.0, 0.5), (-0.3, 0.0, -0.01), 20)
            .y_rot(170.0)
            .row(),
        proj("fireball-standing", "minecraft:fireball", floor(), (0.5, 66.0, 0.5), (0.0, 0.0, 0.0), 10).row(),
        proj("wind-charge-standing", "minecraft:wind_charge", floor(), (0.5, 66.0, 0.5), (0.0, 0.0, 0.0), 10).row(),
    ]
}

fn projectile_2() -> Vec<Value> {
    vec![
        // Blocks: into a wall, and past a ledge.
        proj(
            "fireball-into-the-wall",
            "minecraft:fireball",
            cat![floor(), wall()],
            (0.5, 65.0, 0.5),
            (0.5, 0.0, 0.0),
            20,
        )
        .y_rot(-90.0)
        .row(),
        proj(
            "small-fireball-into-the-wall",
            "minecraft:small_fireball",
            cat![floor(), wall()],
            (0.5, 65.0, 0.5),
            (0.5, 0.0625, 0.03125),
            20,
        )
        .y_rot(-90.0)
        .row(),
        proj(
            "wind-charge-into-the-wall",
            "minecraft:wind_charge",
            cat![floor(), wall()],
            (0.5, 65.0, 0.5),
            (0.5, 0.0, 0.0),
            20,
        )
        .y_rot(-90.0)
        .row(),
        proj("fireball-into-the-floor", "minecraft:fireball", floor(), (0.5, 65.0, 0.5), (0.25, -0.5, 0.0), 20).row(),
        proj(
            "fireball-under-the-ceiling",
            "minecraft:fireball",
            cat![floor(), ceiling()],
            (0.5, 64.5, 0.5),
            (0.0, 0.5, 0.0),
            20,
        )
        .row(),
        // Water: it slows (but a wind charge does not), and a stream pushes it.
        proj("fireball-into-water", "minecraft:fireball", lake(), (0.5, 70.0, 0.5), (0.25, -0.5, 0.0), 40).row(),
    ]
}

fn projectile_3() -> Vec<Value> {
    vec![
        proj("small-fireball-in-water", "minecraft:small_fireball", lake(), (0.5, 65.0, 0.5), (0.25, 0.0, 0.0), 40)
            .row(),
        proj("wind-charge-in-water", "minecraft:wind_charge", lake(), (0.5, 65.0, 0.5), (0.25, 0.0, 0.0), 40).row(),
        proj("wither-skull-in-water", "minecraft:wither_skull", lake(), (0.5, 65.0, 0.5), (0.25, 0.0, 0.0), 40).row(),
        proj(
            "fireball-in-a-stream",
            "minecraft:fireball",
            cat![floor(), stream()],
            (1.5, 64.5, 0.5),
            (0.0, 0.0, 0.0),
            40,
        )
        .row(),
        proj(
            "wind-charge-in-a-stream",
            "minecraft:wind_charge",
            cat![floor(), stream()],
            (1.5, 64.5, 0.5),
            (0.0, 0.0, 0.0),
            40,
        )
        .row(),
        proj(
            "fireball-in-lava",
            "minecraft:fireball",
            cat![floor(), pool("minecraft:lava", 3, -3, 3)],
            (0.5, 65.5, 0.5),
            (0.125, 0.0, 0.0),
            30,
        )
        .row(),
    ]
}

fn projectile_4() -> Vec<Value> {
    vec![
        // Entities: a target in the way, the owner it leaves, one it starts inside.
        proj("fireball-at-a-pig", "minecraft:fireball", floor(), (0.5, 64.5, 0.5), (0.5, 0.0, 0.0), 20)
            .y_rot(-90.0)
            .others(vec![(30, "minecraft:pig", (4.5, 64.0, 0.5))])
            .row(),
        proj("fireball-leaves-its-owner", "minecraft:fireball", floor(), (0.5, 65.0, 0.5), (0.25, 0.0, 0.0), 30)
            .y_rot(-90.0)
            .owner(30)
            .others(vec![(30, "minecraft:blaze", (0.5, 64.0, 0.5)), (31, "minecraft:pig", (5.5, 64.0, 0.5))])
            .row(),
        proj("fireball-from-inside-a-pig", "minecraft:fireball", floor(), (0.5, 64.5, 0.5), (0.25, 0.0, 0.0), 20)
            .y_rot(-90.0)
            .others(vec![(30, "minecraft:pig", (0.5, 64.0, 0.5))])
            .row(),
        proj("fireball-grazes-a-pig", "minecraft:fireball", floor(), (0.5, 64.5, 0.5), (0.5, 0.0, 0.0), 20)
            .y_rot(-90.0)
            .others(vec![(30, "minecraft:pig", (4.5, 64.0, 0.9))])
            .row(),
        // The shulker bullet drifts on the motion it was sent, and turns half the way.
        proj("shulker-bullet", "minecraft:shulker_bullet", floor(), (0.5, 66.0, 0.5), (0.125, 0.0625, -0.25), 20).row(),
        proj("shulker-bullet-turning", "minecraft:shulker_bullet", floor(), (0.5, 66.0, 0.5), (-0.25, 0.0, 0.125), 20)
            .y_rot(120.0)
            .x_rot(30.0)
            .row(),
    ]
}

fn projectile_5() -> Vec<Value> {
    vec![
        proj("shulker-bullet-in-water", "minecraft:shulker_bullet", lake(), (0.5, 65.0, 0.5), (0.125, 0.0, 0.0), 20)
            .row(),
        proj(
            "shulker-bullet-in-a-stream",
            "minecraft:shulker_bullet",
            cat![floor(), stream()],
            (1.5, 64.5, 0.5),
            (0.0, 0.0, 0.0),
            20,
        )
        .row(),
        // The llama's spit: a drag and a fall, and it dies in blocks and in water.
        proj("llama-spit", "minecraft:llama_spit", floor(), (0.5, 66.0, 0.5), (0.5, 0.25, 0.0), 40).y_rot(-90.0).row(),
        proj("llama-spit-into-the-floor", "minecraft:llama_spit", floor(), (0.5, 65.0, 0.5), (0.25, -0.5, 0.0), 20)
            .row(),
        proj(
            "llama-spit-into-the-wall",
            "minecraft:llama_spit",
            cat![floor(), wall()],
            (0.5, 65.0, 0.5),
            (0.5, 0.0, 0.0),
            20,
        )
        .y_rot(-90.0)
        .row(),
        proj("llama-spit-into-water", "minecraft:llama_spit", lake(), (0.5, 70.0, 0.5), (0.0, -0.5, 0.0), 20).row(),
    ]
}

fn projectile_6() -> Vec<Value> {
    vec![
        proj(
            "llama-spit-in-a-stream",
            "minecraft:llama_spit",
            cat![floor(), stream()],
            (1.5, 64.5, 0.5),
            (0.0, 0.0, 0.0),
            20,
        )
        .row(),
        proj("llama-spit-past-a-pig", "minecraft:llama_spit", floor(), (0.5, 64.5, 0.5), (0.5, 0.0, 0.0), 20)
            .y_rot(-90.0)
            .others(vec![(30, "minecraft:pig", (4.5, 64.0, 0.5))])
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [open_air(), projectile_1(), projectile_2(), projectile_3(), projectile_4(), projectile_5(), projectile_6()]
        .concat()
}
