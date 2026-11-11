//! Reading a block transformer entry from the data pack's JSON or from the NBT a server sends.
//! The NBT is turned into the JSON shape first, so one reader serves both.

use super::{BlockSet, Particle, Predicate, Provider, Transform};
use photon_core::json::{JsonObject, JsonValue};
use photon_core::nbt::FieldValue;

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn number(text: String) -> JsonValue {
    JsonValue::Number(text)
}

/// A number or flag of NBT as the JSON a data pack would have: a byte of 0 or 1 is a flag.
fn scalar(value: &FieldValue) -> Option<JsonValue> {
    Some(match value {
        FieldValue::Byte(flag @ (0 | 1)) => JsonValue::Bool(*flag == 1),
        FieldValue::Byte(value) => number(value.to_string()),
        FieldValue::Short(value) => number(value.to_string()),
        FieldValue::Int(value) => number(value.to_string()),
        FieldValue::Long(value) => number(value.to_string()),
        FieldValue::Float(value) => number(value.to_string()),
        FieldValue::Double(value) => number(value.to_string()),
        _ => return None,
    })
}

fn numbers<T: ToString>(items: &[T]) -> JsonValue {
    JsonValue::Array(items.iter().map(|item| number(item.to_string())).collect())
}

fn compound(fields: &[photon_core::nbt::Field]) -> JsonValue {
    let mut object = JsonObject::default();
    for field in fields {
        object.insert(units(&field.key), from_field(&field.value));
    }
    JsonValue::Object(object)
}

/// An NBT value as the JSON a data pack would have.
fn from_field(value: &FieldValue) -> JsonValue {
    if let Some(scalar) = scalar(value) {
        return scalar;
    }
    match value {
        FieldValue::Str(text) => JsonValue::String(units(text)),
        FieldValue::List(items) => JsonValue::Array(items.iter().map(from_field).collect()),
        FieldValue::IntArray(items) => numbers(items),
        FieldValue::ByteArray(items) => numbers(items),
        FieldValue::LongArray(items) => numbers(items),
        FieldValue::Compound(fields) => compound(fields),
        // The numbers were taken above.
        _ => JsonValue::Null,
    }
}

fn text(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(units) => String::from_utf16(units).ok(),
        _ => None,
    }
}

/// A type or entry name without its `minecraft:` namespace; another namespace is kept whole, so
/// it matches none of the names this reader knows.
fn bare(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

fn offset(object: &JsonObject) -> Option<super::Pos> {
    let Some(value) = object.get_str("offset") else {
        return Some((0, 0, 0));
    };
    let JsonValue::Array(items) = value else {
        return None;
    };
    match items.as_slice() {
        [x, y, z] => Some((x.as_i32()?, y.as_i32()?, z.as_i32()?)),
        _ => None,
    }
}

fn block_set(value: &JsonValue) -> Option<BlockSet> {
    match value {
        JsonValue::String(_) => {
            let name = text(value)?;
            Some(match name.strip_prefix('#') {
                Some(tag) => BlockSet::Tag(tag.to_owned()),
                None => BlockSet::Names(vec![name]),
            })
        }
        JsonValue::Array(items) => items
            .iter()
            .map(text)
            .collect::<Option<Vec<_>>>()
            .map(BlockSet::Names),
        _ => None,
    }
}

fn predicates(object: &JsonObject) -> Option<Vec<Predicate>> {
    object.array("predicates")?.iter().map(predicate).collect()
}

/// `BlockPredicate.CODEC`; a kind the client does not decide is [`Predicate::Unsupported`].
fn predicate(value: &JsonValue) -> Option<Predicate> {
    let JsonValue::Object(object) = value else {
        return None;
    };
    Some(match bare(&object.text("type")?) {
        "true" => Predicate::True,
        "all_of" => Predicate::AllOf(predicates(object)?),
        "any_of" => Predicate::AnyOf(predicates(object)?),
        "not" => Predicate::Not(Box::new(predicate(object.get_str("predicate")?)?)),
        "matching_blocks" => Predicate::Blocks {
            offset: offset(object)?,
            blocks: block_set(object.get_str("blocks")?)?,
        },
        "matching_block_tag" => Predicate::Tag {
            offset: offset(object)?,
            tag: object.text("tag")?,
        },
        _ => Predicate::Unsupported,
    })
}

/// The state a provider written as `{id, properties}` stands for.
fn state_provider(object: &JsonObject) -> Option<Provider> {
    let mut properties = Vec::new();
    if let Some(JsonValue::Object(named)) = object.get_str("properties") {
        for (key, value) in named.entries() {
            properties.push((String::from_utf16(key).ok()?, text(value)?));
        }
    }
    let state = photon_world::block::named_state::state_named(&object.text("id")?, &properties)?;
    Some(Provider::State(state))
}

/// `BlockStateProvider.CODEC`: a state written out, or a provider with a `type`. A name of an
/// entry of the `block_state_provider` registry is [`Provider::Unsupported`].
fn provider(value: &JsonValue) -> Option<Provider> {
    let JsonValue::Object(object) = value else {
        return matches!(value, JsonValue::String(_)).then_some(Provider::Unsupported);
    };
    let Some(kind) = object.text("type") else {
        return state_provider(object);
    };
    Some(match bare(&kind) {
        "copy_properties" => {
            Provider::CopyProperties(Box::new(provider(object.get_str("source")?)?))
        }
        "rule_based" => {
            let mut rules = Vec::new();
            for rule in object.array("rules")? {
                let JsonValue::Object(rule) = rule else {
                    return None;
                };
                rules.push((
                    predicate(rule.get_str("if_true")?)?,
                    provider(rule.get_str("then")?)?,
                ));
            }
            let fallback = match object.get_str("fallback") {
                Some(fallback) => Some(Box::new(provider(fallback)?)),
                None => None,
            };
            Provider::Rules { fallback, rules }
        }
        _ => Provider::Unsupported,
    })
}

fn direction(name: &str) -> Option<u8> {
    ["down", "up", "north", "south", "west", "east"]
        .iter()
        .position(|candidate| *candidate == name)
        .map(|place| place as u8)
}

fn particle(name: &str) -> Option<Particle> {
    Some(match name {
        "none" => Particle::None,
        "scrape" => Particle::Scrape,
        "wax_on" => Particle::WaxOn,
        "wax_off" => Particle::WaxOff,
        _ => return None,
    })
}

/// The sound event the entry names (by registry name, or as a written-out event), if the client
/// has it.
fn sound(value: Option<&JsonValue>) -> Option<Option<&'static str>> {
    let name = match value {
        None => return Some(None),
        Some(JsonValue::Object(event)) => event.text("sound_id")?,
        Some(other) => text(other)?,
    };
    Some(photon_data::sound_event::names().find(|candidate| *candidate == name))
}

/// A flag with its default, which must be a flag when it is there.
fn flag(object: &JsonObject, key: &str, default: bool) -> Option<bool> {
    match object.get_str(key) {
        None => Some(default),
        Some(JsonValue::Bool(value)) => Some(*value),
        Some(_) => None,
    }
}

fn transform(value: &JsonValue) -> Option<Transform> {
    let JsonValue::Object(object) = value else {
        return None;
    };
    let mut disallowed_faces = Vec::new();
    if let Some(faces) = object.get_str("disallowed_faces") {
        let JsonValue::Array(faces) = faces else {
            return None;
        };
        for face in faces {
            disallowed_faces.push(direction(&text(face)?)?);
        }
    }
    let particle = match object.get_str("particle") {
        Some(name) => particle(&text(name)?)?,
        None => Particle::None,
    };
    let copper_chest = match object.get_str("transform_type") {
        Some(kind) => match text(kind)?.as_str() {
            "copper_chest" => true,
            "single_block" => false,
            _ => return None,
        },
        None => false,
    };
    Some(Transform {
        provider: provider(object.get_str("block_state_provider")?)?,
        sound: sound(object.get_str("sound"))?,
        particle,
        disallowed_faces,
        update_from_neighbors: flag(object, "update_from_neighbors", true)?,
        copper_chest,
        consume_on_use: flag(object, "consume_on_use", true)?,
    })
}

/// A transformer entry as a data pack writes it (`BlockTransformer.DIRECT_CODEC`): one to two
/// hundred transforms. `None` for one that does not read.
pub fn from_json(value: &JsonValue) -> Option<Vec<Transform>> {
    let JsonValue::Array(items) = value else {
        return None;
    };
    if !(1..=200).contains(&items.len()) {
        return None;
    }
    items.iter().map(transform).collect()
}

/// A transformer entry as a server sends it.
pub fn from_nbt(value: &FieldValue) -> Option<Vec<Transform>> {
    from_json(&from_field(value))
}
