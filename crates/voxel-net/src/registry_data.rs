//! Configuration registry-data packet codec for the pinned Java protocol.

use voxel_core::binary;
use voxel_core::identifier::{self, Identifier};
use voxel_core::nbt::{self, FieldValue};
use voxel_core::wire;
use voxel_world::{
    CardinalLightType, DimensionBounds, DimensionEnvironment, DimensionEnvironmentInput,
    DimensionSkybox,
};

/// Registry-data decoder failures using the runner's error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    BufferUnderflow,
    DecoderError,
    EncoderError,
    TooBig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    EncoderError,
}

/// One registry entry and its optional serialized NBT data.
#[derive(Debug, Clone, PartialEq)]
pub struct PackedRegistryEntry {
    pub id: Identifier,
    pub data: Option<FieldValue>,
}

impl PackedRegistryEntry {
    pub fn new(id: Identifier, data: Option<FieldValue>) -> Self {
        Self { id, data }
    }

    /// Decode the bounds fields used by a serialized DimensionType entry.
    pub(crate) fn dimension_type_bounds(&self) -> Result<Option<DimensionBounds>, ReadError> {
        let Some(data) = &self.data else {
            return Ok(None);
        };
        let FieldValue::Compound(fields) = data else {
            return Err(ReadError::DecoderError);
        };
        let min_y = fields
            .iter()
            .find_map(|field| match field.key.as_str() {
                "min_y" => match &field.value {
                    FieldValue::Int(value) => Some(*value),
                    _ => None,
                },
                _ => None,
            })
            .ok_or(ReadError::DecoderError)?;
        let height = fields
            .iter()
            .find_map(|field| match field.key.as_str() {
                "height" => match &field.value {
                    FieldValue::Int(value) => Some(*value),
                    _ => None,
                },
                _ => None,
            })
            .ok_or(ReadError::DecoderError)?;
        DimensionBounds::from_dimension_type(min_y, height)
            .map(Some)
            .ok_or(ReadError::DecoderError)
    }

    /// Decode the client-consumed environment fields of a serialized DimensionType entry.
    pub(crate) fn dimension_type_environment(
        &self,
    ) -> Result<Option<DimensionEnvironment>, ReadError> {
        let Some(data) = &self.data else {
            return Ok(None);
        };
        let FieldValue::Compound(fields) = data else {
            return Err(ReadError::DecoderError);
        };
        let bounds = self
            .dimension_type_bounds()?
            .ok_or(ReadError::DecoderError)?;
        let has_fixed_time = match find_field(fields, "has_fixed_time") {
            None => false,
            Some(value) => decode_bool(value).ok_or(ReadError::DecoderError)?,
        };
        let has_skylight = find_field(fields, "has_skylight")
            .and_then(decode_bool)
            .ok_or(ReadError::DecoderError)?;
        let has_ceiling = find_field(fields, "has_ceiling")
            .and_then(decode_bool)
            .ok_or(ReadError::DecoderError)?;
        let has_ender_dragon_fight = find_field(fields, "has_ender_dragon_fight")
            .and_then(decode_bool)
            .ok_or(ReadError::DecoderError)?;
        let coordinate_scale = match find_field(fields, "coordinate_scale") {
            Some(FieldValue::Double(value)) => *value,
            _ => return Err(ReadError::DecoderError),
        };
        let logical_height = match find_field(fields, "logical_height") {
            Some(FieldValue::Int(value)) => *value,
            _ => return Err(ReadError::DecoderError),
        };
        let ambient_light = match find_field(fields, "ambient_light") {
            Some(FieldValue::Float(value)) => *value,
            _ => return Err(ReadError::DecoderError),
        };
        let skybox = match find_field(fields, "skybox") {
            None => DimensionSkybox::Overworld,
            Some(FieldValue::Str(value)) => {
                DimensionSkybox::from_wire(value).ok_or(ReadError::DecoderError)?
            }
            Some(_) => return Err(ReadError::DecoderError),
        };
        let cardinal_light_type = match find_field(fields, "cardinal_light") {
            None => CardinalLightType::Default,
            Some(FieldValue::Str(value)) => {
                CardinalLightType::from_wire(value).ok_or(ReadError::DecoderError)?
            }
            Some(_) => return Err(ReadError::DecoderError),
        };

        let piglins_zombify = piglins_zombify(fields)?;

        DimensionEnvironment::from_wire(
            DimensionEnvironmentInput {
                has_fixed_time,
                has_skylight,
                has_ceiling,
                has_ender_dragon_fight,
                coordinate_scale,
                logical_height,
                ambient_light,
                skybox,
                cardinal_light_type,
                piglins_zombify,
            },
            bounds.height,
        )
        .map(Some)
        .ok_or(ReadError::DecoderError)
    }
}

/// The dimension type's `gameplay/piglins_zombify` attribute: the default
/// (true) when the map leaves it out, else the value or the boolean modifier
/// applied to that default.
fn piglins_zombify(fields: &[voxel_core::nbt::Field]) -> Result<bool, ReadError> {
    const DEFAULT: bool = true;
    let Some(attributes) = find_field(fields, "attributes") else {
        return Ok(DEFAULT);
    };
    let FieldValue::Compound(attributes) = attributes else {
        return Err(ReadError::DecoderError);
    };
    match find_field(attributes, "minecraft:gameplay/piglins_zombify") {
        None => Ok(DEFAULT),
        Some(FieldValue::Compound(entry)) => {
            let argument = find_field(entry, "argument")
                .and_then(decode_bool)
                .ok_or(ReadError::DecoderError)?;
            let Some(FieldValue::Str(modifier)) = find_field(entry, "modifier") else {
                return Err(ReadError::DecoderError);
            };
            // `BooleanModifier` and the override every attribute has.
            match modifier.strip_prefix("minecraft:").unwrap_or(modifier) {
                "override" => Ok(argument),
                "and" => Ok(argument && DEFAULT),
                "nand" => Ok(!argument || !DEFAULT),
                "or" => Ok(argument || DEFAULT),
                "nor" => Ok(!argument && !DEFAULT),
                "xor" => Ok(argument ^ DEFAULT),
                "xnor" => Ok(argument == DEFAULT),
                _ => Err(ReadError::DecoderError),
            }
        }
        Some(value) => decode_bool(value).ok_or(ReadError::DecoderError),
    }
}

fn find_field<'a>(fields: &'a [voxel_core::nbt::Field], key: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|field| field.key == key)
        .map(|field| &field.value)
}

fn decode_bool(value: &FieldValue) -> Option<bool> {
    match value {
        FieldValue::Byte(0) => Some(false),
        FieldValue::Byte(1) => Some(true),
        _ => None,
    }
}

/// Clientbound configuration registry-data packet.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundRegistryDataPacket {
    pub registry: Identifier,
    pub entries: Vec<PackedRegistryEntry>,
}

impl ClientboundRegistryDataPacket {
    pub fn new(registry: Identifier, entries: Vec<PackedRegistryEntry>) -> Self {
        Self { registry, entries }
    }

    /// Encode the registry identifier, entry list and optional generic NBT tags.
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), WriteError> {
        identifier::write_identifier(&self.registry, output)
            .map_err(|_| WriteError::EncoderError)?;
        write_count(self.entries.len(), output)?;
        for entry in &self.entries {
            identifier::write_identifier(&entry.id, output)
                .map_err(|_| WriteError::EncoderError)?;
            output.push(u8::from(entry.data.is_some()));
            if let Some(data) = &entry.data {
                nbt::write_any_nbt(Some(data), output);
            }
        }
        Ok(())
    }

    /// Decode one registry-data packet and preserve list order.
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let registry = identifier::read_identifier(input).map_err(map_binary_error)?;
        let count = read_count(input)?;
        let mut entries = Vec::with_capacity(count.min(65_536));
        for _ in 0..count {
            let id = identifier::read_identifier(input).map_err(map_binary_error)?;
            let has_data = read_byte(input)? != 0;
            let data = if has_data {
                match nbt::read_any_nbt(input).map_err(map_nbt_error)? {
                    Some(value) => Some(value),
                    None => return Err(ReadError::DecoderError),
                }
            } else {
                None
            };
            entries.push(PackedRegistryEntry::new(id, data));
        }
        Ok(Self { registry, entries })
    }
}

fn write_count(count: usize, output: &mut Vec<u8>) -> Result<(), WriteError> {
    let count = i32::try_from(count).map_err(|_| WriteError::EncoderError)?;
    wire::write_varint(count, output);
    Ok(())
}

fn read_count(input: &mut &[u8]) -> Result<usize, ReadError> {
    let count = wire::read_varint(input).map_err(map_wire_error)?;
    if count < 0 {
        return Err(ReadError::DecoderError);
    }
    usize::try_from(count).map_err(|_| ReadError::DecoderError)
}

fn read_byte(input: &mut &[u8]) -> Result<u8, ReadError> {
    let (&head, rest) = input.split_first().ok_or(ReadError::BufferUnderflow)?;
    *input = rest;
    Ok(head)
}

fn map_wire_error(error: wire::ReadError) -> ReadError {
    match error {
        wire::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        wire::ReadError::TooBig => ReadError::TooBig,
    }
}

fn map_binary_error(error: binary::ReadError) -> ReadError {
    match error {
        binary::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        binary::ReadError::DecoderError => ReadError::DecoderError,
        binary::ReadError::TooBig => ReadError::TooBig,
    }
}

fn map_nbt_error(error: nbt::ReadError) -> ReadError {
    match error {
        nbt::ReadError::BufferUnderflow => ReadError::BufferUnderflow,
        nbt::ReadError::DecoderError => ReadError::DecoderError,
        nbt::ReadError::EncoderError => ReadError::EncoderError,
        nbt::ReadError::TooBig => ReadError::TooBig,
    }
}

#[cfg(test)]
mod tests {
    use super::{PackedRegistryEntry, ReadError};
    use voxel_core::identifier::Identifier;
    use voxel_core::nbt::{Field, FieldValue};
    use voxel_world::{CardinalLightType, DimensionBounds, DimensionSkybox};

    fn dimension_type_data(min_y: i32, height: i32) -> FieldValue {
        FieldValue::Compound(vec![
            Field {
                key: "height".to_owned(),
                value: FieldValue::Int(height),
            },
            Field {
                key: "min_y".to_owned(),
                value: FieldValue::Int(min_y),
            },
        ])
    }

    fn dimension_type_environment_data() -> FieldValue {
        FieldValue::Compound(vec![
            Field {
                key: "ambient_light".to_owned(),
                value: FieldValue::Float(0.35),
            },
            Field {
                key: "cardinal_light".to_owned(),
                value: FieldValue::Str("nether".to_owned()),
            },
            Field {
                key: "coordinate_scale".to_owned(),
                value: FieldValue::Double(2.5),
            },
            Field {
                key: "has_ceiling".to_owned(),
                value: FieldValue::Byte(1),
            },
            Field {
                key: "has_ender_dragon_fight".to_owned(),
                value: FieldValue::Byte(1),
            },
            Field {
                key: "has_fixed_time".to_owned(),
                value: FieldValue::Byte(1),
            },
            Field {
                key: "has_skylight".to_owned(),
                value: FieldValue::Byte(0),
            },
            Field {
                key: "height".to_owned(),
                value: FieldValue::Int(192),
            },
            Field {
                key: "logical_height".to_owned(),
                value: FieldValue::Int(128),
            },
            Field {
                key: "min_y".to_owned(),
                value: FieldValue::Int(-32),
            },
            Field {
                key: "skybox".to_owned(),
                value: FieldValue::Str("end".to_owned()),
            },
        ])
    }

    #[test]
    fn dimension_type_entry_extracts_valid_bounds_and_rejects_invalid_data() {
        let entry = PackedRegistryEntry::new(
            Identifier::parse("minecraft:custom").expect("identifier"),
            Some(dimension_type_data(-32, 128)),
        );
        assert_eq!(
            entry.dimension_type_bounds(),
            Ok(Some(DimensionBounds::new(-32, 128)))
        );

        let missing = PackedRegistryEntry::new(
            Identifier::parse("minecraft:missing").expect("identifier"),
            None,
        );
        assert_eq!(missing.dimension_type_bounds(), Ok(None));

        let invalid = PackedRegistryEntry::new(
            Identifier::parse("minecraft:invalid").expect("identifier"),
            Some(FieldValue::Compound(vec![Field {
                key: "min_y".to_owned(),
                value: FieldValue::Int(-32),
            }])),
        );
        assert_eq!(
            invalid.dimension_type_bounds(),
            Err(ReadError::DecoderError)
        );

        let misaligned = PackedRegistryEntry::new(
            Identifier::parse("minecraft:misaligned").expect("identifier"),
            Some(dimension_type_data(-31, 128)),
        );
        assert_eq!(
            misaligned.dimension_type_bounds(),
            Err(ReadError::DecoderError)
        );
    }

    #[test]
    fn dimension_type_entry_extracts_client_environment_fields() {
        let entry = PackedRegistryEntry::new(
            Identifier::parse("p08:test").expect("identifier"),
            Some(dimension_type_environment_data()),
        );
        let environment = entry
            .dimension_type_environment()
            .expect("valid environment")
            .expect("environment data");

        assert!(environment.has_fixed_time());
        assert!(!environment.has_skylight());
        assert!(environment.has_ceiling());
        assert!(environment.has_ender_dragon_fight());
        assert_eq!(environment.coordinate_scale().to_bits(), 2.5_f64.to_bits());
        assert_eq!(environment.logical_height(), 128);
        assert_eq!(environment.ambient_light().to_bits(), 0.35_f32.to_bits());
        assert_eq!(environment.skybox(), DimensionSkybox::End);
        assert_eq!(environment.cardinal_light_type(), CardinalLightType::Nether);

        let defaults = PackedRegistryEntry::new(
            Identifier::parse("p08:defaults").expect("identifier"),
            Some(FieldValue::Compound(vec![
                Field {
                    key: "has_skylight".to_owned(),
                    value: FieldValue::Byte(1),
                },
                Field {
                    key: "has_ceiling".to_owned(),
                    value: FieldValue::Byte(0),
                },
                Field {
                    key: "has_ender_dragon_fight".to_owned(),
                    value: FieldValue::Byte(0),
                },
                Field {
                    key: "coordinate_scale".to_owned(),
                    value: FieldValue::Double(1.0),
                },
                Field {
                    key: "height".to_owned(),
                    value: FieldValue::Int(192),
                },
                Field {
                    key: "logical_height".to_owned(),
                    value: FieldValue::Int(192),
                },
                Field {
                    key: "min_y".to_owned(),
                    value: FieldValue::Int(-32),
                },
                Field {
                    key: "ambient_light".to_owned(),
                    value: FieldValue::Float(0.0),
                },
            ])),
        );
        let defaults = defaults
            .dimension_type_environment()
            .expect("valid environment")
            .expect("environment data");
        assert!(!defaults.has_fixed_time());
        assert_eq!(defaults.skybox(), DimensionSkybox::Overworld);
        assert_eq!(defaults.cardinal_light_type(), CardinalLightType::Default);
    }

    #[test]
    fn dimension_type_entry_reads_the_piglins_zombify_attribute() {
        let with_attributes = |attributes: Option<Vec<Field>>| {
            let FieldValue::Compound(mut fields) = dimension_type_environment_data() else {
                unreachable!()
            };
            if let Some(attributes) = attributes {
                fields.push(Field {
                    key: "attributes".to_owned(),
                    value: FieldValue::Compound(attributes),
                });
            }
            PackedRegistryEntry::new(
                Identifier::parse("p08:test").expect("identifier"),
                Some(FieldValue::Compound(fields)),
            )
            .dimension_type_environment()
        };
        let attribute = |value: FieldValue| {
            Some(vec![Field {
                key: "minecraft:gameplay/piglins_zombify".to_owned(),
                value,
            }])
        };
        let modifier = |name: &str, argument: i8| {
            attribute(FieldValue::Compound(vec![
                Field {
                    key: "argument".to_owned(),
                    value: FieldValue::Byte(argument),
                },
                Field {
                    key: "modifier".to_owned(),
                    value: FieldValue::Str(name.to_owned()),
                },
            ]))
        };
        let zombify = |attributes| {
            with_attributes(attributes)
                .expect("valid environment")
                .expect("environment data")
                .piglins_zombify()
        };
        // The attribute's default while the map leaves it out.
        assert!(zombify(None));
        assert!(zombify(Some(Vec::new())));
        assert!(!zombify(attribute(FieldValue::Byte(0))));
        assert!(zombify(attribute(FieldValue::Byte(1))));
        // A modifier applies to that default.
        assert!(!zombify(modifier("and", 0)));
        assert!(zombify(modifier("or", 0)));
        assert!(!zombify(modifier("xor", 1)));
        assert!(!zombify(modifier("minecraft:override", 0)));
        assert_eq!(
            with_attributes(modifier("blend", 0)),
            Err(ReadError::DecoderError)
        );
    }
}
