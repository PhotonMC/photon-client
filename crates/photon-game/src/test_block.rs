//! A test block's contents as its block entity tag carries them (`TestBlockEntity.loadAdditional`),
//! and the values its screen edits.

use crate::structure_block::Fields;

/// `TestBlockMode`, in the order the screen cycles it and the wire counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

impl TestBlockMode {
    pub const ALL: [Self; 4] = [Self::Start, Self::Log, Self::Fail, Self::Accept];

    /// `getSerializedName`, which the translation keys and the block state use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Log => "log",
            Self::Fail => "fail",
            Self::Accept => "accept",
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.name() == name)
    }
}

/// What the test block's screen shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestBlockContents {
    pub mode: TestBlockMode,
    pub message: String,
}

impl TestBlockContents {
    /// The block entity as the client holds it: it starts with the mode of the block state, and
    /// the tag the server sent replaces it (`loadAdditional` takes a mode that is missing or not
    /// one as fail).
    pub fn from_tag(tag: Option<&[u8]>, state_mode: TestBlockMode) -> Self {
        let Some(mut input) = tag else {
            return Self {
                mode: state_mode,
                message: String::new(),
            };
        };
        let fields = photon_core::nbt::read_nbt(&mut input).ok().flatten();
        let fields = Fields(fields.unwrap_or_default());
        Self {
            mode: fields
                .name("mode")
                .and_then(TestBlockMode::named)
                .unwrap_or(TestBlockMode::Fail),
            message: fields.text("message"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::nbt::{Field, FieldValue};

    fn tag(fields: Vec<Field>) -> Vec<u8> {
        let mut out = Vec::new();
        photon_core::nbt::write_nbt(Some(&fields), &mut out);
        out
    }

    fn field(key: &str, value: &str) -> Field {
        Field {
            key: key.to_owned(),
            value: FieldValue::Str(value.to_owned()),
        }
    }

    #[test]
    fn no_tag_keeps_the_mode_of_the_block_state() {
        let contents = TestBlockContents::from_tag(None, TestBlockMode::Accept);
        assert_eq!(contents.mode, TestBlockMode::Accept);
        assert_eq!(contents.message, "");
    }

    #[test]
    fn the_tag_replaces_the_mode_and_a_missing_mode_is_fail() {
        let bytes = tag(vec![field("mode", "log"), field("message", "hello")]);
        let contents = TestBlockContents::from_tag(Some(&bytes), TestBlockMode::Start);
        assert_eq!(contents.mode, TestBlockMode::Log);
        assert_eq!(contents.message, "hello");
        let bytes = tag(vec![field("message", "x")]);
        let contents = TestBlockContents::from_tag(Some(&bytes), TestBlockMode::Start);
        assert_eq!(contents.mode, TestBlockMode::Fail);
        let bytes = tag(vec![field("mode", "nonsense")]);
        let contents = TestBlockContents::from_tag(Some(&bytes), TestBlockMode::Start);
        assert_eq!(contents.mode, TestBlockMode::Fail);
    }
}
