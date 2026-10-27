//! Item, projectile and shadow scenes: the rows the entity cases share.

use serde_json::{Value, json};

use crate::bits::{f32_bits, f64_bits};

type Vec3 = (f64, f64, f64);

/// A dropped item ticked in a scene (`ItemEntity.tick` on the client).
pub struct Item {
    label: String,
    blocks: Vec<Value>,
    position: Vec3,
    motion: Vec3,
    ticks: i64,
    on_ground: bool,
    entity_id: i64,
    events: Vec<Value>,
}

pub fn row(label: impl Into<String>, blocks: Vec<Value>, position: Vec3, motion: Vec3, ticks: i64) -> Item {
    Item { label: label.into(), blocks, position, motion, ticks, on_ground: false, entity_id: 21, events: vec![] }
}

impl Item {
    pub fn on_ground(self, on_ground: bool) -> Self {
        Self { on_ground, ..self }
    }

    pub fn entity_id(self, entity_id: i64) -> Self {
        Self { entity_id, ..self }
    }

    pub fn events(self, events: Vec<Value>) -> Self {
        Self { events, ..self }
    }

    pub fn row(self) -> Value {
        json!({
            "events": self.events, "label": self.label, "blocks": self.blocks, "chunk_radius": 1,
            "entity_id": self.entity_id, "x": f64_bits(self.position.0), "y": f64_bits(self.position.1),
            "z": f64_bits(self.position.2), "motion_x": f64_bits(self.motion.0),
            "motion_y": f64_bits(self.motion.1), "motion_z": f64_bits(self.motion.2),
            "on_ground": self.on_ground, "tick_count": self.ticks,
        })
    }
}

/// A projectile the server moves in coarse steps: the client carries it on
/// between its packets.
pub struct Projectile {
    label: String,
    kind: String,
    blocks: Vec<Value>,
    position: Vec3,
    motion: Vec3,
    ticks: i64,
    y_rot: f64,
    x_rot: f64,
    others: Vec<(i64, &'static str, Vec3)>,
    owner: Option<i64>,
    dangerous: Option<bool>,
    events: Option<Vec<Value>>,
}

pub fn proj(label: &str, kind: &str, blocks: Vec<Value>, position: Vec3, motion: Vec3, ticks: i64) -> Projectile {
    Projectile {
        label: label.to_owned(),
        kind: kind.to_owned(),
        blocks,
        position,
        motion,
        ticks,
        y_rot: 0.0,
        x_rot: 0.0,
        others: vec![],
        owner: None,
        dangerous: None,
        events: None,
    }
}

impl Projectile {
    pub fn y_rot(self, y_rot: f64) -> Self {
        Self { y_rot, ..self }
    }

    pub fn x_rot(self, x_rot: f64) -> Self {
        Self { x_rot, ..self }
    }

    pub fn others(self, others: Vec<(i64, &'static str, Vec3)>) -> Self {
        Self { others, ..self }
    }

    pub fn owner(self, owner: i64) -> Self {
        Self { owner: Some(owner), ..self }
    }

    pub fn dangerous(self, dangerous: bool) -> Self {
        Self { dangerous: Some(dangerous), ..self }
    }

    /// The server's packets between the ticks (see `events`).
    pub fn events(self, events: Vec<Value>) -> Self {
        Self { events: Some(events), ..self }
    }

    pub fn row(self) -> Value {
        let others: Vec<Value> = self
            .others
            .iter()
            .map(|(id, kind, at)| json!({"id": id, "type": kind, "x": f64_bits(at.0), "y": f64_bits(at.1), "z": f64_bits(at.2)}))
            .collect();
        let mut out = json!({
            "label": self.label, "blocks": self.blocks, "chunk_radius": 1, "entity_id": 21, "type": self.kind,
            "tick_count": self.ticks, "y_rot": f32_bits(self.y_rot), "x_rot": f32_bits(self.x_rot), "others": others,
            "x": f64_bits(self.position.0), "y": f64_bits(self.position.1), "z": f64_bits(self.position.2),
            "motion_x": f64_bits(self.motion.0), "motion_y": f64_bits(self.motion.1),
            "motion_z": f64_bits(self.motion.2),
        });
        if let Some(owner) = self.owner {
            out["owner"] = json!(owner);
        }
        if let Some(dangerous) = self.dangerous {
            out["dangerous"] = json!(dangerous);
        }
        if let Some(events) = self.events {
            out["events"] = json!(events);
        }
        out
    }
}

/// An entity's shadow: the block squares under it that are lit and whole
/// enough, how dark each is, and the quad drawn for each.
pub struct Shadow {
    label: String,
    blocks: Vec<Value>,
    position: Vec3,
    radius: f64,
    strength: f64,
    distance_sq: f64,
    invisible: bool,
    option: bool,
}

pub fn shadow(label: impl Into<String>, blocks: Vec<Value>) -> Shadow {
    Shadow {
        label: label.into(),
        blocks,
        position: (0.5, 64.0, 0.5),
        radius: 0.7,
        strength: 1.0,
        distance_sq: 0.0,
        invisible: false,
        option: true,
    }
}

impl Shadow {
    pub fn position(self, x: f64, y: f64, z: f64) -> Self {
        Self { position: (x, y, z), ..self }
    }

    pub fn radius(self, radius: f64) -> Self {
        Self { radius, ..self }
    }

    pub fn strength(self, strength: f64) -> Self {
        Self { strength, ..self }
    }

    pub fn distance_sq(self, distance_sq: f64) -> Self {
        Self { distance_sq, ..self }
    }

    pub fn invisible(self, invisible: bool) -> Self {
        Self { invisible, ..self }
    }

    pub fn option(self, option: bool) -> Self {
        Self { option, ..self }
    }

    pub fn row(self) -> Value {
        json!({
            "label": self.label, "blocks": self.blocks, "chunk_radius": 1,
            "x": f64_bits(self.position.0), "y": f64_bits(self.position.1), "z": f64_bits(self.position.2),
            "radius": f32_bits(self.radius), "strength": f32_bits(self.strength),
            "distance_sq": f64_bits(self.distance_sq), "invisible": self.invisible, "shadows_option": self.option,
            "tx": f32_bits(0.5), "ty": f32_bits(0.0), "tz": f32_bits(3.0),
        })
    }
}
