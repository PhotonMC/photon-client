//! A structure block's contents as its block entity tag carries them
//! (`StructureBlockEntity.loadAdditional`), and the values its screen edits.

use photon_core::identifier::Identifier;
use photon_core::nbt::{Field, FieldValue};

/// `StructureMode`, in the order the wire counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureMode {
    Save,
    Load,
    Corner,
    Data,
}

impl StructureMode {
    pub const ALL: [Self; 4] = [Self::Save, Self::Load, Self::Corner, Self::Data];

    /// `getSerializedName`, which the translation keys use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Save => "save",
            Self::Load => "load",
            Self::Corner => "corner",
            Self::Data => "data",
        }
    }
}

/// `Mirror`, in the order the wire counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirror {
    None,
    LeftRight,
    FrontBack,
}

impl Mirror {
    pub const ALL: [Self; 3] = [Self::None, Self::LeftRight, Self::FrontBack];

    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LeftRight => "left_right",
            Self::FrontBack => "front_back",
        }
    }
}

/// `Rotation`, in the order the wire counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

/// `StructureBlockEntity.UpdateType`, in the order the wire counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    UpdateData,
    SaveArea,
    LoadArea,
    ScanArea,
}

/// What the structure block's screen shows and sends back.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureBlockContents {
    /// `getStructureName`: the identifier written out, or empty.
    pub name: String,
    pub metadata: String,
    pub pos: (i32, i32, i32),
    pub size: (i32, i32, i32),
    pub mirror: Mirror,
    pub rotation: Rotation,
    pub mode: StructureMode,
    pub ignore_entities: bool,
    pub strict: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
    pub integrity: f32,
    pub seed: i64,
}

impl Default for StructureBlockContents {
    /// A structure block whose tag is missing: the entity's own defaults, in data mode.
    fn default() -> Self {
        Self {
            name: String::new(),
            metadata: String::new(),
            pos: (0, 1, 0),
            size: (0, 0, 0),
            mirror: Mirror::None,
            rotation: Rotation::None,
            mode: StructureMode::Data,
            ignore_entities: true,
            strict: false,
            show_air: false,
            show_bounding_box: true,
            integrity: 1.0,
            seed: 0,
        }
    }
}

/// `StructureBlockEntity.setStructureName`: the name as an identifier, which fills in the
/// namespace, or nothing when it is empty or does not parse.
pub fn structure_name(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }
    Identifier::parse(name).map_or_else(|_| String::new(), |identifier| identifier.as_string())
}

/// The fields of a block entity tag, read the way `loadAdditional` reads them.
pub(crate) struct Fields(pub(crate) Vec<Field>);

impl Fields {
    pub(crate) fn get(&self, key: &str) -> Option<&FieldValue> {
        self.0
            .iter()
            .find(|field| field.key == key)
            .map(|field| &field.value)
    }

    pub(crate) fn text(&self, key: &str) -> String {
        self.name(key).unwrap_or_default().to_owned()
    }

    /// A string, if the field is one.
    pub(crate) fn name(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(FieldValue::Str(text)) => Some(text.as_str()),
            _ => None,
        }
    }

    /// `Mth.clamp` of a number the tag holds as a byte, short or int.
    pub(crate) fn int(&self, key: &str, default: i32, (low, high): (i32, i32)) -> i32 {
        let value = match self.get(key) {
            Some(FieldValue::Byte(value)) => i32::from(*value),
            Some(FieldValue::Short(value)) => i32::from(*value),
            Some(FieldValue::Int(value)) => *value,
            _ => default,
        };
        value.max(low).min(high)
    }

    pub(crate) fn flag(&self, key: &str, default: bool) -> bool {
        match self.get(key) {
            Some(FieldValue::Byte(value)) => *value != 0,
            _ => default,
        }
    }

    fn triple(
        &self,
        keys: [&str; 3],
        default: (i32, i32, i32),
        range: (i32, i32),
    ) -> (i32, i32, i32) {
        (
            self.int(keys[0], default.0, range),
            self.int(keys[1], default.1, range),
            self.int(keys[2], default.2, range),
        )
    }
}

fn mirror_named(name: Option<&str>) -> Mirror {
    match name {
        Some("LEFT_RIGHT") => Mirror::LeftRight,
        Some("FRONT_BACK") => Mirror::FrontBack,
        _ => Mirror::None,
    }
}

fn rotation_named(name: Option<&str>) -> Rotation {
    match name {
        Some("CLOCKWISE_90") => Rotation::Clockwise90,
        Some("CLOCKWISE_180") => Rotation::Clockwise180,
        Some("COUNTERCLOCKWISE_90") => Rotation::Counterclockwise90,
        _ => Rotation::None,
    }
}

fn mode_named(name: Option<&str>) -> StructureMode {
    match name {
        Some("SAVE") => StructureMode::Save,
        Some("LOAD") => StructureMode::Load,
        Some("CORNER") => StructureMode::Corner,
        _ => StructureMode::Data,
    }
}

impl StructureBlockContents {
    /// `StructureBlockEntity.loadAdditional` for a tag.
    pub fn from_tag(tag: Option<&[u8]>) -> Self {
        let fields =
            tag.and_then(|mut input| photon_core::nbt::read_nbt(&mut input).ok().flatten());
        let fields = Fields(fields.unwrap_or_default());
        let defaults = Self::default();
        Self {
            name: structure_name(&fields.text("name")),
            metadata: fields.text("metadata"),
            pos: fields.triple(["posX", "posY", "posZ"], defaults.pos, (-48, 48)),
            size: fields.triple(["sizeX", "sizeY", "sizeZ"], (0, 0, 0), (0, 48)),
            mirror: mirror_named(fields.name("mirror")),
            rotation: rotation_named(fields.name("rotation")),
            mode: mode_named(fields.name("mode")),
            ignore_entities: fields.flag("ignoreEntities", true),
            strict: fields.flag("strict", false),
            show_air: fields.flag("showair", false),
            show_bounding_box: fields.flag("showboundingbox", true),
            integrity: match fields.get("integrity") {
                Some(FieldValue::Float(value)) => *value,
                _ => 1.0,
            },
            seed: match fields.get("seed") {
                Some(FieldValue::Long(value)) => *value,
                _ => 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(fields: Vec<Field>) -> Vec<u8> {
        let mut out = Vec::new();
        photon_core::nbt::write_nbt(Some(&fields), &mut out);
        out
    }

    fn field(key: &str, value: FieldValue) -> Field {
        Field {
            key: key.to_owned(),
            value,
        }
    }

    #[test]
    fn a_missing_tag_is_the_default_entity() {
        assert_eq!(
            StructureBlockContents::from_tag(None),
            StructureBlockContents::default()
        );
    }

    #[test]
    fn the_tag_is_read_and_its_numbers_are_kept_in_range() {
        let bytes = tag(vec![
            field("name", FieldValue::Str("house".into())),
            field("posX", FieldValue::Int(100)),
            field("sizeY", FieldValue::Int(-3)),
            field("mode", FieldValue::Str("LOAD".into())),
            field("rotation", FieldValue::Str("CLOCKWISE_180".into())),
            field("showboundingbox", FieldValue::Byte(0)),
            field("integrity", FieldValue::Float(0.5)),
            field("seed", FieldValue::Long(9)),
        ]);
        let contents = StructureBlockContents::from_tag(Some(&bytes));
        assert_eq!(contents.name, "minecraft:house");
        assert_eq!(contents.pos, (48, 1, 0));
        assert_eq!(contents.size, (0, 0, 0));
        assert_eq!(contents.mode, StructureMode::Load);
        assert_eq!(contents.rotation, Rotation::Clockwise180);
        assert!(!contents.show_bounding_box);
        assert_eq!((contents.integrity, contents.seed), (0.5, 9));
    }

    #[test]
    fn a_name_that_does_not_parse_is_empty() {
        assert_eq!(structure_name("Bad Name"), "");
        assert_eq!(structure_name("a:b"), "a:b");
    }
}
