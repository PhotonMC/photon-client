//! Entities in the crosshair: the player picks as `LocalPlayer.raycastHitResult`
//! does, among the blocks and the entities the server told of, and the attack
//! and use keys act on what it lands on. Everything the player sent is
//! compared as the bytes the packets' own codecs write.

use serde_json::{Value, json};

use super::facing;
use crate::bits::f64_bits;
use crate::movement::player::scene::keys;
use crate::movement::world::put;

/// Ticks of the attack, use and pick keys on whatever the aim lands on.
struct Aims {
    count: usize,
    clicks: i64,
    use_clicks: i64,
    down: bool,
    use_down: bool,
    pick_clicks: i64,
    held: &'static [&'static str],
}

fn aims(count: usize) -> Aims {
    Aims { count, clicks: 0, use_clicks: 0, down: false, use_down: false, pick_clicks: 0, held: &[] }
}

impl Aims {
    fn clicks(self, clicks: i64) -> Self {
        Self { clicks, ..self }
    }

    fn use_clicks(self, use_clicks: i64) -> Self {
        Self { use_clicks, ..self }
    }

    fn down(self) -> Self {
        Self { down: true, ..self }
    }

    fn use_down(self) -> Self {
        Self { use_down: true, ..self }
    }

    fn pick(self, pick_clicks: i64) -> Self {
        Self { pick_clicks, ..self }
    }

    fn held(self, held: &'static [&'static str]) -> Self {
        Self { held, ..self }
    }

    fn go(self) -> Vec<Value> {
        let mut ticks = keys(self.count, self.held);
        for (index, tick) in ticks.iter_mut().enumerate() {
            let first = index == 0;
            let mut attack = json!({
                "clicks": if first { self.clicks } else { 0 }, "down": self.down, "pick": true,
                "use": {"clicks": if first { self.use_clicks } else { 0 }, "down": self.use_down},
                "watch": [],
            });
            if self.pick_clicks != 0 {
                attack["pick_item"] = json!({"clicks": if first { self.pick_clicks } else { 0 }});
            }
            tick["attack"] = attack;
        }
        ticks
    }
}

fn mob(id: i64, kind: &str, x: f64, y: f64, z: f64) -> Value {
    json!({"id": id, "type": format!("minecraft:{kind}"), "x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

fn entity_rows_1() -> Vec<Value> {
    let zombie = || mob(20, "zombie", 0.5, 64.0, 2.5);
    let hit_short = || [aims(1).clicks(1).go(), aims(2).go()].concat();
    let use_once = || [aims(1).use_clicks(1).go(), aims(3).go()].concat();
    let at_zombie = |label: &str, ticks: Vec<Value>| facing(label, vec![zombie()], ticks);
    let alone = |label: &str, entity: Value, ticks: Vec<Value>| facing(label, vec![entity], ticks);
    vec![
        at_zombie("attack-zombie", [aims(2).clicks(1).go(), aims(3).go()].concat()).row(),
        at_zombie("attack-twice", [aims(1).clicks(2).go(), aims(3).go()].concat()).row(),
        at_zombie("attack-held", aims(12).down().go()).row(),
        at_zombie("use-on-zombie-empty-hands", use_once()).row(),
        at_zombie("use-on-zombie-with-stone", use_once()).main_hand("minecraft:stone").row(),
        // The swing lasts as long as haste (or the conduit's power) leaves of
        // it, or mining fatigue adds to it (a spear's own length is read the
        // same way for a remote player), and the off hand's arm swings when it
        // is the one that acts.
        at_zombie("attack-with-haste", [aims(2).clicks(1).go(), aims(8).go()].concat())
            .effects(&[("minecraft:haste", 1)])
            .row(),
        at_zombie("attack-with-conduit-power-and-haste", [aims(2).clicks(1).go(), aims(8).go()].concat())
            .effects(&[("minecraft:haste", 0), ("minecraft:conduit_power", 2)])
            .row(),
        at_zombie("attack-with-mining-fatigue", [aims(2).clicks(1).go(), aims(14).go()].concat())
            .effects(&[("minecraft:mining_fatigue", 1)])
            .row(),
        at_zombie("off-hand-use-with-haste", [aims(1).use_clicks(1).go(), aims(6).go()].concat())
            .off_hand("minecraft:cobblestone")
            .effects(&[("minecraft:haste", 0)])
            .row(),
        at_zombie("use-on-zombie-both-hands", use_once())
            .main_hand("minecraft:stone")
            .off_hand("minecraft:cobblestone")
            .row(),
        at_zombie("use-held-on-zombie", aims(10).use_down().go()).row(),
        at_zombie(
            "use-on-zombie-sneaking",
            [aims(1).use_clicks(1).held(&["sneak"]).go(), aims(3).held(&["sneak"]).go()].concat(),
        )
        .row(),
        alone("pig-from-above", mob(21, "pig", 0.5, 64.0, 2.5), hit_short()).x_rot(30.0).row(),
        alone("cow-below-the-aim", mob(22, "cow", 0.5, 64.0, 2.5), hit_short()).row(),
        alone("armor-stand", mob(23, "armor_stand", 0.5, 64.0, 2.5), hit_short()).row(),
    ]
}

fn entity_rows_2() -> Vec<Value> {
    let zombie = || mob(20, "zombie", 0.5, 64.0, 2.5);
    let hit = || [aims(1).clicks(1).go(), aims(3).go()].concat();
    let hit_short = || [aims(1).clicks(1).go(), aims(2).go()].concat();
    let at_zombie = |label: &str, ticks: Vec<Value>| facing(label, vec![zombie()], ticks);
    let alone = |label: &str, entity: Value, ticks: Vec<Value>| facing(label, vec![entity], ticks);
    vec![
        facing(
            "nearest-of-two",
            vec![mob(24, "skeleton", 0.5, 64.0, 2.6), mob(25, "zombie", 0.5, 64.0, 1.8)],
            hit_short(),
        )
        .row(),
        facing(
            "nearest-of-two-reversed",
            vec![mob(25, "zombie", 0.5, 64.0, 1.8), mob(24, "skeleton", 0.5, 64.0, 2.6)],
            hit_short(),
        )
        .row(),
        alone("inside-the-entity", mob(26, "zombie", 0.5, 64.0, 0.5), hit_short()).row(),
        at_zombie("block-in-front", [aims(1).clicks(1).go(), aims(6).down().go()].concat())
            .blocks(vec![put(0, 65, 1, "minecraft:dirt")])
            .row(),
        alone("entity-in-front-of-block", mob(20, "zombie", 0.5, 64.0, 1.8), hit_short())
            .blocks(vec![put(0, 65, 4, "minecraft:stone")])
            .row(),
        alone("out-of-reach", mob(20, "zombie", 0.5, 64.0, 4.0), hit()).row(),
        alone("out-of-reach-then-block", mob(20, "zombie", 0.5, 64.0, 4.0), hit())
            .blocks(vec![put(0, 65, 6, "minecraft:stone")])
            .row(),
        alone("edge-of-reach", mob(20, "zombie", 0.5, 64.0, 3.45), hit()).row(),
        alone("not-pickable-arrow", mob(27, "arrow", 0.5, 65.4, 2.5), hit()).row(),
        alone("fireball-grows-the-aim", mob(28, "fireball", 1.8, 65.1, 2.5), hit()).row(),
        alone("fireball-too-far-aside", mob(28, "fireball", 2.6, 65.1, 2.5), hit()).row(),
        alone("boat-ahead", mob(29, "oak_boat", 0.5, 64.0, 2.5), hit()).x_rot(20.0).row(),
        at_zombie("turned-away", hit()).y_rot(180.0).row(),
        at_zombie("creative-attack", hit()).game_mode("creative").row(),
        at_zombie("adventure-attack", hit()).game_mode("adventure").row(),
        at_zombie("spectator-attack", hit()).game_mode("spectator").row(),
        at_zombie("sword-in-hand", hit()).main_hand("minecraft:iron_sword").row(),
        // The pick item key (the middle button): the aimed entity's or
        // block's item.
    ]
}

/// The pick item key (the middle button): the aimed entity's or block's item.
fn entity_rows_3() -> Vec<Value> {
    let zombie = || mob(20, "zombie", 0.5, 64.0, 2.5);
    let pick = || [aims(1).pick(1).go(), aims(3).go()].concat();
    let pick_short = || [aims(1).pick(1).go(), aims(2).go()].concat();
    let at_zombie = |label: &str, ticks: Vec<Value>| facing(label, vec![zombie()], ticks);
    vec![
        at_zombie("pick-zombie", pick()).row(),
        at_zombie("pick-zombie-twice", [aims(1).pick(2).go(), aims(2).go()].concat()).row(),
        facing("pick-block", vec![], pick()).blocks(vec![put(0, 65, 2, "minecraft:dirt")]).row(),
        at_zombie("pick-block-behind-entity", pick_short()).blocks(vec![put(0, 65, 5, "minecraft:stone")]).row(),
        facing("pick-nothing", vec![], pick()).row(),
        facing("pick-out-of-reach", vec![mob(20, "zombie", 0.5, 64.0, 4.0)], pick_short()).row(),
        at_zombie("pick-after-use-and-attack", [aims(1).clicks(1).use_clicks(1).pick(1).go(), aims(2).go()].concat())
            .row(),
        at_zombie("pick-in-creative", pick_short()).game_mode("creative").row(),
        at_zombie("pick-in-spectator", pick_short()).game_mode("spectator").row(),
    ]
}

/// The use key on a minecart: what `Entity.interact` answers on the client decides whether the
/// click goes on to the hand's item and whether the arm swings.
fn minecart_rows() -> Vec<Value> {
    let use_once = || [aims(1).use_clicks(1).go(), aims(3).go()].concat();
    let cart = |label: &str, kind: &str| facing(label, vec![mob(30, kind, 0.5, 64.0, 2.5)], use_once());
    vec![
        cart("use-minecart", "minecart").x_rot(30.0).row(),
        cart("use-minecart-with-stone", "minecart").main_hand("minecraft:stone").x_rot(30.0).row(),
        cart("use-minecart-both-hands", "minecart")
            .main_hand("minecraft:stone")
            .off_hand("minecraft:cobblestone")
            .x_rot(30.0)
            .row(),
        facing(
            "use-minecart-sneaking",
            vec![mob(30, "minecart", 0.5, 64.0, 2.5)],
            [aims(1).use_clicks(1).held(&["sneak"]).go(), aims(3).held(&["sneak"]).go()].concat(),
        )
        .main_hand("minecraft:stone")
        .x_rot(30.0)
        .row(),
        cart("use-chest-minecart", "chest_minecart").main_hand("minecraft:stone").x_rot(30.0).row(),
        cart("use-hopper-minecart", "hopper_minecart").main_hand("minecraft:stone").x_rot(30.0).row(),
        cart("use-furnace-minecart", "furnace_minecart").main_hand("minecraft:stone").x_rot(30.0).row(),
        cart("use-command-minecart", "command_block_minecart").main_hand("minecraft:stone").x_rot(30.0).row(),
        cart("use-chest-minecart-in-spectator", "chest_minecart")
            .main_hand("minecraft:stone")
            .game_mode("spectator")
            .x_rot(30.0)
            .row(),
    ]
}

pub fn rows() -> Vec<Value> {
    [entity_rows_1(), entity_rows_2(), entity_rows_3(), minecart_rows()].concat()
}
