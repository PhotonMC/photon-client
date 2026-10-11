//! The player riding a boat, a horse, a camel, a pig, a strider or a
//! minecart.
//!
//! The player sits in the vehicle as the server's passengers packet puts it
//! there, and the level ticks the vehicle (its own `tick`, then the rider
//! through `rideTick`). Each tick reports the player and the vehicle, and the
//! packets sent: the input, the rotation, the vehicle's position and the
//! paddles.

use serde_json::{Map, Value, json};

use super::scene::{Scene, bubbles, keys, lava_pool, looking, stream, turned, wall};
use crate::bits::{f32_bits, f64_bits};
use crate::case::case;
use crate::movement::world::{cat, column, column_with, floor, floor_block, put, put_with};

/// A ride scene: the vehicle's start and kind, and what else it holds.
struct Ride {
    scene: Scene,
    boat: (f64, f64, f64),
    y_rot: f64,
    motion: Option<(f64, f64, f64)>,
    kind: &'static str,
    extra: Map<String, Value>,
}

fn ride(label: &str, blocks: Vec<Value>, ticks: Vec<Value>) -> Ride {
    Ride {
        scene: Scene::new(label, blocks, ticks),
        boat: (0.5, 64.0, 0.5),
        y_rot: 0.0,
        motion: None,
        kind: "minecraft:oak_boat",
        extra: Map::new(),
    }
}

impl Ride {
    fn at(self, x: f64, y: f64, z: f64) -> Self {
        Self { boat: (x, y, z), ..self }
    }

    fn y_rot(self, y_rot: f64) -> Self {
        Self { y_rot, ..self }
    }

    fn motion(self, x: f64, y: f64, z: f64) -> Self {
        Self { motion: Some((x, y, z)), ..self }
    }

    fn kind(self, kind: &'static str) -> Self {
        Self { kind, ..self }
    }

    fn set(mut self, key: &str, value: Value) -> Self {
        self.extra.insert(key.to_owned(), value);
        self
    }

    fn unset(mut self, key: &str) -> Self {
        self.extra.remove(key);
        self
    }

    fn row(self) -> Value {
        let (x, y, z) = self.boat;
        let mut row = self.scene.start(x, y, z).row();
        let mut riding = json!({
            "type": self.kind, "id": 8, "x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z),
            "y_rot": f32_bits(self.y_rot),
        });
        if let Some((mx, my, mz)) = self.motion {
            riding["motion_x"] = json!(f64_bits(mx));
            riding["motion_y"] = json!(f64_bits(my));
            riding["motion_z"] = json!(f64_bits(mz));
        }
        for (key, value) in self.extra {
            riding[key] = value;
        }
        row["ride"] = riding;
        row
    }
}

/// A tamed horse in a saddle, steered by the player's keys and look,
/// jumping when the jump key is let go of (`AbstractHorse.tickRidden`,
/// `LocalPlayer.jumpableVehicle`).
fn horse(label: &str, blocks: Vec<Value>, ticks: Vec<Value>) -> Ride {
    ride(label, blocks, ticks).kind("minecraft:horse")
}

/// A horse that dashes and sits: a jump charge is a dash, a sitting camel
/// rises when pushed forward, and one that sits or moves between poses stays
/// put.
fn camel(label: &str, blocks: Vec<Value>, ticks: Vec<Value>) -> Ride {
    ride(label, blocks, ticks).kind("minecraft:camel")
}

/// Saddled and steered with a carrot on a stick it goes straight ahead where
/// the rider looks (`Pig.getRiddenInput`); otherwise the rider only sits in
/// it.
fn pig(label: &str, blocks: Vec<Value>, ticks: Vec<Value>) -> Ride {
    ride(label, blocks, ticks)
        .kind("minecraft:pig")
        .set("saddled", json!(true))
        .set("holding", json!("minecraft:carrot_on_a_stick"))
}

/// Steered with a warped fungus on a stick, it goes straight ahead where the
/// rider looks, warm (fast) in or on warm blocks and in lava, cold (slow)
/// elsewhere, and stands on the surface of lava (`Strider.canStandOnFluid`,
/// `floatStrider`).
fn strider(label: &str, blocks: Vec<Value>, ticks: Vec<Value>) -> Ride {
    ride(label, blocks, ticks)
        .kind("minecraft:strider")
        .set("saddled", json!(true))
        .set("holding", json!("minecraft:warped_fungus_on_a_stick"))
}

fn square(low: i64, high: i64, z_high: i64, y: i64) -> Vec<Value> {
    (low..=high)
        .flat_map(|x| (low..=z_high).map(move |z| (x, z)))
        .map(|(x, z)| put(x, y, z, "minecraft:water"))
        .collect()
}

fn lake() -> Vec<Value> {
    square(-12, 12, 12, 64)
}

fn deep_lake() -> Vec<Value> {
    cat![lake(), square(-12, 12, 12, 65)]
}

fn shore() -> Vec<Value> {
    square(-12, 12, -1, 64)
}

fn row_of_blocks(block: &str, xs: std::ops::Range<i64>, z: i64) -> Vec<Value> {
    xs.map(|x| put(x, 64, z, block)).collect()
}

fn cobwebs() -> Vec<Value> {
    (1..4).map(|z| put(0, 64, z, "minecraft:cobweb")).collect()
}

fn on_lake(label: &str, ticks: Vec<Value>) -> Ride {
    ride(label, cat![floor(), lake()], ticks).at(0.5, 64.2, 0.5)
}

fn on_land(label: &str, block: &str, ticks: Vec<Value>) -> Ride {
    ride(label, floor_block(block), ticks)
}

fn deep(label: &str, ticks: Vec<Value>) -> Ride {
    ride(label, cat![floor(), deep_lake()], ticks).at(0.5, 65.5, 0.5)
}

/// Boats on a lake, and dropped into one.
fn lake_rows() -> Vec<Value> {
    vec![
        on_lake("boat-water-idle", keys(30, &[])).row(),
        on_lake("boat-water-forward", keys(40, &["forward"])).row(),
        on_lake("boat-water-backward", keys(30, &["backward"])).row(),
        on_lake("boat-water-turn-left", keys(30, &["left"])).y_rot(30.0).row(),
        on_lake("boat-water-turn-right-forward", keys(40, &["right", "forward"])).y_rot(-45.0).row(),
        on_lake("boat-water-both-turns", keys(20, &["left", "right"])).row(),
        on_lake(
            "boat-water-input-changes",
            [keys(8, &["forward"]), keys(8, &["forward", "left"]), keys(8, &[]), keys(8, &["backward"])].concat(),
        )
        .row(),
        on_lake("boat-water-motion", keys(30, &[])).motion(0.2, 0.0, -0.1).row(),
        ride("boat-drops-into-water", cat![floor(), lake()], keys(40, &[])).at(0.5, 68.0, 0.5).row(),
    ]
}

/// Boats on land, on each floor, in cobweb and powder snow.
fn land_rows() -> Vec<Value> {
    vec![
        ride("boat-drops-onto-land", floor(), keys(30, &[])).at(0.5, 66.0, 0.5).row(),
        ride("boat-land-forward", floor(), keys(40, &["forward"])).row(),
        ride("boat-land-turn", floor(), keys(30, &["right", "forward"])).row(),
        on_land("boat-land-ice", "minecraft:ice", keys(40, &["forward"])).row(),
        on_land("boat-land-blue-ice", "minecraft:blue_ice", [keys(40, &["forward"]), keys(10, &[])].concat()).row(),
        on_land("boat-land-slime", "minecraft:slime_block", keys(30, &["forward"])).row(),
        on_land("boat-land-honey", "minecraft:honey_block", keys(20, &["forward"])).row(),
        on_land("boat-land-soul-sand", "minecraft:soul_sand", keys(20, &["forward"])).row(),
        ride("boat-land-lily-pad", cat![floor(), vec![put(0, 64, 0, "minecraft:lily_pad")]], keys(10, &[])).row(),
        ride("boat-land-fence", cat![floor(), row_of_blocks("minecraft:oak_fence", -3..4, 3)], keys(30, &["forward"]))
            .row(),
        ride(
            "boat-land-stair",
            cat![
                floor(),
                (-2..3)
                    .flat_map(|x| (2..4).map(move |z| put_with(
                        x,
                        64,
                        z,
                        "minecraft:stone_stairs",
                        &[("facing", "north")]
                    )))
                    .collect::<Vec<Value>>(),
            ],
            keys(30, &["forward"]),
        )
        .row(),
        on_land("boat-drops-onto-slime", "minecraft:slime_block", keys(60, &[])).at(0.5, 70.0, 0.5).row(),
        ride(
            "boat-in-cobweb",
            cat![floor(), (64..66).map(|y| put(0, y, 0, "minecraft:cobweb")).collect::<Vec<Value>>()],
            keys(30, &[]),
        )
        .at(0.5, 68.0, 0.5)
        .row(),
        ride("boat-in-powder-snow", cat![floor(), vec![put(0, 64, 0, "minecraft:powder_snow")]], keys(30, &[]))
            .at(0.5, 66.0, 0.5)
            .row(),
    ]
}

/// Boats at a wall, a shore and in deep water.
fn deep_rows() -> Vec<Value> {
    vec![
        on_lake("boat-into-wall", keys(40, &["forward"])).row_with_wall(),
        ride("boat-shore", cat![floor(), shore()], keys(50, &["forward"])).at(0.5, 64.2, -6.5).row(),
        ride("boat-under-water", cat![floor(), deep_lake()], keys(30, &[])).row(),
        deep("boat-surface", keys(30, &[])).row(),
        deep("boat-surface-forward", [keys(40, &["forward"]), keys(20, &["left", "forward"])].concat())
            .y_rot(20.0)
            .row(),
        ride("boat-drops-into-deep-water", cat![floor(), deep_lake()], keys(50, &[])).at(0.5, 72.0, 0.5).row(),
        deep(
            "boat-sprint-sneak-jump",
            [
                keys(8, &["forward", "sprint"]),
                keys(8, &["sneak"]),
                keys(8, &["jump"]),
                keys(8, &["forward", "sneak", "jump"]),
            ]
            .concat(),
        )
        .row(),
        deep("boat-turn-then-keep-going", [keys(30, &["forward", "left"]), keys(30, &["forward"])].concat()).row(),
        ride("boat-under-flowing-water", cat![floor(), stream()], keys(30, &[])).at(0.5, 64.1, -0.5).row(),
        ride("boat-bubble-column", cat![floor(), bubbles()], keys(40, &[])).row(),
        on_lake("boat-chest", keys(20, &["forward"])).kind("minecraft:oak_chest_boat").row(),
        on_lake("raft", keys(20, &["forward"])).kind("minecraft:bamboo_raft").row(),
    ]
}

impl Ride {
    /// The row with the wall across the way added to its blocks.
    fn row_with_wall(self) -> Value {
        let mut row = self.row();
        let blocks = row["blocks"].as_array_mut();
        if let Some(blocks) = blocks {
            blocks.extend(wall());
        }
        row
    }
}

fn horse_rows() -> Vec<Value> {
    let step = row_of_blocks("minecraft:stone", -3..4, 3);
    let fence = row_of_blocks("minecraft:oak_fence", -3..4, 3);
    let stairs: Vec<Value> = (-2..3)
        .flat_map(|x| [3, 4].map(move |z| put_with(x, 64, z, "minecraft:stone_stairs", &[("facing", "north")])))
        .collect();
    let ladder_wall = column(0, 3, 64, 68);
    let go = |count: usize| keys(count, &["forward"]);
    let jump = |count: usize| keys(count, &["jump"]);
    let on = |label: &str, blocks: Vec<Value>, ticks: Vec<Value>| horse(label, cat![floor(), blocks], ticks);
    let on_block = |label: &str, block: &str, ticks: Vec<Value>| horse(label, floor_block(block), ticks);
    let of = |label: &str, kind: &'static str, ticks: Vec<Value>| horse(label, floor(), ticks).kind(kind);
    vec![
        horse("horse-idle", floor(), keys(30, &[])).row(),
        horse("horse-forward", floor(), go(50)).row(),
        horse("horse-backward", floor(), keys(30, &["backward"])).row(),
        horse("horse-strafe", floor(), [keys(30, &["left"]), keys(30, &["right"])].concat()).row(),
        horse("horse-diagonal", floor(), keys(40, &["forward", "left"])).row(),
        horse("horse-sprint-key", floor(), keys(30, &["forward", "sprint"])).row(),
        horse("horse-sneak", floor(), keys(20, &["forward", "sneak"])).row(),
        horse("horse-turn", floor(), turned(go(40), Some(-90.0), None)).row(),
        horse("horse-look", floor(), looking(go(40), &[(0.0, 0.0), (35.0, 40.0), (-80.0, -60.0)])).row(),
        horse("horse-small-jump", floor(), [keys(10, &[]), jump(3), keys(30, &[])].concat()).row(),
        horse("horse-full-jump", floor(), [keys(10, &[]), jump(15), keys(40, &[])].concat()).row(),
        horse("horse-jump-forward", floor(), [go(8), keys(10, &["forward", "jump"]), go(40)].concat()).row(),
        horse("horse-two-jumps", floor(), [keys(10, &[]), jump(5), keys(25, &[]), jump(8), keys(30, &[])].concat())
            .row(),
        on("horse-step-up", step, go(30)).row(),
        on("horse-fence", fence, go(30)).row(),
        on("horse-stairs", stairs, go(30)).row(),
        on("horse-wall", ladder_wall, go(30)).row(),
        on("horse-water", lake(), go(40)).at(0.5, 64.2, -8.5).row(),
        on("horse-deep-water", deep_lake(), [go(40), keys(20, &[])].concat()).at(0.5, 65.0, 0.5).row(),
        on("horse-swim-jump", deep_lake(), [keys(10, &[]), jump(10), keys(30, &[])].concat()).at(0.5, 65.0, 0.5).row(),
        on_block("horse-ice", "minecraft:ice", [go(30), keys(30, &[])].concat()).row(),
        on_block("horse-slime", "minecraft:slime_block", [keys(10, &[]), jump(10), keys(50, &[])].concat()).row(),
        on_block("horse-honey", "minecraft:honey_block", go(30)).row(),
        on_block("horse-soul-sand", "minecraft:soul_sand", go(30)).row(),
        on("horse-cobweb", cobwebs(), go(30)).row(),
        horse("horse-falls", floor(), keys(40, &[])).at(0.5, 75.0, 0.5).row(),
        on("horse-bubbles", bubbles(), keys(40, &[])).row(),
        on("horse-ladder", column_with(0, 2, 64, 70, "minecraft:ladder", &[("facing", "north")]), go(30)).row(),
        of("donkey", "minecraft:donkey", go(30)).row(),
        of("mule-jump", "minecraft:mule", [keys(5, &[]), jump(10), keys(30, &[])].concat()).row(),
        of("skeleton-horse", "minecraft:skeleton_horse", go(30)).row(),
        of("zombie-horse", "minecraft:zombie_horse", go(30)).row(),
        horse("horse-unsaddled", floor(), [go(30), jump(10)].concat()).set("unsaddled", json!(true)).row(),
    ]
}

fn camel_rows() -> Vec<Value> {
    let go = |count: usize| keys(count, &["forward"]);
    let jump = |count: usize| keys(count, &["jump"]);
    let on = |label: &str, blocks: Vec<Value>, ticks: Vec<Value>| camel(label, cat![floor(), blocks], ticks);
    let sitting = |label: &str, ticks: Vec<Value>, age: i64| camel(label, floor(), ticks).set("sit_age", json!(age));
    let sprint = |count: usize| keys(count, &["forward", "sprint"]);
    vec![
        camel("camel-idle", floor(), keys(30, &[])).row(),
        camel("camel-forward", floor(), go(50)).row(),
        camel("camel-sprint", floor(), [go(10), sprint(40)].concat()).row(),
        camel("camel-strafe-back", floor(), [keys(20, &["left"]), keys(20, &["backward"])].concat()).row(),
        camel("camel-look", floor(), looking(go(50), &[(0.0, 0.0), (50.0, 30.0), (-80.0, -40.0)])).row(),
        camel("camel-dash", floor(), [go(5), keys(12, &["forward", "jump"]), go(70)].concat()).row(),
        camel("camel-small-dash", floor(), [go(5), jump(3), go(60)].concat()).row(),
        camel("camel-dash-twice", floor(), [jump(12), keys(80, &[]), jump(12), keys(30, &[])].concat()).row(),
        camel(
            "camel-dash-looking-up",
            floor(),
            looking([keys(5, &[]), jump(12), keys(50, &[])].concat(), &[(0.0, -90.0)]),
        )
        .row(),
        camel("camel-dash-sprint", floor(), [sprint(5), keys(10, &["forward", "sprint", "jump"]), sprint(60)].concat())
            .row(),
        sitting("camel-sitting", keys(30, &[]), 100).row(),
        sitting("camel-sitting-forward", go(60), 100).row(),
        sitting("camel-sitting-dash", [jump(10), keys(30, &[])].concat(), 100).row(),
        sitting("camel-sitting-down", go(60), 10).row(),
        sitting("camel-standing-up", go(60), 2).row(),
        on("camel-step", row_of_blocks("minecraft:stone", -3..4, 3), go(40)).row(),
        on("camel-fence", row_of_blocks("minecraft:oak_fence", -3..4, 3), go(40)).row(),
        on("camel-water", lake(), go(40)).at(0.5, 64.2, -8.5).row(),
        on("camel-deep-water", deep_lake(), [go(40), keys(20, &[])].concat()).at(0.5, 65.0, 0.5).row(),
        camel("camel-sitting-in-water", cat![floor(), lake()], keys(30, &[]))
            .at(0.5, 64.2, 0.5)
            .set("sit_age", json!(100))
            .row(),
        camel("camel-ice", floor_block("minecraft:ice"), [sprint(30), keys(30, &[])].concat()).row(),
        camel("camel-slime-dash", floor_block("minecraft:slime_block"), [jump(10), keys(60, &[])].concat()).row(),
        camel("camel-falls", floor(), keys(40, &[])).at(0.5, 75.0, 0.5).row(),
        camel("camel-husk", floor(), [go(30), jump(10), keys(40, &[])].concat()).kind("minecraft:camel_husk").row(),
    ]
}

fn pig_rows() -> Vec<Value> {
    let ladder_wall = column(0, 3, 64, 68);
    let on = |label: &str, blocks: Vec<Value>, ticks: Vec<Value>| pig(label, cat![floor(), blocks], ticks);
    vec![
        pig("pig-straight", floor(), keys(50, &[])).row(),
        pig("pig-keys-ignored", floor(), [keys(20, &["backward"]), keys(20, &["left"]), keys(10, &["jump"])].concat())
            .row(),
        pig("pig-steer", floor(), looking(keys(60, &[]), &[(0.0, 0.0), (60.0, 20.0), (-120.0, -40.0)])).row(),
        pig("pig-turn", floor(), turned(keys(40, &[]), Some(-90.0), None)).row(),
        pig("pig-no-stick", floor(), keys(30, &["forward"])).unset("holding").row(),
        pig("pig-other-item", floor(), keys(30, &[])).set("holding", json!("minecraft:stick")).row(),
        pig("pig-unsaddled", floor(), keys(30, &[])).row(),
        on("pig-slab", row_of_blocks("minecraft:stone_slab", -3..4, 3), keys(30, &[])).row(),
        on("pig-wall", ladder_wall, keys(30, &[])).row(),
        on("pig-ladder", column_with(0, 2, 64, 70, "minecraft:ladder", &[("facing", "north")]), keys(30, &[])).row(),
        on("pig-water", lake(), keys(40, &[])).at(0.5, 64.2, -8.5).row(),
        on("pig-deep-water", deep_lake(), keys(60, &[])).at(0.5, 65.0, 0.5).row(),
        pig("pig-ice", floor_block("minecraft:ice"), keys(40, &[])).row(),
        pig("pig-honey", floor_block("minecraft:honey_block"), keys(30, &[])).row(),
        on("pig-cobweb", cobwebs(), keys(30, &[])).row(),
        pig("pig-falls", floor(), keys(30, &[])).at(0.5, 75.0, 0.5).row(),
    ]
}

fn strider_rows() -> Vec<Value> {
    let ladder_wall = column(0, 3, 64, 68);
    let on = |label: &str, blocks: Vec<Value>, ticks: Vec<Value>| strider(label, cat![floor(), blocks], ticks);
    let roof: Vec<Value> = (-5..6).flat_map(|x| (-5..6).map(move |z| put(x, 67, z, "minecraft:stone"))).collect();
    vec![
        strider("strider-cold-land", floor(), keys(40, &[])).row(),
        strider("strider-no-stick", floor(), keys(20, &[])).unset("holding").row(),
        strider("strider-unsaddled", floor(), keys(20, &[])).set("saddled", json!(false)).row(),
        strider("strider-steer", floor(), looking(keys(60, &[]), &[(0.0, 0.0), (70.0, 15.0), (-110.0, -30.0)])).row(),
        on("strider-wall", ladder_wall, keys(30, &[])).row(),
        on("strider-step", row_of_blocks("minecraft:stone_slab", -3..4, 3), keys(30, &[])).row(),
        on("strider-lava-walk", lava_pool(1, -5, 5), keys(50, &[])).at(0.5, 64.5, -3.5).row(),
        on("strider-onto-lava", lava_pool(1, -5, 5), keys(60, &[])).at(0.5, 64.0, -9.5).row(),
        on("strider-lava-deep", lava_pool(3, -5, 5), keys(50, &[])).at(0.5, 66.6, -3.5).row(),
        on("strider-sinks-in", lava_pool(2, -5, 5), keys(40, &[])).at(0.5, 64.2, -3.5).row(),
        on("strider-falls-into-lava", lava_pool(1, -5, 5), keys(40, &[])).at(0.5, 75.0, 0.5).row(),
        on("strider-lava-roof", cat![lava_pool(2, -5, 5), roof], keys(40, &[])).at(0.5, 65.0, -3.5).row(),
        on("strider-lava-edge", lava_pool(1, 0, 5), keys(60, &[])).at(-3.5, 64.0, 2.5).y_rot(-90.0).row(),
        strider("strider-warm-floor", floor_block("minecraft:magma_block"), keys(40, &[])).row(),
        on("strider-water", lake(), keys(40, &[])).at(0.5, 64.2, -8.5).row(),
        strider("strider-ice", floor_block("minecraft:ice"), keys(40, &[])).row(),
        on("strider-cobweb", cobwebs(), keys(30, &[])).row(),
    ]
}

/// Riding a minecart, which the server moves: the rider sits at its seat and
/// keeps its own keys to itself, with the minecart standing, rolled to a
/// heading or off the ground.
fn minecart_rows() -> Vec<Value> {
    let cart = |label: &str, ticks: Vec<Value>| ride(label, floor(), ticks).kind("minecraft:minecart");
    vec![
        cart("minecart-idle", keys(30, &[])).row(),
        cart("minecart-keys", [keys(20, &["forward", "left"]), keys(10, &["jump", "sneak"])].concat()).row(),
        cart("minecart-look", looking(keys(40, &[]), &[(0.0, 0.0), (90.0, 20.0), (-45.0, -60.0)])).y_rot(70.0).row(),
        cart("minecart-chest", keys(20, &[])).kind("minecraft:chest_minecart").y_rot(-30.0).row(),
        cart("minecart-motion", keys(30, &[])).motion(0.2, 0.0, 0.1).row(),
        cart("minecart-air", keys(40, &[])).at(0.5, 70.0, 0.5).row(),
    ]
}

pub fn cases() -> Vec<Value> {
    let local = |id: &str, rows: Vec<Value>| case(id, "movement", "local_player_ticks", json!({"rows": rows}));
    vec![
        local("movement/ride-boat", [lake_rows(), land_rows(), deep_rows()].concat()),
        local("movement/ride-horse", horse_rows()),
        local("movement/ride-camel", camel_rows()),
        local("movement/ride-pig", pig_rows()),
        local("movement/ride-strider", strider_rows()),
        local("movement/ride-minecart", minecart_rows()),
    ]
}
