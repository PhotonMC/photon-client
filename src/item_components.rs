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
        crate::wire::write_varint(self.added.len() as i32, &mut out);
        crate::wire::write_varint(self.removed.len() as i32, &mut out);
        for (id, value) in &self.added {
            crate::wire::write_varint(*id, &mut out);
            if delimited {
                crate::wire::write_varint(value.len() as i32, &mut out);
            }
            out.extend_from_slice(value);
        }
        for id in &self.removed {
            crate::wire::write_varint(*id, &mut out);
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

/// The registry ids of the components whose values are read.
pub mod ids {
    pub const CUSTOM_NAME: i32 = 6;
    pub const ITEM_NAME: i32 = 9;
    pub const LORE: i32 = 11;
    pub const RARITY: i32 = 12;
    pub const ENCHANTMENTS: i32 = 13;
    pub const DYED_COLOR: i32 = 44;
    pub const MAP_COLOR: i32 = 45;
    pub const POTION_CONTENTS: i32 = 51;
    pub const FIREWORK_EXPLOSION: i32 = 68;
    pub const STORED_ENCHANTMENTS: i32 = 42;
}

/// `PotionContents`, as far as the colour and the name read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionContents {
    /// The potion's registry id.
    pub potion: Option<i32>,
    pub custom_color: Option<i32>,
    /// The custom effects: the mob effect's registry id, its amplifier and
    /// whether it shows.
    pub effects: Vec<(i32, i32, bool)>,
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
        cursor.varint().ok()?;
        cursor.boolean().ok()?;
        let visible = cursor.boolean().ok()?;
        cursor.boolean().ok()?;
        if cursor.boolean().ok()? {
            cursor.mob_effect_details().ok()?;
        }
        effects.push((effect, amplifier, visible));
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

struct Cursor<'a> {
    data: &'a [u8],
}

impl Cursor<'_> {
    fn skip(&mut self, count: usize) -> R {
        if self.data.len() < count {
            return Err(Malformed);
        }
        self.data = &self.data[count..];
        Ok(())
    }

    fn byte(&mut self) -> R<u8> {
        let byte = *self.data.first().ok_or(Malformed)?;
        self.data = &self.data[1..];
        Ok(byte)
    }

    fn boolean(&mut self) -> R<bool> {
        Ok(self.byte()? != 0)
    }

    fn varint(&mut self) -> R<i32> {
        let mut value = 0_u32;
        for shift in 0..5 {
            let byte = self.byte()?;
            value |= u32::from(byte & 0x7F) << (shift * 7);
            if byte & 0x80 == 0 {
                return Ok(value as i32);
            }
        }
        Err(Malformed)
    }

    /// A count that a list or a map reads.
    fn count(&mut self) -> R<usize> {
        usize::try_from(self.varint()?).map_err(|_| Malformed)
    }

    fn float(&mut self) -> R {
        self.skip(4)
    }

    fn double(&mut self) -> R {
        self.skip(8)
    }

    fn int(&mut self) -> R {
        self.skip(4)
    }

    fn long(&mut self) -> R {
        self.skip(8)
    }

    fn string(&mut self) -> R {
        let length = self.count()?;
        self.skip(length)
    }

    fn identifier(&mut self) -> R {
        self.string()
    }

    fn uuid(&mut self) -> R {
        self.skip(16)
    }

    /// `ByteBufCodecs.optional`.
    fn optional(&mut self, value: impl FnOnce(&mut Self) -> R) -> R {
        if self.boolean()? { value(self) } else { Ok(()) }
    }

    /// `ByteBufCodecs.list`: a count and that many values.
    fn list(&mut self, mut value: impl FnMut(&mut Self) -> R) -> R {
        for _ in 0..self.count()? {
            value(self)?;
        }
        Ok(())
    }

    /// A network tag: its type, then its payload, with no name.
    fn tag(&mut self) -> R {
        let kind = self.byte()?;
        self.tag_payload(kind)
    }

    /// `ByteBufCodecs.COMPOUND_TAG`: a compound, or a single end byte for
    /// none.
    fn compound_tag(&mut self) -> R {
        match self.byte()? {
            0 => Ok(()),
            10 => self.tag_payload(10),
            _ => Err(Malformed),
        }
    }

    fn tag_payload(&mut self, kind: u8) -> R {
        match kind {
            0 => Ok(()),
            1 => self.skip(1),
            2 => self.skip(2),
            3 | 5 => self.skip(4),
            4 | 6 => self.skip(8),
            7 => {
                let length = self.tag_length()?;
                self.skip(length)
            }
            8 => {
                let length = self.short()?;
                self.skip(length)
            }
            9 => {
                let element = self.byte()?;
                let length = self.tag_length()?;
                for _ in 0..length {
                    self.tag_payload(element)?;
                }
                Ok(())
            }
            10 => loop {
                let member = self.byte()?;
                if member == 0 {
                    return Ok(());
                }
                let name = self.short()?;
                self.skip(name)?;
                self.tag_payload(member)?;
            },
            11 => {
                let length = self.tag_length()?;
                self.skip(length.checked_mul(4).ok_or(Malformed)?)
            }
            12 => {
                let length = self.tag_length()?;
                self.skip(length.checked_mul(8).ok_or(Malformed)?)
            }
            _ => Err(Malformed),
        }
    }

    fn short(&mut self) -> R<usize> {
        let high = self.byte()?;
        let low = self.byte()?;
        Ok(usize::from(u16::from_be_bytes([high, low])))
    }

    fn tag_length(&mut self) -> R<usize> {
        let bytes = [self.byte()?, self.byte()?, self.byte()?, self.byte()?];
        usize::try_from(i32::from_be_bytes(bytes)).map_err(|_| Malformed)
    }

    /// `ByteBufCodecs.holder`: a registry id plus one, or a zero and the
    /// direct value.
    fn holder(&mut self, direct: impl FnOnce(&mut Self) -> R) -> R {
        if self.varint()? == 0 {
            direct(self)
        } else {
            Ok(())
        }
    }

    /// `ByteBufCodecs.holderSet`: a count plus one and that many ids, or a
    /// zero and a tag's name.
    fn holder_set(&mut self) -> R {
        match self.varint()? {
            0 => self.identifier(),
            count if count > 0 => {
                for _ in 0..count - 1 {
                    self.varint()?;
                }
                Ok(())
            }
            _ => Err(Malformed),
        }
    }

    /// `SoundEvent.STREAM_CODEC`.
    fn sound(&mut self) -> R {
        self.holder(|cursor| {
            cursor.identifier()?;
            cursor.optional(Cursor::float)
        })
    }

    fn patch(&mut self) -> R<Patch> {
        let added_count = self.count()?;
        let removed_count = self.count()?;
        let mut patch = Patch {
            added: Vec::new(),
            removed: Vec::new(),
        };
        for _ in 0..added_count {
            let id = self.varint()?;
            let start = self.data;
            self.component(id)?;
            let length = start.len() - self.data.len();
            patch.added.push((id, start[..length].to_vec()));
        }
        for _ in 0..removed_count {
            patch.removed.push(self.varint()?);
        }
        Ok(patch)
    }

    /// `ItemStackTemplate.STREAM_CODEC`: an item, a count and a patch.
    fn template(&mut self) -> R {
        self.varint()?;
        self.varint()?;
        self.patch().map(|_| ())
    }

    fn mob_effect_instance(&mut self) -> R {
        self.varint()?;
        self.mob_effect_details()
    }

    fn mob_effect_details(&mut self) -> R {
        self.varint()?;
        self.varint()?;
        self.skip(3)?;
        self.optional(Cursor::mob_effect_details)
    }

    fn consume_effect(&mut self) -> R {
        match self.varint()? {
            0 => {
                self.list(Cursor::mob_effect_instance)?;
                self.float()
            }
            1 => self.holder_set(),
            2 => Ok(()),
            3 => self.float(),
            4 => self.sound(),
            _ => Err(Malformed),
        }
    }

    /// `Filterable.streamCodec`: the raw value and an optional filtered one.
    fn filterable(&mut self, mut value: impl FnMut(&mut Self) -> R) -> R {
        value(self)?;
        self.optional(&mut value)
    }

    fn game_profile_properties(&mut self) -> R {
        self.list(|cursor| {
            cursor.string()?;
            cursor.string()?;
            cursor.optional(Cursor::string)
        })
    }

    /// `BlockPredicate.STREAM_CODEC` of the advancement predicates.
    fn block_predicate(&mut self) -> R {
        self.optional(Cursor::holder_set)?;
        self.optional(|cursor| {
            cursor.list(|cursor| {
                cursor.string()?;
                // Exact, or a range of two optional bounds.
                if cursor.boolean()? {
                    cursor.string()
                } else {
                    cursor.optional(Cursor::string)?;
                    cursor.optional(Cursor::string)
                }
            })
        })?;
        self.optional(Cursor::compound_tag)?;
        // `DataComponentMatchers`: the exact components, then the partial
        // predicates, which this does not read.
        self.list(|cursor| {
            let id = cursor.varint()?;
            cursor.component(id)
        })?;
        if self.count()? != 0 {
            return Err(Malformed);
        }
        Ok(())
    }

    /// One component's value, by the type's registry id.
    fn component(&mut self, id: i32) -> R {
        match id {
            // Holder registries, ids, enumerations and plain numbers.
            1 | 2 | 3 | 8 | 12 | 19 | 31 | 41 | 43 | 46 | 48 | 63 | 73 | 82..=102 | 104..=110 => {
                self.varint().map(|_| ())
            }
            // Unit components.
            4 | 20 | 34 => Ok(()),
            // Components with no network codec of their own are sent as the
            // tag their persistent codec makes: `custom_data`,
            // `intangible_projectile`, `map_decorations`, `debug_stick_state`,
            // `recipes`, `lock` and `container_loot`.
            0 | 22 | 47 | 57 | 66 | 79 | 80 => self.tag(),
            // `use_effects`: two flags and a speed.
            5 => self.skip(6),
            // Text components.
            6 | 9 => self.tag(),
            7 | 52 => self.float(),
            10 | 35 | 71 => self.identifier(),
            11 => self.list(Cursor::tag),
            // Enchantments, the stored ones alike.
            13 | 42 => self.list(|cursor| {
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            // What an item can be placed on or break.
            14 | 15 => self.list(Cursor::block_predicate),
            16 => self.list(|cursor| {
                cursor.varint()?;
                cursor.identifier()?;
                cursor.double()?;
                cursor.varint()?;
                cursor.varint()?;
                match cursor.varint()? {
                    0 | 1 => Ok(()),
                    2 => cursor.tag(),
                    _ => Err(Malformed),
                }
            }),
            // `custom_model_data`.
            17 => {
                self.list(Cursor::float)?;
                self.list(|cursor| cursor.byte().map(|_| ()))?;
                self.list(Cursor::string)?;
                self.list(Cursor::int)
            }
            18 => {
                self.boolean()?;
                self.list(|cursor| cursor.varint().map(|_| ()))
            }
            21 => self.boolean().map(|_| ()),
            23 => {
                self.varint()?;
                self.float()?;
                self.boolean().map(|_| ())
            }
            24 => {
                self.float()?;
                self.varint()?;
                self.sound()?;
                self.boolean()?;
                self.list(Cursor::consume_effect)
            }
            25 | 78 => self.template(),
            26 => {
                self.float()?;
                self.optional(Cursor::identifier)
            }
            27 | 33 | 65 => self.holder_set(),
            28 => {
                self.list(|cursor| {
                    cursor.holder_set()?;
                    cursor.optional(Cursor::float)?;
                    cursor.optional(|cursor| cursor.boolean().map(|_| ()))
                })?;
                self.float()?;
                self.varint()?;
                self.boolean().map(|_| ())
            }
            29 => {
                self.varint()?;
                self.float()
            }
            30 => self.skip(24),
            32 => {
                self.varint()?;
                self.sound()?;
                self.optional(Cursor::identifier)?;
                self.optional(Cursor::identifier)?;
                self.optional(Cursor::holder_set)?;
                self.skip(5)?;
                self.sound()
            }
            36 => self.list(Cursor::consume_effect),
            37 => {
                self.float()?;
                self.float()?;
                self.list(|cursor| {
                    cursor.float()?;
                    cursor.optional(Cursor::holder_set)?;
                    cursor.float()?;
                    cursor.float()
                })?;
                self.skip(12)?;
                self.optional(Cursor::holder_set)?;
                self.optional(Cursor::sound)?;
                self.optional(Cursor::sound)
            }
            38 => {
                self.skip(2)?;
                self.optional(Cursor::sound)?;
                self.optional(Cursor::sound)
            }
            39 => {
                self.varint()?;
                self.varint()?;
                for _ in 0..3 {
                    self.optional(|cursor| {
                        cursor.varint()?;
                        cursor.skip(8)
                    })?;
                }
                self.float()?;
                self.float()?;
                self.optional(Cursor::sound)?;
                self.optional(Cursor::sound)
            }
            40 => {
                self.varint()?;
                self.varint().map(|_| ())
            }
            44 | 45 => self.int(),
            49 | 50 => self.list(Cursor::template),
            // `potion_contents`.
            51 => {
                self.optional(|cursor| cursor.varint().map(|_| ()))?;
                self.optional(Cursor::int)?;
                self.list(Cursor::mob_effect_instance)?;
                self.optional(Cursor::string)
            }
            53 => self.list(|cursor| {
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            54 => self.list(|cursor| cursor.filterable(Cursor::string)),
            55 => {
                self.filterable(Cursor::string)?;
                self.string()?;
                self.varint()?;
                self.list(|cursor| cursor.filterable(Cursor::tag))?;
                self.boolean().map(|_| ())
            }
            // `trim`: the material, then the pattern.
            56 => {
                self.holder(|cursor| {
                    cursor.string()?;
                    cursor.list(|cursor| {
                        cursor.identifier()?;
                        cursor.string()
                    })?;
                    cursor.tag()
                })?;
                self.holder(|cursor| {
                    cursor.identifier()?;
                    cursor.tag()?;
                    cursor.boolean().map(|_| ())
                })
            }
            // Entity and block entity data: a type and a compound.
            58 | 60 => {
                self.varint()?;
                self.compound_tag()
            }
            59 => self.compound_tag(),
            61 => self.holder(|cursor| {
                cursor.sound()?;
                cursor.float()?;
                cursor.float()?;
                cursor.tag()
            }),
            62 => self.holder(|cursor| {
                cursor.string()?;
                cursor.list(|cursor| {
                    cursor.identifier()?;
                    cursor.string()
                })?;
                cursor.tag()
            }),
            64 => self.holder(|cursor| {
                cursor.sound()?;
                cursor.tag()?;
                cursor.float()?;
                cursor.varint().map(|_| ())
            }),
            // `lodestone_tracker`: a dimension and a position, and a flag.
            67 => {
                self.optional(|cursor| {
                    cursor.identifier()?;
                    cursor.long()
                })?;
                self.boolean().map(|_| ())
            }
            68 => self.firework_explosion(),
            69 => {
                self.varint()?;
                self.list(Cursor::firework_explosion)
            }
            70 => {
                if self.boolean()? {
                    self.uuid()?;
                    self.string()?;
                    self.game_profile_properties()?;
                } else {
                    self.optional(Cursor::string)?;
                    self.optional(Cursor::uuid)?;
                    self.game_profile_properties()?;
                }
                self.optional(Cursor::identifier)?;
                self.optional(Cursor::identifier)?;
                self.optional(Cursor::identifier)?;
                self.optional(|cursor| cursor.boolean().map(|_| ()))
            }
            72 => self.list(|cursor| {
                cursor.holder(|cursor| {
                    cursor.identifier()?;
                    cursor.string()
                })?;
                cursor.varint().map(|_| ())
            }),
            74 => self.list(|cursor| cursor.varint().map(|_| ())),
            75 => self.list(|cursor| cursor.optional(Cursor::template)),
            76 => self.list(|cursor| {
                cursor.string()?;
                cursor.string()
            }),
            77 => self.list(|cursor| {
                cursor.varint()?;
                cursor.compound_tag()?;
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            81 => self.sound(),
            // The painting: a registry id plus one, or the variant itself.
            103 => self.holder(|cursor| {
                cursor.varint()?;
                cursor.varint()?;
                cursor.identifier()?;
                cursor.optional(Cursor::tag)?;
                cursor.optional(Cursor::tag)
            }),
            _ => Err(Malformed),
        }
    }

    fn firework_explosion(&mut self) -> R {
        self.varint()?;
        self.list(Cursor::int)?;
        self.list(Cursor::int)?;
        self.skip(2)
    }
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
    fn every_creative_stack_reads_whole() {
        for stack in crate::creative_tabs_table::CREATIVE_STACKS.iter() {
            let mut input = stack.patch;
            let patch = read_patch(&mut input).expect("the reference's own patch reads");
            assert!(input.is_empty());
            let ids: Vec<i32> = patch.added.iter().map(|(id, _)| *id).collect();
            assert_eq!(ids, stack.positive);
            assert_eq!(patch.removed, stack.negative);
            assert_eq!(patch.trusted(), stack.patch);
        }
    }

    #[test]
    fn the_empty_patch_reads_as_two_zeros() {
        let mut input = &[0_u8, 0][..];
        let patch = read_patch(&mut input).expect("a patch");
        assert!(patch.added.is_empty() && patch.removed.is_empty());
        assert_eq!(delimit(&[0, 0]), Some(vec![0, 0]));
    }
}
