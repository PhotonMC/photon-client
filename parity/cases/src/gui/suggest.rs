//! What a command tree suggests for text at a cursor: literals, the values
//! of each argument type, selectors and names, coordinates, and whether the
//! server would be asked.

use serde_json::{Value, json};

use super::gui;
use super::tree::Tree;

fn parameters(id: i64, variant: u8) -> Vec<u8> {
    match id {
        1..=4 => vec![0],
        5 | 6 | 31 => vec![variant],
        43 => 0i32.to_be_bytes().to_vec(),
        44..=48 => [vec![14], b"minecraft:item".to_vec()].concat(),
        _ => vec![],
    }
}

fn suggest_tree() -> Tree {
    let mut tree = Tree::new();
    let mut branches = Vec::new();
    for id in 0..57 {
        let argument = tree.argument("x", (id, parameters(id, 0)), vec![], true);
        branches.push(tree.literal(&format!("t{id}"), vec![argument], false));
    }
    let single = tree.argument("x", (6, parameters(6, 3)), vec![], true);
    branches.push(tree.literal("single", vec![single], false));
    for (name, provider) in
        [("ask", "minecraft:ask_server"), ("sounds", "minecraft:available_sounds"), ("bogus", "minecraft:bogus")]
    {
        let argument = tree.argument("x", (5, parameters(5, 0)), vec![], true);
        tree.set_provider(argument, provider);
        branches.push(tree.literal(name, vec![argument], false));
    }
    let literal = tree.literal("literal", vec![], true);
    let argument = tree.argument("x", (5, parameters(5, 0)), vec![], true);
    tree.set_provider(argument, "minecraft:ask_server");
    branches.push(tree.literal("mixed", vec![literal, argument], false));
    for name in ["alpha", "Alphabet", "beta", "gamma"] {
        branches.push(tree.literal(name, vec![], true));
    }
    tree.set_children(Tree::ROOT, branches);
    tree
}

/// Rows labelled by their position.
#[derive(Default)]
struct Rows(Vec<Value>);

impl Rows {
    fn add(&mut self, text: &str) -> &mut Value {
        let row = json!({
            "label": format!("{}-{text}", self.0.len()), "text": text, "players": ["Alex", "Miner", "miner2"],
            "teams": ["red", "blue"],
        });
        self.0.push(row);
        self.0.last_mut().expect("just pushed")
    }
}

/// The game lists these from its registries; the port asks the server.
const REGISTRY: [i64; 15] = [12, 13, 14, 15, 25, 27, 34, 35, 41, 44, 45, 46, 47, 48, 55];

const SAMPLES: &[(i64, &[&str])] = &[
    (0, &["tr", "F", "x"]),
    (42, &["cre", "ive", "s", "x"]),
    (38, &["f", "e", "ey"]),
    (26, &["=", "+", "<", "x"]),
    (16, &["re", "ed", "dark_", "_blue", "x"]),
    (30, &["side", "sidebar.t", "team.r", "x"]),
    (33, &["r", "b", "x"]),
    (6, &["@", "@a", "@z", "@a[", "@a[t", "Mi", "mi", "x", "@a ", "@p[type=player,"]),
    (7, &["@", "Mi"]),
    (31, &["@", "Mi", "*"]),
    (8, &["1", "1 2", "~", "^", "1 2 3"]),
    (10, &["1", "1 2", "~", "^", "1 2 3"]),
    (9, &["1", "1 2"]),
    (11, &["1", "1 2"]),
    (29, &["1"]),
    (43, &["1", "1d", "x"]),
    (5, &["a"]),
];

fn typed_rows(rows: &mut Rows) {
    for text in ["/", "/t", "/T1", "/a", "/al", "/AL", "/alpha", "/alpha ", "/nope", "/be", "/zzz", ""] {
        rows.add(text);
    }
    for id in 0..57 {
        let row = rows.add(&format!("/t{id} "));
        if REGISTRY.contains(&id) {
            row["registry_backed"] = json!(true);
        }
    }
    for (id, texts) in SAMPLES {
        for text in *texts {
            let row = rows.add(&format!("/t{id} {text}"));
            if text.starts_with("@a[") || text.starts_with("@p[") {
                row["registry_backed"] = json!(true);
            }
        }
    }
    rows.add("/single ");
    rows.add("/single @");
}

/// A player who may not use selectors is offered names only, and an `@` is
/// a name that no player has.
fn selector_rows(rows: &mut Rows) {
    let lists: [(i64, &[&str]); 3] = [
        (6, &["", "@", "@a", "@a[", "@z", "Mi", "mi", "x", "@p[type=player,"]),
        (7, &["", "@", "Mi"]),
        (31, &["", "@", "Mi", "*"]),
    ];
    for (id, texts) in lists {
        for text in texts {
            rows.add(&format!("/t{id} {text}"))["selectors"] = json!(false);
        }
    }
    rows.add("/single ")["selectors"] = json!(false);
    rows.add("/single @")["selectors"] = json!(false);
    for name in ["ask", "sounds", "bogus", "mixed"] {
        let blank = rows.0.len();
        rows.add(&format!("/{name} "));
        rows.add(&format!("/{name} a"));
        if name == "sounds" {
            // A provider the game lists locally (the sounds); the port asks.
            rows.0[blank]["registry_backed"] = json!(true);
            rows.0[blank + 1]["registry_backed"] = json!(true);
        }
    }
}

/// The entity in the crosshair is offered by an entity argument that takes
/// more than players, as its string UUID; not by the players-only ones.
fn aimed_rows(rows: &mut Rows) {
    let aimed = json!(["5e8c9a1b-0000-4000-8000-000000000001"]);
    for text in
        ["/t6 ", "/t6 5", "/t6 5e8c", "/t6 Mi", "/t6 @", "/t6 x", "/single ", "/t7 ", "/t7 5", "/t31 ", "/t31 5"]
    {
        rows.add(text)["selected_entities"] = aimed.clone();
    }
    let row = rows.add("/t6 ");
    row["selected_entities"] = aimed;
    row["selectors"] = json!(false);
    rows.add("/t42 creative ");
    rows.add("/t6 Alex ");
    rows.add("/t42 cr")["cursor"] = json!(5);
    rows.add("/t42 creative")["cursor"] = json!(5);
}

pub fn command_suggest() -> Value {
    let mut rows = Rows::default();
    typed_rows(&mut rows);
    selector_rows(&mut rows);
    aimed_rows(&mut rows);
    let mut case = gui("command-suggest", "command_suggest", rows.0);
    case["tree"] = json!(suggest_tree().payload());
    case
}
