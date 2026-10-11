//! Container clicks: the inventory screen's clicks into the reference's own
//! inventory menu and game mode, which predicts each click and sends it; and
//! the hashes of item components. Stacks are named by item and count.
//!
//! A scenario writes its stacks as `"slot item count, ..."` and its clicks as
//! `"slot button type, ..."`.

mod anvil;
mod chest;
mod crafting;
mod enchanted;
mod furnace;
mod hashes;
mod inventory;
mod mounts;
mod stacks;
mod workshops;

use serde_json::{Value, json};

use crate::case::{case, merge};
use crate::components::{
    BINDING_CURSE, CUSTOM_NAME, DAMAGE, EFFICIENCY, ENCHANTMENTS, MAX_DAMAGE, MAX_STACK_SIZE, MENDING, REPAIR_COST,
    SHARPNESS, SMITE, STORED_ENCHANTMENTS, UNBREAKING, VANISHING_CURSE, bundle, enchantments, nbt_text, patch,
    patch_of, stack, vi,
};

const SMELTS: [&str; 4] = ["raw_iron", "beef", "cobblestone", "sand"];

fn st(slot: i64, item: &str, count: i64) -> Value {
    json!({"slot": slot, "item": format!("minecraft:{item}"), "count": count})
}

fn cl(slot: i64, button: i64, kind: &str) -> Value {
    json!({"slot": slot, "button": button, "type": kind})
}

/// Stacks written as `"slot item count, ..."`.
fn stacks(spec: &str) -> Vec<Value> {
    spec.split(',')
        .map(|part| {
            let fields: Vec<&str> = part.split_whitespace().collect();
            let number = |text: &str| text.parse().expect("a number in a stack");
            st(number(fields[0]), fields[1], number(fields[2]))
        })
        .collect()
}

/// The click type a short name stands for.
fn kind(name: &str) -> &'static str {
    match name {
        "P" => "PICKUP",
        "Q" => "QUICK_MOVE",
        "SW" => "SWAP",
        "CL" => "CLONE",
        "TH" => "THROW",
        "QC" => "QUICK_CRAFT",
        "PA" => "PICKUP_ALL",
        "S" => "SCROLL",
        "U" => "UNHOVER",
        other => panic!("no click type {other}"),
    }
}

const QC: &str = "QUICK_CRAFT";

/// Clicks written as `"slot button type, ..."`.
fn taps(spec: &str) -> Vec<Value> {
    spec.split(',')
        .map(|part| {
            let fields: Vec<&str> = part.split_whitespace().collect();
            let number = |text: &str| text.parse().expect("a number in a click");
            cl(number(fields[0]), number(fields[1]), kind(fields[2]))
        })
        .collect()
}

/// `item` x `count` in each slot of `range`.
fn slots(range: std::ops::Range<i64>, item: &str, count: i64) -> Vec<Value> {
    range.map(|slot| st(slot, item, count)).collect()
}

/// The stack with a component patch.
fn with_patch(mut stack: Value, hex_patch: String) -> Value {
    stack["patch"] = json!(hex_patch);
    stack
}

/// A container clicks scenario.
#[derive(Clone)]
pub struct Clicks {
    label: &'static str,
    slots: Vec<Value>,
    clicks: Vec<Value>,
    carried: Option<Value>,
    game_mode: &'static str,
    state_id: i64,
    menu: Option<&'static str>,
    furnace_inputs: bool,
    extra: Option<Value>,
}

fn clicks(label: &'static str, slots: Vec<Value>, list: Vec<Value>) -> Clicks {
    Clicks {
        label,
        slots,
        clicks: list,
        carried: None,
        game_mode: "survival",
        state_id: 0,
        menu: None,
        furnace_inputs: false,
        extra: None,
    }
}

impl Clicks {
    fn carried(self, item: &str, count: i64) -> Self {
        Self { carried: Some(json!({"item": format!("minecraft:{item}"), "count": count})), ..self }
    }

    /// The carried stack with a component patch.
    fn carried_with(mut self, hex_patch: String) -> Self {
        if let Some(carried) = &mut self.carried {
            carried["patch"] = json!(hex_patch);
        }
        self
    }

    fn mode(self, game_mode: &'static str) -> Self {
        Self { game_mode, ..self }
    }

    fn state(self, state_id: i64) -> Self {
        Self { state_id, ..self }
    }

    fn menu(self, menu: &'static str) -> Self {
        Self { menu: Some(menu), ..self }
    }

    fn furnace(self, _inputs: &[&str]) -> Self {
        Self { furnace_inputs: true, ..self }
    }

    fn extra(self, extra: Value) -> Self {
        Self { extra: Some(extra), ..self }
    }

    fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "blocks": [], "chunk_radius": 0, "game_mode": self.game_mode, "selected": 0,
            "state_id": self.state_id, "slots": self.slots, "clicks": self.clicks,
        });
        if let Some(menu) = self.menu {
            row["menu"] = json!(format!("minecraft:{menu}"));
            row["container_id"] = json!(4);
        }
        if let Some(extra) = self.extra {
            merge(&mut row, extra);
        }
        if self.furnace_inputs {
            row["furnace_inputs"] = json!(SMELTS.iter().map(|name| format!("minecraft:{name}")).collect::<Vec<_>>());
        }
        if let Some(carried) = self.carried {
            row["carried"] = carried;
        }
        row
    }
}

fn offer(buy: (&str, i64), sell: (&str, i64), buy_b: Option<(&str, i64)>, more: &[(&str, Value)]) -> Value {
    let mut made = json!({
        "buy": [format!("minecraft:{}", buy.0), buy.1], "sell": [format!("minecraft:{}", sell.0), sell.1],
        "uses": 0, "max_uses": 12, "xp": 2, "special": 0, "demand": 0, "multiplier": 0.05,
    });
    if let Some((item, count)) = buy_b {
        made["buy_b"] = json!([format!("minecraft:{item}"), count]);
    }
    for (key, value) in more {
        made[*key] = value.clone();
    }
    made
}

/// The trades a merchant menu offers.
fn merchant_offers() -> Value {
    json!([
        offer(("emerald", 1), ("bread", 6), None, &[]),
        offer(("wheat", 20), ("emerald", 1), None, &[("max_uses", json!(16)), ("demand", json!(4))]),
        offer(("emerald", 3), ("bookshelf", 1), Some(("paper", 2)), &[("special", json!(-1))]),
        offer(("diamond", 2), ("cake", 1), None, &[("uses", json!(12))]),
    ])
}

/// What a smithing table takes in each slot.
fn smith_sets() -> Value {
    json!({"recipe_sets": {
        "minecraft:smithing_template": ["netherite_upgrade_smithing_template"],
        "minecraft:smithing_base": ["diamond_sword", "diamond_chestplate"],
        "minecraft:smithing_addition": ["netherite_ingot"],
    }})
}

/// A stack of one with an enchantments-like component.
fn enchanted_in(slot: i64, item: &str, levels: &[(i64, i64)], component: i64) -> Value {
    with_patch(st(slot, item, 1), patch(&[(component, enchantments(levels))]))
}

fn enchanted(slot: i64, item: &str, levels: &[(i64, i64)]) -> Value {
    enchanted_in(slot, item, levels, ENCHANTMENTS)
}

/// An enchanted book.
fn book(slot: i64, levels: &[(i64, i64)]) -> Value {
    enchanted_in(slot, "enchanted_book", levels, STORED_ENCHANTMENTS)
}

/// A stack of one that has taken `damage`.
fn worn(slot: i64, item: &str, damage: i64) -> Value {
    with_patch(st(slot, item, 1), patch(&[(DAMAGE, vi(damage))]))
}

/// An iron helmet that cannot be taken off.
fn cursed(slot: i64) -> Value {
    enchanted(slot, "iron_helmet", &[(BINDING_CURSE, 1)])
}

/// A bundle of three kinds of stack.
fn holding_three() -> Value {
    let three = bundle(&[stack("dirt", 5), stack("ender_pearl", 2), stack("stone", 3)]);
    with_patch(st(20, "bundle", 1), three)
}

fn rock() -> String {
    patch(&[(CUSTOM_NAME, nbt_text("Rock"))])
}

fn damaged_stick() -> String {
    patch(&[(MAX_STACK_SIZE, vi(4)), (MAX_DAMAGE, vi(10)), (DAMAGE, vi(3))])
}

fn max_four() -> String {
    patch(&[(MAX_STACK_SIZE, vi(4))])
}

/// Two stacks that carry the work an anvil remembers.
fn prior_work_stacks() -> Vec<Value> {
    let prior_work = patch(&[(REPAIR_COST, vi(31))]);
    vec![with_patch(st(0, "diamond_sword", 1), prior_work.clone()), with_patch(book(1, &[(UNBREAKING, 3)]), prior_work)]
}

/// What a mount's menu is for.
struct Mount {
    entity: &'static str,
    columns: i64,
    baby: bool,
    tame: bool,
}

fn mount(entity: &'static str) -> Mount {
    Mount { entity, columns: 0, baby: false, tame: true }
}

impl Mount {
    fn columns(self, columns: i64) -> Self {
        Self { columns, ..self }
    }

    fn baby(self) -> Self {
        Self { baby: true, ..self }
    }

    fn untamed(self) -> Self {
        Self { tame: false, ..self }
    }
}

/// A mount's menu (`HorseInventoryMenu`, `NautilusInventoryMenu`): the
/// saddle and the body armour take what the entity may wear, by its tame and
/// baby state and the item's allowed entities; the chest is the columns'
/// three rows; shift-click goes by the reference's order.
fn mounted(label: &'static str, mount: Mount, slots: Vec<Value>, list: Vec<Value>) -> Clicks {
    let described = json!({
        "entity": format!("minecraft:{}", mount.entity), "columns": mount.columns, "baby": mount.baby,
        "tame": mount.tame,
    });
    clicks(label, slots, list).menu("mount").extra(json!({ "mount": described }))
}

/// The saddle, the horse armour and the carpet at the first hotbar slots of
/// a mount menu without a chest.
fn gear() -> Vec<Value> {
    stacks("29 saddle 1, 30 iron_horse_armor 1, 31 white_carpet 1")
}

fn wear() -> Vec<Value> {
    taps("29 0 P, 0 0 P, 30 0 P, 1 0 P, 31 0 P, 0 0 P")
}

pub fn cases() -> Vec<Value> {
    let sections = [
        inventory::rows(),
        chest::rows(),
        crafting::rows(),
        furnace::rows(),
        workshops::rows(),
        stacks::rows(),
        enchanted::rows(),
        anvil::rows(),
        mounts::rows(),
    ];
    let rows: Vec<Value> = sections.into_iter().flatten().map(Clicks::row).collect();
    vec![
        case("movement/container-clicks", "movement", "container_clicks", json!({"rows": rows})),
        hashes::component_hashes(),
    ]
}
