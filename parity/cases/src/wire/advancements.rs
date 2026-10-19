//! The advancement packets.

use serde_json::{Value, json};

use super::codec::{string_frame, varint_hex};
use crate::case::case;

/// A text component as a trusted network component: a compound with its text.
fn component(text: &str) -> String {
    format!("0a080004{}{:04x}{}00", "74657874", text.len(), crate::bits::hex(text.as_bytes()))
}

fn float(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

/// An advancement's display on the wire: the title and description, an icon of an item id and count
/// with a component patch, the frame, the flags, the background if the flags say so, and the place.
struct Display {
    frame: i64,
    flags: u32,
    background: Option<&'static str>,
    icon: (i64, i64, &'static str),
    place: (f32, f32),
}

impl Display {
    fn task() -> Self {
        Self { frame: 0, flags: 2, background: None, icon: (1, 1, "0000"), place: (0.0, 0.0) }
    }

    fn hex(&self, title: &str) -> String {
        let (item, count, patch) = self.icon;
        let mut hex = component(title) + &component(&format!("{title} description"));
        hex += &varint_hex(item);
        hex += &varint_hex(count);
        hex += patch;
        hex += &varint_hex(self.frame);
        hex += &format!("{:08x}", self.flags);
        hex += &self.background.map(string_frame).unwrap_or_default();
        hex += &float(self.place.0);
        hex += &float(self.place.1);
        hex
    }
}

/// One advancement: the id, the parent, the display (with its title), the requirements and the
/// telemetry flag.
fn advancement(
    id: &str,
    parent: Option<&str>,
    display: Option<(&Display, &str)>,
    requirements: &[&[&str]],
    telemetry: bool,
) -> String {
    let mut hex = string_frame(id);
    hex += &parent.map_or("00".to_owned(), |parent| format!("01{}", string_frame(parent)));
    hex += &display.map_or("00".to_owned(), |(display, title)| format!("01{}", display.hex(title)));
    hex += &varint_hex(requirements.len() as i64);
    for group in requirements {
        hex += &varint_hex(group.len() as i64);
        for criterion in *group {
            hex += &string_frame(criterion);
        }
    }
    hex += if telemetry { "01" } else { "00" };
    hex
}

/// The packet: the reset flag, the advancements, the removed ids, the progress, the show flag.
/// The criteria of an advancement, each with when it was obtained.
type Criteria<'a> = &'a [(&'a str, Option<i64>)];

fn update(reset: bool, added: &[String], removed: &[&str], progress: &[(&str, Criteria<'_>)], show: bool) -> String {
    let mut hex = String::from(if reset { "01" } else { "00" });
    hex += &varint_hex(added.len() as i64);
    hex += &added.concat();
    hex += &varint_hex(removed.len() as i64);
    for id in removed {
        hex += &string_frame(id);
    }
    hex += &varint_hex(progress.len() as i64);
    for (id, criteria) in progress {
        hex += &string_frame(id);
        hex += &varint_hex(criteria.len() as i64);
        for (name, obtained) in *criteria {
            hex += &string_frame(name);
            hex += &obtained.map_or("00".to_owned(), |time| format!("01{:016x}", time as u64));
        }
    }
    hex += if show { "01" } else { "00" };
    hex
}

pub fn cases() -> Vec<Value> {
    let mut cases: Vec<Value> = update_reads()
        .into_iter()
        .map(|(label, hex)| {
            case(format!("game/update-advancements/{label}"), "wire", "update_advancements", json!({"hex": hex}))
        })
        .collect();
    for (label, hex) in [
        ("none", "00".to_owned()),
        ("tab", format!("01{}", string_frame("minecraft:story/root"))),
        ("tab-bad", format!("01{}", string_frame("Bad:Tab"))),
        ("tab-truncated", "01".to_owned()),
        ("empty-bytes", String::new()),
    ] {
        cases.push(case(
            format!("game/select-advancements-tab/{label}"),
            "wire",
            "select_advancements_tab",
            json!({"hex": hex}),
        ));
    }
    cases
}

/// The update packets: the flags, a tree, an advancement with no display, odd floats, removals,
/// progress, and the ways bytes can be wrong.
fn update_reads() -> Vec<(&'static str, String)> {
    let mut reads = vec![
        ("empty", update(false, &[], &[], &[], true)),
        ("reset", update(true, &[], &[], &[], false)),
        ("removed", update(false, &[], &["minecraft:story/root", "test:other", "minecraft:story/root"], &[], true)),
        ("truncated-count", format!("00{}", varint_hex(2))),
        ("empty-bytes", String::new()),
    ];
    reads.extend(display_reads());
    reads.extend(progress_reads());
    reads
}

fn display_reads() -> Vec<(&'static str, String)> {
    let root =
        Display { background: Some("minecraft:gui/advancements/backgrounds/stone"), flags: 3, ..Display::task() };
    let challenge = Display {
        frame: 1,
        flags: 6,
        icon: (300, 3, "0001" /* a removed component, id 0 */),
        place: (-1.5, 3.25),
        ..Display::task()
    };
    let goal = Display { frame: 2, flags: 0, place: (1.0e10, f32::MIN_POSITIVE), ..Display::task() };
    let nan = Display { place: (f32::NAN, -0.0), ..Display::task() };
    let story = |id: &str, parent: Option<&str>, display: Option<(&Display, &str)>, requirements: &[&[&str]]| {
        advancement(id, parent, display, requirements, false)
    };
    let tree = vec![
        advancement("minecraft:story/root", None, Some((&root, "Minecraft")), &[&["crafting_table"]], true),
        story(
            "minecraft:story/mine_stone",
            Some("minecraft:story/root"),
            Some((&Display::task(), "Stone Age")),
            &[&["get_stone"]],
        ),
        story(
            "minecraft:story/upgrade_tools",
            Some("minecraft:story/mine_stone"),
            Some((&challenge, "Getting an Upgrade")),
            &[&["a", "b"], &["c"]],
        ),
        story("minecraft:story/goal", Some("minecraft:story/mine_stone"), Some((&goal, "A Goal")), &[]),
    ];
    let one = |advancement: String| update(false, &[advancement], &[], &[], true);
    vec![
        ("root", one(story("minecraft:story/root", None, Some((&root, "Minecraft")), &[&["crafting_table"]]))),
        ("tree", update(false, &tree, &[], &[], true)),
        ("no-display", one(story("test:hidden", Some("test:root"), None, &[&["x"]]))),
        ("nan-place", one(story("test:odd", None, Some((&nan, "Odd")), &[&["x"]]))),
        ("unicode-title", one(story("test:unicode", None, Some((&Display::task(), "\u{e9}\u{4e2d}")), &[&["x"]]))),
        ("unknown-frame", one(story("test:frame", None, Some((&Display { frame: 3, ..Display::task() }, "F")), &[]))),
        ("bad-identifier", one(story("Test:Upper", None, None, &[]))),
        ("truncated", one(story("test:a", None, None, &[&["x"]]))[..30].to_owned()),
        (
            "everything",
            update(
                true,
                &[advancement("a:b", None, Some((&root, "T")), &[&["c"]], true)],
                &["x:y"],
                &[("a:b", &[("c", Some(7))])],
                true,
            ),
        ),
    ]
}

fn progress_reads() -> Vec<(&'static str, String)> {
    let progress: &[(&str, Criteria<'_>)] = &[
        ("minecraft:story/root", &[("crafting_table", Some(1_700_000_000_123))]),
        ("minecraft:adventure/kill", &[("zombie", None), ("skeleton", Some(-5)), ("creeper", Some(0))]),
        ("test:none", &[]),
    ];
    vec![
        ("progress", update(false, &[], &[], progress, true)),
        ("progress-twice", update(false, &[], &[], &[("a:b", &[("x", None)]), ("a:b", &[("y", Some(1))])], true)),
    ]
}
