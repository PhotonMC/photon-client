//! Item component patches as the network carries them.
//!
//! A stack's patch says which data components it adds, with their values,
//! and which it removes. The client has to read every value to find where
//! the stack ends, and to send a stack back it has to write each value
//! behind its length (`DataComponentPatch.DELIMITED_STREAM_CODEC`). Neither
//! needs the values themselves, so each component's stream codec is ported
//! as a skip: it moves over the bytes the reference's codec would read.
//!
//! A stack whose component predicate names a partial predicate is refused:
//! this port does not read those.

/// The bytes are not what the codec reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Malformed;

type R<T = ()> = Result<T, Malformed>;

/// A patch, split into its parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// The added components: the type's registry id and the value's bytes.
    pub added: Vec<(i32, Vec<u8>)>,
    /// The removed components' type ids.
    pub removed: Vec<i32>,
}

impl Patch {
    /// The patch as the trusted codec writes it, the way a server sends a
    /// stack.
    pub fn trusted(&self) -> Vec<u8> {
        self.write(false)
    }

    /// The patch as the delimited codec writes it, the way a client sends a
    /// stack to the server: each value behind its length.
    pub fn delimited(&self) -> Vec<u8> {
        self.write(true)
    }

    fn write(&self, delimited: bool) -> Vec<u8> {
        let mut out = Vec::new();
        voxel_core::wire::write_varint(self.added.len() as i32, &mut out);
        voxel_core::wire::write_varint(self.removed.len() as i32, &mut out);
        for (id, value) in &self.added {
            voxel_core::wire::write_varint(*id, &mut out);
            if delimited {
                voxel_core::wire::write_varint(value.len() as i32, &mut out);
            }
            out.extend_from_slice(value);
        }
        for id in &self.removed {
            voxel_core::wire::write_varint(*id, &mut out);
        }
        out
    }
}

/// `DataComponentPatch.STREAM_CODEC`, read from the start of a patch.
pub fn read_patch(input: &mut &[u8]) -> R<Patch> {
    let mut cursor = Cursor { data: input };
    let patch = cursor.patch()?;
    *input = cursor.data;
    Ok(patch)
}

/// The registry ids of the data component types, named.
pub use voxel_data::item::component_types as ids;

/// `PotionContents`, as far as the colour and the name read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionContents {
    /// The potion's registry id.
    pub potion: Option<i32>,
    pub custom_color: Option<i32>,
    /// The custom effects: the mob effect's registry id, its amplifier, its
    /// duration in ticks (-1 for no end) and whether it shows.
    pub effects: Vec<(i32, i32, i32, bool)>,
    pub custom_name: Option<String>,
}

/// A component value that is one `int`: `dyed_color`, `map_color`.
pub fn int_value(value: &[u8]) -> Option<i32> {
    Some(i32::from_be_bytes(value.get(..4)?.try_into().ok()?))
}

/// A component value that is one `VarInt`: a rarity.
pub fn varint_value(value: &[u8]) -> Option<i32> {
    let mut cursor = Cursor { data: value };
    cursor.varint().ok()
}

/// `block_state` (`BlockItemStateProperties`): the property names and the
/// values the stack gives them.
pub fn block_state_properties(value: &[u8]) -> Option<Vec<(String, String)>> {
    let mut cursor = Cursor { data: value };
    let string = |cursor: &mut Cursor| -> Option<String> {
        let length = cursor.count().ok()?;
        let bytes = cursor.data.get(..length)?;
        let text = String::from_utf8_lossy(bytes).into_owned();
        cursor.skip(length).ok()?;
        Some(text)
    };
    let mut properties = Vec::new();
    for _ in 0..cursor.count().ok()? {
        let name = string(&mut cursor)?;
        properties.push((name, string(&mut cursor)?));
    }
    Some(properties)
}

/// `potion_contents`.
pub fn potion_contents(value: &[u8]) -> Option<PotionContents> {
    let mut cursor = Cursor { data: value };
    let potion = if cursor.boolean().ok()? {
        Some(cursor.varint().ok()?)
    } else {
        None
    };
    let custom_color = if cursor.boolean().ok()? {
        let bytes = cursor.data.get(..4)?;
        let color = i32::from_be_bytes(bytes.try_into().ok()?);
        cursor.skip(4).ok()?;
        Some(color)
    } else {
        None
    };
    let mut effects = Vec::new();
    for _ in 0..cursor.count().ok()? {
        let effect = cursor.varint().ok()?;
        let amplifier = cursor.varint().ok()?;
        let duration = cursor.varint().ok()?;
        cursor.boolean().ok()?;
        let visible = cursor.boolean().ok()?;
        cursor.boolean().ok()?;
        if cursor.boolean().ok()? {
            cursor.mob_effect_details().ok()?;
        }
        effects.push((effect, amplifier, duration, visible));
    }
    let custom_name = if cursor.boolean().ok()? {
        let length = cursor.count().ok()?;
        let bytes = cursor.data.get(..length)?;
        Some(String::from_utf8_lossy(bytes).into_owned())
    } else {
        None
    };
    Some(PotionContents {
        potion,
        custom_color,
        effects,
        custom_name,
    })
}

/// `firework_explosion`'s colours.
pub fn firework_colors(value: &[u8]) -> Option<Vec<i32>> {
    let mut cursor = Cursor { data: value };
    cursor.varint().ok()?;
    let mut colors = Vec::new();
    for _ in 0..cursor.count().ok()? {
        let bytes = cursor.data.get(..4)?;
        colors.push(i32::from_be_bytes(bytes.try_into().ok()?));
        cursor.skip(4).ok()?;
    }
    Some(colors)
}

/// A patch as the trusted codec wrote it, sent back as the delimited codec
/// writes it. `None` when the patch is not one this reads.
pub fn delimit(trusted: &[u8]) -> Option<Vec<u8>> {
    let mut input = trusted;
    let patch = read_patch(&mut input).ok()?;
    input.is_empty().then(|| patch.delimited())
}

pub struct Cursor<'a> {
    pub data: &'a [u8],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_patch_splits_into_its_values_and_writes_them_back() {
        // A painting variant (103) by registry id, and a removed component.
        let wire = [1, 1, 103, 5, 4];
        let mut input = &wire[..];
        let patch = read_patch(&mut input).expect("a patch");
        assert!(input.is_empty());
        assert_eq!(patch.added, vec![(103, vec![5])]);
        assert_eq!(patch.removed, vec![4]);
        assert_eq!(patch.trusted(), wire);
        assert_eq!(patch.delimited(), [1, 1, 103, 1, 5, 4]);
    }

    #[test]
    fn a_component_without_a_network_codec_is_a_tag() {
        // `custom_data` is sent as the compound its codec makes.
        let mut input = &[1_u8, 0, 0, 10, 0][..];
        let patch = read_patch(&mut input).expect("a patch");
        assert!(input.is_empty());
        assert_eq!(patch.added, vec![(0, vec![10, 0])]);
    }

    #[test]
    fn an_unknown_component_is_refused() {
        let mut input = &[1_u8, 0, 111, 0][..];
        assert_eq!(read_patch(&mut input), Err(Malformed));
    }

    #[test]
    fn nested_tags_are_skipped_whole() {
        // A compound holding a list of two ints, then a string.
        let tag = [
            10, 9, 0, 1, b'a', 3, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2, 8, 0, 1, b'b', 0, 1, b'x', 0,
        ];
        let mut cursor = Cursor { data: &tag };
        cursor.tag().expect("a tag");
        assert!(cursor.data.is_empty());
    }

    #[test]
    fn the_empty_patch_reads_as_two_zeros() {
        let mut input = &[0_u8, 0][..];
        let patch = read_patch(&mut input).expect("a patch");
        assert!(patch.added.is_empty() && patch.removed.is_empty());
        assert_eq!(delimit(&[0, 0]), Some(vec![0, 0]));
    }
}

mod cursor_firework;
mod cursor_tag;
