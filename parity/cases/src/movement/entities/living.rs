//! A living entity hurt, killed or animated by the packets that do it,
//! ticked on.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};

/// The tags of the extra state an entity row asks for.
pub fn swing() -> &'static [&'static str] {
    &["swing"]
}

/// A living entity scenario.
pub struct Living {
    label: String,
    entity: String,
    ticks: i64,
    events: Vec<Value>,
    position: (f64, f64, f64),
    blocks: Vec<Value>,
    player_x: Option<f64>,
    extras: Option<Vec<String>>,
    data: Option<i64>,
    second: Option<Value>,
    water: Option<&'static str>,
    squid_speed: Option<f64>,
    fields: Vec<(String, Value)>,
}

pub fn living(label: impl Into<String>, entity: &str, ticks: i64, events: Vec<Value>) -> Living {
    Living {
        label: label.into(),
        entity: entity.to_owned(),
        ticks,
        events,
        position: (0.5, 64.0, 0.5),
        blocks: vec![],
        player_x: None,
        extras: None,
        data: None,
        second: None,
        water: None,
        squid_speed: None,
        fields: vec![],
    }
}

/// A swimmer at the middle of the scene's only loaded chunk, in a pool of
/// water, a layer of it that reaches only part of its box, or none.
pub fn fish(label: &str, entity: &str, ticks: i64, events: Vec<Value>) -> Living {
    Living {
        extras: Some(vec!["fish".to_owned()]),
        position: (8.5, 64.0, 8.5),
        water: Some("deep"),
        ..living(label, entity, ticks, events)
    }
}

/// A squid in a pool of water, its tentacle speed (drawn unseeded by the
/// constructor) fixed.
pub fn squid(label: &str, ticks: i64, events: Vec<Value>) -> Living {
    Living {
        extras: Some(vec!["squid".to_owned()]),
        position: (8.5, 64.0, 8.5),
        water: Some("deep"),
        squid_speed: Some(0.15),
        ..living(label, "squid", ticks, events)
    }
}

/// A shulker put where the packet says, not yet held to a block.
pub fn shulker_at(label: &str, ticks: i64, events: Vec<Value>, x: f64, y: f64, z: f64) -> Living {
    Living { position: (x, y, z), ..living(label, "shulker", ticks, events) }.extras(&["shulker"])
}

/// The culling scenarios: how far each kind of entity is drawn.
pub fn cull(label: &str, entity: &str, events: Vec<Value>) -> Living {
    living(format!("cull-{label}"), entity, 3, events).extras(&["cull"]).player_x(30.5)
}

fn water_blocks(water: &str) -> Vec<Value> {
    let layers = match water {
        "deep" => 60..70,
        "shallow" => 64..65,
        _ => 0..0,
    };
    layers
        .flat_map(|y| {
            (4..13).flat_map(move |x| (4..13).map(move |z| crate::movement::world::put(x, y, z, "minecraft:water")))
        })
        .collect()
}

impl Living {
    pub fn extras(self, extras: &[&str]) -> Self {
        Self { extras: Some(extras.iter().map(|extra| (*extra).to_owned()).collect()), ..self }
    }

    /// The one extra of a swimmer.
    pub fn tag(self, tag: &str) -> Self {
        self.extras(&[tag])
    }

    pub fn player_x(self, player_x: f64) -> Self {
        Self { player_x: Some(player_x), ..self }
    }

    /// `ClientboundAddEntityPacket.getData()`: a hanging entity's 3D
    /// direction value.
    pub fn data(self, data: i64) -> Self {
        Self { data: Some(data), ..self }
    }

    pub fn water(self, water: &'static str) -> Self {
        Self { water: Some(water), ..self }
    }

    /// The feet of a swimmer.
    pub fn y(self, y: f64) -> Self {
        Self { position: (self.position.0, y, self.position.2), ..self }
    }

    pub fn speed(self, speed: f64) -> Self {
        Self { squid_speed: Some(speed), ..self }
    }

    pub fn entity(self, entity: &str) -> Self {
        Self { entity: entity.to_owned(), ..self }
    }

    /// A boat beside a shulker to ride.
    pub fn vehicle(self, vehicle: bool) -> Self {
        if vehicle { self.beside("oak_boat", 3.5, 64.0, 5.5) } else { self }
    }

    /// With another entity, which stays where it is.
    pub fn beside(self, entity: &str, x: f64, y: f64, z: f64) -> Self {
        let second = json!({
            "entity": format!("minecraft:{entity}"), "entity_id": 41, "x": f64_bits(x), "y": f64_bits(y),
            "z": f64_bits(z),
        });
        Self { second: Some(second), ..self }
    }

    /// The row with a field set.
    pub fn set(mut self, key: &str, value: Value) -> Self {
        self.fields.push((key.to_owned(), value));
        self
    }

    pub fn row(self) -> Value {
        let mut events = self.events;
        if let Some(speed) = self.squid_speed {
            events.insert(0, json!({"tick": 0, "type": "squid_speed", "speed": f32_bits(speed)}));
        }
        let blocks = self.water.map_or(self.blocks, water_blocks);
        let mut row = json!({
            "label": self.label, "blocks": blocks, "chunk_radius": 0, "entity": format!("minecraft:{}", self.entity),
            "entity_id": 40, "x": f64_bits(self.position.0), "y": f64_bits(self.position.1),
            "z": f64_bits(self.position.2), "tick_count": self.ticks, "events": events,
        });
        if let Some(player_x) = self.player_x {
            row["player_x"] = json!(f64_bits(player_x));
        }
        if let Some(extras) = self.extras {
            row["extras"] = json!(extras);
        }
        if let Some(data) = self.data {
            row["data"] = json!(data);
        }
        if let Some(second) = self.second {
            row["second"] = second;
        }
        for (key, value) in self.fields {
            row[key] = value;
        }
        row
    }
}
