//! Item models: the flat item extruded into a slab of pixels, as the
//! reference's item model generator bakes `builtin/generated`.

use std::collections::BTreeSet;

/// Which way a side face of an extruded item faces in the texture: up and
/// down along the rows, left and right along the columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SideDirection {
    Up,
    Down,
    Left,
    Right,
}

impl SideDirection {
    /// The reference's name, for reports.
    pub fn name(self) -> &'static str {
        match self {
            SideDirection::Up => "UP",
            SideDirection::Down => "DOWN",
            SideDirection::Left => "LEFT",
            SideDirection::Right => "RIGHT",
        }
    }

    /// The step, in texture pixels, the side looks towards (`Direction`'s
    /// step: up is +y, down is -y, left is east +x, right is west -x).
    fn step(self) -> (i32, i32) {
        match self {
            SideDirection::Up => (0, 1),
            SideDirection::Down => (0, -1),
            SideDirection::Left => (1, 0),
            SideDirection::Right => (-1, 0),
        }
    }

    /// The model direction the face points.
    pub fn direction(self) -> crate::direction::Direction {
        use crate::direction::Direction;
        match self {
            SideDirection::Up => Direction::Up,
            SideDirection::Down => Direction::Down,
            SideDirection::Left => Direction::East,
            SideDirection::Right => Direction::West,
        }
    }
}

/// `ItemModelGenerator.getSideFaces`: for every opaque pixel, a face on
/// each side whose neighbour (against the side's step) is transparent or
/// outside the texture. `alpha` is the texture's alpha, row by row.
pub fn side_faces(alpha: &[u8], width: i32, height: i32) -> BTreeSet<(SideDirection, i32, i32)> {
    let transparent = |x: i32, y: i32| {
        if x < 0 || y < 0 || x >= width || y >= height {
            return true;
        }
        alpha[(y * width + x) as usize] == 0
    };
    let mut faces = BTreeSet::new();
    for y in 0..height {
        for x in 0..width {
            if transparent(x, y) {
                continue;
            }
            for facing in [
                SideDirection::Up,
                SideDirection::Down,
                SideDirection::Left,
                SideDirection::Right,
            ] {
                let (dx, dy) = facing.step();
                if transparent(x - dx, y - dy) {
                    faces.insert((facing, x, y));
                }
            }
        }
    }
    faces
}

use crate::identifier::Identifier;
use crate::json::{JsonObject, JsonValue};
use crate::render::model::DisplayContext;

/// A tint source a model layer names (`ItemTintSources`), for the
/// default stack.
#[derive(Debug, Clone, PartialEq)]
pub enum TintSource {
    /// A colour of its own (RGB; made opaque).
    Constant(u32),
    /// The grass colour map at a climate.
    Grass { temperature: f32, downfall: f32 },
    /// A source that reads a component the stack may carry; the default
    /// stack takes its default colour (RGB; made opaque).
    Default(u32),
}

/// A client item's model tree (`ItemModel.Unbaked`), as `items/*.json`
/// writes it.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemModelDef {
    Model {
        model: Identifier,
        tints: Vec<TintSource>,
    },
    Composite(Vec<ItemModelDef>),
    Condition {
        property: String,
        on_true: Box<ItemModelDef>,
        on_false: Box<ItemModelDef>,
    },
    Select {
        property: String,
        cases: Vec<(Vec<String>, ItemModelDef)>,
        fallback: Option<Box<ItemModelDef>>,
    },
    RangeDispatch {
        property: String,
        entries: Vec<(f32, ItemModelDef)>,
        fallback: Option<Box<ItemModelDef>>,
    },
    /// A special renderer over a base model (chests, banners, heads…): the
    /// base model gives the display transforms and the light. Only chests
    /// are drawn yet.
    Special {
        base: Identifier,
        model: SpecialModel,
    },
    Empty,
    /// A kind this client does not know.
    Unknown(String),
}

/// The special renderers this client draws.
#[derive(Debug, Clone, PartialEq)]
pub enum SpecialModel {
    /// `ChestSpecialRenderer`: the chest model shut, with the texture under
    /// `entity/chest`.
    Chest { texture: Identifier },
}

fn text_of(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(units) => Some(String::from_utf16_lossy(units)),
        _ => None,
    }
}

fn parse_tint(object: &JsonObject) -> TintSource {
    let kind = object.text("type").unwrap_or_default();
    let rgb = |key: &str| {
        object
            .get_str(key)
            .and_then(JsonValue::as_i64)
            .unwrap_or(-1) as u32
    };
    match kind.as_str() {
        "minecraft:constant" => TintSource::Constant(rgb("value")),
        "minecraft:grass" => TintSource::Grass {
            temperature: object.f32("temperature").unwrap_or(0.5),
            downfall: object.f32("downfall").unwrap_or(1.0),
        },
        _ => TintSource::Default(rgb("default")),
    }
}

/// Parse one `ItemModel.Unbaked`.
pub fn parse_item_model(object: &JsonObject) -> ItemModelDef {
    let kind = object.text("type").unwrap_or_default();
    let child = |key: &str| object.object(key).map(parse_item_model);
    match kind.as_str() {
        "minecraft:model" => ItemModelDef::Model {
            model: object
                .text("model")
                .and_then(|name| Identifier::parse(&name).ok())
                .unwrap_or_else(|| Identifier::parse("minecraft:missing").expect("fixed")),
            tints: object
                .array("tints")
                .unwrap_or(&[])
                .iter()
                .filter_map(|value| match value {
                    JsonValue::Object(tint) => Some(parse_tint(tint)),
                    _ => None,
                })
                .collect(),
        },
        "minecraft:composite" => ItemModelDef::Composite(
            object
                .array("models")
                .unwrap_or(&[])
                .iter()
                .filter_map(|value| match value {
                    JsonValue::Object(model) => Some(parse_item_model(model)),
                    _ => None,
                })
                .collect(),
        ),
        "minecraft:condition" => ItemModelDef::Condition {
            property: object.text("property").unwrap_or_default(),
            on_true: Box::new(child("on_true").unwrap_or(ItemModelDef::Empty)),
            on_false: Box::new(child("on_false").unwrap_or(ItemModelDef::Empty)),
        },
        "minecraft:select" => ItemModelDef::Select {
            property: object.text("property").unwrap_or_default(),
            cases: object
                .array("cases")
                .unwrap_or(&[])
                .iter()
                .filter_map(|value| {
                    let JsonValue::Object(case) = value else {
                        return None;
                    };
                    let when = match case.get_str("when")? {
                        JsonValue::Array(values) => values.iter().filter_map(text_of).collect(),
                        other => vec![text_of(other)?],
                    };
                    Some((when, parse_item_model(case.object("model")?)))
                })
                .collect(),
            fallback: child("fallback").map(Box::new),
        },
        "minecraft:range_dispatch" => ItemModelDef::RangeDispatch {
            property: object.text("property").unwrap_or_default(),
            entries: object
                .array("entries")
                .unwrap_or(&[])
                .iter()
                .filter_map(|value| {
                    let JsonValue::Object(entry) = value else {
                        return None;
                    };
                    Some((
                        entry.f32("threshold")?,
                        parse_item_model(entry.object("model")?),
                    ))
                })
                .collect(),
            fallback: child("fallback").map(Box::new),
        },
        "minecraft:special" => {
            let base = object
                .text("base")
                .and_then(|name| Identifier::parse(&name).ok());
            let model = object.object("model");
            let kind = model
                .and_then(|model| model.text("type"))
                .unwrap_or_default();
            let special = match (kind.as_str(), model) {
                ("minecraft:chest", Some(model)) => Identifier::parse(&format!(
                    "minecraft:entity/chest/{}",
                    model
                        .text("texture")
                        .map(|name| name.trim_start_matches("minecraft:").to_owned())
                        .unwrap_or_default()
                ))
                .ok()
                .map(|texture| SpecialModel::Chest { texture }),
                _ => None,
            };
            match (base, special) {
                (Some(base), Some(model)) => ItemModelDef::Special { base, model },
                _ => ItemModelDef::Unknown(format!("special {kind}")),
            }
        }
        "minecraft:empty" => ItemModelDef::Empty,
        other => ItemModelDef::Unknown(other.to_owned()),
    }
}

/// Parse a client item file (`items/<name>.json`) to its model tree.
pub fn parse_client_item(text: &str) -> Option<ClientItem> {
    let JsonValue::Object(object) = crate::json::parse(text).ok()? else {
        return None;
    };
    Some(ClientItem {
        model: parse_item_model(object.object("model")?),
        hand_animation_on_swap: object.bool("hand_animation_on_swap").unwrap_or(true),
        swap_animation_scale: object.f32("swap_animation_scale").unwrap_or(1.0),
    })
}

/// A client item (`ClientItem`): its model tree and the properties the
/// hand reads when the held stack changes.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientItem {
    pub model: ItemModelDef,
    /// Whether a new stack lowers the hand and raises it again.
    pub hand_animation_on_swap: bool,
    /// How far the hand drops while it swaps.
    pub swap_animation_scale: f32,
}

/// One model layer the item draws, with its tints.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemLayerRef {
    pub model: Identifier,
    pub tints: Vec<TintSource>,
    /// The special renderer drawn over `model`'s transforms, if any.
    pub special: Option<SpecialModel>,
}

/// The layers a default stack (no components, not in use) draws in a
/// display context. Properties read the default stack: conditions are
/// false, selections other than the display context and numeric
/// properties take their fallbacks.
pub fn layers_for(def: &ItemModelDef, context: DisplayContext, out: &mut Vec<ItemLayerRef>) {
    match def {
        ItemModelDef::Model { model, tints } => out.push(ItemLayerRef {
            model: model.clone(),
            tints: tints.clone(),
            special: None,
        }),
        ItemModelDef::Special { base, model } => out.push(ItemLayerRef {
            model: base.clone(),
            tints: Vec::new(),
            special: Some(model.clone()),
        }),
        ItemModelDef::Composite(models) => {
            for model in models {
                layers_for(model, context, out);
            }
        }
        ItemModelDef::Condition { on_false, .. } => layers_for(on_false, context, out),
        ItemModelDef::Select {
            property,
            cases,
            fallback,
        } => {
            let chosen = (property == "minecraft:display_context")
                .then(|| {
                    cases
                        .iter()
                        .find(|(when, _)| when.iter().any(|name| name == context.name()))
                        .map(|(_, model)| model)
                })
                .flatten();
            match (chosen, fallback) {
                (Some(model), _) => layers_for(model, context, out),
                (None, Some(fallback)) => layers_for(fallback, context, out),
                (None, None) => {}
            }
        }
        ItemModelDef::RangeDispatch {
            entries, fallback, ..
        } => {
            // The value is zero: the last entry whose threshold it reaches.
            let chosen = entries
                .iter()
                .filter(|(threshold, _)| *threshold <= 0.0)
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, model)| model);
            match (chosen, fallback) {
                (Some(model), _) => layers_for(model, context, out),
                (None, Some(fallback)) => layers_for(fallback, context, out),
                (None, None) => {}
            }
        }
        ItemModelDef::Empty | ItemModelDef::Unknown(_) => {}
    }
}

/// A tint's colour, ARGB. `grass` reads the grass colour map.
pub fn tint_colour(tint: &TintSource, grass: &dyn Fn(f64, f64) -> u32) -> u32 {
    match *tint {
        TintSource::Constant(rgb) | TintSource::Default(rgb) => 0xFF00_0000 | rgb,
        TintSource::Grass {
            temperature,
            downfall,
        } => 0xFF00_0000 | grass(f64::from(temperature), f64::from(downfall)),
    }
}

/// A texture's alpha, row by row, with its width and height.
pub type LayerAlpha = (Vec<u8>, i32, i32);

/// The elements the item model generator extrudes a layered flat item
/// into: each layer's front and back, then a thin side face along every
/// edge between an opaque and a transparent pixel. `layer_alpha` answers a
/// layer's texture alpha and size.
pub fn generated_elements(
    textures: &crate::render::model::TextureSlots,
    layer_alpha: &dyn Fn(&Identifier) -> Option<LayerAlpha>,
) -> Vec<crate::render::model::ModelElement> {
    use crate::direction::Direction;
    use crate::render::geometry::{FaceUvs, Quadrant};
    use crate::render::model::{ModelElement, ModelFace};
    let mut elements = Vec::new();
    let element =
        |from: [f32; 3], to: [f32; 3], facing: Direction, slot: &str, layer: i32, uvs: FaceUvs| {
            ModelElement {
                from,
                to,
                rotation: None,
                shade: true,
                light_emission: 0,
                faces: vec![(
                    facing,
                    ModelFace {
                        cull_for_direction: None,
                        tint_index: layer,
                        texture: format!("#{slot}"),
                        uvs: Some(uvs),
                        rotation: Quadrant::R0,
                    },
                )],
            }
        };
    for (layer, slot) in ["layer0", "layer1", "layer2", "layer3", "layer4"]
        .iter()
        .enumerate()
    {
        let Some(material) = textures.material(slot) else {
            break;
        };
        let layer = layer as i32;
        let (from, to) = ([0.0, 0.0, 7.5], [16.0, 16.0, 8.5]);
        elements.push(element(
            from,
            to,
            Direction::South,
            slot,
            layer,
            FaceUvs {
                min_u: 0.0,
                min_v: 0.0,
                max_u: 16.0,
                max_v: 16.0,
            },
        ));
        elements.push(element(
            from,
            to,
            Direction::North,
            slot,
            layer,
            FaceUvs {
                min_u: 16.0,
                min_v: 0.0,
                max_u: 0.0,
                max_v: 16.0,
            },
        ));
        let Some((alpha, width, height)) = layer_alpha(&material.sprite) else {
            continue;
        };
        let x_scale = 16.0 / width as f32;
        let y_scale = 16.0 / height as f32;
        for (facing, x, y) in side_faces(&alpha, width, height) {
            let (x, y) = (x as f32, y as f32);
            let u0 = x + 0.1;
            let u1 = x + 1.0 - 0.1;
            let (v0, v1) = if matches!(facing, SideDirection::Up | SideDirection::Down) {
                (y + 0.1, y + 1.0 - 0.1)
            } else {
                (y + 1.0 - 0.1, y + 0.1)
            };
            let (mut start_x, mut start_y, mut end_x, mut end_y) = (x, y, x, y);
            match facing {
                SideDirection::Up => end_x += 1.0,
                SideDirection::Down => {
                    end_x += 1.0;
                    start_y += 1.0;
                    end_y += 1.0;
                }
                SideDirection::Left => end_y += 1.0,
                SideDirection::Right => {
                    start_x += 1.0;
                    end_x += 1.0;
                    end_y += 1.0;
                }
            }
            start_x *= x_scale;
            end_x *= x_scale;
            start_y = 16.0 - start_y * y_scale;
            end_y = 16.0 - end_y * y_scale;
            let (from, to) = match facing {
                SideDirection::Up => ([start_x, start_y, 7.5], [end_x, start_y, 8.5]),
                SideDirection::Down => ([start_x, end_y, 7.5], [end_x, end_y, 8.5]),
                SideDirection::Left => ([start_x, start_y, 7.5], [start_x, end_y, 8.5]),
                SideDirection::Right => ([end_x, start_y, 7.5], [end_x, end_y, 8.5]),
            };
            elements.push(element(
                from,
                to,
                facing.direction(),
                slot,
                layer,
                FaceUvs {
                    min_u: u0 * x_scale,
                    min_v: v0 * y_scale,
                    max_u: u1 * x_scale,
                    max_v: v1 * y_scale,
                },
            ));
        }
    }
    elements
}

#[cfg(test)]
mod special_tests {
    use super::*;

    #[test]
    fn a_chest_draws_its_model_over_the_base_models_transforms() {
        let text = r#"{"model":{"type":"minecraft:select","property":"minecraft:local_time",
            "pattern":"MM-dd","cases":[],"fallback":{"type":"minecraft:special",
            "base":"minecraft:item/chest","model":{"type":"minecraft:chest",
            "texture":"minecraft:normal"}}}}"#;
        let item = parse_client_item(text).expect("a client item");
        let mut layers = Vec::new();
        layers_for(&item.model, DisplayContext::ALL[0], &mut layers);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].model.as_string(), "minecraft:item/chest");
        assert_eq!(
            layers[0].special,
            Some(SpecialModel::Chest {
                texture: Identifier::parse("minecraft:entity/chest/normal").expect("valid")
            })
        );
    }

    #[test]
    fn other_special_renderers_draw_nothing() {
        let text = r#"{"model":{"type":"minecraft:special","base":"minecraft:item/shield",
            "model":{"type":"minecraft:shield"}}}"#;
        let item = parse_client_item(text).expect("a client item");
        let mut layers = Vec::new();
        layers_for(&item.model, DisplayContext::ALL[0], &mut layers);
        assert!(layers.is_empty());
    }
}
