//! The `profile` of a head block's tag (`ResolvableProfile`): the id and the properties that
//! carry the skin.

use photon_core::binary::Uuid;
use photon_core::nbt::{Field, FieldValue, read_nbt};
use photon_net::skins::ProfileProperty;

use super::dynamic::{Dynamic, Resolvable};

/// A profile that names its skin itself (`ResolvableProfile.Static`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StaticProfile {
    pub id: Uuid,
    pub properties: Vec<ProfileProperty>,
}

/// `ResolvableProfile.create` of a partial profile: a name or an id alone, with no properties,
/// is looked up; any other is the profile it says (its id the given one, or the nil one).
pub(crate) fn resolvable(
    name: Option<String>,
    id: Option<Uuid>,
    properties: Vec<ProfileProperty>,
) -> Resolvable {
    match (name, id) {
        (Some(name), None) if properties.is_empty() => Resolvable::Dynamic(Dynamic::Name(name)),
        (None, Some(id)) if properties.is_empty() => Resolvable::Dynamic(Dynamic::Id(id)),
        (_, id) => Resolvable::Static(StaticProfile {
            id: id.unwrap_or(Uuid::new(0, 0)),
            properties,
        }),
    }
}

/// The profile of a head block entity's tag: a name alone, or the compound with a name, an id
/// and properties of which each may be missing.
pub(crate) fn resolvable_profile(tag: &[u8]) -> Option<Resolvable> {
    let fields = read_nbt(&mut &tag[..]).ok().flatten()?;
    let profile = match field(&fields, "profile")? {
        FieldValue::Str(name) => return Some(resolvable(Some(name.clone()), None, Vec::new())),
        FieldValue::Compound(profile) => profile,
        _ => return None,
    };
    let name = match field(profile, "name") {
        Some(FieldValue::Str(name)) => Some(name.clone()),
        _ => None,
    };
    let id = match field(profile, "id") {
        Some(FieldValue::IntArray(parts)) if parts.len() == 4 => Some(uuid(parts)),
        _ => None,
    };
    let properties = match field(profile, "properties") {
        Some(value) => properties(value)?,
        None => Vec::new(),
    };
    Some(resolvable(name, id, properties))
}

fn field<'a>(fields: &'a [Field], key: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|field| field.key == key)
        .map(|f| &f.value)
}

/// `UUIDUtil.uuidFromIntArray`.
fn uuid(parts: &[i32]) -> Uuid {
    let pair = |high: i32, low: i32| u64::from(high as u32) << 32 | u64::from(low as u32);
    Uuid::new(pair(parts[0], parts[1]), pair(parts[2], parts[3]))
}

/// `ExtraCodecs.PROPERTY_MAP`: a list of `{name, value, signature?}` or a map of names to lists
/// of values.
fn properties(value: &FieldValue) -> Option<Vec<ProfileProperty>> {
    let text = |value: &FieldValue| match value {
        FieldValue::Str(text) => Some(text.clone()),
        _ => None,
    };
    match value {
        FieldValue::List(entries) => entries
            .iter()
            .map(|entry| {
                let FieldValue::Compound(entry) = entry else {
                    return None;
                };
                Some(ProfileProperty {
                    name: text(field(entry, "name")?)?,
                    value: text(field(entry, "value")?)?,
                    signature: field(entry, "signature").and_then(text),
                })
            })
            .collect(),
        FieldValue::Compound(map) => {
            let mut found = Vec::new();
            for entry in map {
                let FieldValue::List(values) = &entry.value else {
                    return None;
                };
                for value in values {
                    found.push(ProfileProperty {
                        name: entry.key.clone(),
                        value: text(value)?,
                        signature: None,
                    });
                }
            }
            Some(found)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::nbt::write_nbt;

    fn static_of(tag: &[u8]) -> StaticProfile {
        match resolvable_profile(tag) {
            Some(Resolvable::Static(profile)) => profile,
            other => panic!("{other:?}"),
        }
    }

    fn tag(profile: FieldValue) -> Vec<u8> {
        let root = [Field {
            key: "profile".into(),
            value: profile,
        }];
        let mut out = Vec::new();
        write_nbt(Some(&root), &mut out);
        out
    }

    fn string(key: &str, value: &str) -> Field {
        Field {
            key: key.into(),
            value: FieldValue::Str(value.into()),
        }
    }

    #[test]
    fn a_list_of_properties_and_an_id_make_a_static_profile() {
        let property =
            FieldValue::Compound(vec![string("name", "textures"), string("value", "abc")]);
        let profile = FieldValue::Compound(vec![
            Field {
                key: "id".into(),
                value: FieldValue::IntArray(vec![1, 2, 3, -4]),
            },
            Field {
                key: "properties".into(),
                value: FieldValue::List(vec![property]),
            },
        ]);
        let found = static_of(&tag(profile));
        assert_eq!(found.id, Uuid::new(0x1_0000_0002, 0x3_ffff_fffc));
        assert_eq!(found.properties[0].name, "textures");
        assert_eq!(found.properties[0].signature, None);
    }

    #[test]
    fn a_map_of_properties_is_read_too() {
        let map = FieldValue::Compound(vec![Field {
            key: "textures".into(),
            value: FieldValue::List(vec![FieldValue::Str("abc".into())]),
        }]);
        let profile = FieldValue::Compound(vec![Field {
            key: "properties".into(),
            value: map,
        }]);
        let found = static_of(&tag(profile));
        assert_eq!(found.properties[0].value, "abc");
        assert_eq!(found.id, Uuid::new(0, 0));
    }

    #[test]
    fn a_name_alone_is_dynamic() {
        assert_eq!(
            resolvable_profile(&tag(FieldValue::Str("Steve".into()))),
            Some(Resolvable::Dynamic(Dynamic::Name("Steve".into())))
        );
    }
}
