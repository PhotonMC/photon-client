//! The reference's own sound engine and music manager, with the sound
//! device replaced by channels that write down their calls.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::{case, merge};

const REGISTRY: &str = concat!(
    r#"{"a": {"sounds": [{"name": "x", "volume": 0.5, "pitch": 2.0, "attenuation_distance": 32}]}, "#,
    r#""s": {"sounds": [{"name": "mus", "stream": true, "volume": 0.8}]}, "#,
    r#""wrap": {"sounds": [{"name": "a", "type": "event", "volume": 0.5, "pitch": 0.5}]}, "#,
    r#""multi": {"sounds": ["x", "y", {"name": "z", "weight": 2}]}, "#,
    r#""gone": {"sounds": ["nothing"]}, "intentionally_empty": {"sounds": ["x"]}}"#,
);
const FILES: &[&str] = &["sounds/x.ogg", "sounds/y.ogg", "sounds/z.ogg", "sounds/mus.ogg"];

/// One sound played through the engine; `id: None` takes the next number.
#[derive(Clone)]
struct Play {
    event: &'static str,
    op: &'static str,
    id: Option<u32>,
    source: &'static str,
    volume: f64,
    pitch: f64,
    seed: i64,
    looping: bool,
    delay: u32,
    attenuation: &'static str,
    relative: bool,
    pos: [f64; 3],
    start_silent: bool,
    can_play: bool,
    tickable: Value,
    delay_ticks: u32,
}

const PLAY: Play = Play {
    event: "minecraft:a",
    op: "play",
    id: None,
    source: "BLOCKS",
    volume: 1.0,
    pitch: 1.0,
    seed: 1,
    looping: false,
    delay: 0,
    attenuation: "linear",
    relative: false,
    pos: [10.5, 64.0, -3.25],
    start_silent: false,
    can_play: true,
    tickable: Value::Null,
    delay_ticks: 0,
};

/// Numbers the plays of one scenario from 1; explicit ids still count.
#[derive(Default)]
struct Counter(u32);

impl Counter {
    fn play(&mut self, play: Play) -> Value {
        self.0 += 1;
        let mut step = json!({
            "op": play.op, "id": play.id.unwrap_or(self.0), "event": play.event,
            "source": play.source, "volume": f32_bits(play.volume), "pitch": f32_bits(play.pitch),
            "seed": play.seed.to_string(), "looping": play.looping, "delay": play.delay,
            "attenuation": play.attenuation, "relative": play.relative, "pos": play.pos,
            "start_silent": play.start_silent, "can_play": play.can_play, "tickable": play.tickable,
        });
        if play.op == "play_delayed" {
            merge(&mut step, json!({"delay_ticks": play.delay_ticks}));
        }
        step
    }

    fn plays(&mut self, plays: Vec<Play>) -> Vec<Value> {
        plays.into_iter().map(|play| self.play(play)).collect()
    }
}

fn scenario(label: &str, steps: Vec<Value>, volumes: Value, channels: (u32, u32)) -> Value {
    json!({
        "label": label, "json": REGISTRY, "files": FILES, "volumes": volumes,
        "static_channels": channels.0, "streaming_channels": channels.1, "steps": steps,
    })
}

fn plain(label: &str, steps: Vec<Value>) -> Value {
    scenario(label, steps, json!({}), (8, 2))
}

fn tick() -> Value {
    json!({"op": "tick", "paused": false})
}

fn ticks(count: usize) -> Vec<Value> {
    vec![tick(); count]
}

fn op(name: &str, id: u32) -> Value {
    json!({"op": name, "id": id})
}

fn with_id(id: u32) -> Play {
    Play { id: Some(id), ..PLAY }
}

fn streamed(source: &'static str) -> Play {
    Play { event: "minecraft:s", source, ..PLAY }
}

pub fn engine() -> Value {
    let rows = vec![
        plain_sounds(),
        refusals(),
        volume_sliders(),
        pools_run_out(),
        stopping(),
        lifetimes(),
        delays(),
        ticking(),
        pausing(),
    ];
    case("sound/engine", "sound", "engine", json!({ "rows": rows }))
}

fn plain_sounds() -> Value {
    let event = |event| Play { event, ..PLAY };
    let plays = vec![
        PLAY,
        Play { event: "minecraft:multi", seed: 7, volume: 0.75, pitch: 1.5, ..PLAY },
        Play { event: "minecraft:wrap", seed: 3, ..PLAY },
        Play { seed: 9, ..streamed("MUSIC") },
        Play { looping: true, seed: 9, ..streamed("MUSIC") },
        Play { looping: true, seed: 2, ..PLAY },
        Play { attenuation: "none", ..PLAY },
        Play { relative: true, ..event("minecraft:a") },
        Play { volume: 3.5, pitch: 9.0, pos: [-100.5, 5.0, 1e6], ..PLAY },
        Play { volume: 0.1, pitch: 0.01, ..PLAY },
    ];
    plain("plain-sounds", Counter::default().plays(plays))
}

fn refusals() -> Value {
    let event = |event| Play { event, ..PLAY };
    let plays = vec![
        event("minecraft:unknown"),
        event("minecraft:gone"),
        event("minecraft:intentionally_empty"),
        Play { can_play: false, ..PLAY },
        Play { volume: 0.0, ..PLAY },
        Play { volume: 0.0, start_silent: true, ..PLAY },
        Play { volume: 0.0, ..streamed("MUSIC") },
        Play { source: "MASTER", volume: 0.5, ..PLAY },
        Play { volume: -1.0, ..PLAY },
    ];
    plain("refusals", Counter::default().plays(plays))
}

fn volume_sliders() -> Value {
    let mut counter = Counter::default();
    let source = |source| Play { source, ..PLAY };
    let slider = |source: &str, value: f64| json!({"op": "slider", "source": source, "value": value});
    let category =
        |source: &str, value: f64| json!({"op": "category_volume", "source": source, "value": f32_bits(value)});
    let mut steps = counter.plays(vec![Play { volume: 0.9, ..source("MUSIC") }, source("HOSTILE"), source("MASTER")]);
    steps.extend([
        slider("MUSIC", 0.4),
        slider("MASTER", 0.5),
        category("HOSTILE", 0.25),
        category("MASTER", 2.0),
        counter.play(source("HOSTILE")),
        counter.play(source("MUSIC")),
        slider("BLOCKS", 0.0),
    ]);
    let volumes = json!({"MASTER": 0.8, "MUSIC": 0.6, "HOSTILE": 0.7, "BLOCKS": 1.0});
    scenario("volume-sliders", steps, volumes, (8, 2))
}

fn pools_run_out() -> Value {
    let mut plays = vec![PLAY; 4];
    plays.extend(vec![streamed("MUSIC"); 3]);
    scenario("pools-run-out", Counter::default().plays(plays), json!({}), (3, 2))
}

fn stopping() -> Value {
    let mut counter = Counter::default();
    let named =
        |event: Option<&str>, source: Option<&str>| json!({"op": "stop_named", "event": event, "source": source});
    let mut steps = counter.plays(vec![
        with_id(1),
        Play { event: "minecraft:multi", source: "HOSTILE", ..with_id(2) },
        Play { event: "minecraft:wrap", source: "HOSTILE", ..with_id(3) },
        Play { id: Some(4), ..streamed("MUSIC") },
    ]);
    steps.extend([
        op("stop", 1),
        op("stop", 1),
        named(Some("minecraft:multi"), Some("HOSTILE")),
        named(Some("minecraft:a"), Some("HOSTILE")),
        named(Some("minecraft:wrap"), None),
        named(None, Some("MUSIC")),
        counter.play(with_id(5)),
        named(None, None),
        op("active", 5),
        counter.play(with_id(6)),
    ]);
    plain("stop-by-instance-event-and-source", steps)
}

fn lifetimes() -> Value {
    let mut counter = Counter::default();
    let mut steps = vec![counter.play(with_id(1)), op("active", 1), tick(), op("finish", 1)];
    steps.extend([tick(), tick(), op("active", 1)]);
    steps.extend(ticks(20));
    steps.extend([op("active", 1), counter.play(with_id(2)), op("stop", 2), tick(), tick()]);
    scenario("lives-end-after-twenty-ticks", steps, json!({}), (1, 2))
}

fn delays() -> Value {
    let mut counter = Counter::default();
    let delayed = Play { op: "play_delayed", delay_ticks: 3, ..with_id(1) };
    let mut steps = vec![counter.play(delayed)];
    steps.extend(ticks(4));
    steps.push(counter.play(Play { looping: true, delay: 5, ..with_id(2) }));
    steps.push(op("finish", 2));
    steps.extend(ticks(24));
    steps.extend([op("stop", 2), op("finish", 2)]);
    steps.extend(ticks(24));
    plain("delay-and-looping-by-hand", steps)
}

fn ticking() -> Value {
    let mut counter = Counter::default();
    let first = json!({
        "stop_at": 4, "moves": [[1.0, 2.0, 3.0], [1.5, 2.5, 3.5]],
        "volumes": [0.9, 0.8], "pitches": [1.1, 1.2],
    });
    let queued = json!({"stop_at": 2, "moves": [[0.0, 0.0, 0.0]], "volumes": [0.5], "pitches": [0.5]});
    let delayed = json!({"stop_at": 9, "moves": [], "volumes": [], "pitches": []});
    let mut steps = vec![counter.play(Play { tickable: first, ..with_id(1) })];
    steps.extend(ticks(5));
    steps.push(counter.play(Play { op: "queue_ticking", tickable: queued, ..with_id(2) }));
    steps.extend(ticks(3));
    steps.push(op("active", 2));
    let third = Play { op: "play_delayed", delay_ticks: 2, tickable: delayed, ..with_id(3) };
    steps.push(counter.play(third));
    steps.extend(ticks(3));
    plain("ticking-sounds", steps)
}

fn pausing() -> Value {
    let mut counter = Counter::default();
    let pause = |sources: &[&str]| json!({"op": "pause_all_except", "sources": sources});
    let resume = json!({"op": "resume"});
    let paused = json!({"op": "tick", "paused": true});
    let mut steps = counter.plays(vec![
        with_id(1),
        Play { id: Some(2), ..streamed("MUSIC") },
        Play { source: "HOSTILE", ..with_id(3) },
    ]);
    steps.extend([
        pause(&["MUSIC"]),
        resume.clone(),
        pause(&[]),
        pause(&["MUSIC", "BLOCKS"]),
        resume,
        op("finish", 2),
        paused.clone(),
        op("finish", 1),
        paused,
        op("active", 2),
        op("active", 1),
    ]);
    plain("pause-and-resume", steps)
}

fn music(sound: &str, min: u32, max: u32, replace: bool) -> Value {
    json!({"sound": format!("minecraft:music.{sound}"), "min": min, "max": max, "replace": replace})
}

/// Music, volume, song state (when given) and count of identical ticks.
type Span<'a> = (&'a Value, f64, Option<bool>, usize);

fn music_row(label: &str, spans: &[Span], frequency: &str, seed: u32) -> Value {
    let mut ticks = Vec::new();
    for (music, volume, active, count) in spans {
        let mut step = json!({"music": music, "volume": f32_bits(*volume)});
        if let Some(active) = active {
            merge(&mut step, json!({ "active": active }));
        }
        ticks.extend(vec![step; *count]);
    }
    json!({"label": label, "frequency": frequency, "seed": seed, "ticks": ticks})
}

/// The reference's own music manager: the wait before the first song, the
/// wait after one, music that replaces the song playing, none to play, the
/// volume fading a song out, and each music frequency.
#[allow(dead_code, reason = "the gui family moves next")]
pub fn music_manager() -> Value {
    let rows = [music_waits(), music_changes()].concat();
    case("gui/music-manager", "gui", "music_manager", json!({ "rows": rows }))
}

fn music_waits() -> Vec<Value> {
    let game = music("game", 12000, 24000, false);
    let short = music("menu", 20, 600, true);
    let none = Value::Null;
    let song: Span = (&game, 1.0, None, 105);
    vec![
        music_row("first-song", &[song], "DEFAULT", 7),
        music_row(
            "short-delays-song-ends",
            &[(&short, 1.0, None, 40), (&short, 1.0, Some(false), 3), (&short, 1.0, None, 30)],
            "DEFAULT",
            7,
        ),
        music_row("no-music", &[(&none, 1.0, None, 5)], "DEFAULT", 7),
        music_row("music-comes-after-none", &[(&none, 1.0, None, 3), song], "DEFAULT", 7),
    ]
}

fn music_changes() -> Vec<Value> {
    let game = music("game", 12000, 24000, false);
    let other = music("creative", 12000, 24000, false);
    let end = music("end", 6000, 24000, true);
    let none = Value::Null;
    let song: Span = (&game, 1.0, None, 105);
    vec![
        music_row("replace-current", &[song, (&end, 1.0, None, 12)], "DEFAULT", 11),
        music_row("creative-does-not-replace", &[song, (&other, 1.0, None, 3)], "DEFAULT", 7),
        music_row("constant", &[song, (&game, 1.0, Some(false), 3), song], "CONSTANT", 7),
        music_row("frequent", &[song, (&game, 1.0, Some(false), 3), (&game, 1.0, None, 3)], "FREQUENT", 3),
        music_row("fade-out", &[song, (&game, 0.0, None, 200)], "DEFAULT", 7),
        music_row("fade-down-and-back", &[song, (&game, 0.4, None, 60), (&game, 1.0, None, 400)], "DEFAULT", 7),
        music_row(
            "silent-biome-after-song",
            &[song, (&none, 1.0, Some(false), 3), (&none, 1.0, None, 3)],
            "DEFAULT",
            7,
        ),
    ]
}
