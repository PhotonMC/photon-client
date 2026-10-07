use super::*;

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

/// `a * b`: `b` first, then `a`.
pub fn multiply(a: &Matrix4, b: &Matrix4) -> Matrix4 {
    std::array::from_fn(|row| {
        std::array::from_fn(|column| (0..4).map(|k| a[row][k] * b[k][column]).sum())
    })
}

/// A rotation by a quaternion `(x, y, z, w)`, as JOML's `rotate` turns.
pub(super) fn rotation([x, y, z, w]: [f32; 4]) -> Matrix4 {
    let (xx, yy, zz) = (x * x, y * y, z * z);
    [
        [
            1.0 - 2.0 * (yy + zz),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
            0.0,
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (xx + zz),
            2.0 * (y * z - x * w),
            0.0,
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (xx + yy),
            0.0,
        ],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// `Transformation.EXTENDED_CODEC`: the parts (translation, left rotation,
/// scale, right rotation) composed in that order, or a matrix of sixteen
/// numbers, column by column.
pub(super) fn parse_transformation(value: &JsonValue) -> Option<Matrix4> {
    let numbers = |value: &JsonValue| -> Option<Vec<f32>> {
        match value {
            JsonValue::Array(items) => items.iter().map(JsonValue::as_f32).collect(),
            _ => None,
        }
    };
    match value {
        JsonValue::Array(_) => {
            let values = numbers(value)?;
            if values.len() != 16 {
                return None;
            }
            // Written column by column.
            Some(std::array::from_fn(|row| {
                std::array::from_fn(|column| values[column * 4 + row])
            }))
        }
        JsonValue::Object(object) => {
            let get = |key: &str, len: usize| {
                let values = numbers(object.get_str(key)?)?;
                (values.len() == len).then_some(values)
            };
            let translation = get("translation", 3)?;
            let left = get("left_rotation", 4)?;
            let scale = get("scale", 3)?;
            let right = get("right_rotation", 4)?;
            let mut matrix = IDENTITY;
            for (row, value) in translation.iter().enumerate() {
                matrix[row][3] = *value;
            }
            let scaling: Matrix4 = std::array::from_fn(|row| {
                std::array::from_fn(|column| {
                    if row == column && row < 3 {
                        scale[row]
                    } else if row == column {
                        1.0
                    } else {
                        0.0
                    }
                })
            });
            let left = rotation([left[0], left[1], left[2], left[3]]);
            let right = rotation([right[0], right[1], right[2], right[3]]);
            Some(multiply(
                &multiply(&multiply(&matrix, &left), &scaling),
                &right,
            ))
        }
        _ => None,
    }
}

/// `DyeColor.getTextureDiffuseColor`, opaque, by dye name.
pub(super) fn dye_colour(name: &str) -> Option<u32> {
    Some(
        0xFF00_0000
            | match name {
                "white" => 16_383_998,
                "orange" => 16_351_261,
                "magenta" => 13_061_821,
                "light_blue" => 3_847_130,
                "yellow" => 16_701_501,
                "lime" => 8_439_583,
                "pink" => 15_961_002,
                "gray" => 4_673_362,
                "light_gray" => 10_329_495,
                "cyan" => 1_481_884,
                "purple" => 8_991_416,
                "blue" => 3_949_738,
                "brown" => 8_606_770,
                "green" => 6_192_150,
                "red" => 11_546_150,
                "black" => 1_908_001,
                _ => return None,
            },
    )
}

/// A special renderer's model, as `special` names it, when this client draws
/// it.
pub(super) fn parse_special(model: &JsonObject) -> Option<SpecialModel> {
    let kind = model.text("type")?;
    // A texture name under a folder of `textures`: `minecraft:shulker` under
    // `entity/shulker`.
    let named = |folder: &str| {
        let name = model.text("texture")?;
        Some(format!(
            "{folder}/{}",
            name.trim_start_matches("minecraft:")
        ))
    };
    Some(match kind.as_str() {
        "minecraft:chest" => SpecialModel::Chest {
            texture: named("entity/chest")?,
        },
        "minecraft:shulker_box" => SpecialModel::ShulkerBox {
            texture: named("entity/shulker")?,
        },
        "minecraft:head" => {
            let head = model.text("kind")?;
            let texture = match head.as_str() {
                "skeleton" => "entity/skeleton/skeleton",
                "wither_skeleton" => "entity/skeleton/wither_skeleton",
                "zombie" => "entity/zombie/zombie",
                "creeper" => "entity/creeper/creeper",
                "piglin" => "entity/piglin/piglin",
                "dragon" => "entity/enderdragon/dragon",
                _ => return None,
            };
            SpecialModel::Head {
                kind: head,
                texture: texture.to_owned(),
            }
        }
        "minecraft:player_head" => SpecialModel::Head {
            kind: "player".to_owned(),
            texture: "entity/player/slim/steve".to_owned(),
        },
        "minecraft:banner" => SpecialModel::Banner {
            color: dye_colour(&model.text("color")?)?,
        },
        "minecraft:shield" => SpecialModel::Shield,
        "minecraft:conduit" => SpecialModel::Conduit,
        "minecraft:decorated_pot" => SpecialModel::DecoratedPot,
        "minecraft:copper_golem_statue" => {
            if model.text("pose")? != "standing" {
                return None;
            }
            let path = model.text("texture")?;
            SpecialModel::CopperGolemStatue {
                texture: path
                    .trim_start_matches("minecraft:textures/")
                    .trim_end_matches(".png")
                    .to_owned(),
            }
        }
        _ => return None,
    })
}

pub(super) fn text_of(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(units) => Some(String::from_utf16_lossy(units)),
        _ => None,
    }
}

pub(super) fn parse_tint(object: &JsonObject) -> TintSource {
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
        "minecraft:dye" => TintSource::Component {
            source: ComponentTint::Dye,
            default: rgb("default"),
        },
        "minecraft:potion" => TintSource::Component {
            source: ComponentTint::Potion,
            default: object
                .get_str("default")
                .and_then(JsonValue::as_i64)
                .unwrap_or(-13_083_194) as u32,
        },
        "minecraft:firework" => TintSource::Component {
            source: ComponentTint::Firework,
            default: object
                .get_str("default")
                .and_then(JsonValue::as_i64)
                .unwrap_or(-7_697_782) as u32,
        },
        "minecraft:map_color" => TintSource::Component {
            source: ComponentTint::MapColor,
            default: rgb("default"),
        },
        _ => TintSource::Default(rgb("default")),
    }
}

/// Parse one `ItemModel.Unbaked`: its node, under its `transformation` when
/// it has one.
pub fn parse_item_model(object: &JsonObject) -> ItemModelDef {
    let node = parse_item_model_node(object);
    match object
        .get_str("transformation")
        .and_then(parse_transformation)
    {
        Some(matrix) => ItemModelDef::Transformed {
            matrix,
            model: Box::new(node),
        },
        None => node,
    }
}

pub(super) fn parse_item_model_node(object: &JsonObject) -> ItemModelDef {
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
            match (base, model.and_then(parse_special)) {
                (Some(base), Some(model)) => ItemModelDef::Special { base, model },
                _ => ItemModelDef::Unknown(format!(
                    "special {}",
                    model
                        .and_then(|model| model.text("type"))
                        .unwrap_or_default()
                )),
            }
        }
        "minecraft:empty" => ItemModelDef::Empty,
        other => ItemModelDef::Unknown(other.to_owned()),
    }
}

/// Parse a client item file (`items/<name>.json`) to its model tree.
pub fn parse_client_item(text: &str) -> Option<ClientItem> {
    let JsonValue::Object(object) = voxel_core::json::parse(text).ok()? else {
        return None;
    };
    Some(ClientItem {
        model: parse_item_model(object.object("model")?),
        hand_animation_on_swap: object.bool("hand_animation_on_swap").unwrap_or(true),
        swap_animation_scale: object.f32("swap_animation_scale").unwrap_or(1.0),
    })
}

/// The layers a default stack (no components, not in use) draws in a
/// display context. Properties read the default stack: conditions are
/// false, selections other than the display context and numeric
/// properties take their fallbacks.
pub fn layers_for(def: &ItemModelDef, context: DisplayContext, out: &mut Vec<ItemLayerRef>) {
    layers_under(def, context, &IDENTITY, out);
}

pub(super) fn layers_under(
    def: &ItemModelDef,
    context: DisplayContext,
    local: &Matrix4,
    out: &mut Vec<ItemLayerRef>,
) {
    match def {
        ItemModelDef::Transformed { matrix, model } => {
            layers_under(model, context, &multiply(local, matrix), out);
        }
        ItemModelDef::Model { model, tints } => out.push(ItemLayerRef {
            model: model.clone(),
            tints: tints.clone(),
            special: None,
            local: *local,
        }),
        ItemModelDef::Special { base, model } => out.push(ItemLayerRef {
            model: base.clone(),
            tints: Vec::new(),
            special: Some(model.clone()),
            local: *local,
        }),
        ItemModelDef::Composite(models) => {
            for model in models {
                layers_under(model, context, local, out);
            }
        }
        ItemModelDef::Condition { on_false, .. } => layers_under(on_false, context, local, out),
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
                (Some(model), _) => layers_under(model, context, local, out),
                (None, Some(fallback)) => layers_under(fallback, context, local, out),
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
                (Some(model), _) => layers_under(model, context, local, out),
                (None, Some(fallback)) => layers_under(fallback, context, local, out),
                (None, None) => {}
            }
        }
        ItemModelDef::Empty | ItemModelDef::Unknown(_) => {}
    }
}
