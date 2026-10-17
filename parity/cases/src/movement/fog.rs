//! The fog a camera is drawn with: each fluid and the air, with the effects that change it.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};
use crate::case::case;

/// A row: a camera in a fluid or the air, with some effects, at some height, looking some way.
struct Fog<'a> {
    label: String,
    fog_type: &'a str,
    local: bool,
    spectator: bool,
    effects: Vec<Value>,
    y: f64,
    render_distance: i64,
    water_vision_time: i64,
    darken_world: f64,
    sun_angle: f64,
    forward_x: f64,
}

impl<'a> Fog<'a> {
    fn new(label: &str, fog_type: &'a str) -> Self {
        Self {
            label: label.to_owned(),
            fog_type,
            local: true,
            spectator: false,
            effects: Vec::new(),
            y: 80.0,
            render_distance: 12,
            water_vision_time: 0,
            darken_world: 0.0,
            sun_angle: 40.0,
            forward_x: 0.5,
        }
    }

    fn effect(mut self, name: &str, duration: i64, blend: bool, ticks: i64) -> Self {
        self.effects.push(json!({"name": name, "duration": duration, "blend": blend, "ticks": ticks}));
        self
    }

    fn row(self) -> Value {
        json!({
            "label": self.label, "blocks": [], "chunk_radius": 1, "fog_type": self.fog_type, "local": self.local,
            "spectator": self.spectator, "effects": self.effects, "x": f64_bits(8.5), "y": f64_bits(self.y), "z": f64_bits(8.5),
            "render_distance": self.render_distance, "water_vision_time": self.water_vision_time,
            "darken_world": f32_bits(self.darken_world), "forward_x": f32_bits(self.forward_x), "partial": f32_bits(0.4),
            "attributes": {
                "fog_color": 0xFFC0_D8FF_u32 as i32, "sunrise": 0xB3FF_8040_u32 as i32, "sky_color": 0xFF78_A7FF_u32 as i32,
                "water_color": 0xFF3F_76E4_u32 as i32, "sun_angle": f32_bits(self.sun_angle),
                "fog_start": f32_bits(-8.0), "fog_end": f32_bits(192.0), "sky_fog_end": f32_bits(512.0),
                "cloud_fog_end": f32_bits(300.0), "water_start": f32_bits(-8.0), "water_end": f32_bits(96.0),
            },
        })
    }
}

/// The air, towards and away from the sun, over a short and a long render distance; water; lava and
/// powder snow.
fn fluid_rows(rows: &mut Vec<Value>) {
    // The air, towards and away from the sun, and over a short and a long render distance.
    for (angle, forward) in [(0.0, 1.0), (40.0, 0.5), (100.0, -1.0), (260.0, 1.0), (260.0, -0.3), (180.0, 1.0)] {
        for distance in [2, 12, 32] {
            let mut fog = Fog::new(&format!("air, sun {angle}, facing {forward}, {distance} chunks"), "atmospheric");
            fog.sun_angle = angle;
            fog.forward_x = forward;
            fog.render_distance = distance;
            rows.push(fog.row());
        }
    }
    // Water, as well as the player has seen it, and as another's camera sees it.
    for time in [0, 50, 99, 100, 250, 599, 600] {
        let mut fog = Fog::new(&format!("water, {time} ticks"), "water");
        fog.water_vision_time = time;
        rows.push(fog.row());
    }
    let mut remote = Fog::new("water, another's camera", "water");
    remote.local = false;
    rows.push(remote.row());
    // Lava and powder snow, as a spectator, with fire resistance, and as a player.
    for fog_type in ["lava", "powder_snow"] {
        rows.push(Fog::new(fog_type, fog_type).row());
        let mut spectator = Fog::new(&format!("{fog_type}, spectator"), fog_type);
        spectator.spectator = true;
        rows.push(spectator.row());
    }
    rows.push(Fog::new("lava with fire resistance", "lava").effect("minecraft:fire_resistance", 600, false, 3).row());
}

/// The effects that change the fog.
fn effect_rows(rows: &mut Vec<Value>) {
    // Blindness, with all its time, little, and none, in the air and in water.
    for (duration, ticks) in [(-1, 5), (1200, 5), (100, 85), (40, 30), (25, 6), (19, 0), (10, 4), (1, 0)] {
        rows.push(
            Fog::new(&format!("blindness {duration} for {ticks}"), "atmospheric")
                .effect("minecraft:blindness", duration, false, ticks)
                .row(),
        );
    }
    rows.push(Fog::new("blindness in water", "water").effect("minecraft:blindness", 600, false, 0).row());
    rows.push(Fog::new("blindness in lava", "lava").effect("minecraft:blindness", 600, false, 0).row());
    // Darkness, fading in, whole, and fading out.
    for (duration, blend, ticks) in
        [(400, true, 0), (400, true, 10), (400, true, 40), (400, false, 3), (60, true, 70), (-1, true, 15)]
    {
        rows.push(
            Fog::new(&format!("darkness {duration} {blend} for {ticks}"), "atmospheric")
                .effect("minecraft:darkness", duration, blend, ticks)
                .row(),
        );
    }
    rows.push(
        Fog::new("darkness and blindness", "atmospheric")
            .effect("minecraft:darkness", 400, true, 30)
            .effect("minecraft:blindness", 200, false, 0)
            .row(),
    );
    // Night vision, steady and flickering, and with darkness.
    for (duration, ticks) in [(2000, 0), (260, 0), (150, 0), (60, 20), (7, 0)] {
        rows.push(
            Fog::new(&format!("night vision {duration} for {ticks}"), "atmospheric")
                .effect("minecraft:night_vision", duration, false, ticks)
                .row(),
        );
    }
    rows.push(Fog::new("night vision under water", "water").effect("minecraft:night_vision", 100, false, 0).row());
    rows.push(
        Fog::new("night vision and darkness", "atmospheric")
            .effect("minecraft:night_vision", 100, false, 0)
            .effect("minecraft:darkness", 400, true, 30)
            .row(),
    );
}

/// The void's darkness near the bottom of the world, and the bosses'.
fn world_rows(rows: &mut Vec<Value>) {
    for y in [-64.0, -60.0, -48.0, -33.0, -20.0] {
        let mut fog = Fog::new(&format!("air at {y}"), "atmospheric");
        fog.y = y;
        rows.push(fog.row());
        let mut lava = Fog::new(&format!("lava at {y}"), "lava");
        lava.y = y;
        rows.push(lava.row());
    }
    for amount in [0.25, 1.0] {
        let mut fog = Fog::new(&format!("darkened by {amount}"), "atmospheric");
        fog.darken_world = amount;
        rows.push(fog.row());
    }
}

pub fn fog_data() -> Value {
    let mut rows = Vec::new();
    fluid_rows(&mut rows);
    effect_rows(&mut rows);
    world_rows(&mut rows);
    case("movement/fog-data", "movement", "fog_data", json!({"rows": rows}))
}
