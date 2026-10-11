//! Titles, boss bars, the sidebar and the player list.

use serde_json::{Value, json};

use super::gui;

/// Titles at a point of their times; `None` texts are left out.
struct Titles {
    elapsed: i64,
    partial: f64,
    title: Option<&'static str>,
    subtitle: Option<&'static str>,
    overlay: Option<&'static str>,
    overlay_elapsed: i64,
    times: (i64, i64, i64),
    size: (i64, i64),
}

const TITLES: Titles = Titles {
    elapsed: 0,
    partial: 0.0,
    title: Some("Hello"),
    subtitle: None,
    overlay: None,
    overlay_elapsed: 0,
    times: (10, 70, 20),
    size: (427, 240),
};

fn titles_row(label: &str, titles: Titles) -> Value {
    let mut row = json!({
        "label": label, "width": titles.size.0, "height": titles.size.1, "fade_in": titles.times.0,
        "stay": titles.times.1, "fade_out": titles.times.2, "elapsed": titles.elapsed,
        "overlay_elapsed": titles.overlay_elapsed, "partial": titles.partial,
    });
    for (key, text) in [("title", titles.title), ("subtitle", titles.subtitle), ("overlay", titles.overlay)] {
        if let Some(text) = text {
            row[key] = json!(text);
        }
    }
    row
}

/// The title, the subtitle and the action bar at points of their times:
/// the fade in and out, the hold, the scales and places, and a subtitle
/// with no title.
pub fn titles() -> Value {
    let bar = Titles { title: None, overlay: Some("Action bar text"), ..TITLES };
    let rows = vec![
        titles_row("fade-in-start", Titles { partial: 0.5, ..TITLES }),
        titles_row("fade-in-middle", Titles { elapsed: 5, partial: 0.25, ..TITLES }),
        titles_row("hold", Titles { elapsed: 30, subtitle: Some("below"), ..TITLES }),
        titles_row("fade-out", Titles { elapsed: 90, partial: 0.75, subtitle: Some("below"), ..TITLES }),
        titles_row("last-tick", Titles { elapsed: 99, ..TITLES }),
        titles_row(
            "title-only-wide",
            Titles { elapsed: 30, title: Some("A much longer title to centre"), size: (640, 480), ..TITLES },
        ),
        titles_row("custom-times", Titles { elapsed: 3, times: (5, 10, 5), subtitle: Some("x"), ..TITLES }),
        titles_row("no-fade-in", Titles { times: (0, 70, 20), ..TITLES }),
        titles_row("subtitle-without-title", Titles { title: None, subtitle: Some("lonely"), ..TITLES }),
        titles_row("action-bar", Titles { overlay_elapsed: 10, ..bar }),
        titles_row("action-bar-fading", Titles { overlay_elapsed: 50, partial: 0.5, ..bar }),
        titles_row(
            "both",
            Titles { elapsed: 20, subtitle: Some("sub"), overlay: Some("bar"), overlay_elapsed: 5, ..TITLES },
        ),
    ];
    gui("titles", "titles", rows)
}

fn bar(name: &str, progress: f64, color: i64, overlay: i64) -> Value {
    json!({"name": name, "progress": progress, "color": color, "overlay": overlay})
}

fn boss(label: &str, bars: Vec<Value>, (width, height): (i64, i64)) -> Value {
    json!({"label": label, "width": width, "height": height, "bars": bars})
}

/// The boss bars: every colour and notch, progress at its ends and in the
/// middle (the discrete width), several bars down the screen and the
/// cut-off at a third of its height.
pub fn boss_bar() -> Value {
    let usual = (427, 240);
    let mut rows = vec![
        boss("full", vec![bar("Ender Dragon", 1.0, 0, 0)], usual),
        boss("empty", vec![bar("Wither", 0.0, 4, 0)], usual),
        boss("a-sliver", vec![bar("Wither", 0.001, 2, 0)], usual),
        boss("half", vec![bar("Half", 0.5, 3, 0)], usual),
        boss("almost-full", vec![bar("Almost", 0.999, 5, 0)], usual),
    ];
    rows.extend((0..7).map(|color| boss(&format!("colour-{color}"), vec![bar("Colour", 0.7, color, 0)], usual)));
    rows.extend(
        (1..5).map(|overlay| boss(&format!("notches-{overlay}"), vec![bar("Notched", 0.6, 1, overlay)], usual)),
    );
    rows.push(boss("three", vec![bar("One", 0.3, 0, 0), bar("Two", 0.6, 1, 2), bar("Three", 0.9, 2, 4)], usual));
    let eight = || (0..8).map(|index| bar(&format!("Bar {index}"), 0.5, index % 7, 0)).collect::<Vec<_>>();
    rows.push(boss("cut-off", eight(), usual));
    rows.push(boss("cut-off-tall", eight(), (427, 480)));
    rows.push(boss("narrow-screen", vec![bar("A long boss name to centre", 0.4, 0, 0)], (200, 150)));
    gui("boss_bar", "boss_bar", rows)
}

fn objective(name: &str, title: &str, format: Option<&str>) -> Value {
    let mut described = json!({"name": name, "title": title});
    if let Some(format) = format {
        described["format"] = json!(format);
    }
    described
}

fn score(owner: &str, value: i64) -> Value {
    json!({"owner": owner, "objective": "o", "value": value})
}

fn board(label: &str, objectives: Vec<Value>, scores: Vec<Value>, slots: Option<Value>, teams: Option<Value>) -> Value {
    let slots = slots.unwrap_or_else(|| json!({"1": objectives[0]["name"]}));
    let mut row = json!({
        "label": label, "width": 427, "height": 240, "local": "me", "objectives": objectives, "scores": scores,
        "slots": slots,
    });
    if let Some(teams) = teams {
        row["teams"] = teams;
    }
    row
}

/// The scoreboard's sidebar: ordering, hidden `#` owners, the 15 line
/// limit, number formats, display names, team prefixes, suffixes and
/// colours, and a team colour's own sidebar over the general one.
pub fn sidebar() -> Value {
    gui("sidebar", "sidebar", [sidebar_rows(), team_rows()].concat())
}

fn sidebar_rows() -> Vec<Value> {
    let plain = || vec![objective("o", "Title", None)];
    let with = |mut score: Value, key: &str, value: &str| {
        score[key] = json!(value);
        score
    };
    let dress = json!([{"name": "t", "prefix": "[T] ", "suffix": "!", "color": 12, "players": ["al"]}]);
    vec![
        board("basic", plain(), vec![score("Alice", 5), score("Bob", 12), score("Cy", 1)], None, None),
        board(
            "ties-by-name",
            plain(),
            vec![score("bob", 5), score("Al", 5), score("cy", 5), score("AL2", 5)],
            None,
            None,
        ),
        board("negative", plain(), vec![score("a", -3), score("b", 0), score("c", 7)], None, None),
        board("hidden", plain(), vec![score("#total", 99), score("visible", 1)], None, None),
        board("fifteen-limit", plain(), (0..20).map(|i| score(&format!("p{i:02}"), i)).collect(), None, None),
        board(
            "wide-title",
            vec![objective("o", "A rather long scoreboard title", None)],
            vec![score("a", 1)],
            None,
            None,
        ),
        board(
            "long-name",
            vec![objective("o", "T", None)],
            vec![score("a_name_that_is_quite_long", 123456)],
            None,
            None,
        ),
        board(
            "blank-format",
            vec![objective("o", "Title", Some("blank"))],
            vec![score("a", 1), score("b", 2)],
            None,
            None,
        ),
        board("fixed-format", vec![objective("o", "Title", Some("fixed:many"))], vec![score("a", 1)], None, None),
        board("styled-format", vec![objective("o", "Title", Some("style:green"))], vec![score("a", 1)], None, None),
        board("per-score-format", plain(), vec![with(score("a", 1), "format", "fixed:x"), score("b", 2)], None, None),
        board("display-name", plain(), vec![with(score("a", 1), "display", "Shown")], None, None),
        board("team-dress", plain(), vec![score("al", 3), score("bo", 2)], None, Some(dress)),
    ]
}

fn team_rows() -> Vec<Value> {
    let plain = || vec![objective("o", "Title", None)];
    let in_objective = |owner: &str, value: i64, name: &str| json!({"owner": owner, "objective": name, "value": value});
    let mine = json!([{"name": "t", "color": 12, "players": ["me"]}]);
    let mut small = board("small-screen", plain(), vec![score("a", 1), score("b", 2)], None, None);
    small["width"] = json!(200);
    small["height"] = json!(150);
    vec![
        board(
            "team-colour-sidebar",
            vec![objective("all", "Title", None), objective("mine", "Team only", None)],
            vec![in_objective("al", 1, "all"), in_objective("x", 9, "mine")],
            Some(json!({"1": "all", "15": "mine"})),
            Some(mine.clone()),
        ),
        board(
            "team-without-slot",
            vec![objective("all", "Title", None)],
            vec![in_objective("al", 1, "all")],
            None,
            Some(mine),
        ),
        board("no-sidebar", plain(), vec![score("a", 1)], Some(json!({"0": "o"})), None),
        small,
    ]
}

fn player(name: &str, mode: i64, latency: i64, order: i64) -> Value {
    json!({"name": name, "mode": mode, "latency": latency, "order": order})
}

fn named(name: &str) -> Value {
    player(name, 0, 0, 0)
}

fn tab(label: &str, players: Vec<Value>, width: i64) -> Value {
    json!({"label": label, "width": width, "height": 240, "players": players})
}

fn texts(mut row: Value, header: Option<&str>, footer: Option<&str>) -> Value {
    if let Some(header) = header {
        row["header"] = json!(header);
    }
    if let Some(footer) = footer {
        row["footer"] = json!(footer);
    }
    row
}

fn tab_rows() -> Vec<Value> {
    let twenty: Vec<Value> = (0..20).map(|n| player(&format!("player{n:02}"), 0, n * 40, 0)).collect();
    let many: Vec<Value> = (0..45).map(|n| player(&format!("player{n:02}"), 0, n * 20 - 10, 0)).collect();
    let pings = [-1, 100, 200, 400, 800, 2000];
    let pinged = ["a", "b", "c", "d", "e", "f"].iter().zip(pings).map(|(name, latency)| player(name, 0, latency, 0));
    let mut display = named("a");
    display["display"] = json!("Alice the Great");
    let header = "a header long enough that the tab list has to break it over lines ".repeat(2);
    let sorted =
        vec![named("zed"), player("Amy", 3, 0, 0), named("bob"), player("Cat", 0, 0, 5), player("dan", 3, 0, 5)];
    vec![
        tab("alone", vec![player("Miner", 0, 23, 0)], 427),
        tab("pings", pinged.collect(), 427),
        texts(tab("header-footer", vec![named("Miner")], 427), Some("Welcome to the server"), Some("Have fun")),
        texts(tab("footer-only", vec![named("Miner")], 427), None, Some("Bottom line")),
        texts(tab("header-only", vec![named("Miner")], 427), Some("Top line"), None),
        texts(tab("wrapped-header", vec![named("Miner")], 427), Some(&header), None),
        tab("sorted", sorted, 427),
        tab("display-names", vec![display, named("b")], 427),
        tab("twenty", twenty, 427),
        tab("columns", many.clone(), 427),
        tab("narrow", many, 320),
        tab("eighty-cap", (0..95).map(|n| named(&format!("p{n:03}"))).collect(), 427),
    ]
}

/// A player list with the objective `o` in the list slot.
fn scored(label: &str, players: Vec<Value>, scores: &[(&str, i64)], objective: Value, width: i64) -> Value {
    let mut row = tab(label, players, width);
    row["objectives"] = json!([objective]);
    row["scores"] = json!(scores.iter().map(|(owner, value)| score(owner, *value)).collect::<Vec<_>>());
    row["slots"] = json!({"0": "o"});
    row
}

fn scored_rows() -> Vec<Value> {
    let trio = || vec![named("Amy"), named("Bob"), named("Cy")];
    let plain = || json!({"name": "o", "title": "T"});
    let format = |format: &str| json!({"name": "o", "title": "T", "format": format});
    let hearts = || json!({"name": "o", "title": "T", "hearts": true});
    let thirty: Vec<(String, i64)> = (0..30).map(|n| (format!("player{n:02}"), n * 13)).collect();
    let thirty: Vec<(&str, i64)> = thirty.iter().map(|(name, value)| (name.as_str(), *value)).collect();
    let column_players = (0..30).map(|n| named(&format!("player{n:02}"))).collect();
    let mut teams = scored("teams", vec![named("zed"), named("amy"), named("bob")], &[], plain(), 427);
    teams["teams"] = json!([
        {"name": "b", "prefix": "[B] ", "color": 12, "players": ["amy"]},
        {"name": "a", "suffix": " (A)", "color": 10, "players": ["zed", "bob"]},
    ]);
    let mut shown = named("zed");
    shown["display"] = json!("Shown");
    let mut with_display = scored("team-with-display", vec![shown], &[], plain(), 427);
    with_display["teams"] = json!([{"name": "a", "prefix": "[A] ", "color": 12, "players": ["zed"]}]);
    vec![
        scored("scores", trio(), &[("Amy", 3), ("Bob", 120), ("Cy", 7)], plain(), 427),
        scored("score-missing", trio(), &[("Amy", 3)], plain(), 427),
        scored("score-spectator", vec![named("Amy"), player("Bob", 3, 0, 0)], &[("Amy", 3), ("Bob", 5)], plain(), 427),
        scored("score-blank", trio(), &[("Amy", 3), ("Bob", 4)], format("blank"), 427),
        scored("score-fixed", trio(), &[("Amy", 3)], format("fixed:lots"), 427),
        scored("score-styled", trio(), &[("Amy", 3)], format("style:aqua"), 427),
        scored("score-columns", column_players, &thirty, plain(), 427),
        scored("hearts", trio(), &[("Amy", 20), ("Bob", 7), ("Cy", 1)], hearts(), 427),
        scored("hearts-over-twenty", trio(), &[("Amy", 30), ("Bob", 22)], hearts(), 427),
        scored("hearts-text", trio(), &[("Amy", 80), ("Bob", 100)], hearts(), 427),
        scored("hearts-zero", trio(), &[("Amy", 0)], hearts(), 427),
        scored("hearts-narrow", trio(), &[("Amy", 20)], hearts(), 160),
        teams,
        with_display,
    ]
}

/// The player list: one column and several, the header and the footer
/// (and their wrapping), spectators, display names, list order and the
/// ping icons, scores and teams.
pub fn tab_list() -> Value {
    gui("tab-list", "tab_list", [tab_rows(), scored_rows()].concat())
}
