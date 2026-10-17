//! The crosshair and hotbar, and the status bars of a real local player.

use serde_json::{Value, json};

use super::gui;
use crate::bits::f32_bits;
use crate::line::float_text;

/// One hotbar scenario.
struct Hotbar {
    size: (i64, i64),
    indicator: i64,
    ticks: i64,
    speed: f64,
    target: &'static str,
    spectator: bool,
    third_person: bool,
    slot: i64,
    /// Ticks frozen, which also draws the camera overlays; none leaves them out.
    frozen: Option<i64>,
    /// The portal effect's intensity, which also draws the camera overlays.
    portal: Option<f64>,
}

const HOTBAR: Hotbar = Hotbar {
    size: (427, 240),
    indicator: 1,
    ticks: 0,
    speed: 4.0,
    target: "none",
    spectator: false,
    third_person: false,
    slot: 0,
    frozen: None,
    portal: None,
};

fn hotbar_row(label: &str, hotbar: Hotbar) -> Value {
    let mut row = json!({
        "label": label, "width": hotbar.size.0, "height": hotbar.size.1, "attack_indicator": hotbar.indicator,
        "attack_ticks": hotbar.ticks, "attack_speed": hotbar.speed, "target": hotbar.target,
        "spectator": hotbar.spectator, "third_person": hotbar.third_person, "selected_slot": hotbar.slot,
    });
    if let Some(frozen) = hotbar.frozen {
        row["frozen_ticks"] = json!(frozen);
    }
    if let Some(portal) = hotbar.portal {
        row["portal"] = json!(portal);
    }
    row
}

/// The crosshair, the attack indicator under it or beside the hotbar, and
/// the hotbar: the option's three values, the attack strength from none to
/// full, what the crosshair is on, the game mode and the camera, the
/// selected slot, and several window sizes.
pub fn hud_hotbar() -> Value {
    let mut rows = Vec::new();
    for (indicator, name) in [(0, "off"), (1, "crosshair"), (2, "hotbar")] {
        for ticks in [0, 1, 2, 3, 4, 5, 9] {
            rows.push(hotbar_row(&format!("{name}-fast-{ticks}"), Hotbar { indicator, ticks, ..HOTBAR }));
        }
        for ticks in [0, 1, 6, 12, 13] {
            rows.push(hotbar_row(&format!("{name}-sword-{ticks}"), Hotbar { indicator, ticks, speed: 1.6, ..HOTBAR }));
        }
        for target in ["living", "dead", "item"] {
            for (ticks, speed) in [(20, 1.6), (6, 1.6), (5, 4.0), (5, 3.9), (6, 3.9)] {
                let label = format!("{name}-{target}-{ticks}-{}", float_text(speed));
                rows.push(hotbar_row(&label, Hotbar { indicator, ticks, speed, target, ..HOTBAR }));
            }
        }
    }
    let sword = Hotbar { ticks: 2, speed: 1.6, ..HOTBAR };
    rows.extend([
        hotbar_row("slot-0", HOTBAR),
        hotbar_row("slot-4", Hotbar { slot: 4, ..HOTBAR }),
        hotbar_row("slot-8", Hotbar { slot: 8, ..HOTBAR }),
        hotbar_row("small", Hotbar { size: (320, 240), ..HOTBAR }),
        hotbar_row("wide", Hotbar { size: (1000, 500), ..HOTBAR }),
        hotbar_row("odd", Hotbar { size: (641, 361), ..HOTBAR }),
        hotbar_row("tiny", Hotbar { size: (213, 120), ..HOTBAR }),
        hotbar_row("hotbar-odd", Hotbar { size: (641, 361), indicator: 2, ..sword }),
        hotbar_row("third-person", Hotbar { third_person: true, ..sword }),
        hotbar_row("third-person-hotbar", Hotbar { third_person: true, indicator: 2, ..sword }),
        hotbar_row("spectator", Hotbar { spectator: true, ..sword }),
        hotbar_row("spectator-hotbar", Hotbar { spectator: true, indicator: 2, ..sword }),
        hotbar_row("slow-speed", Hotbar { speed: 0.5, ticks: 20, ..HOTBAR }),
        hotbar_row("fast-speed", Hotbar { speed: 20.0, ticks: 0, ..HOTBAR }),
        hotbar_row("hotbar-slow-speed", Hotbar { indicator: 2, speed: 0.5, ticks: 20, ..HOTBAR }),
    ]);
    // The powder snow frame: its alpha grows with the ticks frozen up to a full one.
    for frozen in [0, 1, 20, 70, 139, 140, 300] {
        rows.push(hotbar_row(&format!("frozen-{frozen}"), Hotbar { frozen: Some(frozen), ..HOTBAR }));
    }
    // The portal overlay: its alpha is lifted to the fourth power below a full effect.
    for portal in [0.0125, 0.25, 0.5, 0.75, 0.9999, 1.0] {
        rows.push(hotbar_row(&format!("portal-{portal}"), Hotbar { portal: Some(portal), ..HOTBAR }));
    }
    rows.push(hotbar_row("portal-and-frozen", Hotbar { portal: Some(0.5), frozen: Some(70), ..HOTBAR }));
    rows.extend([
        hotbar_row("frozen-third-person", Hotbar { frozen: Some(70), third_person: true, ..HOTBAR }),
        hotbar_row("frozen-small", Hotbar { frozen: Some(35), size: (320, 240), ..HOTBAR }),
        hotbar_row("frozen-odd", Hotbar { frozen: Some(100), size: (641, 361), ..HOTBAR }),
    ]);
    gui("hud-hotbar", "hud_hotbar", rows)
}

fn frame(tick: i64, ms: i64, health: f64, invulnerable: i64) -> Value {
    json!({"tick": tick, "ms": ms, "health": f32_bits(health), "invulnerable": invulnerable})
}

/// One status scenario.
#[derive(Clone)]
struct Status {
    frames: Vec<Value>,
    size: (i64, i64),
    max_health: f64,
    absorption: f64,
    armor: f64,
    food: i64,
    saturation: f64,
    air: i64,
    eye_in_water: bool,
    effects: Vec<&'static str>,
    frozen_ticks: i64,
    hardcore: bool,
    xp: (i64, f64),
}

fn status() -> Status {
    Status {
        frames: vec![frame(0, 0, 20.0, 0)],
        size: (427, 240),
        max_health: 20.0,
        absorption: 0.0,
        armor: 0.0,
        food: 20,
        saturation: 5.0,
        air: 300,
        eye_in_water: false,
        effects: vec![],
        frozen_ticks: 0,
        hardcore: false,
        xp: (0, 0.0),
    }
}

fn at(tick: i64, health: f64) -> Status {
    Status { frames: vec![frame(tick, 0, health, 0)], ..status() }
}

impl Status {
    fn row(self, label: &str) -> Value {
        json!({
            "label": label, "width": self.size.0, "height": self.size.1, "hardcore": self.hardcore,
            "max_health": self.max_health, "absorption": f32_bits(self.absorption), "armor": self.armor,
            "food": self.food, "saturation": f32_bits(self.saturation), "air": self.air,
            "eye_in_water": self.eye_in_water, "effects": self.effects, "frozen_ticks": self.frozen_ticks,
            "xp_level": self.xp.0, "xp_progress": f32_bits(self.xp.1), "frames": self.frames,
        })
    }
}

fn health_rows() -> Vec<Value> {
    let mut rows = vec![
        status().row("full"),
        Status { size: (320, 240), ..status() }.row("small"),
        Status { size: (1000, 500), ..status() }.row("wide"),
        Status { size: (641, 361), ..status() }.row("odd"),
    ];
    for health in [0.0, 0.5, 1.0, 3.0, 4.0, 4.5, 5.0, 10.0, 10.5, 19.0, 19.5, 20.0] {
        rows.push(at(7, health).row(&format!("health-{}", float_text(health))));
    }
    for max_health in [2.0, 10.0, 30.0, 40.0, 60.0, 100.0, 200.0] {
        let label = format!("max-{}", float_text(max_health));
        rows.push(Status { max_health, ..at(3, max_health) }.row(&label));
        rows.push(Status { max_health, ..at(3, max_health / 2.0 + 0.5) }.row(&format!("{label}-half")));
    }
    for absorption in [0.5, 1.0, 4.0, 10.0, 20.0, 25.0, 40.0] {
        let label = format!("absorption-{}", float_text(absorption));
        rows.push(Status { absorption, ..at(2, 20.0) }.row(&label));
        rows.push(Status { absorption, ..at(2, 7.0) }.row(&format!("{label}-hurt")));
    }
    for armor in [0.0, 1.0, 2.0, 7.0, 19.0, 20.0, 30.0] {
        rows.push(Status { armor, ..status() }.row(&format!("armor-{}", float_text(armor))));
    }
    rows.push(Status { armor: 20.0, max_health: 60.0, ..at(1, 40.0) }.row("armor-over-rows"));
    rows
}

fn food_and_air_rows() -> Vec<Value> {
    let mut rows: Vec<Value> = [0, 1, 2, 9, 10, 19, 20]
        .iter()
        .map(|food| Status { food: *food, ..status() }.row(&format!("food-{food}")))
        .collect();
    for tick in [0, 1, 5, 6, 13] {
        for food in [0, 4, 20] {
            rows.push(Status { food, saturation: 0.0, ..at(tick, 20.0) }.row(&format!("starving-{food}-{tick}")));
        }
    }
    rows.push(Status { effects: vec!["hunger"], food: 11, ..status() }.row("hunger"));
    rows.push(Status { effects: vec!["hunger"], food: 0, saturation: 0.0, ..at(0, 20.0) }.row("hunger-starving"));
    for air in [300, 299, 290, 250, 150, 31, 30, 1, 0, -20] {
        rows.push(Status { air, eye_in_water: true, ..at(1, 20.0) }.row(&format!("air-{air}")));
        rows.push(Status { air, ..at(1, 20.0) }.row(&format!("air-{air}-out")));
    }
    for tick in 0..4 {
        rows.push(Status { air: 0, eye_in_water: true, ..at(tick, 20.0) }.row(&format!("drowned-{tick}")));
    }
    rows.push(Status { eye_in_water: true, ..status() }.row("in-water-full"));
    let armored = Status { air: 100, eye_in_water: true, armor: 10.0, max_health: 40.0, food: 20, ..at(1, 40.0) };
    rows.push(armored.row("air-with-armor"));
    rows
}

fn effect_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for effect in ["poison", "wither", "regeneration"] {
        for tick in [0, 3, 9, 24] {
            rows.push(Status { effects: vec![effect], ..at(tick, 13.0) }.row(&format!("{effect}-{tick}")));
        }
        rows.push(
            Status { effects: vec![effect], absorption: 6.0, ..at(4, 13.0) }.row(&format!("{effect}-absorption")),
        );
        rows.push(Status { effects: vec![effect], hardcore: true, ..at(4, 13.0) }.row(&format!("{effect}-hardcore")));
    }
    rows.extend([
        Status { effects: vec!["regeneration"], max_health: 40.0, ..at(31, 30.0) }.row("regeneration-big"),
        Status { effects: vec!["poison", "wither"], ..status() }.row("poison-wither"),
        Status { frozen_ticks: 139, ..at(0, 13.0) }.row("frozen-short"),
        Status { frozen_ticks: 140, ..at(0, 13.0) }.row("frozen"),
        Status { frozen_ticks: 200, effects: vec!["poison"], ..at(0, 13.0) }.row("frozen-poisoned"),
        Status { hardcore: true, ..at(0, 13.0) }.row("hardcore"),
        Status { hardcore: true, absorption: 5.0, ..at(0, 13.0) }.row("hardcore-absorption"),
    ]);
    let levels = [
        (0, 0.0),
        (0, 0.5),
        (1, 0.0),
        (1, 0.01),
        (5, 0.5),
        (9, 0.99),
        (10, 1.0),
        (30, 0.37),
        (99, 0.5),
        (100, 0.25),
        (1000, 0.9),
        (0, 0.001),
        (3, 0.0054),
    ];
    for (level, progress) in levels {
        rows.push(Status { xp: (level, progress), ..status() }.row(&format!("xp-{level}-{}", float_text(progress))));
    }
    rows
}

/// Frames after a change of health, every tick from `first` to `last`.
fn falling(health: f64, first: i64, last: i64) -> Vec<Value> {
    (first..last).map(|tick| frame(tick, tick * 50, health, (20 - (tick - 1)).max(0))).collect()
}

fn frames(list: &[(i64, i64, f64, i64)]) -> Vec<Value> {
    list.iter().map(|(tick, ms, health, invulnerable)| frame(*tick, *ms, *health, *invulnerable)).collect()
}

fn timed_rows() -> Vec<Value> {
    let over = |frames: Vec<Value>| Status { frames, ..status() };
    let hurt = frames(&[(0, 0, 20.0, 0), (1, 50, 14.0, 20)]);
    let three = frames(&[(0, 0, 20.0, 0), (1, 50, 14.0, 20), (2, 100, 14.0, 19)]);
    vec![
        over([hurt.clone(), falling(14.0, 2, 26)].concat()).row("hurt"),
        over([frames(&[(0, 0, 10.0, 0), (1, 50, 16.0, 20)]), falling(16.0, 2, 14)].concat()).row("healed"),
        over(frames(&[
            (0, 0, 20.0, 0),
            (1, 50, 14.0, 20),
            (5, 250, 11.0, 15),
            (6, 300, 11.0, 14),
            (14, 700, 11.0, 6),
            (30, 1500, 11.0, 0),
            (31, 1550, 11.0, 0),
        ]))
        .row("hurt-twice"),
        over(frames(&[(0, 0, 20.0, 0), (1, 50, 14.0, 0), (2, 100, 14.0, 0)])).row("hurt-no-immunity"),
        over(frames(&[
            (0, 0, 20.0, 0),
            (1, 50, 8.0, 20),
            (10, 500, 8.0, 11),
            (20, 1000, 8.0, 1),
            (21, 1100, 8.0, 0),
            (22, 1200, 8.0, 0),
            (23, 2300, 8.0, 0),
        ]))
        .row("catches-up"),
        Status {
            absorption: 8.0,
            ..over([hurt, frames(&[(2, 100, 14.0, 19), (3, 150, 14.0, 18), (4, 200, 14.0, 17)])].concat())
        }
        .row("hurt-with-absorption"),
        Status {
            max_health: 40.0,
            ..over(frames(&[(0, 0, 40.0, 0), (1, 50, 25.0, 20), (2, 100, 25.0, 19), (3, 150, 25.0, 18)]))
        }
        .row("hurt-big"),
        over(frames(&[(0, 0, 6.0, 0), (1, 50, 2.0, 20), (2, 100, 2.0, 19), (3, 150, 2.0, 18), (4, 200, 2.0, 17)]))
            .row("hurt-low"),
        Status { effects: vec!["regeneration"], ..over(three.clone()) }.row("hurt-regeneration"),
        Status { hardcore: true, ..over(three) }.row("hurt-hardcore"),
    ]
}

/// The status bars of a real local player: the hearts, the armor, the food,
/// the air bubbles, the experience bar and its level, several window sizes,
/// and frames over time.
pub fn hud_status() -> Value {
    gui("hud-status", "hud_status", [health_rows(), food_and_air_rows(), effect_rows(), timed_rows()].concat())
}
