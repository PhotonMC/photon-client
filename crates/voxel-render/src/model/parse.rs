use super::*;

pub(super) fn reject(reason: &'static str) -> ModelParseError {
    ModelParseError { reason }
}

/// Parse a model file the way the reference's configured reader does.
pub fn parse_model(text: &str) -> Result<CuboidModel, ModelParseError> {
    let value = voxel_core::json::parse(text).map_err(|_| reject("malformed json"))?;
    let JsonValue::Object(object) = value else {
        return Err(reject("model must be an object"));
    };

    let parent = match object.text("parent") {
        Some(name) if !name.is_empty() => Some(
            voxel_core::identifier::Identifier::parse(&name)
                .map_err(|_| reject("bad parent name"))?,
        ),
        Some(_) => None,
        None if object.has("parent") => return Err(reject("parent must be a string")),
        None => None,
    };

    let ambient_occlusion = if object.has("ambientocclusion") {
        Some(
            object
                .bool("ambientocclusion")
                .ok_or_else(|| reject("ambientocclusion must be a boolean"))?,
        )
    } else {
        None
    };

    let gui_light = if object.has("gui_light") {
        match object
            .text("gui_light")
            .ok_or_else(|| reject("gui_light must be a string"))?
            .as_str()
        {
            "front" => Some(GuiLight::Front),
            "side" => Some(GuiLight::Side),
            // The reference's lookup returns null for an unknown name rather
            // than failing, and the field then stays unset.
            _ => None,
        }
    } else {
        None
    };

    let texture_slots = if object.has("textures") {
        let textures = object
            .object("textures")
            .ok_or_else(|| reject("textures must be an object"))?;
        parse_texture_map(textures)?
    } else {
        TextureSlotsData::default()
    };

    let elements = if object.has("elements") {
        let items = object
            .array("elements")
            .ok_or_else(|| reject("elements must be an array"))?;
        let mut parsed = Vec::with_capacity(items.len());
        for item in items {
            parsed.push(parse_element(item)?);
        }
        Some(parsed)
    } else {
        None
    };

    let display = if object.has("display") {
        let display = object
            .object("display")
            .ok_or_else(|| reject("display must be an object"))?;
        Some(parse_display(display)?)
    } else {
        None
    };

    Ok(CuboidModel {
        parent,
        ambient_occlusion,
        gui_light,
        texture_slots,
        elements,
        display,
    })
}

/// `ItemTransforms.Deserializer`: a left hand left out takes the right
/// hand's transform.
pub(super) fn parse_display(display: &JsonObject) -> Result<ItemTransforms, ModelParseError> {
    let mut transforms = ItemTransforms::default();
    for context in DisplayContext::ALL {
        if display.has(context.name()) {
            let object = display
                .object(context.name())
                .ok_or_else(|| reject("transform must be an object"))?;
            transforms.transforms[context as usize] = Some(parse_transform(object)?);
        }
    }
    for (left, right) in [
        (
            DisplayContext::ThirdPersonLeftHand,
            DisplayContext::ThirdPersonRightHand,
        ),
        (
            DisplayContext::FirstPersonLeftHand,
            DisplayContext::FirstPersonRightHand,
        ),
    ] {
        if transforms.transforms[left as usize].is_none() {
            transforms.transforms[left as usize] = transforms.transforms[right as usize];
        }
    }
    Ok(transforms)
}

/// `ItemTransform.Deserializer`: the translation in sixteenths, then both
/// translation and scale kept within their limits.
pub(super) fn parse_transform(object: &JsonObject) -> Result<ItemTransform, ModelParseError> {
    let vector = |key: &'static str, default: [f32; 3]| {
        if object.has(key) {
            parse_vector(object, key)
        } else {
            Ok(default)
        }
    };
    let rotation = vector("rotation", [0.0; 3])?;
    let translation =
        vector("translation", [0.0; 3])?.map(|value| (value * 0.0625).clamp(-5.0, 5.0));
    let scale = vector("scale", [1.0; 3])?.map(|value| value.clamp(-4.0, 4.0));
    Ok(ItemTransform {
        rotation,
        translation,
        scale,
    })
}

/// Java `TextureSlots.parseTextureMap`.
pub(super) fn parse_texture_map(
    textures: &JsonObject,
) -> Result<TextureSlotsData, ModelParseError> {
    let mut entries = Vec::with_capacity(textures.len());
    for (key, value) in textures.entries() {
        let slot = String::from_utf16(key).map_err(|_| reject("texture slot name"))?;
        let contents = match value {
            JsonValue::String(units) => {
                let text = String::from_utf16(units).map_err(|_| reject("texture value"))?;
                SlotContents::parse_string(&text).map_err(|_| reject("texture value"))?
            }
            JsonValue::Object(material) => SlotContents::Value(Material {
                sprite: voxel_core::identifier::Identifier::parse(
                    &material
                        .text("sprite")
                        .ok_or_else(|| reject("material sprite"))?,
                )
                .map_err(|_| reject("material sprite"))?,
                force_translucent: if material.has("force_translucent") {
                    material
                        .bool("force_translucent")
                        .ok_or_else(|| reject("force_translucent"))?
                } else {
                    false
                },
            }),
            _ => return Err(reject("texture value must be a string or object")),
        };
        entries.push((slot, contents));
    }
    Ok(TextureSlotsData::from_entries(entries))
}

pub(super) fn parse_element(value: &JsonValue) -> Result<ModelElement, ModelParseError> {
    let JsonValue::Object(object) = value else {
        return Err(reject("element must be an object"));
    };
    let from = parse_position(object, "from")?;
    let to = parse_position(object, "to")?;
    let rotation = parse_rotation(object)?;
    let faces = parse_faces(object)?;

    // The reference checks the type before converting, so a number here is a
    // rejection rather than a coercion.
    if object.has("shade") && object.bool("shade").is_none() {
        return Err(reject("shade must be a boolean"));
    }
    let shade = object.bool("shade").unwrap_or(true);

    let mut light_emission = 0;
    if object.has("light_emission") {
        let Some(value) = object.i32("light_emission") else {
            return Err(reject("light_emission must be an integer"));
        };
        if !(0..=15).contains(&value) {
            return Err(reject("light_emission out of range"));
        }
        light_emission = value;
    }

    Ok(ModelElement {
        from,
        to,
        rotation,
        shade,
        light_emission,
        faces,
    })
}

/// A corner, rejected if it reaches past what an element may occupy.
pub(super) fn parse_position(
    object: &JsonObject,
    key: &'static str,
) -> Result<[f32; 3], ModelParseError> {
    let vector = parse_vector(object, key)?;
    if vector
        .iter()
        .any(|axis| *axis < MIN_EXTENT || *axis > MAX_EXTENT)
    {
        return Err(reject("position exceeds the allowed boundaries"));
    }
    Ok(vector)
}

pub(super) fn parse_vector(
    object: &JsonObject,
    key: &'static str,
) -> Result<[f32; 3], ModelParseError> {
    let array = object
        .array(key)
        .ok_or_else(|| reject("expected an array"))?;
    if array.len() != 3 {
        return Err(reject("expected three values"));
    }
    let mut out = [0.0_f32; 3];
    for (slot, item) in out.iter_mut().zip(array) {
        *slot = item.as_f32().ok_or_else(|| reject("expected a number"))?;
    }
    Ok(out)
}

pub(super) fn parse_rotation(
    object: &JsonObject,
) -> Result<Option<ElementRotation>, ModelParseError> {
    if !object.has("rotation") {
        return Ok(None);
    }
    let block = object
        .object("rotation")
        .ok_or_else(|| reject("rotation must be an object"))?;
    let mut origin = parse_vector(block, "origin")?;
    // Written in sixteenths, used in block units.
    for axis in origin.iter_mut() {
        *axis *= 0.0625;
    }
    let rescale = if block.has("rescale") {
        block.bool("rescale").ok_or_else(|| reject("rescale"))?
    } else {
        false
    };

    let transform = if !block.has("axis") && !block.has("angle") {
        if !block.has("x") && !block.has("y") && !block.has("z") {
            return Err(reject("missing rotation value"));
        }
        let read = |key: &str| -> Result<f32, ModelParseError> {
            if !block.has(key) {
                return Ok(0.0);
            }
            block.f32(key).ok_or_else(|| reject("rotation angle"))
        };
        euler_element_transform(read("x")?, read("y")?, read("z")?, rescale)
    } else {
        let axis = match block
            .text("axis")
            .ok_or_else(|| reject("axis must be a string"))?
            .to_lowercase()
            .as_str()
        {
            "x" => RotationAxis::X,
            "y" => RotationAxis::Y,
            "z" => RotationAxis::Z,
            _ => return Err(reject("invalid rotation axis")),
        };
        let angle = block.f32("angle").ok_or_else(|| reject("angle"))?;
        element_transform(axis, angle, rescale)
    };

    Ok(Some(ElementRotation {
        origin,
        rescale,
        transform,
    }))
}

pub(super) fn parse_faces(
    object: &JsonObject,
) -> Result<Vec<(Direction, ModelFace)>, ModelParseError> {
    let faces = object
        .object("faces")
        .ok_or_else(|| reject("faces must be an object"))?;
    let mut out = Vec::with_capacity(faces.len());
    for (key, value) in faces.entries() {
        let name = String::from_utf16(key).map_err(|_| reject("face name"))?;
        let direction = direction_by_name(&name).ok_or_else(|| reject("unknown facing"))?;
        let JsonValue::Object(face) = value else {
            return Err(reject("face must be an object"));
        };
        out.push((direction, parse_face(face)?));
    }
    if out.is_empty() {
        return Err(reject("expected between 1 and 6 unique faces"));
    }
    Ok(out)
}

pub(super) fn parse_face(face: &JsonObject) -> Result<ModelFace, ModelParseError> {
    // An absent or unrecognised cullface is simply no cullface.
    let cull_for_direction = match face.text("cullface") {
        Some(name) => direction_by_name(&name),
        None if face.has("cullface") => return Err(reject("cullface must be a string")),
        None => None,
    };
    let tint_index = if face.has("tintindex") {
        face.i32("tintindex")
            .ok_or_else(|| reject("tintindex must be an integer"))?
    } else {
        -1
    };
    let texture = face
        .text("texture")
        .ok_or_else(|| reject("face needs a texture"))?;
    let uvs = if face.has("uv") {
        let array = face
            .array("uv")
            .ok_or_else(|| reject("uv must be an array"))?;
        if array.len() != 4 {
            return Err(reject("expected four uv values"));
        }
        let mut values = [0.0_f32; 4];
        for (slot, item) in values.iter_mut().zip(array) {
            *slot = item.as_f32().ok_or_else(|| reject("uv value"))?;
        }
        Some(FaceUvs {
            min_u: values[0],
            min_v: values[1],
            max_u: values[2],
            max_v: values[3],
        })
    } else {
        None
    };
    let degrees = if face.has("rotation") {
        face.i32("rotation")
            .ok_or_else(|| reject("rotation must be an integer"))?
    } else {
        0
    };
    let rotation =
        Quadrant::from_degrees(degrees).ok_or_else(|| reject("invalid face rotation"))?;

    Ok(ModelFace {
        cull_for_direction,
        tint_index,
        texture,
        uvs,
        rotation,
    })
}

/// Java `Direction.byName`, which is case sensitive and has no default.
pub(super) fn direction_by_name(name: &str) -> Option<Direction> {
    Some(match name {
        "down" => Direction::Down,
        "up" => Direction::Up,
        "north" => Direction::North,
        "south" => Direction::South,
        "west" => Direction::West,
        "east" => Direction::East,
        _ => return None,
    })
}
