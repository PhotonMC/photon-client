//! The social interactions screen.

use serde_json::{Value, json};

use super::gui;

const SHIFTS: [u32; 16] = [7, 12, 17, 22, 5, 9, 14, 20, 4, 11, 16, 23, 6, 10, 15, 21];

/// The MD5 digest of `data` (RFC 1321): the offline player ids are derived
/// from it, as `UUID.nameUUIDFromBytes` does.
fn md5(data: &[u8]) -> [u8; 16] {
    let table: Vec<u32> = (0..64).map(|i| ((f64::from(i + 1).sin().abs()) * 4_294_967_296.0) as u32).collect();
    let mut state: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend(((data.len() as u64) * 8).to_le_bytes());
    for block in message.chunks(64) {
        let words: Vec<u32> =
            block.chunks(4).map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]])).collect();
        let [mut a, mut b, mut c, mut d] = state;
        for i in 0..64usize {
            let (mix, index) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let sum = a.wrapping_add(mix).wrapping_add(table[i]).wrapping_add(words[index]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(sum.rotate_left(SHIFTS[(i / 16) * 4 + i % 4]));
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut digest = [0u8; 16];
    for (bytes, word) in digest.chunks_mut(4).zip(state) {
        bytes.copy_from_slice(&word.to_le_bytes());
    }
    digest
}

/// `UUID.nameUUIDFromBytes`: the MD5 of the name as a version 3 id.
fn offline_id(name: &str) -> String {
    let mut digest = md5(name.as_bytes());
    digest[6] = (digest[6] & 0x0f) | 0x30;
    digest[8] = (digest[8] & 0x3f) | 0x80;
    let text = crate::bits::hex(&digest);
    format!("{}-{}-{}-{}-{}", &text[..8], &text[8..12], &text[12..16], &text[16..20], &text[20..])
}

/// One social interactions scenario.
#[derive(Clone)]
struct Social {
    label: &'static str,
    players: Vec<&'static str>,
    gone: Vec<&'static str>,
    hidden: Vec<&'static str>,
    page: &'static str,
    search: &'static str,
    size: (i64, i64),
    mouse: (i64, i64),
    reporting: bool,
    frames: i64,
    steps: Vec<Value>,
}

fn row(label: &'static str) -> Social {
    Social {
        label,
        players: vec!["Alex", "bob", "_Zed", "Carol"],
        gone: vec![],
        hidden: vec![],
        page: "all",
        search: "",
        size: (427, 240),
        mouse: (0, 0),
        reporting: true,
        frames: 2,
        steps: vec![],
    }
}

impl Social {
    fn row(self) -> Value {
        let person = |name: &str| json!({"name": name, "id": offline_id(name)});
        let players: Vec<Value> = ["Steve"].iter().chain(&self.players).map(|name| person(name)).collect();
        json!({
            "label": self.label, "players": players,
            "gone": self.gone.iter().map(|name| person(name)).collect::<Vec<_>>(),
            "hidden": self.hidden.iter().map(|name| offline_id(name)).collect::<Vec<_>>(),
            "frames": self.frames, "local": offline_id("Steve"), "page": self.page, "search": self.search,
            "width": self.size.0, "height": self.size.1, "mouse": [self.mouse.0, self.mouse.1],
            "reporting": self.reporting, "server": "Test Server", "steps": self.steps,
        })
    }
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn scroll(dy: i64) -> Value {
    json!({"do": "scroll", "x": 200, "y": 120, "dy": dy})
}

fn key(code: i64) -> Value {
    json!({"do": "key", "key": code})
}

fn typed(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

fn drag(x: i64, y: i64, dy: i64) -> Value {
    json!({"do": "drag", "x": x, "y": y, "dy": dy})
}

fn release(x: i64, y: i64) -> Value {
    json!({"do": "release", "x": x, "y": y})
}

fn view_rows() -> Vec<Social> {
    let at = |label: &'static str, x: i64, y: i64| Social { mouse: (x, y), ..row(label) };
    vec![
        Social { players: vec![], ..row("alone") },
        row("crowd"),
        Social { size: (320, 240), ..row("small") },
        Social { size: (854, 480), ..row("wide") },
        Social { size: (427, 160), ..row("short") },
        Social { reporting: false, ..row("no-reporting") },
        Social { gone: vec!["Dave", "erin"], ..row("gone") },
        Social { hidden: vec!["Alex"], ..row("hidden-online") },
        Social { gone: vec!["Dave"], hidden: vec!["Dave"], ..row("hidden-gone") },
        Social { page: "hidden", hidden: vec!["Alex", "bob"], ..row("page-hidden") },
        Social { page: "hidden", ..row("page-hidden-empty") },
        Social { page: "blocked", ..row("page-blocked-empty") },
        Social { search: "a", ..row("search-hit") },
        Social { search: "zzz", ..row("search-miss") },
        Social { page: "hidden", hidden: vec!["Alex"], search: "q", ..row("search-miss-hidden") },
        at("hover-hide", 283, 144),
        at("hover-report", 307, 144),
        Social { frames: 1, ..row("first-frame") },
        Social { page: "blocked", frames: 1, ..row("blocked-first-frame") },
        Social { page: "blocked", frames: 2, ..row("page-blocked-second-frame") },
    ]
}

fn input_rows() -> Vec<Social> {
    let with = |label: &'static str, steps: Vec<Value>| Social { steps, ..row(label) };
    let hide = || click(283, 144);
    let bar = || click(333, 100);
    vec![
        with("click-hide", vec![hide()]),
        with("click-hide-show", vec![hide(), hide()]),
        Social { mouse: (283, 144), ..with("click-hide-hover", vec![hide()]) },
        Social { page: "hidden", hidden: vec!["Alex", "bob"], ..with("click-show-page-hidden", vec![click(283, 108)]) },
        with("click-hide-scrolled", vec![scroll(-2), hide()]),
        with("click-local-row", vec![click(283, 108)]),
        Social { gone: vec!["Dave"], ..with("click-gone-row", vec![scroll(-6), click(283, 200)]) },
        Social { hidden: vec!["Alex"], ..with("tab-hidden", vec![click(200, 55)]) },
        with("tab-blocked", vec![click(280, 55)]),
        Social { hidden: vec!["Alex"], page: "hidden", ..with("tab-back-to-all", vec![click(120, 55)]) },
        Social { search: "a", hidden: vec!["Alex"], ..with("tab-keeps-search", vec![click(200, 55)]) },
        with("scroll-down", vec![scroll(-1)]),
        with("scroll-past-bottom", vec![scroll(-20)]),
        with("scroll-past-top", vec![scroll(3)]),
        with("scroll-outside-list", vec![json!({"do": "scroll", "x": 200, "y": 30, "dy": -1})]),
        Social { players: vec![], ..with("scroll-short-list", vec![scroll(-1)]) },
        with("drag-bar", vec![bar(), drag(333, 120, 20)]),
        with("drag-bar-released", vec![bar(), release(333, 100), drag(333, 140, 20)]),
        with("drag-bar-past-end", vec![bar(), drag(333, 200, 5)]),
        with("drag-without-bar", vec![click(200, 100), drag(333, 140, 20)]),
        with("search-unfocus", vec![click(120, 55)]),
        with("search-unfocus-typed", vec![click(120, 55), typed("zz")]),
        with("search-refocus", vec![click(120, 55), click(200, 80)]),
        Social { search: "Carol", ..with("search-click-cursor", vec![click(140, 80)]) },
        with("search-type", vec![typed("a")]),
        Social { search: "al", ..with("search-type-backspace", vec![key(259)]) },
        with("search-type-scrolled", vec![scroll(-3), typed("a")]),
        with("search-type-miss", vec![typed("zz"), key(259), key(259)]),
        with("done", vec![click(200, 223)]),
        with("key-escape", vec![key(256)]),
        with("key-social-in-search", vec![key(80)]),
        with("key-social-unfocused", vec![click(120, 55), key(80)]),
        with("key-social-list-focus", vec![click(283, 108), key(80)]),
        Social { page: "blocked", frames: 1, ..with("hint-first-frame", vec![click(200, 165)]) },
        with("hint-hidden-later", vec![click(200, 165)]),
        with("click-nothing", vec![click(5, 5)]),
    ]
}

/// The player list on its three pages: online players beside hidden and
/// departed ones, the search over them (a hit, a miss), the empty pages,
/// and several screen sizes.
pub fn social_interactions_screen() -> Value {
    let rows = [view_rows(), input_rows()].concat();
    gui("social-interactions-screen", "social_interactions_screen", rows.into_iter().map(Social::row).collect())
}
