//! The screens of the menus that change their items, over the reference's
//! own menu: labels, textures, sprites, name box and what each shows of its
//! data. Slots hold items, which are not recorded.

use serde_json::{Value, json};

use super::gui;
use crate::bits::{hex, varint};

/// A container screen scenario; `None` fields are left out of the row.
#[derive(Clone)]
pub struct Screen {
    menu: &'static str,
    title_key: &'static str,
    slots: Vec<(i64, &'static str, i64)>,
    data: Option<Value>,
    xp: Option<i64>,
    keys: Vec<&'static str>,
    sync_slot: Option<i64>,
    mouse: (i64, i64),
    size: (i64, i64),
    stonecutter: Vec<&'static str>,
    data_after: Option<Value>,
    tooltips: bool,
    game_mode: &'static str,
    books: Vec<(i64, Vec<&'static str>)>,
    offers: Option<Value>,
    merchant: Option<Value>,
    patches: Vec<(i64, String)>,
    cooldowns: Option<(Vec<(&'static str, i64)>, i64)>,
}

pub fn screen(menu: &'static str, title_key: &'static str) -> Screen {
    Screen {
        menu,
        title_key,
        slots: vec![],
        data: None,
        xp: None,
        keys: vec![],
        sync_slot: None,
        mouse: (0, 0),
        size: (427, 240),
        stonecutter: vec![],
        data_after: None,
        tooltips: true,
        game_mode: "survival",
        books: vec![],
        offers: None,
        merchant: None,
        patches: vec![],
        cooldowns: None,
    }
}

impl Screen {
    fn slot_rows(&self) -> Vec<Value> {
        self.slots
            .iter()
            .map(|(slot, item, count)| {
                let mut row = json!({"slot": slot, "item": format!("minecraft:{item}"), "count": count});
                if let Some((_, pages)) = self.books.iter().find(|(book, _)| book == slot) {
                    row["book_pages"] = json!(pages);
                }
                if let Some((_, patch)) = self.patches.iter().find(|(patched, _)| patched == slot) {
                    row["patch"] = json!(patch);
                }
                row
            })
            .collect()
    }

    pub fn row(&self, label: &str) -> Value {
        let mut row = json!({
            "label": label, "width": self.size.0, "height": self.size.1, "mouse_x": self.mouse.0,
            "mouse_y": self.mouse.1, "menu": format!("minecraft:{}", self.menu), "title_key": self.title_key,
            "container_id": 4, "game_mode": self.game_mode, "selected": 0, "state_id": 0, "blocks": [],
            "chunk_radius": 0, "clicks": [], "keys": self.keys, "slots": self.slot_rows(),
        });
        let optional = [
            ("data", self.data.clone()),
            ("xp_level", self.xp.map(Value::from)),
            ("sync_slot", self.sync_slot.map(Value::from)),
            ("data_after", self.data_after.clone()),
            ("offers", self.offers.clone()),
            ("merchant", self.merchant.clone()),
        ];
        for (key, value) in optional {
            if let Some(value) = value {
                row[key] = value;
            }
        }
        if let Some((cooldowns, ticks)) = &self.cooldowns {
            let list: Vec<Value> =
                cooldowns.iter().map(|(group, duration)| json!({"group": group, "duration": duration})).collect();
            row["cooldowns"] = json!(list);
            row["cooldown_ticks"] = json!(ticks);
        }
        if !self.tooltips {
            row["tooltips"] = json!(false);
        }
        if !self.stonecutter.is_empty() {
            row["stonecutter_inputs"] = json!(self.stonecutter);
            // Its entries show no item the reference can name here.
            row["tooltips"] = json!(false);
        }
        row
    }
}

fn trader(level: i64, xp: i64, progress: bool) -> Value {
    json!({"level": level, "xp": xp, "progress": progress, "restock": true})
}

fn offer(buy: (&str, i64), sell: (&str, i64), (uses, max_uses, special, demand): (i64, i64, i64, i64)) -> Value {
    json!({
        "buy": [format!("minecraft:{}", buy.0), buy.1], "sell": [format!("minecraft:{}", sell.0), sell.1],
        "uses": uses, "max_uses": max_uses, "xp": 2, "special": special, "demand": demand, "multiplier": 0.05,
    })
}

/// The offers of the trading screens: a plain one, one the demand has
/// raised, one that asks for two items, and one that is used up.
fn trades() -> Vec<Value> {
    let mut two = offer(("emerald", 3), ("bookshelf", 1), (0, 12, -1, 0));
    two["buy_b"] = json!(["minecraft:paper", 2]);
    vec![
        offer(("emerald", 1), ("bread", 6), (0, 12, 0, 0)),
        offer(("wheat", 20), ("emerald", 1), (0, 16, 0, 4)),
        two,
        offer(("diamond", 2), ("cake", 1), (12, 12, 0, 0)),
    ]
}

fn small_menus() -> Vec<Value> {
    let anvil = || screen("anvil", "container.repair");
    let grindstone = || screen("grindstone", "container.grindstone_title");
    let stonecutter = || screen("stonecutter", "container.stonecutter");
    let ingot = |count: usize, data: i64| Screen {
        slots: vec![(0, "iron_ingot", 1)],
        stonecutter: vec!["minecraft:iron_ingot"; count],
        data: Some(json!([data])),
        sync_slot: Some(0),
        ..stonecutter()
    };
    let cartography = || screen("cartography_table", "container.cartography_table");
    vec![
        anvil().row("anvil-empty"),
        Screen { slots: vec![(0, "iron_sword", 1)], sync_slot: Some(0), ..anvil() }.row("anvil-item"),
        Screen { slots: vec![(0, "iron_sword", 1)], data: Some(json!([0])), sync_slot: Some(0), ..anvil() }
            .row("anvil-no-result"),
        grindstone().row("grindstone-empty"),
        Screen { slots: vec![(0, "iron_sword", 1)], ..grindstone() }.row("grindstone-error"),
        stonecutter().row("stonecutter-empty"),
        ingot(5, 2).row("stonecutter-entries"),
        Screen { mouse: (198, 56), ..ingot(5, -1) }.row("stonecutter-hover"),
        ingot(14, -1).row("stonecutter-scrollable"),
        screen("loom", "container.loom").row("loom-empty"),
        cartography().row("cartography-empty"),
        Screen { slots: vec![(1, "paper", 1)], sync_slot: Some(1), ..cartography() }.row("cartography-paper"),
        Screen { slots: vec![(1, "glass_pane", 1)], sync_slot: Some(1), ..cartography() }.row("cartography-glass"),
    ]
}

fn brewing_and_beacons() -> Vec<Value> {
    let brewing = |data: Option<Value>| Screen { data, ..screen("brewing_stand", "container.brewing") };
    let beacon = |level: i64| Screen { data: Some(json!([level, 0, 0])), ..screen("beacon", "container.beacon") };
    let after = |pairs: Value| Screen { data_after: Some(pairs), ..beacon(4) };
    let paid = Screen { slots: vec![(0, "emerald", 1)], sync_slot: Some(0), ..after(json!([[1, 1]])) };
    vec![
        brewing(None).row("brewing-empty"),
        brewing(Some(json!([0, 20]))).row("brewing-fuel"),
        brewing(Some(json!([0, 7]))).row("brewing-partial-fuel"),
        brewing(Some(json!([399, 20]))).row("brewing-start"),
        brewing(Some(json!([201, 3]))).row("brewing-half"),
        brewing(Some(json!([1, 0]))).row("brewing-end"),
        Screen {
            slots: vec![(0, "potion", 1), (3, "nether_wart", 2)],
            sync_slot: Some(3),
            ..brewing(Some(json!([120, 11])))
        }
        .row("brewing-items"),
        beacon(0).row("beacon-empty"),
        beacon(1).row("beacon-level-one"),
        beacon(3).row("beacon-level-three"),
        beacon(4).row("beacon-level-four"),
        after(json!([[1, 1]])).row("beacon-primary"),
        after(json!([[1, 11], [2, 10]])).row("beacon-secondary"),
        after(json!([[1, 10], [2, 10]])).row("beacon-upgrade"),
        paid.row("beacon-payment"),
        Screen { mouse: (160, 40), tooltips: false, ..beacon(4) }.row("beacon-hover-power"),
        Screen { mouse: (270, 125), tooltips: false, ..paid }.row("beacon-hover-confirm"),
    ]
}

fn crafters_and_lecterns() -> Vec<Value> {
    let crafter = |data: Value| Screen { data: Some(data), ..screen("crafter_3x3", "container.crafter") };
    let lectern = || screen("lectern", "container.lectern");
    let three = vec!["Hello, lectern.", "A second page", "The third"];
    let book = |pages: Vec<&'static str>| Screen {
        slots: vec![(0, "written_book", 1)],
        sync_slot: Some(0),
        books: vec![(0, pages)],
        ..lectern()
    };
    let long = "The quick brown fox jumps over the lazy dog, again and again, until the \
                page is full of words that wrap at the width of the book's text column \
                and run on and on and on and on and on and on and on and on and on and \
                on and on and on and on and on and on and on and on and on and on and on \
                and on and on and on and on and on and on and on and on and on and on";
    vec![
        crafter(json!([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])).row("crafter-empty"),
        crafter(json!([0, 1, 0, 0, 1, 0, 1, 0, 0, 0])).row("crafter-disabled"),
        crafter(json!([1, 0, 0, 0, 0, 0, 0, 0, 0, 1])).row("crafter-powered"),
        Screen {
            slots: vec![(0, "stone", 3), (1, "stick", 1), (4, "dirt", 2)],
            sync_slot: Some(0),
            ..crafter(json!([0, 1, 0, 0, 1, 0, 0, 0, 0, 0]))
        }
        .row("crafter-items"),
        Screen { mouse: (155, 58), tooltips: false, ..crafter(json!([0, 0, 0, 0, 0, 0, 0, 0, 0, 0])) }
            .row("crafter-hover-slot"),
        Screen { mouse: (155, 58), tooltips: false, ..crafter(json!([1, 0, 0, 0, 0, 0, 0, 0, 0, 0])) }
            .row("crafter-hover-disabled"),
        lectern().row("lectern-empty"),
        book(three.clone()).row("lectern-book"),
        Screen { data_after: Some(json!([[0, 1]])), ..book(three.clone()) }.row("lectern-middle-page"),
        Screen { data_after: Some(json!([[0, 2]])), ..book(three) }.row("lectern-last-page"),
        book(vec![long]).row("lectern-long-page"),
        Screen { mouse: (240, 165), ..book(vec!["One", "Two"]) }.row("lectern-hover-forward"),
        Screen { mouse: (150, 205), ..lectern() }.row("lectern-hover-done"),
        Screen { game_mode: "adventure", ..book(vec!["One", "Two"]) }.row("lectern-adventure"),
    ]
}

fn merchants() -> Vec<Value> {
    let merchant = |offers: Vec<Value>, trader_: Value| Screen {
        offers: Some(json!(offers)),
        keys: vec!["frame"],
        merchant: Some(trader_),
        ..screen("merchant", "entity.minecraft.villager")
    };
    let rotated = [trades()[3..].to_vec(), trades()[..3].to_vec()].concat();
    vec![
        merchant(vec![], trader(1, 0, true)).row("merchant-empty"),
        merchant(trades(), trader(2, 15, true)).row("merchant-offers"),
        merchant(trades().iter().cycle().take(trades().len() * 3).cloned().collect::<Vec<_>>(), trader(3, 100, true))
            .row("merchant-scroll-bar"),
        Screen { slots: vec![(0, "emerald", 1)], ..merchant(trades(), trader(2, 15, true)) }.row("merchant-payment"),
        Screen { slots: vec![(0, "wheat", 30)], ..merchant(trades(), trader(2, 40, true)) }.row("merchant-priced"),
        merchant(rotated, trader(4, 180, true)).row("merchant-out-of-stock"),
        merchant(trades(), trader(5, 400, true)).row("merchant-last-level"),
        merchant(trades(), trader(2, 15, false)).row("merchant-no-bar"),
        Screen { mouse: (160, 70), tooltips: false, ..merchant(trades(), trader(2, 15, true)) }
            .row("merchant-hover-trade"),
    ]
}

fn smithing_and_enchanting() -> Vec<Value> {
    let smithing = || screen("smithing", "container.upgrade");
    let enchant = || screen("enchantment", "container.enchant");
    let rows = |lapis: i64, xp: i64, seed: i64| Screen {
        slots: vec![(0, "diamond_sword", 1), (1, "lapis_lazuli", lapis)],
        xp: Some(xp),
        data: Some(json!([10, 20, 30, seed, -1, -1, -1, -1, -1, -1])),
        ..enchant()
    };
    vec![
        smithing().row("smithing-empty"),
        Screen { slots: vec![(0, "netherite_upgrade_smithing_template", 1)], ..smithing() }.row("smithing-template"),
        Screen { slots: vec![(0, "coast_armor_trim_smithing_template", 1), (1, "iron_chestplate", 1)], ..smithing() }
            .row("smithing-trim"),
        enchant().row("enchanting-empty"),
        rows(3, 30, 12345).row("enchanting-rows"),
        rows(1, 15, 777).row("enchanting-poor"),
        Screen { mouse: (195, 56), ..rows(3, 30, 12345) }.row("enchanting-hover"),
        Screen { size: (320, 240), ..screen("anvil", "container.repair") }.row("narrow"),
        Screen { size: (640, 480), ..screen("grindstone", "container.grindstone_title") }.row("tall"),
    ]
}

fn damaged(value: i64) -> String {
    format!("010003{}", hex(&varint(value)))
}

fn bundle(items: &[(i64, i64)]) -> String {
    let stacks: String =
        items.iter().map(|(item, count)| format!("{}{}0000", hex(&varint(*item)), hex(&varint(*count)))).collect();
    format!("010032{}{stacks}", hex(&varint(items.len() as i64)))
}

/// `UseCooldown.STREAM_CODEC`: the seconds (a float), then the group.
fn use_cooldown(group: &str) -> String {
    format!("01001a3f80000001{}{}", hex(&varint(group.len() as i64)), hex(group.as_bytes()))
}

/// `itemDecorations`' bar: durability, a bundle's fullness and cooldowns.
fn decorations() -> Vec<Value> {
    let grindstone = || Screen { tooltips: false, ..screen("grindstone", "container.grindstone_title") };
    let worn = [(3, 0), (4, 1), (5, 30), (6, 125), (7, 200), (8, 249), (9, 250)];
    let mut slots: Vec<(i64, &str, i64)> = worn.iter().map(|(slot, _)| (*slot, "iron_sword", 1)).collect();
    slots.extend([(10, "diamond_pickaxe", 1), (11, "elytra", 1)]);
    let mut patches: Vec<(i64, String)> = worn.iter().map(|(slot, value)| (*slot, damaged(*value))).collect();
    patches.extend([(10, damaged(1000)), (11, damaged(431))]);
    let bundles = (12..17).map(|slot| (slot, "bundle", 1)).collect();
    let fullness =
        vec![(13, bundle(&[(1, 32)])), (14, bundle(&[(1, 64)])), (15, bundle(&[(1144, 3)])), (16, bundle(&[(1, 63)]))];
    let mut rows = vec![
        Screen { slots, patches, ..grindstone() }.row("decorations-durability"),
        Screen { slots: bundles, patches: fullness, ..grindstone() }.row("decorations-bundle"),
    ];
    for ticks in [0, 5, 7, 19, 20] {
        let cooled = Screen {
            slots: vec![
                (20, "ender_pearl", 16),
                (21, "ender_pearl", 1),
                (22, "chorus_fruit", 3),
                (23, "stick", 1),
                (24, "stick", 1),
                (25, "wind_charge", 2),
            ],
            patches: vec![(23, use_cooldown("minecraft:charge")), (24, use_cooldown("pearls"))],
            cooldowns: Some((
                vec![
                    ("minecraft:ender_pearl", 20),
                    ("minecraft:chorus_fruit", 30),
                    ("minecraft:charge", 13),
                    ("minecraft:pearls", 7),
                    ("minecraft:wind_charge", 3),
                ],
                ticks,
            )),
            ..grindstone()
        };
        rows.push(cooled.row(&format!("decorations-cooldown-{ticks}")));
    }
    rows
}

fn mount(label: &str, entity: &str, (columns, tame, baby): (i64, bool, bool), base: Screen) -> Value {
    let mut row = Screen { menu: "mount", title_key: base.title_key, ..base }.row(label);
    row["mount"] = json!({"entity": format!("minecraft:{entity}"), "columns": columns, "baby": baby, "tame": tame});
    row
}

/// `HorseInventoryScreen`, `NautilusInventoryScreen`: the texture, the
/// chest's slots sprite, the saddle and armour frames where the entity wears
/// them, the empty icons and the title. The mount's model is not recorded.
fn mounts() -> Vec<Value> {
    let titled = |key: &'static str| screen("mount", key);
    let tame = (0, true, false);
    vec![
        mount("mount-horse", "horse", tame, titled("entity.minecraft.horse")),
        mount(
            "mount-horse-saddled",
            "horse",
            tame,
            Screen {
                slots: vec![(0, "saddle", 1), (1, "iron_horse_armor", 1)],
                mouse: (220, 100),
                ..titled("entity.minecraft.horse")
            },
        ),
        mount(
            "mount-donkey-chest",
            "donkey",
            (5, true, false),
            Screen { slots: vec![(2, "stone", 3), (9, "bread", 2)], ..titled("entity.minecraft.donkey") },
        ),
        mount("mount-donkey-untamed", "donkey", (0, false, false), titled("entity.minecraft.donkey")),
        mount("mount-donkey-foal", "donkey", (0, true, true), titled("entity.minecraft.donkey")),
        mount("mount-mule-chest", "mule", (3, true, false), titled("entity.minecraft.mule")),
        mount("mount-llama", "llama", (2, false, false), titled("entity.minecraft.llama")),
        mount("mount-camel", "camel", tame, titled("entity.minecraft.camel")),
        mount("mount-skeleton-horse", "skeleton_horse", tame, titled("entity.minecraft.skeleton_horse")),
        mount("mount-nautilus", "nautilus", tame, titled("entity.minecraft.nautilus")),
        mount("mount-nautilus-untamed", "nautilus", (0, false, false), titled("entity.minecraft.nautilus")),
        mount("mount-zombie-nautilus", "zombie_nautilus", tame, titled("entity.minecraft.zombie_nautilus")),
    ]
}

pub fn container_screen() -> Value {
    let rows = [
        small_menus(),
        brewing_and_beacons(),
        crafters_and_lecterns(),
        merchants(),
        smithing_and_enchanting(),
        decorations(),
        mounts(),
    ]
    .concat();
    gui("container-screen", "container_screen", rows)
}
