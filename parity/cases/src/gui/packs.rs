//! The Resource Packs screen over synthetic packs: the two lists, the selection and focus a
//! press leaves, the search and the lists that scroll.

use serde_json::{Value, json};

use super::gui;
use super::setting::{at, click, drag, key, release, wheel_at};

/// A pack the screen lists; `selected` puts it in the right list.
fn pack(id: &str, title: &str, description: &str, selected: bool) -> Value {
    json!({"id": id, "title": title, "description": description, "selected": selected})
}

fn with(mut pack: Value, field: &str, value: Value) -> Value {
    pack[field] = value;
    pack
}

/// The game's own pack as the reference lists it: required, at the bottom, not fixed, named by
/// the game's texts and from the built-in source.
fn vanilla() -> Value {
    let mut vanilla = pack("vanilla", "", "", true);
    vanilla["title_key"] = json!("resourcePack.vanilla.name");
    vanilla["description_key"] = json!("resourcePack.vanilla.description");
    vanilla["builtin"] = json!(true);
    vanilla["required"] = json!(true);
    vanilla["top"] = json!(false);
    vanilla
}

fn few() -> Vec<Value> {
    vec![
        vanilla(),
        pack("file/Alpha", "Alpha", "A pack that fits", false),
        pack(
            "file/Beta",
            "Beta",
            "Another pack, with a description long enough to need a second line and a third one besides",
            false,
        ),
        pack("file/Gamma", "Gamma", "In use", true),
    ]
}

fn three() -> Vec<Value> {
    vec![
        vanilla(),
        pack("file/One", "One", "First in use", true),
        pack("file/Two", "Two", "Second in use", true),
        pack("file/Three", "Three", "Third in use", true),
    ]
}

fn many() -> Vec<Value> {
    let mut packs = vec![vanilla()];
    packs.push(pack(
        "file/Wordy",
        "A pack with a title that is much too long to fit beside its icon",
        "A description that runs on and on, past the end of the second line, so that the text is cut and ends in dots",
        false,
    ));
    for number in 0..9 {
        packs.push(pack(
            &format!("file/Pack{number}"),
            &format!("Pack number {number}"),
            "Some text about the pack",
            number % 3 == 0,
        ));
    }
    packs
}

fn row(label: &str, packs: Vec<Value>, steps: Vec<Value>) -> Value {
    let mut row = at(label).steps(steps).row();
    row["packs"] = json!(packs);
    row
}

fn typed(text: &str) -> Vec<Value> {
    text.chars().map(|ch| json!({"do": "char", "char": ch.to_string()})).collect()
}

pub fn pack_selection_screen() -> Value {
    // At 427 by 240 the lists are 200 wide at -2 and 228; the first row's icon is at (6, 72).
    let available_icon = click(6, 72);
    let available_text = click(60, 72);
    let selected_text = click(290, 72);
    let selected_icon = click(234, 72);
    let mut rows = vec![
        row("initial", few(), vec![]),
        row("select-icon", few(), vec![available_icon.clone()]),
        row("select-icon-then-text", few(), vec![available_icon.clone(), available_text.clone()]),
        row("text-click", few(), vec![available_text.clone()]),
        row("text-click-selected", few(), vec![selected_text.clone()]),
        row("text-then-other-list", few(), vec![available_text, selected_text]),
        row("unselect", few(), vec![selected_icon]),
        row("click-heading", few(), vec![click(100, 59)]),
        row("search", few(), typed("alp")),
        row("search-then-click", few(), [typed("a"), vec![click(60, 72)]].concat()),
        row("search-erased", few(), [typed("al"), vec![key("backspace"), key("backspace")]].concat()),
        {
            let mut hover = row("hover-icon", few(), vec![]);
            hover["mouse"] = json!([10, 76]);
            hover
        },
        {
            let mut hover = row("hover-selected-arrows", few(), vec![]);
            hover["mouse"] = json!([238, 76]);
            hover
        },
        row(
            "incompatible",
            vec![
                vanilla(),
                with(pack("file/Old", "Old", "Made for an older game", false), "compatibility", json!("TOO_OLD")),
                with(pack("file/New", "New", "Made for a newer game", false), "compatibility", json!("TOO_NEW")),
            ],
            vec![],
        ),
        {
            let mut hover = row(
                "incompatible-hover",
                vec![
                    vanilla(),
                    with(pack("file/Old", "Old", "Made for an older game", false), "compatibility", json!("TOO_OLD")),
                ],
                vec![],
            );
            hover["mouse"] = json!([10, 76]);
            hover
        },
    ];
    rows.extend(scrolling_rows());
    gui("pack-selection-screen", "pack_selection_screen", rows)
}

/// The rows with lists that scroll, three packs in use, and the arrows that move them.
fn scrolling_rows() -> Vec<Value> {
    vec![
        row("click-empty-area", few(), vec![click(100, 200)]),
        row("click-empty-area-after-text", few(), vec![click(60, 72), click(300, 200)]),
        {
            let mut hover = row(
                "unknown-and-hover",
                vec![
                    vanilla(),
                    with(
                        pack("file/Faithful", "Faithful32.zip", "The go-to 32x resource pack.", false),
                        "compatibility",
                        json!("UNKNOWN"),
                    ),
                ],
                vec![],
            );
            hover["mouse"] = json!([10, 76]);
            hover
        },
        row("scrolling", many(), vec![]),
        row("three-selected", three(), vec![]),
        {
            let mut hover = row("hover-middle-arrows", three(), vec![]);
            hover["mouse"] = json!([258, 126]);
            hover
        },
        row("move-up", three(), vec![click(258, 108)]),
        row("move-down", three(), vec![click(258, 126)]),
        row("move-up-in-many", many(), vec![click(258, 108)]),
        row("scrolled", many(), vec![wheel_at(-3, 100, 120, false)]),
        row("scroll-bar-drag", many(), vec![click(193, 90), drag(193, 140), release(193, 140)]),
        row("select-in-scrolled", many(), vec![wheel_at(-3, 100, 120, false), click(6, 100)]),
        {
            let mut outside = row("outside-a-world", few(), vec![]);
            outside["in_world"] = json!(false);
            outside
        },
        {
            let mut small = row("small-window", few(), vec![]);
            small["framebuffer"] = json!([640, 360]);
            small
        },
    ]
}
