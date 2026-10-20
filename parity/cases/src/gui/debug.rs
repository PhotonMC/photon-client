//! The debug overlay (F3).

use serde_json::{Map, Value, json};

use super::gui;
use crate::bits::{f32_bits, f64_bits};

type Place = (f64, f64, f64);

/// One overlay row: a window, the camera entity at `place` and the usual frame and connection, with
/// `changes` laid over it.
fn row(label: String, place: (Place, (f64, f64)), changes: Value) -> Value {
    let ((x, y, z), (yaw, pitch)) = place;
    let mut row = json!({
        "label": label, "width": 427, "height": 240, "launched": "26.2",
        "fps": 60, "limit": 120, "iconified": false, "present": "FIFO",
        "brand": "vanilla", "frozen": false, "stepping": false,
        "sent": f32_bits(0.0), "received": f32_bits(0.0),
        "overlay": true, "reduced": false, "hidden": false,
        "x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z),
        "yaw": f32_bits(yaw), "pitch": f32_bits(pitch),
    });
    let fields: &mut Map<String, Value> = row.as_object_mut().expect("an object");
    for (key, value) in changes.as_object().expect("an object") {
        fields.insert(key.clone(), value.clone());
    }
    row
}

/// The debug overlay (F3): the entries that need no running game with the overlay up or down, with
/// reduced debug info, with the interface hidden, and the camera entity at places that round, wrap
/// and floor differently (ties at the third decimal, negative coordinates, the four facings, an
/// angle past a half turn); then the frame rate line (the limit at the cutoff, a minimised window,
/// each present mode, no configuration) and the server line (the brand, frozen and stepping, the
/// averages rounded, no brand).
pub fn hud_debug_overlay() -> Value {
    let mut rows = place_rows();
    rows.extend(frame_and_server_rows());
    rows.extend(impactor_rows());
    rows.extend(system_rows());
    gui("hud-debug-overlay", "hud_debug_overlay", rows)
}

/// The rows of the camera entity's places and the overlay's states.
fn place_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let places: [(Place, (f64, f64)); 7] = [
        ((1.0005, 64.0625, 2.5), (22.25, 22.25)),
        ((-0.5, -60.5, -15.999), (-135.0, -90.0)),
        ((100.125, 70.0, -300.9995), (190.5, 0.04)),
        ((0.0, 0.0, 0.0), (0.0, 0.0)),
        ((33.3333333, 12.12345678, 47.0), (-45.0, 45.0)),
        ((-1234.5678, 319.99999, 5000.0005), (359.9, -0.05)),
        ((15.99999, 15.0, 16.0), (720.0, 33.35)),
    ];
    for (index, place) in places.into_iter().enumerate() {
        for (overlay, reduced, hidden) in
            [(true, false, false), (false, false, false), (true, true, false), (true, false, true)]
        {
            if index > 2 && (!overlay || reduced || hidden) {
                continue;
            }
            rows.push(row(
                format!("debug-{index}-{overlay}-{reduced}-{hidden}"),
                place,
                json!({"fps": 60 + index, "overlay": overlay, "reduced": reduced, "hidden": hidden}),
            ));
        }
    }
    rows
}

/// The rows of the frame rate and the server lines, and a small window.
fn frame_and_server_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let origin = ((1.0, 2.0, 3.0), (0.0, 0.0));
    rows.push(row(
        "debug-small-window".to_owned(),
        origin,
        json!({"width": 160, "height": 90, "fps": 144, "limit": 260, "present": "MAILBOX"}),
    ));
    for (label, fps, limit, iconified, present) in [
        ("unlimited-immediate", 1000, 260, false, "IMMEDIATE"),
        ("limit-250", 5, 250, false, "FIFO_RELAXED"),
        ("iconified", 0, 120, true, "MAILBOX"),
        ("no-surface", 61, 60, false, "none"),
        ("limit-10", 9, 10, false, "FIFO"),
    ] {
        rows.push(row(
            format!("debug-fps-{label}"),
            origin,
            json!({"fps": fps, "limit": limit, "iconified": iconified, "present": present}),
        ));
    }
    for (label, brand, frozen, stepping, (sent, received)) in [
        ("vanilla", json!("vanilla"), false, false, (0.0, 0.0)),
        ("tie-up", json!("vanilla"), false, false, (2.5, 3.5)),
        ("frozen", json!("Pumpkin"), true, false, (11.4, 120.6)),
        ("stepping", json!("vanilla"), true, true, (1.0, 2.0)),
        ("stepping-only", json!("vanilla"), false, true, (0.49, 0.5)),
        ("no-brand", Value::Null, false, false, (7.0, 8.0)),
    ] {
        rows.push(row(
            format!("debug-server-{label}"),
            origin,
            json!({
                "brand": brand, "frozen": frozen, "stepping": stepping,
                "sent": f32_bits(sent), "received": f32_bits(received),
            }),
        ));
    }
    rows
}

/// The performance options entry: each cloud status and filtering method, the blend radius at its
/// ends, improved transparency, and the anisotropy held to what the device supports.
fn impactor_rows() -> Vec<Value> {
    let origin = ((1.0, 2.0, 3.0), (0.0, 0.0));
    let mut rows = Vec::new();
    for (label, transparency, clouds, blend, filtering, bit, limit) in [
        ("defaults", false, 2, 2, 0, 2, 16),
        ("off", true, 0, 0, 1, 2, 16),
        ("fast-max", false, 1, 7, 0, 2, 16),
        ("aniso-4", true, 2, 3, 2, 2, 16),
        ("aniso-2", false, 1, 5, 2, 1, 16),
        ("aniso-8", false, 0, 1, 2, 3, 16),
        ("aniso-capped", false, 2, 2, 2, 3, 4),
        ("aniso-unsupported", false, 2, 2, 2, 2, 1),
    ] {
        let impactors = json!({
            "transparency": transparency, "clouds": clouds, "blend": blend,
            "filtering": filtering, "bit": bit, "limit": limit,
        });
        rows.push(row(format!("debug-impactors-{label}"), origin, json!({"impactors": impactors})));
    }
    for (label, overlay, reduced) in [("hidden-overlay", false, false), ("reduced", true, true)] {
        rows.push(row(
            format!("debug-impactors-{label}"),
            origin,
            json!({"overlay": overlay, "reduced": reduced,
                "impactors": {"transparency": true, "clouds": 1, "blend": 4, "filtering": 2,
                    "bit": 3, "limit": 16}}),
        ));
    }
    rows
}

/// The system entry: each device type, a driver with several lines, vendors by name, the window's
/// size, and the entry beside the performance options and under reduced debug info.
fn system_rows() -> Vec<Value> {
    let origin = ((1.0, 2.0, 3.0), (0.0, 0.0));
    let system = |runtime: &str, cpu: &str, size: (i32, i32), device: (&str, &str, i32)| {
        json!({
            "runtime": runtime, "cpu": cpu, "width": size.0, "height": size.1,
            "vendor": device.0, "name": device.1, "type": device.2,
            "backend": "Vulkan", "driver": "1.4.311 NVIDIA 570.133.07",
        })
    };
    let mut rows = Vec::new();
    for (label, type_code) in [("other", 0), ("integrated", 1), ("discrete", 2), ("virtual", 3), ("cpu", 4)] {
        rows.push(row(
            format!("debug-system-{label}"),
            origin,
            json!({"system": system("25.0.1", "16x AMD Ryzen 7 5800X 8-Core Processor", (854, 480),
                ("NVIDIA", "NVIDIA GeForce RTX 3070", type_code))}),
        ));
    }
    let mut lines = system("21.0.7", "<unknown>", (1, 1), ("0x1af4", "llvmpipe (LLVM 19.1.7)", 4));
    lines["driver"] = json!("1.3.296 llvmpipe Mesa 25.0.3\nsecond line\nthird");
    lines["backend"] = json!("");
    rows.push(row("debug-system-multi-line-driver".to_owned(), origin, json!({"system": lines})));
    rows.push(row(
        "debug-system-with-options".to_owned(),
        origin,
        json!({"system": system("25.0.1", "4x Intel Core", (1920, 1080), ("INTEL", "Intel UHD", 1)),
            "impactors": {"transparency": false, "clouds": 2, "blend": 2, "filtering": 0,
                "bit": 2, "limit": 16}}),
    ));
    rows.push(row(
        "debug-system-reduced".to_owned(),
        origin,
        json!({"reduced": true, "system": system("25.0.1", "2x X", (427, 240), ("AMD", "Radeon", 2))}),
    ));
    rows
}

/// The crosshair under F3: with the overlay up the 3D crosshair is shown and the flat one is not,
/// attack indicator and all; reduced debug info, no overlay and another camera keep the flat one.
/// Each has a half-charged attack indicator, which goes with it.
pub fn crosshair_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for (label, overlay, reduced, third_person, spectator) in [
        ("overlay", true, false, false, false),
        ("overlay-reduced", true, true, false, false),
        ("no-overlay", false, false, false, false),
        ("no-overlay-reduced", false, true, false, false),
        ("overlay-third-person", true, false, true, false),
        ("overlay-spectator", true, false, false, true),
    ] {
        for (indicator, name) in [(1, "crosshair"), (2, "hotbar")] {
            rows.push(json!({
                "label": format!("debug-{label}-{name}"), "width": 427, "height": 240,
                "attack_indicator": indicator, "attack_ticks": 2, "attack_speed": 1.6,
                "target": "none", "spectator": spectator, "third_person": third_person,
                "selected_slot": 0,
                "debug": {"overlay": overlay, "reduced": reduced},
            }));
        }
    }
    rows
}

/// `extractLines` with lines in both columns: empty lines between, long lines that overlap in a narrow
/// window, and a column with nothing.
pub fn hud_debug_lines() -> Value {
    let long = "Facing: south (Towards positive Z) (22.3 / 22.3)";
    let rows = vec![
        json!({"label": "both", "width": 427, "height": 240,
            "left": ["Minecraft 26.2 (26.2/vanilla)", "", "XYZ: 1.001 / 64.06250 / 2.500", "Block: 1 64 2"],
            "right": ["Allocated: 12% 123/1024MB", "", "CPU: 8x some processor", "Display: 854x480"]}),
        json!({"label": "narrow-overlap", "width": 160, "height": 90,
            "left": [long, "", long], "right": [long, "", "Chunk: 0 4 0 [0 0 in r.0.0.mca]"]}),
        json!({"label": "right-only", "width": 427, "height": 240, "left": [], "right": ["a", "", "bb"]}),
        json!({"label": "left-only", "width": 427, "height": 240, "left": ["a", "", "bb"], "right": []}),
    ];
    gui("hud-debug-lines", "hud_debug_lines", rows)
}

/// The debug screen's model: what is registered with its group and its reduced-info rule, the
/// profiles, and an entry list put through toggles, profile loads and files (what it saves, and
/// what it makes of what it reads).
pub fn debug_entries() -> Value {
    let toggle = |id: &str| json!({"op": "toggle", "id": id});
    let overlay = |visible: bool| json!({"op": "overlay", "visible": visible});
    let write = |text: &str| json!({"op": "write", "text": text});
    let rows = vec![
        json!({"label": "registry", "registry": true}),
        // An entry nothing was set for comes on always, with the overlay up or not.
        json!({"label": "toggle", "ops": [
            overlay(true), toggle("chunk_borders"), toggle("chunk_borders"), toggle("chunk_borders"),
            toggle("chunk_borders"), toggle("fps"), toggle("fps"), toggle("fps"),
            overlay(false), toggle("fps"), toggle("fps"), toggle("entity_hitboxes"),
        ]}),
        json!({"label": "profiles", "ops": [
            {"op": "profile", "profile": "PERFORMANCE"},
            {"op": "set", "id": "fps", "status": "NEVER"},
            {"op": "set", "id": "biome", "status": "ALWAYS_ON"},
            overlay(true), {"op": "reduced", "on": true}, {"op": "reduced", "on": false},
            {"op": "profile", "profile": "DEFAULT"},
        ]}),
        json!({"label": "files", "ops": [
            write(r#"{"custom":{"minecraft:fps":"alwaysOn","minecraft:bogus":"never"},"DataVersion":4903}"#),
            toggle("fps"),
            write(r#"{"profile":"performance","DataVersion":4903}"#),
            write(r#"{"DataVersion":4903}"#),
            write("not json"),
            write(r#"{"profile":"nope"}"#),
            write(r#"{"custom":{"minecraft:tps":"sometimes"}}"#),
            write(r#"{"custom":{"minecraft:tps":"inOverlay"}}"#),
        ]}),
    ];
    gui("debug-entries", "debug_entries", rows)
}
