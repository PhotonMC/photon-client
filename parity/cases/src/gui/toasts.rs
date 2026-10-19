//! The `toasts` case: the toast manager through scripts of events.

use serde_json::{Value, json};

use super::gui;
use crate::bits::f32_bits;

fn system(id: &str, title: &str, message: Option<&str>) -> Value {
    json!({"do": "system", "id": id, "title": title, "message": message})
}

fn system_update(id: &str, title: &str, message: Option<&str>) -> Value {
    json!({"do": "system_update", "id": id, "title": title, "message": message})
}

fn update(at: i64) -> Value {
    json!({"do": "update", "at": at})
}

fn draw() -> Value {
    json!({"do": "draw"})
}

/// An update at each time, each followed by a draw.
fn run(times: &[i64]) -> Vec<Value> {
    times.iter().flat_map(|time| [update(*time), draw()]).collect()
}

struct Setup {
    music_toast: &'static str,
    display_time: f64,
    hud_hidden: bool,
    pause_screen: bool,
    song: Option<&'static str>,
    width: i64,
}

const SETUP: Setup = Setup {
    music_toast: "PAUSE_AND_TOAST",
    display_time: 1.0,
    hud_hidden: false,
    pause_screen: false,
    song: None,
    width: 427,
};

fn row(label: &str, setup: &Setup, events: Vec<Value>) -> Value {
    let mut row = json!({
        "label": label, "width": setup.width, "height": 240,
        "music_toast": setup.music_toast, "display_time": setup.display_time,
        "hud_hidden": setup.hud_hidden, "pause_screen": setup.pause_screen,
        "events": events,
    });
    if let Some(song) = setup.song {
        row["song"] = json!(song);
    }
    row
}

/// A toast across its slide in, its wait and its slide out; six at once, so some wait for room; a
/// toast of two rows; a toast changed while shown and while waiting; one hidden by force; the
/// display time option; the music toast shown, hidden and left out by its option and by the pause
/// screen; the interface hidden; and the colours of the music notes by tick.
pub fn toasts() -> Value {
    let mut rows = system_rows();
    rows.extend(changed_rows());
    rows.extend(music_rows());
    rows.extend(advancement_rows());
    rows.extend(recipe_rows());
    rows.extend(tutorial_rows());
    let ticks: Vec<f64> = (0..400).map(|tick| f64::from(tick) + [0.0, 0.25, 0.5, 0.99][tick as usize % 4]).collect();
    rows.push(row("music-note-colours", &SETUP, vec![json!({"do": "note_colours", "ticks": ticks})]));
    gui("toasts", "toasts", rows)
}

/// System toasts alone: across their life, six at once, a tall one, in a narrow window.
fn system_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let lifetime = [1000, 1100, 1300, 1599, 1600, 1601, 3000, 6599, 6600, 6601, 6900, 7200, 7201, 7202];
    let mut events = vec![system("PERIODIC_NOTIFICATION", "Hello", Some("A message of the system"))];
    events.extend(run(&lifetime));
    rows.push(row("one-toast", &SETUP, events));
    let mut events = vec![system("WORLD_BACKUP", "", Some("an empty title"))];
    events.extend(run(&[0, 700, 1300]));
    rows.push(row("empty-system-title", &SETUP, events));
    let mut events = vec![system("WORLD_BACKUP", "", None)];
    events.extend(run(&[0, 700, 1300]));
    rows.push(row("empty-system-words", &SETUP, events));
    let mut events = vec![system("WORLD_BACKUP", "Only a title", None)];
    events.extend(run(&[0, 300, 600, 601, 9000]));
    rows.push(row("title-only", &SETUP, events));
    let mut events: Vec<Value> = (0..6)
        .map(|index| system("PERIODIC_NOTIFICATION", &format!("Toast {index}"), Some("waits for room")))
        .collect();
    events.extend(run(&[0, 700, 1300, 6000, 6700, 7300, 7400, 8000, 12_000]));
    rows.push(row("six-toasts", &SETUP, events));
    let long = "A message long enough to be broken into several lines at the width of a toast";
    let mut events = vec![
        system("CHUNK_LOAD_FAILURE", "A title that is long enough to need more than one line to be shown", Some(long)),
        system("WORLD_BACKUP", "Next", Some("small")),
        system("PACK_COPY_FAILURE", "Third", None),
    ];
    events.extend(run(&[0, 700, 1300, 6000, 7300]));
    rows.push(row("tall-toast", &SETUP, events));
    let mut events = vec![system("PERIODIC_NOTIFICATION", "Narrow", Some("window"))];
    events.extend(run(&[0, 700, 1300]));
    rows.push(row("small-window", &Setup { width: 200, ..SETUP }, events));
    rows
}

/// Toasts changed while shown and while waiting, hidden by force, and kept by the display time.
fn changed_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let mut events = vec![system("PERIODIC_NOTIFICATION", "First", Some("text"))];
    events.extend(run(&[0, 700, 4000]));
    events.push(system_update("PERIODIC_NOTIFICATION", "Second", Some("changed")));
    events.extend(run(&[4100, 8500, 9100, 9200, 9800, 10_400, 11_000]));
    rows.push(row("updated-while-shown", &SETUP, events));
    let mut events: Vec<Value> =
        (0..5).map(|_| system("PERIODIC_NOTIFICATION", "Fills the slots", Some("x"))).collect();
    events.push(system("FILE_DROP_FAILURE", "Waiting", Some("old words")));
    events.extend(run(&[0, 700]));
    events.push(system_update("FILE_DROP_FAILURE", "Waiting", Some("new words")));
    events.extend(run(&[1300, 6000, 7300, 7400, 8000, 13_000]));
    rows.push(row("updated-while-waiting", &SETUP, events));
    let mut events = vec![system("WORLD_BACKUP", "Sent away", Some("early"))];
    events.extend(run(&[0, 700, 1300]));
    events.push(json!({"do": "force_hide", "id": "WORLD_BACKUP"}));
    events.extend(run(&[1400, 1700, 2000, 2100, 2500]));
    rows.push(row("force-hidden", &SETUP, events));
    for (name, time) in [("doubled", 2.0), ("least", 0.5), ("tenfold", 10.0)] {
        let setup = Setup { display_time: time, ..SETUP };
        let mut events = vec![system("LOW_DISK_SPACE", "Kept", Some("by the option"))];
        events.extend(run(&[0, 700, 2900, 3100, 6000, 6700, 12_000, 12_700, 13_300, 70_000, 110_000]));
        rows.push(row(&format!("display-time-{name}"), &setup, events));
    }
    let mut events = vec![system("PERIODIC_NOTIFICATION", "Hidden with the interface", Some("x"))];
    events.extend(run(&[0, 700, 1300]));
    rows.push(row("hud-hidden", &Setup { hud_hidden: true, ..SETUP }, events));
    rows
}

/// The music toast: shown and hidden, and left out by its option, by the pause screen and by the
/// interface being hidden.
fn music_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let mut events = vec![json!({"do": "show_now_playing"})];
    events.extend(run(&[0, 300, 600, 601, 3000, 5599, 5600, 5601, 6000, 6200, 6201, 6202]));
    events.push(json!({"do": "hide_now_playing"}));
    events.extend(run(&[6300, 6600, 6900, 7200, 7201]));
    rows.push(row("now-playing", &song_setup(), events));
    for (name, setup) in [
        ("now-playing-pause-only", Setup { music_toast: "PAUSE", ..song_setup() }),
        ("now-playing-never", Setup { music_toast: "NEVER", ..song_setup() }),
        ("now-playing-pause-screen", Setup { pause_screen: true, ..song_setup() }),
        ("now-playing-hud-hidden", Setup { hud_hidden: true, ..song_setup() }),
        ("now-playing-no-song", Setup { song: None, ..song_setup() }),
        ("now-playing-display-time", Setup { display_time: 2.0, ..song_setup() }),
    ] {
        let mut events = vec![json!({"do": "show_now_playing"})];
        events.extend(run(&[0, 700, 5000, 9000, 11_000, 12_000]));
        rows.push(row(name, &setup, events));
    }
    rows
}

fn song_setup() -> Setup {
    Setup { song: Some("minecraft:music/game/aerie"), ..SETUP }
}

/// An advancement toast event: a title of the given kind (0 task, 1 challenge, 2 goal).
fn advancement(id: &str, title: &str, frame: i32) -> Value {
    json!({"do": "advancement", "advancement": {
        "id": id, "parent": null, "requirements": [],
        "display": {"title": title, "description": "d", "frame": frame, "show_toast": true, "hidden": false,
            "x": f32_bits(0.0), "y": f32_bits(0.0)},
    }})
}

/// Advancement toasts: each kind with a title of one line, titles of none, two and three lines
/// across the fade at 1500 ms and the one that follows, the challenge's sound, the display time
/// option, and several at once.
fn advancement_rows() -> Vec<Value> {
    let lifetime = [
        0, 600, 601, 1300, 2099, 2100, 2101, 2250, 2399, 2400, 2401, 2500, 3000, 5599, 5600, 5601, 6199, 6200, 6201,
        6900, 7201,
    ];
    let two = "An advancement title long enough for two lines";
    let three = "An advancement title that is long enough to need three lines at the width a toast gives it";
    let mut rows = Vec::new();
    for (name, title, frame) in [
        ("task", "Stone Age", 0),
        ("challenge", "Return to Sender", 1),
        ("goal", "Monster Hunter", 2),
        ("empty-title", "", 0),
        ("two-lines", two, 0),
        ("three-lines-challenge", three, 1),
    ] {
        let mut events = vec![advancement("minecraft:test/toast", title, frame)];
        events.extend(run(&lifetime));
        rows.push(row(&format!("advancement-{name}"), &SETUP, events));
    }
    for (name, time) in [("least", 0.5), ("tenfold", 10.0)] {
        let setup = Setup { display_time: time, ..SETUP };
        let mut events = vec![advancement("minecraft:test/toast", two, 1)];
        events.extend(run(&[0, 600, 601, 2100, 2800, 2801, 3400, 3401, 6000, 20_000, 51_000, 51_700]));
        rows.push(row(&format!("advancement-display-time-{name}"), &setup, events));
    }
    let mut events = vec![advancement("minecraft:test/x", "Sound", 1), advancement("minecraft:test/y", "Once", 1)];
    events.extend(run(&[0, 700, 1300]));
    rows.push(row("advancement-two-challenges", &SETUP, events));
    let mut events = vec![
        advancement("minecraft:test/a", "First", 0),
        advancement("minecraft:test/b", two, 1),
        system("PERIODIC_NOTIFICATION", "Between", Some("a system toast")),
        advancement("minecraft:test/c", "Third", 2),
    ];
    events.extend(run(&[0, 700, 1300, 2300, 6000, 6700, 7300, 8000]));
    rows.push(row("advancement-several", &SETUP, events));
    rows
}

/// A recipe toast event: the items named by a recipe's crafting station and result.
fn recipe(station: Option<&str>, result: Option<&str>) -> Value {
    json!({"do": "recipe", "station": station, "result": result})
}

fn table(result: &str) -> Value {
    recipe(Some("minecraft:crafting_table"), Some(result))
}

/// Recipe toasts: one recipe, several of them cycling through the display time option, ones
/// added while it is shown, while it slides out and while it waits behind other toasts, and a
/// recipe with no items.
fn recipe_rows() -> Vec<Value> {
    let lifetime = [0, 600, 601, 1700, 2500, 3000, 3400, 4400, 5599, 5600, 5601, 6199, 6200, 6201, 7000, 11_300];
    let mut rows = Vec::new();
    for (name, time) in [("least", 0.5), ("normal", 1.0), ("tenfold", 10.0)] {
        let setup = Setup { display_time: time, ..SETUP };
        for count in [1, 2, 3, 7] {
            let mut events: Vec<Value> = (0..count).map(|_| table("minecraft:stick")).collect();
            events.extend(run(&lifetime));
            rows.push(row(&format!("recipe-{count}-{name}"), &setup, events));
        }
    }
    rows.push(row("recipe-empty", &SETUP, {
        let mut events = vec![recipe(None, None)];
        events.extend(run(&[0, 700, 2000, 5700, 6400, 6401]));
        events
    }));
    let mut events = vec![table("minecraft:stick")];
    events.extend(run(&[0, 700, 3000]));
    events.push(table("minecraft:torch"));
    events.extend(run(&[3001, 5000, 8000, 8001, 9000, 12_000, 13_700]));
    rows.push(row("recipe-added-while-shown", &SETUP, events));
    let mut events = vec![table("minecraft:stick")];
    events.extend(run(&[0, 700, 5700, 6000]));
    events.push(table("minecraft:torch"));
    events.extend(run(&[6001, 6100, 6400, 6700, 12_000]));
    rows.push(row("recipe-added-while-sliding-out", &SETUP, events));
    let mut events: Vec<Value> = (0..5).map(|n| system("PERIODIC_NOTIFICATION", &format!("n{n}"), None)).collect();
    events.push(table("minecraft:stick"));
    events.extend(run(&[0, 700]));
    events.push(table("minecraft:torch"));
    events.extend(run(&[701, 5000, 5800, 6000, 6500, 7000, 12_000]));
    rows.push(row("recipe-queued-behind-five", &SETUP, events));
    rows
}

/// A tutorial toast event: the icon by its index, the title, the message, whether it has a bar
/// of progress and how long it stays.
fn tutorial(icon: u32, (title, message): (&str, Option<&str>), progressable: bool, time_ms: i32) -> Value {
    json!({"do": "tutorial", "icon": icon, "title": title, "message": message,
        "progressable": progressable, "time_ms": time_ms})
}

fn tutorial_progress(icon: u32, value: f32) -> Value {
    json!({"do": "tutorial_progress", "icon": icon, "value": f64::from(value)})
}

fn tutorial_hide(icon: u32) -> Value {
    json!({"do": "tutorial_hide", "icon": icon})
}

/// Tutorial toasts: the timed one the social interactions key gets, one with a bar of progress
/// that moves and is hidden, titles of several lines, and no message.
fn tutorial_rows() -> Vec<Value> {
    let lifetime = [0, 600, 601, 1000, 2000, 4000, 8000, 8599, 8600, 8601, 9000, 9200, 9201, 9800];
    let mut rows = Vec::new();
    let mut events = vec![tutorial(5, ("Social Interactions", Some("Press P to view the players")), true, 8000)];
    events.extend(run(&lifetime));
    rows.push(row("tutorial-timed", &SETUP, events));
    let long = "A title that is long enough to need several lines in a toast";
    let mut events =
        vec![tutorial(0, (long, Some("and a message that is long enough as well to go over lines")), true, 0)];
    events.extend(run(&[0, 700]));
    events.push(tutorial_progress(0, 0.5));
    events.extend(run(&[701, 750, 800, 850, 1000, 2000]));
    events.push(tutorial_progress(0, 1.0));
    events.extend(run(&[2100, 2150, 2400, 3000]));
    events.push(tutorial_hide(0));
    events.extend(run(&[3001, 3500, 3700, 3701, 3900, 4400]));
    rows.push(row("tutorial-progress-hidden", &SETUP, events));
    let mut events = vec![tutorial(3, ("Only a title", None), false, 0)];
    events.extend(run(&[0, 700, 2000]));
    rows.push(row("tutorial-no-message", &SETUP, events));
    let mut events = vec![tutorial(6, ("One", Some("Two")), false, 3000), table("minecraft:stick")];
    events.extend(run(&[0, 700, 2000, 3500, 3600, 4300]));
    rows.push(row("tutorial-with-recipe", &SETUP, events));
    rows
}
