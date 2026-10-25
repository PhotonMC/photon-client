//! The player breaking, using and attacking: the attack and use keys as
//! `handleKeybinds` spends them, into the reference's own game mode, and the
//! entities in the crosshair.

mod aiming;
mod bottling;
mod consuming;
mod digging;
mod effects;
mod gliding;
mod item_on;
mod opening;
mod pistons;
mod placing;
mod throwing;
mod utility;

use serde_json::{Value, json};

use super::scene::Scene;
use crate::bits::f64_bits;
use crate::case::case;
use crate::movement::world::{cat, floor, put};

/// A scene whose player holds items, with the effects it has and the
/// entities around it.
pub struct Interaction {
    scene: Scene,
    main_hand: Option<&'static str>,
    count: Option<i64>,
    /// A `use_cooldown` component (seconds) on the main hand's stack.
    use_cooldown: Option<f64>,
    /// A `use_remainder` component (one of this item) on the main hand's stack.
    use_remainder: Option<&'static str>,
    /// A `custom_name` component (plain text) on the main hand's stack.
    custom_name: Option<&'static str>,
    /// Components on the main hand's stack, as `DataComponentPatch.STREAM_CODEC` hex.
    patch: Option<String>,
    off_hand: Option<&'static str>,
    /// Components on the off hand's stack, as `DataComponentPatch.STREAM_CODEC` hex.
    off_hand_patch: Option<String>,
    chest: Option<&'static str>,
    rockets: Option<i64>,
    effects: Option<Vec<Value>>,
    /// The attributes the server sent, each with its base and modifiers.
    attributes: Vec<Value>,
    entities: Option<Vec<Value>>,
}

fn interaction(label: &str, blocks: Vec<Value>, ticks: Vec<Value>, count: Option<i64>) -> Interaction {
    Interaction {
        scene: Scene::new(label, cat![floor(), blocks], ticks),
        main_hand: None,
        count,
        use_cooldown: None,
        use_remainder: None,
        custom_name: None,
        patch: None,
        off_hand: None,
        off_hand_patch: None,
        chest: None,
        rockets: None,
        effects: None,
        attributes: Vec::new(),
        entities: None,
    }
}

/// Breaking: the scene of a player at the attack key.
pub fn breaking(label: &str, blocks: Vec<Value>, ticks: Vec<Value>) -> Interaction {
    interaction(label, blocks, ticks, None)
}

/// Using: the scene of a player at the use key, holding `count` items.
pub fn using(label: &str, blocks: Vec<Value>, ticks: Vec<Value>) -> Interaction {
    interaction(label, blocks, ticks, Some(1))
}

/// Entities in the crosshair.
pub fn facing(label: &str, entities: Vec<Value>, ticks: Vec<Value>) -> Interaction {
    Interaction { entities: Some(entities), ..interaction(label, vec![], ticks, None) }
}

impl Interaction {
    pub fn blocks(self, blocks: Vec<Value>) -> Self {
        let scene = self.scene.with_blocks(cat![floor(), blocks]);
        Self { scene, ..self }
    }

    pub fn block_entity(self, at: (i64, i64, i64), kind: &str) -> Self {
        Self { scene: self.scene.block_entity(at, kind), ..self }
    }

    pub fn main_hand(self, item: &'static str) -> Self {
        Self { main_hand: Some(item), ..self }
    }

    pub fn count(self, count: i64) -> Self {
        Self { count: Some(count), ..self }
    }

    pub fn use_cooldown(self, seconds: f64) -> Self {
        Self { use_cooldown: Some(seconds), ..self }
    }

    pub fn custom_name(self, name: &'static str) -> Self {
        Self { custom_name: Some(name), ..self }
    }

    pub fn patch(self, hex: String) -> Self {
        Self { patch: Some(hex), ..self }
    }

    pub fn use_remainder(self, item: &'static str) -> Self {
        Self { use_remainder: Some(item), ..self }
    }

    pub fn off_hand_patch(self, hex: String) -> Self {
        Self { off_hand_patch: Some(hex), ..self }
    }

    pub fn off_hand(self, item: &'static str) -> Self {
        Self { off_hand: Some(item), ..self }
    }

    /// Fireworks that fly with the player.
    pub fn rockets(self, count: i64) -> Self {
        Self { rockets: Some(count), ..self }
    }

    pub fn chest(self, item: &'static str) -> Self {
        Self { chest: Some(item), ..self }
    }

    pub fn effects(self, effects: &[(&str, i64)]) -> Self {
        let effects = effects.iter().map(|(name, amplifier)| json!({"effect": name, "amplifier": amplifier})).collect();
        Self { effects: Some(effects), ..self }
    }

    /// An attribute as the server sends it: its base and its modifiers as `(id, amount, operation)`.
    pub fn attribute(mut self, name: &str, base: f64, modifiers: &[(&str, f64, i64)]) -> Self {
        let modifiers: Vec<Value> = modifiers
            .iter()
            .map(|(id, amount, operation)| json!({"id": id, "amount": f64_bits(*amount), "operation": operation}))
            .collect();
        self.attributes.push(json!({"attribute": name, "base": f64_bits(base), "modifiers": modifiers}));
        self
    }

    pub fn start(self, x: f64, y: f64, z: f64) -> Self {
        Self { scene: self.scene.start(x, y, z), ..self }
    }

    pub fn y_rot(self, y_rot: f64) -> Self {
        Self { scene: self.scene.y_rot(y_rot), ..self }
    }

    pub fn x_rot(self, x_rot: f64) -> Self {
        Self { scene: self.scene.x_rot(x_rot), ..self }
    }

    pub fn game_mode(self, game_mode: &'static str) -> Self {
        Self { scene: self.scene.game_mode(game_mode), ..self }
    }

    /// The components a row puts on the main hand's stack.
    fn add_stack_components(&self, held: &mut Value) {
        if let Some(seconds) = self.use_cooldown {
            held["main_hand_use_cooldown"] = json!(seconds);
        }
        if let Some(item) = self.use_remainder {
            held["main_hand_use_remainder"] = json!(item);
        }
        if let Some(hex) = &self.patch {
            held["main_hand_patch"] = json!(hex);
        }
        if let Some(name) = self.custom_name {
            held["main_hand_custom_name"] = json!(name);
        }
    }

    pub fn row(self) -> Value {
        let mut held = json!({});
        self.add_stack_components(&mut held);
        let mut row = self.scene.row();
        if let Some(item) = self.main_hand {
            held["main_hand"] = json!(format!("minecraft:{}", item.trim_start_matches("minecraft:")));
            if let Some(count) = self.count {
                held["main_hand_count"] = json!(count);
            }
        }
        if let Some(item) = self.off_hand {
            held["off_hand"] = json!(item);
            if let Some(hex) = &self.off_hand_patch {
                held["off_hand_patch"] = json!(hex);
            }
        }
        if let Some(count) = self.rockets {
            held["rockets"] = json!(count);
        }
        if let Some(item) = self.chest {
            held["chest"] = json!(item);
        }
        if let Some(effects) = self.effects {
            held["effects"] = json!(effects);
        }
        if !self.attributes.is_empty() {
            held["attributes"] = json!(self.attributes);
        }
        row["breaking"] = held;
        if let Some(entities) = self.entities {
            row["entities"] = json!(entities);
        }
        row
    }
}

fn location(x: f64, y: f64, z: f64) -> Value {
    json!({"x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

/// An aim: a block, a face and where on the face it lands.
pub(super) fn aim_at(block: (i64, i64, i64), face: &str, at: (f64, f64, f64)) -> Value {
    json!({"x": block.0, "y": block.1, "z": block.2, "face": face, "location": location(at.0, at.1, at.2)})
}

pub(super) fn place(place: &Value, block: &str) -> Value {
    put(place["x"].as_i64().unwrap_or(0), place["y"].as_i64().unwrap_or(0), place["z"].as_i64().unwrap_or(0), block)
}

pub fn cases() -> Vec<Value> {
    let local = |id: &str, rows: Vec<Value>| case(id, "movement", "local_player_ticks", json!({"rows": rows}));
    vec![
        local(
            "movement/using",
            [
                placing::rows(),
                consuming::rows(),
                throwing::rows(),
                bottling::rows(),
                utility::rows(),
                opening::rows(),
                item_on::rows(),
            ]
            .concat(),
        ),
        local("movement/breaking", digging::rows()),
        local("movement/entity-aim", aiming::rows()),
        local("movement/gliding", gliding::rows()),
        local("movement/pistons", pistons::rows()),
        local("movement/effects", effects::rows()),
    ]
}
