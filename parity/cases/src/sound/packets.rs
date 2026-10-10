//! The three sound packets decoded by the reference's own codecs.

use serde_json::{Value, json};

use crate::bits::{hex, varint};

fn identifier(text: &str) -> Vec<u8> {
    [varint(text.len() as i64), text.as_bytes().to_vec()].concat()
}

fn registered(id: i64) -> Vec<u8> {
    varint(id + 1)
}

fn direct(name: &str, range: Option<f32>) -> Vec<u8> {
    let mut out = [varint(0), identifier(name)].concat();
    match range {
        Some(range) => {
            out.push(1);
            out.extend(range.to_be_bytes());
        }
        None => out.push(0),
    }
    out
}

struct Sound {
    holder: Vec<u8>,
    source: i64,
    at: (i32, i32, i32),
    volume: f32,
    pitch: f32,
    seed: i64,
}

impl Sound {
    fn new(holder: Vec<u8>, source: i64, at: (i32, i32, i32)) -> Self {
        Self { holder, source, at, volume: 1.0, pitch: 1.0, seed: 1 }
    }

    fn sound(&self, volume: f32, pitch: f32, seed: i64) -> Self {
        Self { holder: self.holder.clone(), volume, pitch, seed, ..*self }
    }

    fn bytes(&self) -> Vec<u8> {
        let mut data = [self.holder.clone(), varint(self.source)].concat();
        for coordinate in [self.at.0, self.at.1, self.at.2] {
            data.extend(coordinate.to_be_bytes());
        }
        data.extend(self.volume.to_be_bytes());
        data.extend(self.pitch.to_be_bytes());
        data.extend(self.seed.to_be_bytes());
        data
    }
}

fn row(label: String, kind: &str, data: &[u8]) -> Value {
    json!({"label": label, "kind": kind, "hex": hex(data)})
}

fn sound(label: &str, sound: &Sound) -> Value {
    row(format!("sound-{label}"), "sound", &sound.bytes())
}

fn entity(label: &str, holder: Vec<u8>, source: i64, entity: i64, levels: (f32, f32, i64)) -> Value {
    let mut data = [holder, varint(source), varint(entity)].concat();
    data.extend(levels.0.to_be_bytes());
    data.extend(levels.1.to_be_bytes());
    data.extend(levels.2.to_be_bytes());
    row(format!("entity-{label}"), "sound_entity", &data)
}

fn stop(label: &str, flags: u8, source: Option<i64>, name: Option<&str>, extra: &[u8]) -> Value {
    let mut data = vec![flags];
    if let Some(source) = source {
        data.extend(varint(source));
    }
    if let Some(name) = name {
        data.extend(identifier(name));
    }
    data.extend(extra);
    row(format!("stop-{label}"), "stop_sound", &data)
}

fn sound_rows() -> Vec<Value> {
    let plain = Sound::new(registered(3), 4, (0, 0, 0));
    let truncated = plain.bytes()[..20].to_vec();
    let trailing = [plain.bytes(), vec![0, 1]].concat();
    vec![
        sound("registered-first", &Sound::new(registered(0), 4, (80, 520, -24)).sound(1.0, 1.0, 0)),
        sound("registered-later", &Sound::new(registered(1234), 0, (1, 2, 3)).sound(0.5, 1.25, 123456789012345)),
        sound("every-source", &Sound::new(registered(7), 10, (0, 0, 0)).sound(2.0, 0.5, -1)),
        sound("direct", &Sound::new(direct("my:sound.custom", None), 2, (-8, 8, 80000)).sound(1.5, 2.0, 9)),
        sound(
            "direct-range",
            &Sound::new(direct("minecraft:block.stone.break", Some(24.5)), 1, (3, -5, 7)).sound(1.0, 1.0, 5),
        ),
        sound("big-coordinates", &Sound::new(registered(3), 4, (i32::MAX, i32::MIN, 16777217))),
        sound("source-too-big", &Sound::new(registered(3), 11, (0, 0, 0))),
        sound("registered-too-big", &Sound::new(registered(99999), 4, (0, 0, 0))),
        row("sound-truncated".to_owned(), "sound", &truncated),
        row("sound-trailing".to_owned(), "sound", &trailing),
        sound("nan-volume", &plain.sound(f32::NAN, -0.0, 1)),
    ]
}

pub fn packets() -> Value {
    let mut rows = sound_rows();
    rows.extend([
        entity("registered", registered(15), 5, 77, (0.8, 1.1, 42)),
        entity("direct", direct("a:b", Some(0.0)), 8, 100000, (1.0, 1.0, -77)),
        entity("bad-source", registered(15), 12, 1, (1.0, 1.0, 1)),
        stop("everything", 0, None, None, &[]),
        stop("source", 1, Some(4), None, &[]),
        stop("name", 2, None, Some("minecraft:ui.button.click"), &[]),
        stop("both", 3, Some(9), Some("minecraft:block.anvil.land"), &[]),
        stop("bad-source", 1, Some(11), None, &[]),
        stop("trailing", 0, None, None, &[0]),
        stop("ignored-bits", 4, None, None, &[]),
    ]);
    crate::case::case("sound/packets", "sound", "packets", json!({ "rows": rows }))
}
