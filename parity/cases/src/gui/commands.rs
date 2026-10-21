//! Commands typed against a tree like a server's, and what a tree suggests.

use serde_json::{Value, json};

use super::gui;
use super::tree::Tree;

type Parser = (i64, Vec<u8>);

fn bounds<const N: usize>(id: i64, low: Option<[u8; N]>, high: Option<[u8; N]>) -> Parser {
    let flags = u8::from(low.is_some()) | (u8::from(high.is_some()) << 1);
    let mut data = vec![flags];
    data.extend(low.into_iter().flatten());
    data.extend(high.into_iter().flatten());
    (id, data)
}

fn integer(low: Option<i32>, high: Option<i32>) -> Parser {
    bounds(3, low.map(i32::to_be_bytes), high.map(i32::to_be_bytes))
}

fn plain(id: i64) -> Parser {
    (id, vec![])
}

/// The tree of the parse cases: gamemode, give, tp, say, time, execute
/// (with redirects), scoreboard, setblock, weather and some argument types.
fn parse_tree() -> Tree {
    let mut tree = Tree::new();
    let entity = || (6, vec![0]);
    let gamemode = tree.argument("gamemode", plain(42), vec![], true);
    let gamemode = tree.literal("gamemode", vec![gamemode], false);
    let count = tree.argument("count", integer(Some(1), None), vec![], true);
    let item = tree.argument("item", plain(14), vec![count], true);
    let targets = tree.argument("targets", entity(), vec![item], false);
    let give = tree.literal("give", vec![targets], false);
    let destination = tree.argument("destination", entity(), vec![], true);
    let location = tree.argument("location", plain(10), vec![], true);
    let then = tree.argument("location", plain(10), vec![], true);
    let targets = tree.argument("targets", entity(), vec![then], false);
    let tp = tree.literal("tp", vec![destination, location, targets], false);
    let message = tree.argument("message", plain(20), vec![], true);
    let say = tree.literal("say", vec![message], false);
    let time = time_branch(&mut tree);
    let execute = execute_branch(&mut tree, entity());
    let mut roots = vec![gamemode, give, tp, say, time, execute];
    roots.extend(more_branches(&mut tree));
    tree.set_children(Tree::ROOT, roots);
    tree
}

fn time_branch(tree: &mut Tree) -> usize {
    let time = || (43, 0i32.to_be_bytes().to_vec());
    let day = tree.literal("day", vec![], true);
    let night = tree.literal("night", vec![], true);
    let value = tree.argument("time", time(), vec![], true);
    let set = tree.literal("set", vec![day, night, value], false);
    let value = tree.argument("time", time(), vec![], true);
    let add = tree.literal("add", vec![value], false);
    tree.literal("time", vec![set, add], false)
}

fn execute_branch(tree: &mut Tree, entity: Parser) -> usize {
    let run = tree.literal("run", vec![], false);
    tree.set_redirect(run, Tree::ROOT);
    let targets = tree.argument("targets", entity, vec![], false);
    let as_ = tree.literal("as", vec![targets], false);
    let execute = tree.literal("execute", vec![run, as_], false);
    tree.set_redirect(targets, execute);
    execute
}

fn more_branches(tree: &mut Tree) -> Vec<usize> {
    let score = tree.argument("score", integer(None, None), vec![], true);
    let objective = tree.argument("objective", plain(24), vec![score], false);
    let targets = tree.argument("targets", (31, vec![1]), vec![objective], false);
    let set = tree.literal("set", vec![targets], false);
    let players = tree.literal("players", vec![set], false);
    let scoreboard = tree.literal("scoreboard", vec![players], false);
    let block = tree.argument("block", plain(12), vec![], true);
    let pos = tree.argument("pos", plain(8), vec![block], false);
    let setblock = tree.literal("setblock", vec![pos], false);
    let duration = tree.argument("duration", integer(Some(1), Some(1_000_000)), vec![], true);
    let clear = tree.literal("clear", vec![duration], true);
    let rain = tree.literal("rain", vec![], true);
    let thunder = tree.literal("thunder", vec![], true);
    let weather = tree.literal("weather", vec![clear, rain, thunder], false);
    let num = number_branch(tree);
    let string = |id: u8| (5, vec![id]);
    let rest = tree.argument("rest", string(2), vec![], true);
    let phrase = tree.argument("phrase", string(1), vec![rest], false);
    let word = tree.argument("word", string(0), vec![phrase], false);
    let str_ = tree.literal("str", vec![word], false);
    let misc = misc_branch(tree);
    let alias = tree.literal("alias", vec![], true);
    let alias2 = tree.literal("alias2", vec![], true);
    vec![scoreboard, setblock, weather, num, str_, misc, alias, alias2]
}

fn number_branch(tree: &mut Tree) -> usize {
    let boolean = tree.argument("b", plain(0), vec![], true);
    let long = bounds(4, Some(0i64.to_be_bytes()), Some(100i64.to_be_bytes()));
    let long = tree.argument("l", long, vec![boolean], true);
    let double = bounds(2, Some((-5.0f64).to_be_bytes()), None);
    let double = tree.argument("d", double, vec![long], false);
    let float = bounds(1, Some(0.0f32.to_be_bytes()), Some(1.0f32.to_be_bytes()));
    let float = tree.argument("f", float, vec![double], false);
    tree.literal("num", vec![float], false)
}

fn misc_branch(tree: &mut Tree) -> usize {
    let kinds = [
        ("a", "angle", 28),
        ("c", "col", 9),
        ("v", "v2", 11),
        ("i", "id", 36),
        ("s", "slot", 30),
        ("w", "swizzle", 32),
        ("o", "op", 26),
        ("e", "anchor", 38),
        ("u", "uuid", 56),
        ("t", "color", 16),
        ("h", "hex", 17),
    ];
    let branches = kinds
        .iter()
        .map(|(literal, name, id)| {
            let argument = tree.argument(name, plain(*id), vec![], true);
            tree.literal(literal, vec![argument], false)
        })
        .collect();
    tree.literal("misc", branches, false)
}

const TEXTS: &[&str] = &[
    "/",
    "/g",
    "/gamemode",
    "/gamemode ",
    "/gamemode creative",
    "/gamemode creative ",
    "/gamemode nope",
    "/gamemode creative extra",
    "/unknown",
    "/unknown arg",
    "/give @p",
    "/give @p ",
    "/give @p minecraft:stone",
    "/give @p minecraft:stone 64",
    "/give @p minecraft:stone 0",
    "/give @e[type=pig,distance=..5] minecraft:stone 3 junk",
    "/tp @p",
    "/tp @p @a",
    "/tp 1 2 3",
    "/tp 1 2",
    "/tp ~ ~1 ~-2",
    "/tp @p 1 2 3",
    "/tp ^ ^ ^1",
    "/tp ^ 1 ^",
    "/say hello world",
    "/say",
    "/say ",
    "/time set day",
    "/time set 10s",
    "/time set 5x",
    "/time set night extra",
    "/time add 1d",
    "/time ",
    "/execute run gamemode survival",
    "/execute as @a run say hi",
    "/execute as @a ",
    "/execute run ",
    "/scoreboard players set @a obj 5",
    "/scoreboard players set @a obj x",
    "/scoreboard players ",
    "/setblock 1 2 3",
    "/setblock 1 2 3 minecraft:oak_log[axis=x]",
    "/setblock 1 2",
    "/setblock ~ ~ ~ minecraft:chest{Items:[{id:\"x\"}]} tail",
    "/weather clear",
    "/weather clear 100",
    "/weather clear 0",
    "/weather clear 2000000",
    "/weather snow",
    "/num 0.5 1.5 10 true",
    "/num 2 1 1 1",
    "/num 0.5 -9 1 1",
    "/num 0.5 1 101 true",
    "/num 0.5 1 1 maybe",
    "/num x",
    "/num 0.5 1e3",
    "/str word \"a phrase\" the rest",
    "/str \"unclosed",
    "/misc a ~5",
    "/misc a nan",
    "/misc c 1 2",
    "/misc c ~ ~1",
    "/misc v 1.5 2",
    "/misc i minecraft:stone",
    "/misc i Bad:Id",
    "/misc s weapon.mainhand",
    "/misc w xz",
    "/misc w xx",
    "/misc w q",
    "/misc o +=",
    "/misc o ><",
    "/misc o ?",
    "/misc e feet",
    "/misc e head",
    "/misc u 123e4567-e89b-12d3-a456-426614174000",
    "/misc u nope",
    "/misc t red",
    "/misc t pink",
    "/misc h 1a2",
    "/misc h abcd",
    "/alias",
    "/alias x",
    "/alias2 ",
    "no slash",
    "/gamemode  creative",
    "/give @p minecraft:stone 64 ",
    "/tp @p 1 2 3 4",
];

/// A player who may not use selectors: the `@` is refused where it stands,
/// and names and coordinates read as before.
const NO_SELECTORS: &[&str] = &[
    "/give @p",
    "/give @p ",
    "/give @a[type=player] minecraft:stone",
    "/tp @p @a",
    "/tp @s 1 2 3",
    "/tp Alex 1 2 3",
    "/tp Alex",
    "/give Alex minecraft:stone 3",
    "/execute as @a run say hi",
    "/execute as Alex run say hi",
    "/scoreboard players set @a obj 5",
    "/scoreboard players set Alex obj 5",
    "/scoreboard players set * obj 5",
    "/give @",
    "/give @x",
    "/give @e[",
    "/say @a hello",
    "/give @p minecraft:stone 64 ",
    "/tp @p 1 2 3 4",
];

/// Commands typed against a tree like a server's: which node read which
/// part, what could not be read and why, the text's colours, and the usage
/// and error lines under it.
pub fn command_parse() -> Value {
    let mut rows = Vec::new();
    for (number, text) in TEXTS.iter().enumerate() {
        for empty in [true, false] {
            rows.push(
                json!({"label": format!("{number}-{text}-{}", title(empty)), "text": text, "suggestions_empty": empty}),
            );
        }
    }
    for (number, text) in NO_SELECTORS.iter().enumerate() {
        for empty in [true, false] {
            let label = format!("no-selectors-{number}-{text}-{}", title(empty));
            rows.push(json!({"label": label, "text": text, "suggestions_empty": empty, "selectors": false}));
        }
    }
    let mut case = gui("command-parse", "command_parse", rows);
    case["tree"] = json!(parse_tree().payload());
    case
}

/// `True` / `False`, as the labels spell them.
pub fn title(flag: bool) -> &'static str {
    if flag { "True" } else { "False" }
}

/// The tree of the verify cases: commands anyone may run, one that takes a message, one that
/// only a source with the restricted permission may use, one with a restricted argument, some
/// that take a selector, and `execute` with its redirects.
fn verify_tree() -> Tree {
    let mut tree = Tree::new();
    let entity = || (6, vec![0]);
    let help = tree.literal("help", vec![], true);
    let message = tree.argument("message", plain(20), vec![], true);
    let say = tree.literal("say", vec![message], false);
    let target = tree.argument("target", plain(24), vec![], true);
    let kick = tree.literal("kick", vec![target], false);
    tree.restrict(kick);
    let reason = tree.argument("reason", plain(24), vec![], true);
    tree.restrict(reason);
    let ban = tree.literal("ban", vec![reason], true);
    let targets = tree.argument("targets", entity(), vec![], true);
    let op = tree.literal("op", vec![targets], false);
    let gamemode = tree.argument("gamemode", plain(42), vec![], true);
    let gamemode = tree.literal("gamemode", vec![gamemode], false);
    let execute = execute_branch(&mut tree, entity());
    tree.set_children(Tree::ROOT, vec![help, say, kick, ban, op, gamemode, execute]);
    tree
}

const VERIFY: &[&str] = &[
    "help",
    "help me",
    "help ",
    "say hi",
    "say",
    "kick bob",
    "kick",
    "kick bob extra",
    "nope",
    "",
    "/help",
    "ban",
    "ban spam",
    "op Steve",
    "op @s",
    "gamemode creative",
    "gamemode nonsense",
    "execute run help",
    "execute run kick bob",
    "execute run say hi",
    "execute run",
    "execute as @a run help",
    "execute as Steve run help",
];

/// What running a command for the player needs first (`verifyCommand`).
pub fn command_verify() -> Value {
    let mut rows = Vec::new();
    for (number, text) in VERIFY.iter().enumerate() {
        for selectors in [true, false] {
            let label = format!("{number}-{text}-selectors-{}", title(selectors));
            rows.push(json!({"label": label, "text": text, "selectors": selectors}));
        }
    }
    let mut case = gui("command-verify", "command_verify", rows);
    case["tree"] = json!(verify_tree().payload());
    case
}
