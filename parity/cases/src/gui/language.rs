//! The language screen, the language text, and the keys on the settings
//! lists.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::gui;
use super::setting::{at, frame, key, release, times, wheel, wheel_at};

/// Latin letters: the Latin blocks without their signs, as named in
/// Unicode.
const LATIN: &[(u32, u32)] = &[
    (0x41, 0x5a),
    (0x61, 0x7a),
    (0xc0, 0xd6),
    (0xd8, 0xf6),
    (0xf8, 0x2af),
    (0x1d00, 0x1d25),
    (0x1d6b, 0x1d77),
    (0x1d79, 0x1d7f),
    (0x1e00, 0x1eff),
    (0x2c60, 0x2c7f),
    (0xa722, 0xa787),
    (0xa78b, 0xa7ff),
];

fn plain(text: &str) -> bool {
    text.chars().all(|ch| {
        " ()-'.,".contains(ch)
            || ch.is_ascii_digit()
            || LATIN.iter().any(|(low, high)| (*low..=*high).contains(&u32::from(ch)))
    })
}

fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.data/minecraft/assets")
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The languages whose names are all in Latin letters, from the asset
/// pack's own list: the font's other scripts need fallbacks and right to
/// left shaping, which are not ported, so the list shows these.
pub fn latin_language_codes() -> Vec<String> {
    let root = assets();
    let mut indexes: Vec<PathBuf> = fs::read_dir(root.join("indexes"))
        .map(|entries| entries.filter_map(Result::ok).map(|entry| entry.path()).collect())
        .unwrap_or_default();
    indexes.retain(|path| path.extension().is_some_and(|extension| extension == "json"));
    indexes.sort();
    let index = read_json(indexes.first().expect("an asset index under .data/minecraft/assets/indexes"));
    let digest = index["objects"]["pack.mcmeta"]["hash"].as_str().expect("pack.mcmeta in the asset index");
    let pack = read_json(&root.join("objects").join(&digest[..2]).join(digest));
    let mut codes: Vec<(&String, &Value)> = pack["language"].as_object().expect("a language list").iter().collect();
    codes.sort_by_key(|(code, _)| *code);
    codes
        .into_iter()
        .filter(|(_, info)| {
            let name = info["name"].as_str().unwrap_or_default();
            let region = info["region"].as_str().unwrap_or_default();
            plain(&format!("{name} ({region})"))
        })
        .map(|(code, _)| code.clone())
        .collect()
}

/// A language screen row: the settings row, its language and the codes the
/// list shows.
fn lang(setting: super::setting::Setting, language: &str, codes: &[String]) -> Value {
    let mut row = setting.row();
    if let Value::Object(map) = &mut row {
        map.remove("ms");
    }
    row["language"] = json!(language);
    row["codes"] = json!(codes);
    row
}

fn press(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y, "double": false})
}

fn drag(x: i64, y: i64) -> Value {
    json!({"do": "drag", "x": x, "y": y})
}

fn typed(text: &str) -> Vec<Value> {
    text.chars().map(|ch| json!({"do": "char", "char": ch.to_string()})).collect()
}

fn then(first: Vec<Value>, rest: Vec<Value>) -> Vec<Value> {
    [first, rest].concat()
}

fn mouse_rows(codes: &[String]) -> Vec<Value> {
    let en = |setting| lang(setting, "en_us", codes);
    let back = || key("backspace");
    vec![
        en(at("english")),
        lang(at("german"), "de_de", codes),
        lang(at("turkish"), "tr_tr", codes),
        en(at("out-of-world").outside()),
        en(at("small").window(640, 480)),
        en(at("large").window(1280, 720)),
        en(at("tiny").window(640, 480).scale(3)),
        en(at("short").window(854, 300)),
        en(at("auto-large").window(1920, 1080).scale(0)),
        lang(at("first-language"), "af_za", codes),
        lang(at("unknown-language"), "xx_xx", codes),
        en(at("hover-done").mouse(300, 225)),
        en(at("hover-fonts").mouse(130, 225)),
        en(at("hover-entry").mouse(213, 120)),
        en(at("scroll-1").steps(vec![wheel(-1)])),
        en(at("scroll-5").steps(vec![wheel(-5)])),
        en(at("scroll-end").steps(vec![wheel(-500)])),
        en(at("scroll-top").steps(vec![wheel(500)])),
        en(at("scroll-over-header").steps(vec![wheel_at(-3, 200, 10, false)])),
        en(at("bar-drag").steps(vec![press(356, 90), drag(356, 120), release(356, 120)])),
        en(at("bar-drag-end").steps(vec![press(356, 90), drag(356, 400)])),
        en(at("bar-drag-top").steps(vec![press(356, 90), drag(356, 0)])),
        en(at("press-entry").steps(vec![press(213, 90)])),
        en(at("press-entry-then-search").steps(vec![press(213, 90), press(200, 25)])),
        en(at("press-entry-twice").steps(vec![press(213, 90), press(213, 125)])),
        en(at("press-entry-clipped-bottom").steps(vec![press(213, 190)])),
        en(at("press-gap").steps(vec![press(20, 90)])),
        en(at("press-search").steps(vec![press(150, 25)])),
        en(at("type-tr").steps(typed("tr"))),
        lang(at("type-de").steps(typed("de")), "de_de", codes),
        en(at("type-german").steps(typed("German"))),
        en(at("type-capital").steps(typed("TUR"))),
        en(at("type-nothing-matches").steps(typed("zzzz"))),
        en(at("type-region").steps(typed("Deutschland"))),
        en(at("type-then-clear").steps(then(typed("tr"), vec![back(), back()]))),
        en(at("type-one-back").steps(then(typed("abc"), vec![back()]))),
        en(at("type-then-press-entry").steps(then(typed("fran"), vec![press(213, 55)]))),
        en(at("type-then-scroll").steps(then(typed("e"), vec![wheel(-2)]))),
        en(at("type-after-list-focus").steps(then(vec![press(213, 90)], typed("x")))),
        en(at("hint-after-list").steps(vec![press(213, 90)])),
        lang(at("type-selection-lost").steps(typed("e")), "tr_tr", codes),
        en(at("type-long").steps(typed("abcdefghijklmnopqrstuvwxyz"))),
    ]
}

/// Key presses written as `"tab*3 down back"`: `back` is Shift+Tab and
/// `*n` repeats a key.
pub fn keys(spec: &str) -> Vec<Value> {
    let mut steps = Vec::new();
    for token in spec.split_whitespace() {
        let (name, count) =
            token.split_once('*').map_or((token, 1), |(name, count)| (name, count.parse().unwrap_or(1)));
        let name = if name == "back" { "shift+tab" } else { name };
        steps.extend(times(key(name), count));
    }
    steps
}

/// Label, language and keys of the plain key rows.
const KEY_ROWS: &[(&str, &str, &str)] = &[
    ("tab-1", "en_us", "tab"),
    ("tab-2", "en_us", "tab*2"),
    ("tab-3", "en_us", "tab*3"),
    ("tab-4", "en_us", "tab*4"),
    ("tab-5", "en_us", "tab*5"),
    ("tab-9", "en_us", "tab*9"),
    ("back-1", "en_us", "back"),
    ("back-2", "en_us", "back*2"),
    ("back-3", "en_us", "back*3"),
    ("back-4", "en_us", "back*4"),
    ("back-5", "en_us", "back*5"),
    ("tab-then-back", "en_us", "tab tab back"),
    ("tab-1-turkish", "tr_tr", "tab"),
    ("tab-1-german", "de_de", "tab"),
    ("tab-1-first", "af_za", "tab"),
    ("tab-1-unknown", "xx_xx", "tab"),
    ("search-down", "en_us", "down"),
    ("search-up", "en_us", "up"),
    ("search-left", "en_us", "left"),
    ("search-right", "en_us", "right"),
];

/// Label, typed text and the keys after it.
const TYPED_ROWS: &[(&str, &str, &str)] = &[
    ("typed-left", "abc", "left"),
    ("typed-left-right", "abc", "left left right"),
    ("typed-down", "e", "down"),
    ("typed-up", "e", "up"),
    ("typed-tab", "e", "tab"),
    ("typed-tab-tab", "e", "tab tab"),
    ("typed-back", "e", "back"),
    ("typed-back-back", "e", "back*2"),
    ("typed-nothing-tab", "zzzz", "tab"),
    ("typed-nothing-down", "zzzz", "down"),
    ("typed-nothing-back", "zzzz", "back"),
    ("typed-one-match-down", "Deutsch", "down down"),
];

const LIST_ROWS: &[(&str, &str, &str)] = &[
    ("list-down", "en_us", "tab down"),
    ("list-down-3", "en_us", "tab down*3"),
    ("list-up", "en_us", "tab up"),
    ("list-up-3", "en_us", "tab up*3"),
    ("list-down-20", "en_us", "tab down*20"),
    ("list-up-20", "en_us", "tab up*20"),
    ("list-down-end", "en_us", "tab down*200"),
    ("list-up-top", "af_za", "tab up"),
    ("list-up-top-twice", "af_za", "tab up up"),
    ("list-left", "en_us", "tab left"),
    ("list-right", "en_us", "tab right"),
    ("list-left-down", "en_us", "tab left down"),
    ("list-left-tab", "en_us", "tab left tab"),
    ("list-left-back", "en_us", "tab left back"),
    ("list-right-up", "en_us", "tab right up"),
    ("list-down-up", "en_us", "tab down up"),
    ("list-down-tab", "en_us", "tab down tab"),
    ("list-down-back", "en_us", "tab down back"),
    ("list-down-back-back", "en_us", "tab down back back"),
    ("list-end-down-fonts", "en_us", "tab down*400"),
    ("list-end-down-down", "en_us", "tab down*401"),
    ("list-top-up-search", "af_za", "tab up"),
    ("list-top-up-down", "af_za", "tab up down"),
    ("fonts-up", "en_us", "tab tab up"),
    ("fonts-down", "en_us", "tab tab down"),
    ("fonts-left", "en_us", "tab tab left"),
    ("fonts-right", "en_us", "tab tab right"),
    ("done-up", "en_us", "tab*3 up"),
    ("done-down", "en_us", "tab*3 down"),
    ("done-left", "en_us", "tab*3 left"),
    ("done-right", "en_us", "tab*3 right"),
    ("done-left-up", "en_us", "tab*3 left up"),
    ("fonts-up-down", "en_us", "tab tab up down"),
    ("fonts-up-up", "en_us", "tab tab up up"),
    ("search-down-down", "en_us", "down down"),
    ("search-down-up", "en_us", "down up"),
    ("search-down-right", "en_us", "down right"),
    ("search-down-up-up", "en_us", "down up up"),
    ("search-down-up-down", "en_us", "down up down"),
];

fn key_rows(codes: &[String]) -> Vec<Value> {
    let mut rows: Vec<Value> =
        KEY_ROWS.iter().map(|(label, language, spec)| lang(at(label).steps(keys(spec)), language, codes)).collect();
    for (label, text, spec) in TYPED_ROWS {
        rows.push(lang(at(label).steps(then(typed(text), keys(spec))), "en_us", codes));
    }
    rows.extend(LIST_ROWS.iter().map(|(label, language, spec)| lang(at(label).steps(keys(spec)), language, codes)));
    rows
}

fn scroll_rows(codes: &[String]) -> Vec<Value> {
    let en = |setting| lang(setting, "en_us", codes);
    let bar = || vec![press(356, 90), release(356, 90)];
    let clicked = |x: i64, y: i64, spec: &str| then(vec![press(x, y)], keys(spec));
    vec![
        lang(at("scroll-tab").window(854, 300).steps(keys("tab")), "tr_tr", codes),
        en(at("scroll-down-short").window(854, 300).steps(keys("tab down*30"))),
        en(at("scroll-up-short").window(854, 300).steps(keys("tab up*30"))),
        en(at("scroll-tiny-down").window(640, 480).scale(3).steps(keys("tab down*12"))),
        en(at("scroll-wheel-then-tab").steps(vec![wheel(-5), key("tab")])),
        en(at("scroll-wheel-then-down").steps(vec![key("tab"), wheel(-8), key("down")])),
        en(at("scroll-end-then-up").steps(vec![key("tab"), wheel(-500), key("up")])),
        en(at("out-of-world-tab").outside().steps(keys("tab down"))),
        en(at("hover-and-focus").mouse(213, 120).steps(keys("tab down"))),
        en(at("hover-fonts-focus-done").mouse(130, 225).steps(keys("tab*3"))),
        en(at("click-down").steps(clicked(213, 90, "down"))),
        en(at("click-up").steps(clicked(213, 90, "up"))),
        en(at("click-left").steps(clicked(213, 90, "left"))),
        en(at("click-tab").steps(clicked(213, 90, "tab"))),
        en(at("click-back").steps(clicked(213, 90, "back"))),
        en(at("click-back-back").steps(clicked(213, 90, "back back"))),
        en(at("click-search-tab").steps(clicked(150, 25, "tab"))),
        en(at("click-search-back").steps(clicked(150, 25, "back"))),
        en(at("click-bar-tab").steps(then(bar(), keys("tab")))),
        en(at("click-bar-back").steps(then(bar(), keys("back")))),
        en(at("click-bar-down").steps(then(bar(), keys("down")))),
        en(at("click-bar-up").steps(then(bar(), keys("up")))),
        en(at("click-bar-left").steps(then(bar(), keys("left")))),
        en(at("click-bar-then-entry-down").steps(then(bar(), vec![press(213, 90), key("down")]))),
        en(at("click-gap-down").steps(clicked(20, 90, "down"))),
    ]
}

/// The language screen as the reference draws it, in the languages it
/// lists, and the text of some keys in several languages.
pub fn language_cases() -> Vec<Value> {
    let codes = latin_language_codes();
    let rows = [mouse_rows(&codes), key_rows(&codes), scroll_rows(&codes)].concat();
    let keys = [
        "gui.done",
        "options.language.title",
        "options.video",
        "options.language.accuracy",
        "menu.quit",
        "gui.cancel",
        "options.on",
        "options.off",
        "deathScreen.title",
        "options.fov",
        "not.a.key",
        "options.generic_value",
        "chat.type.text",
    ];
    let texts = ["en_us", "tr_tr", "de_de", "fr_fr", "ja_jp", "ar_sa", "ru_ru", "xx_xx"]
        .iter()
        .map(|code| json!({"label": code, "language": code, "keys": keys}))
        .collect();
    vec![gui("language-screen", "language_screen", rows), gui("lang-text", "lang_text", texts)]
}

fn walk_rows() -> Vec<Value> {
    let walks = [
        ("tab-1", "tab"),
        ("tab-2", "tab*2"),
        ("tab-3", "tab*3"),
        ("tab-6", "tab*6"),
        ("tab-12", "tab*12"),
        ("tab-30", "tab*30"),
        ("back-1", "back"),
        ("back-2", "back*2"),
        ("back-3", "back*3"),
        ("back-8", "back*8"),
        ("tab-back", "tab tab back"),
        ("tab-3-back-3", "tab*3 back*3"),
        ("back-tab", "back tab"),
        ("tab-past-end", "tab*40"),
        ("back-past-start", "back*40"),
        ("down-1", "down"),
        ("down-2", "down*2"),
        ("down-4", "down*4"),
        ("down-20", "down*20"),
        ("up-1", "up"),
        ("up-2", "up*2"),
        ("up-5", "up*5"),
        ("left-1", "left"),
        ("right-1", "right"),
        ("tab-right", "tab right"),
        ("tab-left", "tab left"),
        ("tab-right-right", "tab right right"),
        ("tab-right-left", "tab right left"),
        ("tab-down-up", "tab down up"),
        ("tab-up", "tab up"),
        ("tab-down", "tab down"),
        ("down-right-down", "down right down"),
        ("down-right-down-down", "down right down down"),
        ("right-down-up", "right down up"),
        ("down-down-right-up-up", "down down right up up"),
        ("down-to-done", "down*31"),
        ("done-up", "down*40 up"),
        ("done-up-up", "down*40 up up"),
        ("done-left", "down*40 left"),
        ("done-right", "down*40 right"),
        ("done-tab", "back tab"),
        ("done-back", "back back"),
        ("up-up-down", "up up down"),
        ("tab-tab-left-down", "tab tab left down"),
        ("hover-and-focus", "tab tab"),
    ];
    let mut rows: Vec<Value> = walks.iter().map(|(label, spec)| at(label).steps(keys(spec)).row()).collect();
    let frames = [
        ("focus-tooltip", vec![key("tab"), frame(50), frame(700)]),
        ("focus-tooltip-second", vec![key("tab"), key("tab"), frame(50), frame(700)]),
        ("focus-tooltip-moved", vec![key("tab"), frame(50), frame(700), key("tab"), frame(900), frame(1700)]),
        ("focus-tooltip-late", vec![key("tab"), frame(100), frame(2000)]),
    ];
    rows.extend(frames.into_iter().map(|(label, steps)| at(label).steps(steps).row()));
    rows
}

/// Tab, Shift+Tab and the arrows on the settings lists: the walk through
/// rows, the end of the list wrapping round through Done, the list scrolled
/// to a focused entry, and the keyboard focus tooltip.
fn settings_key_rows() -> Vec<Value> {
    let mut rows = walk_rows();
    rows.extend([
        at("hover-and-focus-mouse").mouse(100, 60).steps(keys("tab tab")).row(),
        at("hover-done-focus-list").mouse(213, 223).steps(keys("tab")).row(),
        at("focus-done-hover-other").mouse(100, 60).steps(keys("back")).row(),
    ]);
    // The list scrolls to what it focuses.
    let windows = [
        ("short", (854, 300), 2),
        ("very-short", (640, 400), 4),
        ("tiny", (640, 480), 3),
        ("large", (1280, 720), 2),
        ("auto-large", (1920, 1080), 0),
    ];
    for (name, (width, height), scale) in windows {
        for (label, spec) in [
            ("scroll-tab-6", "tab*6"),
            ("scroll-tab-12", "tab*12"),
            ("scroll-down-8", "down*8"),
            ("scroll-back-3", "back*3"),
            ("scroll-up-3", "down*20 up*3"),
        ] {
            rows.push(at(&format!("{label}-{name}")).window(width, height).scale(scale).steps(keys(spec)).row());
        }
    }
    let clicked = |spec: &str| then(vec![super::setting::click(100, 60), release(100, 60)], keys(spec));
    rows.extend([
        at("out-of-world-tab").outside().steps(keys("tab*3")).row(),
        at("scrolled-then-tab").steps(vec![wheel(-3), key("tab")]).row(),
        at("scrolled-then-down").steps(vec![wheel(-5), key("down")]).row(),
        at("clicked-then-tab").steps(clicked("tab")).row(),
        at("clicked-then-down").steps(clicked("down")).row(),
        at("clicked-then-up").steps(clicked("up")).row(),
    ]);
    rows
}

/// Enter and the arrows on a widget the keys reached: a slider takes Enter
/// to let the arrows move it, then steps by them; a cycle button takes
/// Enter and Space.
fn press_rows(slider_tabs: usize, toggle_tabs: usize) -> Vec<Value> {
    let slider = |label: &str, spec: &str| at(label).steps(then(times(key("tab"), slider_tabs), keys(spec))).row();
    let toggle = |label: &str, spec: &str| at(label).steps(then(times(key("tab"), toggle_tabs), keys(spec))).row();
    vec![
        slider("slider-arrows-before-enter", "right"),
        slider("slider-enter-right", "enter right"),
        slider("slider-enter-right-right", "enter right right"),
        slider("slider-enter-left", "enter left"),
        slider("slider-enter-left-left-left", "enter left left left"),
        slider("slider-enter-twice-right", "enter enter right"),
        slider("slider-space", "space right"),
        slider("slider-enter-right-tab", "enter right tab"),
        slider("slider-enter-down", "enter down"),
        slider("slider-enter-many", "enter right*12"),
        slider("slider-enter-many-left", "enter left*12"),
        toggle("toggle-enter", "enter"),
        toggle("toggle-space", "space"),
        toggle("toggle-enter-twice", "enter enter"),
        toggle("toggle-enter-right", "enter right"),
        toggle("toggle-left", "left"),
    ]
}

/// The key cases of each settings list, with the Tab presses that reach a
/// slider and a cycle button that is safe to press.
pub fn settings_keys() -> Vec<Value> {
    let lists = [
        ("video", "video_settings_screen", 2, 3),
        ("chat-options", "chat_options_screen", 5, 2),
        ("mouse-settings", "mouse_settings_screen", 1, 3),
        ("controls", "controls_screen", 8, 3),
    ];
    lists
        .iter()
        .map(|(name, operation, slider, toggle)| {
            let rows = [settings_key_rows(), press_rows(*slider, *toggle)].concat();
            gui(&format!("{name}-keys"), operation, rows)
        })
        .collect()
}
