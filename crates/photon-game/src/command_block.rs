//! A command block's contents as its block entity tag carries them (`BaseCommandBlock.load`).

use photon_core::nbt::FieldValue;

use crate::text::Component;

/// What the command block's screen shows and sends back.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandBlockContents {
    pub command: String,
    /// `TrackOutput`, true when missing.
    pub track_output: bool,
    /// `LastOutput`, kept while the output is tracked.
    pub last_output: Option<Component>,
    /// `auto`.
    pub automatic: bool,
}

impl Default for CommandBlockContents {
    /// A command block whose tag is missing or unread: no command, output tracked.
    fn default() -> Self {
        Self {
            command: String::new(),
            track_output: true,
            last_output: None,
            automatic: false,
        }
    }
}

impl CommandBlockContents {
    /// `CommandBlockEntity.loadAdditional` for a tag.
    pub fn from_tag(tag: Option<&[u8]>) -> Self {
        let fields =
            tag.and_then(|mut input| photon_core::nbt::read_nbt(&mut input).ok().flatten());
        let fields = fields.unwrap_or_default();
        let get = |key: &str| {
            fields
                .iter()
                .find(|field| field.key == key)
                .map(|field| &field.value)
        };
        let flag = |key: &str, default: bool| match get(key) {
            Some(FieldValue::Byte(value)) => *value != 0,
            _ => default,
        };
        let track_output = flag("TrackOutput", true);
        Self {
            command: match get("Command") {
                Some(FieldValue::Str(text)) => text.clone(),
                _ => String::new(),
            },
            track_output,
            last_output: get("LastOutput")
                .filter(|_| track_output)
                .and_then(Component::from_nbt),
            automatic: flag("auto", false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_core::nbt::Field;

    fn tag(fields: Vec<Field>) -> Vec<u8> {
        let mut out = Vec::new();
        photon_core::nbt::write_nbt(Some(&fields), &mut out);
        out
    }

    fn field(key: &str, value: FieldValue) -> Field {
        Field {
            key: key.into(),
            value,
        }
    }

    #[test]
    fn a_tag_gives_the_command_the_output_and_the_auto_flag() {
        let bytes = tag(vec![
            field("Command", FieldValue::Str("say hi".into())),
            field("LastOutput", FieldValue::Str("done".into())),
            field("auto", FieldValue::Byte(1)),
        ]);
        let contents = CommandBlockContents::from_tag(Some(&bytes));
        assert_eq!(contents.command, "say hi");
        assert!(contents.track_output && contents.automatic);
        assert_eq!(contents.last_output, Some(Component::literal("done")));
    }

    #[test]
    fn an_untracked_block_keeps_no_output_and_a_missing_tag_is_blank() {
        let bytes = tag(vec![
            field("TrackOutput", FieldValue::Byte(0)),
            field("LastOutput", FieldValue::Str("stale".into())),
        ]);
        let contents = CommandBlockContents::from_tag(Some(&bytes));
        assert!(!contents.track_output);
        assert_eq!(contents.last_output, None);
        assert_eq!(
            CommandBlockContents::from_tag(None),
            CommandBlockContents::default()
        );
    }
}
