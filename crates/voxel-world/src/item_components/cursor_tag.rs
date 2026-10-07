use super::*;

impl Cursor<'_> {
    pub fn skip(&mut self, count: usize) -> R {
        if self.data.len() < count {
            return Err(Malformed);
        }
        self.data = &self.data[count..];
        Ok(())
    }

    pub fn byte(&mut self) -> R<u8> {
        let byte = *self.data.first().ok_or(Malformed)?;
        self.data = &self.data[1..];
        Ok(byte)
    }

    pub fn boolean(&mut self) -> R<bool> {
        Ok(self.byte()? != 0)
    }

    pub fn varint(&mut self) -> R<i32> {
        voxel_core::wire::read_varint(&mut self.data).map_err(|_| Malformed)
    }

    /// A count that a list or a map reads.
    pub fn count(&mut self) -> R<usize> {
        usize::try_from(self.varint()?).map_err(|_| Malformed)
    }

    pub fn float(&mut self) -> R {
        self.skip(4)
    }

    pub fn double(&mut self) -> R {
        self.skip(8)
    }

    pub fn int(&mut self) -> R {
        self.skip(4)
    }

    pub fn long(&mut self) -> R {
        self.skip(8)
    }

    pub fn string(&mut self) -> R {
        let length = self.count()?;
        self.skip(length)
    }

    pub fn identifier(&mut self) -> R {
        self.string()
    }

    pub fn uuid(&mut self) -> R {
        self.skip(16)
    }

    /// `ByteBufCodecs.optional`.
    pub fn optional(&mut self, value: impl FnOnce(&mut Self) -> R) -> R {
        if self.boolean()? { value(self) } else { Ok(()) }
    }

    /// `ByteBufCodecs.list`: a count and that many values.
    pub fn list(&mut self, mut value: impl FnMut(&mut Self) -> R) -> R {
        for _ in 0..self.count()? {
            value(self)?;
        }
        Ok(())
    }

    /// A network tag: its type, then its payload, with no name.
    pub fn tag(&mut self) -> R {
        let kind = self.byte()?;
        self.tag_payload(kind)
    }

    /// `ByteBufCodecs.COMPOUND_TAG`: a compound, or a single end byte for
    /// none.
    pub fn compound_tag(&mut self) -> R {
        match self.byte()? {
            0 => Ok(()),
            10 => self.tag_payload(10),
            _ => Err(Malformed),
        }
    }

    pub fn tag_payload(&mut self, kind: u8) -> R {
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

    pub fn short(&mut self) -> R<usize> {
        let high = self.byte()?;
        let low = self.byte()?;
        Ok(usize::from(u16::from_be_bytes([high, low])))
    }

    pub fn tag_length(&mut self) -> R<usize> {
        let bytes = [self.byte()?, self.byte()?, self.byte()?, self.byte()?];
        usize::try_from(i32::from_be_bytes(bytes)).map_err(|_| Malformed)
    }

    /// `ByteBufCodecs.holder`: a registry id plus one, or a zero and the
    /// direct value.
    pub fn holder(&mut self, direct: impl FnOnce(&mut Self) -> R) -> R {
        if self.varint()? == 0 {
            direct(self)
        } else {
            Ok(())
        }
    }

    /// `ByteBufCodecs.holderSet`: a count plus one and that many ids, or a
    /// zero and a tag's name.
    pub fn holder_set(&mut self) -> R {
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
    pub fn sound(&mut self) -> R {
        self.holder(|cursor| {
            cursor.identifier()?;
            cursor.optional(Cursor::float)
        })
    }

    pub fn patch(&mut self) -> R<Patch> {
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
    pub fn template(&mut self) -> R {
        self.varint()?;
        self.varint()?;
        self.patch().map(|_| ())
    }

    pub fn mob_effect_instance(&mut self) -> R {
        self.varint()?;
        self.mob_effect_details()
    }

    pub fn mob_effect_details(&mut self) -> R {
        self.varint()?;
        self.varint()?;
        self.skip(3)?;
        self.optional(Cursor::mob_effect_details)
    }

    pub fn consume_effect(&mut self) -> R {
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
    pub fn filterable(&mut self, mut value: impl FnMut(&mut Self) -> R) -> R {
        value(self)?;
        self.optional(&mut value)
    }

    pub fn game_profile_properties(&mut self) -> R {
        self.list(|cursor| {
            cursor.string()?;
            cursor.string()?;
            cursor.optional(Cursor::string)
        })
    }

    /// `BlockPredicate.STREAM_CODEC` of the advancement predicates.
    pub fn block_predicate(&mut self) -> R {
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
}
