//! What a test instance block's screen shows and sends
//! (`TestInstanceBlockEditScreen`, `ServerboundTestInstanceBlockActionPacket`).

use crate::structure_block::{Fields, Rotation};
use crate::text::Component;
use photon_core::nbt::FieldValue;

/// What the screen shows of a block: the test it names, the size of its structure, the rotation
/// and whether entities are left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceContents {
    /// The test's identifier as written, `None` for a block that names none.
    pub test: Option<String>,
    pub size: (i32, i32, i32),
    pub rotation: Rotation,
    pub ignore_entities: bool,
}

/// `ServerboundTestInstanceBlockActionPacket.Action`, in the order the wire counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceAction {
    Init,
    Query,
    Set,
    Reset,
    Save,
    Export,
    Run,
}

impl InstanceAction {
    pub fn name(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Query => "query",
            Self::Set => "set",
            Self::Reset => "reset",
            Self::Save => "save",
            Self::Export => "export",
            Self::Run => "run",
        }
    }
}

/// One action sent to the server with the values the screen holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRequest {
    pub action: InstanceAction,
    /// The test's identifier, normalised (`Identifier.toString`), `None` when the box did not
    /// parse.
    pub test: Option<String>,
    pub size: (i32, i32, i32),
    pub rotation: Rotation,
    pub ignore_entities: bool,
}

impl Rotation {
    pub const ALL: [Self; 4] = [
        Self::None,
        Self::Clockwise90,
        Self::Clockwise180,
        Self::Counterclockwise90,
    ];

    /// The enum constant's name in lower case.
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "clockwise_180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }
}

impl TestInstanceContents {
    /// The block as its entity tag says (`Data.CODEC`): a tag that lacks the data or holds it in a
    /// form the codec refuses leaves the entity's defaults.
    pub fn from_tag(tag: Option<&[u8]>) -> Self {
        Self::read(tag).map_or_else(Self::default, |(contents, _)| contents)
    }

    /// The error the last run left on the block, if the tag holds one.
    pub fn error_from_tag(tag: Option<&[u8]>) -> Option<Component> {
        Self::read(tag).and_then(|(_, error)| error)
    }

    fn read(mut tag: Option<&[u8]>) -> Option<(Self, Option<Component>)> {
        let input = tag.as_mut()?;
        let fields = Fields(photon_core::nbt::read_nbt(input).ok().flatten()?);
        let Some(FieldValue::Compound(data)) = fields.get("data") else {
            return None;
        };
        let data = Fields(data.clone());
        let Some(FieldValue::IntArray(size)) = data.get("size") else {
            return None;
        };
        let &[x, y, z] = size.as_slice() else {
            return None;
        };
        let rotation = Rotation::ALL
            .into_iter()
            .find(|rotation| Some(rotation.name()) == data.name("rotation"))?;
        let FieldValue::Byte(ignore) = data.get("ignore_entities")? else {
            return None;
        };
        data.name("status")?;
        // An optional field that is there but wrong fails the whole record.
        let test = match data.get("test") {
            None => None,
            Some(FieldValue::Str(name)) => Some(
                photon_core::identifier::Identifier::parse(name)
                    .ok()?
                    .as_string(),
            ),
            Some(_) => return None,
        };
        let error = match data.get("error_message") {
            None => None,
            Some(value) => Some(Component::from_nbt(value)?),
        };
        Some((
            Self {
                test,
                size: (x, y, z),
                rotation,
                ignore_entities: *ignore != 0,
            },
            error,
        ))
    }
}

impl Default for TestInstanceContents {
    /// `TestInstanceBlockEntity.Data.DEFAULT`.
    fn default() -> Self {
        Self {
            test: None,
            size: (0, 0, 0),
            rotation: Rotation::None,
            ignore_entities: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::nbt::Field;

    fn field(key: &str, value: FieldValue) -> Field {
        Field {
            key: key.to_owned(),
            value,
        }
    }

    fn tag(data: Vec<Field>) -> Vec<u8> {
        let mut out = Vec::new();
        photon_core::nbt::write_nbt(Some(&[field("data", FieldValue::Compound(data))]), &mut out);
        out
    }

    fn full(extra: Vec<Field>) -> Vec<Field> {
        let mut data = vec![
            field("test", FieldValue::Str("example".to_owned())),
            field("size", FieldValue::IntArray(vec![3, 4, 5])),
            field("rotation", FieldValue::Str("clockwise_90".to_owned())),
            field("ignore_entities", FieldValue::Byte(1)),
            field("status", FieldValue::Str("cleared".to_owned())),
        ];
        data.extend(extra);
        data
    }

    #[test]
    fn the_tag_gives_the_data_the_block_holds() {
        let bytes = tag(full(vec![field(
            "error_message",
            FieldValue::Str("bad".to_owned()),
        )]));
        let contents = TestInstanceContents::from_tag(Some(&bytes));
        assert_eq!(contents.test.as_deref(), Some("minecraft:example"));
        assert_eq!(contents.size, (3, 4, 5));
        assert_eq!(contents.rotation, Rotation::Clockwise90);
        assert!(contents.ignore_entities);
        assert_eq!(
            TestInstanceContents::error_from_tag(Some(&bytes)),
            Some(Component::literal("bad"))
        );
    }

    #[test]
    fn an_optional_field_that_does_not_parse_fails_the_record() {
        let mut data = full(Vec::new());
        data[0] = field("test", FieldValue::Str("Not Valid".to_owned()));
        let bytes = tag(data);
        assert_eq!(
            TestInstanceContents::from_tag(Some(&bytes)),
            TestInstanceContents::default()
        );
    }

    #[test]
    fn a_tag_the_codec_refuses_leaves_the_defaults() {
        assert_eq!(
            TestInstanceContents::from_tag(None),
            TestInstanceContents::default()
        );
        let mut data = full(Vec::new());
        data.retain(|field| field.key != "status");
        let bytes = tag(data);
        assert_eq!(
            TestInstanceContents::from_tag(Some(&bytes)),
            TestInstanceContents::default()
        );
        assert_eq!(TestInstanceContents::error_from_tag(Some(&bytes)), None);
    }
}
