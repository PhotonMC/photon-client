//! Item tooltip lines (`ItemStack.getTooltipLines`) and a bundle's tooltip.

use serde_json::{Value, json};

use super::gui;
use crate::components::{
    BANNER_PATTERNS, CONTAINER, CUSTOM_NAME, DAMAGE, INSTRUMENT, LORE, MAP_ID, MAP_POST_PROCESSING,
    OMINOUS_BOTTLE_AMPLIFIER, POTION_CONTENTS, Stack, WRITTEN_BOOK_CONTENT, bundle, item_id, nbt_text as nbt, patch,
    stack, text, vi,
};

/// `ItemContainerContents`: an optional stack per slot.
fn container(items: &[Option<Stack>]) -> String {
    let slots: String = items
        .iter()
        .map(|slot| match slot {
            None => "00".to_owned(),
            Some((item, count, inner)) => format!("01{}{}{inner}", vi(item_id(item)), vi(*count)),
        })
        .collect();
    format!("{}{slots}", vi(items.len() as i64))
}

/// `WrittenBookContent`: a title with no filtered form, no pages, resolved.
fn book(author: &str, generation: i64) -> String {
    format!("{}00{}{}{}01", text("Title"), text(author), vi(generation), vi(0))
}

/// `Instrument.DIRECT_STREAM_CODEC` after the holder's 0: a written-out sound with no fixed
/// range, the use duration and range, and the description.
fn written_horn() -> String {
    let float = |value: f32| value.to_be_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    format!(
        "{}{}{}00{}{}{}",
        vi(0),
        vi(0),
        text("minecraft:item.goat_horn.sound.2"),
        float(3.5),
        float(64.0),
        nbt("Old Horn")
    )
}

const LOCK: i64 = 0;
const SCALE: i64 = 1;

/// A tooltip scenario: `None` fields are left out of the row.
struct Tooltip {
    label: &'static str,
    item: &'static str,
    added: Vec<(i64, String)>,
    advanced: bool,
    map: Option<(i64, bool)>,
    /// The level's tick rate the effect times are read at, when not twenty.
    tick_rate: Option<f64>,
}

fn row(label: &'static str, item: &'static str) -> Tooltip {
    Tooltip { label, item, added: vec![], advanced: false, map: None, tick_rate: None }
}

impl Tooltip {
    fn row(self) -> Value {
        let mut made = json!({
            "label": self.label, "item": format!("minecraft:{}", self.item), "count": 1, "creative": false,
            "advanced": self.advanced,
        });
        if !self.added.is_empty() {
            made["patch"] = json!(patch(&self.added));
        }
        if let Some((scale, locked)) = self.map {
            made["map"] = json!({"scale": scale, "locked": locked});
        }
        if let Some(rate) = self.tick_rate {
            made["tick_rate"] = json!(rate);
        }
        made
    }
}

/// A potion's and an ominous bottle's effect times at the level's tick rate: twenty, slower and a
/// fraction.
fn effect_time_rows() -> Vec<Tooltip> {
    // `PotionContents`: the potion by registry id, no colour, no custom effects, no name.
    let potion = |id: i64| vec![(POTION_CONTENTS, format!("01{}000000", vi(id)))];
    let (swiftness, long_swiftness) = (13, 14);
    let at = |label, item, added, rate| Tooltip { added, tick_rate: Some(rate), ..row(label, item) };
    vec![
        at("swiftness", "potion", potion(swiftness), 20.0),
        at("swiftness tick rate 10", "potion", potion(swiftness), 10.0),
        at("swiftness tick rate 2.5", "potion", potion(swiftness), 2.5),
        at("long swiftness tick rate 10", "potion", potion(long_swiftness), 10.0),
        at("ominous bottle tick rate 10", "ominous_bottle", vec![(OMINOUS_BOTTLE_AMPLIFIER, vi(2))], 10.0),
    ]
}

/// Tooltip lines of stacks whose items or components add lines: maps with
/// and without the client's map data and post-processing, container
/// contents up to the cut-off, written books, smithing templates, the
/// advanced (F3+H) lines, and the order of patterns against lore.
pub fn tooltip_lines() -> Value {
    let named = || patch(&[(CUSTOM_NAME, nbt("Rock"))]);
    let map_id = || (MAP_ID, vi(3));
    let with = |label, item, added: Vec<(i64, String)>| Tooltip { added, ..row(label, item) };
    let advanced = |label, item, added: Vec<(i64, String)>| Tooltip { advanced: true, ..with(label, item, added) };
    let contents = |items: &[Option<Stack>]| vec![(CONTAINER, container(items))];
    let many: Vec<Option<Stack>> = (1..8).map(|n| Some(stack("stone", n))).collect();
    let mut rows = vec![
        row("plain", "stone"),
        advanced("plain advanced", "stone", vec![]),
        advanced("named advanced", "stone", vec![(CUSTOM_NAME, nbt("Rock"))]),
        with("damaged", "iron_sword", vec![(DAMAGE, vi(10))]),
        advanced("damaged advanced", "iron_sword", vec![(DAMAGE, vi(10))]),
        with("map unknown", "filled_map", vec![map_id()]),
        Tooltip { map: Some((2, false)), ..with("map", "filled_map", vec![map_id()]) },
        Tooltip { map: Some((4, true)), ..advanced("map locked advanced", "filled_map", vec![map_id()]) },
        Tooltip {
            map: Some((1, false)),
            ..advanced("map scaling advanced", "filled_map", vec![map_id(), (MAP_POST_PROCESSING, vi(SCALE))])
        },
        Tooltip {
            map: Some((0, false)),
            ..with(
                "map locking named",
                "filled_map",
                vec![map_id(), (MAP_POST_PROCESSING, vi(LOCK)), (CUSTOM_NAME, nbt("Atlas"))],
            )
        },
        with(
            "shulker contents",
            "shulker_box",
            contents(&[Some(stack("stone", 10)), None, Some(stack("dirt", 3)), Some(("stone", 1, named()))]),
        ),
        with("shulker many", "shulker_box", contents(&many)),
        advanced("shulker advanced", "shulker_box", contents(&[Some(stack("ender_pearl", 16))])),
        with("written book", "written_book", vec![(WRITTEN_BOOK_CONTENT, book("Alex", 1))]),
        // `InstrumentComponent`: a registry call's description, and one written out in full.
        with("horn sing", "goat_horn", vec![(INSTRUMENT, vi(7))]),
        with("horn written out", "goat_horn", vec![(INSTRUMENT, written_horn())]),
        with("written book no author", "written_book", vec![(WRITTEN_BOOK_CONTENT, book(" ", 3))]),
        row("netherite template", "netherite_upgrade_smithing_template"),
        advanced("trim template advanced", "sentry_armor_trim_smithing_template", vec![]),
        with(
            "banner and lore",
            "white_banner",
            vec![
                (BANNER_PATTERNS, format!("{}{}{}", vi(1), vi(1), vi(14))),
                (LORE, format!("{}{}", vi(1), nbt("Note"))),
            ],
        ),
    ];
    rows.extend(effect_time_rows());
    gui("tooltip-lines", "tooltip_lines", rows.into_iter().map(Tooltip::row).collect())
}

/// A bundle's tooltip in a container screen: empty, some stacks with and
/// without a selected one (a named one, a damaged one), full, more than
/// twelve stacks, and placed by the screen's edges.
pub fn bundle_tooltip() -> Value {
    struct Row {
        label: &'static str,
        contents: Option<Vec<Stack>>,
        selected: Option<i64>,
        mouse: (i64, i64),
        size: (i64, i64),
    }
    let row = |label| Row { label, contents: None, selected: None, mouse: (120, 100), size: (427, 240) };
    let three = || vec![stack("dirt", 5), stack("ender_pearl", 2), stack("stone", 3)];
    let damaged = patch(&[(DAMAGE, vi(100))]);
    let rows = [
        row("empty"),
        Row { contents: Some(three()), ..row("three") },
        Row { contents: Some(three()), selected: Some(1), ..row("three, second selected") },
        Row {
            contents: Some([vec![("stone", 2, patch(&[(CUSTOM_NAME, nbt("Rock"))]))], three()].concat()),
            selected: Some(0),
            ..row("named selected")
        },
        Row {
            contents: Some(vec![("iron_sword", 1, damaged), stack("stone", 9)]),
            selected: Some(1),
            ..row("damaged sword")
        },
        Row { contents: Some(vec![stack("stone", 64)]), ..row("full") },
        Row { contents: Some(vec![stack("stone", 1); 9]), selected: Some(8), ..row("nine") },
        Row { contents: Some((1..14).map(|n| stack("stone", n)).collect()), selected: Some(2), ..row("thirteen") },
        Row { contents: Some(three()), selected: Some(2), mouse: (400, 60), ..row("by the right edge") },
        Row { contents: Some(vec![stack("stone", 1); 12]), mouse: (50, 230), size: (320, 240), ..row("by the bottom") },
    ];
    let rows = rows
        .into_iter()
        .map(|row| {
            let mut made = json!({
                "label": row.label, "item": "minecraft:bundle", "count": 1, "width": row.size.0,
                "height": row.size.1, "mouse_x": row.mouse.0, "mouse_y": row.mouse.1,
            });
            if let Some(contents) = &row.contents {
                made["patch"] = json!(bundle(contents));
            }
            if let Some(selected) = row.selected {
                made["selected"] = json!(selected);
            }
            made
        })
        .collect();
    gui("bundle-tooltip", "bundle_tooltip", rows)
}
