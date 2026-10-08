//! A sign's text as its block entity tag carries it (`SignText.DIRECT_CODEC`).

use voxel_core::nbt::{Field, FieldValue};

use crate::text::Component;

pub use voxel_world::DYES;

/// `DyeColor.getTextColor` in `DyeColor` order, opaque.
pub const DYE_TEXT_COLORS: [u32; 16] = [
    0xFFFF_FFFF,
    0xFFFF_681F,
    0xFFFF_00FF,
    0xFF9A_C0CD,
    0xFFFF_FF00,
    0xFFBF_FF00,
    0xFFFF_69B4,
    0xFF80_8080,
    0xFFD3_D3D3,
    0xFF00_FFFF,
    0xFFA0_20F0,
    0xFF00_00FF,
    0xFF8B_4513,
    0xFF00_FF00,
    0xFFFF_0000,
    0xFF00_0000,
];

/// The index of `black` in [`DYES`].
pub const BLACK: usize = 15;

/// One face of a sign: its four lines as written and as filtered, the dye and whether it glows.
#[derive(Debug, Clone, PartialEq)]
pub struct SignText {
    pub messages: [Component; 4],
    pub filtered_messages: [Component; 4],
    /// An index into [`DYES`].
    pub color: usize,
    pub has_glowing_text: bool,
}

impl Default for SignText {
    /// `new SignText()`: four empty lines in black, not glowing.
    fn default() -> Self {
        let empty = || std::array::from_fn(|_| Component::literal(""));
        Self {
            messages: empty(),
            filtered_messages: empty(),
            color: BLACK,
            has_glowing_text: false,
        }
    }
}

impl SignText {
    /// `SignText.getMessages`: the filtered lines when the player filters text.
    pub fn messages(&self, filtered: bool) -> &[Component; 4] {
        if filtered {
            &self.filtered_messages
        } else {
            &self.messages
        }
    }

    /// `SignText.DIRECT_CODEC`: four lines (anything else fails), the filtered ones only when
    /// they are four too (the written ones otherwise), a dye (black when missing), the glow.
    fn from_fields(fields: &[Field]) -> Option<Self> {
        let get = |key: &str| {
            fields
                .iter()
                .find(|field| field.key == key)
                .map(|field| &field.value)
        };
        let lines = |value: Option<&FieldValue>| -> Option<[Component; 4]> {
            let FieldValue::List(items) = value? else {
                return None;
            };
            let lines: Vec<Component> = items
                .iter()
                .map(Component::from_nbt)
                .collect::<Option<_>>()?;
            lines.try_into().ok()
        };
        let messages = lines(get("messages"))?;
        let filtered_messages = lines(get("filtered_messages")).unwrap_or_else(|| messages.clone());
        let color = match get("color") {
            None => BLACK,
            Some(FieldValue::Str(name)) => DYES.iter().position(|dye| dye == name)?,
            Some(_) => return None,
        };
        let has_glowing_text = match get("has_glowing_text") {
            None => false,
            Some(FieldValue::Byte(value)) => *value != 0,
            Some(_) => return None,
        };
        Some(Self {
            messages,
            filtered_messages,
            color,
            has_glowing_text,
        })
    }
}

/// `SignBlockEntity.loadAdditional`: the front and back text of a block entity tag, each a blank
/// one when missing or not read.
pub fn sign_texts(tag: Option<&[u8]>) -> (SignText, SignText) {
    let fields = tag.and_then(|mut input| voxel_core::nbt::read_nbt(&mut input).ok().flatten());
    let face = |key: &str| {
        fields
            .as_deref()
            .and_then(|fields| fields.iter().find(|field| field.key == key))
            .and_then(|field| match &field.value {
                FieldValue::Compound(fields) => SignText::from_fields(fields),
                _ => None,
            })
            .unwrap_or_default()
    };
    (face("front_text"), face("back_text"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(key: &str, value: FieldValue) -> Field {
        Field {
            key: key.into(),
            value,
        }
    }

    fn lines(texts: &[&str]) -> FieldValue {
        FieldValue::List(
            texts
                .iter()
                .map(|text| FieldValue::Str((*text).into()))
                .collect(),
        )
    }

    #[test]
    fn reads_both_faces_and_drops_a_malformed_one() {
        let front = vec![
            field("messages", lines(&["a", "b", "c", "d"])),
            field("color", FieldValue::Str("red".into())),
            field("has_glowing_text", FieldValue::Byte(1)),
        ];
        // Three lines fail the codec: the face is blank.
        let back = vec![field("messages", lines(&["x", "y", "z"]))];
        let mut tag = Vec::new();
        voxel_core::nbt::write_nbt(
            Some(&[
                field("front_text", FieldValue::Compound(front)),
                field("back_text", FieldValue::Compound(back)),
            ]),
            &mut tag,
        );
        let (front, back) = sign_texts(Some(&tag));
        assert_eq!(front.messages[3], Component::literal("d"));
        assert_eq!(front.filtered_messages, front.messages);
        assert_eq!((front.color, front.has_glowing_text), (14, true));
        assert_eq!(back, SignText::default());
        assert_eq!(sign_texts(None).0, SignText::default());
    }
}
