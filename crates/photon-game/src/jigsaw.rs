//! A jigsaw block's contents as its block entity tag carries them (`JigsawBlockEntity.loadAdditional`),
//! and the values its screen edits.

use crate::structure_block::Fields;
use photon_core::identifier::Identifier;

/// `JigsawBlockEntity.JointType`, in the order the screen cycles it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Joint {
    Rollable,
    Aligned,
}

impl Joint {
    pub const ALL: [Self; 2] = [Self::Rollable, Self::Aligned];

    /// `getSerializedName`, which the packet and the translation key use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Rollable => "rollable",
            Self::Aligned => "aligned",
        }
    }

    /// `StructureTemplate.getDefaultJointType`: a block that faces up or down rolls.
    pub fn default_for(vertical: bool) -> Self {
        if vertical {
            Self::Rollable
        } else {
            Self::Aligned
        }
    }
}

/// What the jigsaw block's screen shows.
#[derive(Debug, Clone, PartialEq)]
pub struct JigsawContents {
    pub name: String,
    pub target: String,
    pub pool: String,
    pub final_state: String,
    pub joint: Joint,
    pub selection_priority: i32,
    pub placement_priority: i32,
    /// Whether the block faces up or down (`JigsawBlock.getFrontFacing(state).getAxis().isVertical()`).
    pub vertical: bool,
}

/// An identifier the tag holds, written out; `minecraft:empty` when it is missing or does not parse.
fn identifier_in(fields: &Fields, key: &str) -> String {
    fields
        .name(key)
        .and_then(|text| Identifier::parse(text).ok())
        .map_or_else(|| "minecraft:empty".to_owned(), |id| id.as_string())
}

impl JigsawContents {
    /// `JigsawBlockEntity.loadAdditional` for a tag, of a block that faces up or down or not.
    pub fn from_tag(tag: Option<&[u8]>, vertical: bool) -> Self {
        let fields =
            tag.and_then(|mut input| photon_core::nbt::read_nbt(&mut input).ok().flatten());
        let fields = Fields(fields.unwrap_or_default());
        let int = |key: &str| fields.int(key, 0, (i32::MIN, i32::MAX));
        Self {
            name: identifier_in(&fields, "name"),
            target: identifier_in(&fields, "target"),
            pool: identifier_in(&fields, "pool"),
            final_state: fields
                .name("final_state")
                .unwrap_or("minecraft:air")
                .to_owned(),
            joint: match fields.name("joint") {
                Some("rollable") => Joint::Rollable,
                Some("aligned") => Joint::Aligned,
                _ => Joint::default_for(vertical),
            },
            selection_priority: int("selection_priority"),
            placement_priority: int("placement_priority"),
            vertical,
        }
    }
}

/// Whether a text parses as an identifier (`JigsawBlockEditScreen.isValidIdentifier`).
pub fn is_valid_identifier(text: &str) -> bool {
    Identifier::parse(text).is_ok()
}

/// `Identifier.parse` written out, as `sendToServer` sends it.
pub fn identifier_text(text: &str) -> String {
    Identifier::parse(text).map_or_else(|_| text.to_owned(), |id| id.as_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_tag_is_the_blocks_defaults() {
        let up = JigsawContents::from_tag(None, true);
        assert_eq!(up.name, "minecraft:empty");
        assert_eq!(up.final_state, "minecraft:air");
        assert_eq!(up.joint, Joint::Rollable);
        assert_eq!(JigsawContents::from_tag(None, false).joint, Joint::Aligned);
    }

    #[test]
    fn identifiers_are_checked_and_written_out() {
        assert!(is_valid_identifier("a:b/c"));
        assert!(!is_valid_identifier("A B"));
        assert_eq!(identifier_text("tower"), "minecraft:tower");
    }
}
