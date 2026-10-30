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
        photon_core::wire::read_varint(&mut self.data).map_err(|_| Malformed)
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

    /// A holder set of `site`'s registry: its ids, or a tag's name.
    pub fn registry_set(&mut self, site: Site) -> R {
        match self.varint()? {
            0 => self.identifier(),
            count if count > 0 => {
                for _ in 0..count - 1 {
                    self.id(site, 0)?;
                }
                Ok(())
            }
            _ => Err(Malformed),
        }
    }

    pub fn skip_item_set(&mut self) -> R {
        self.registry_set(Site::Item)
    }

    pub fn skip_block_set(&mut self) -> R {
        self.registry_set(Site::Block)
    }

    pub fn skip_entity_type_set(&mut self) -> R {
        self.registry_set(Site::EntityType)
    }

    /// `SoundEvent.STREAM_CODEC`.
    pub fn sound(&mut self) -> R {
        if self.id(Site::SoundEvent, 1)? == 0 {
            self.identifier()?;
            self.optional(Cursor::float)?;
        }
        Ok(())
    }

    /// `ItemStackTemplate.STREAM_CODEC`: an item, a count and a patch.
    pub fn template(&mut self) -> R {
        self.id(Site::Item, 0)?;
        self.varint()?;
        self.skip_patch()
    }
}

pub(super) type R<T = ()> = Result<T, Malformed>;

/// A component value that is one `int`: `dyed_color`, `map_color`.
pub fn int_value(value: &[u8]) -> Option<i32> {
    Some(i32::from_be_bytes(value.get(..4)?.try_into().ok()?))
}

/// A component value that is one `VarInt`: a rarity.
pub fn varint_value(value: &[u8]) -> Option<i32> {
    let mut cursor = Cursor::new(value);
    cursor.varint().ok()
}

/// A patch as the trusted codec wrote it, sent back as the delimited codec
/// writes it. `None` when the patch is not one this reads.
pub fn delimit(trusted: &[u8], numbering: &Numbering<'_>) -> Option<Vec<u8>> {
    let mut input = trusted;
    let patch = read_patch(&mut input).ok()?;
    input
        .is_empty()
        .then(|| patch.delimited_for(numbering))
        .flatten()
}

/// The registries whose ids a component value carries as numbers, which a server numbers by its
/// version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Site {
    ComponentType,
    Attribute,
    SoundEvent,
    Item,
    Block,
    EntityType,
    BlockEntityType,
}

/// The ids of a version translated as a cursor reads: what it has passed, with each id site
/// changed, is gathered in `out`, which is complete up to `mark`.
struct Rewrite<'a> {
    map: &'a dyn Fn(Site, i32) -> i32,
    out: Vec<u8>,
    mark: &'a [u8],
}

pub struct Cursor<'a> {
    pub data: &'a [u8],
    rewrite: Option<Rewrite<'a>>,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            rewrite: None,
        }
    }

    /// A cursor that renumbers the ids it reads with `map`; [`Self::finish`] gives the bytes.
    pub fn rewriting(data: &'a [u8], map: &'a dyn Fn(Site, i32) -> i32) -> Self {
        Self {
            data,
            rewrite: Some(Rewrite {
                map,
                out: Vec::new(),
                mark: data,
            }),
        }
    }

    /// The bytes read so far, renumbered.
    pub fn finish(self) -> Vec<u8> {
        self.rewrite.map_or_else(Vec::new, |mut rewrite| {
            let kept = rewrite.mark.len() - self.data.len();
            rewrite.out.extend_from_slice(&rewrite.mark[..kept]);
            rewrite.out
        })
    }

    /// A registry id of `site`, read as a varint; with a rewrite, the renumbered one is the
    /// value and is what is written. `bias` is what the codec adds to an id (a holder's id is
    /// one more, zero meaning a direct value); a value below it is not an id.
    pub fn id(&mut self, site: Site, bias: i32) -> R<i32> {
        let before = self.data;
        let value = self.varint()?;
        let Some(rewrite) = &mut self.rewrite else {
            return Ok(value);
        };
        let mapped = if value >= bias {
            (rewrite.map)(site, value - bias).saturating_add(bias)
        } else {
            value
        };
        rewrite
            .out
            .extend_from_slice(&rewrite.mark[..rewrite.mark.len() - before.len()]);
        photon_core::wire::write_varint(mapped, &mut rewrite.out);
        rewrite.mark = self.data;
        Ok(mapped)
    }

    /// What `read` passes over, as the bytes it was (renumbered, with a rewrite).
    pub fn captured(&mut self, read: impl FnOnce(&mut Self) -> R) -> R<Vec<u8>> {
        let start = self.data;
        if let Some(rewrite) = &mut self.rewrite {
            rewrite.out = Vec::new();
            rewrite.mark = start;
        }
        read(self)?;
        match &mut self.rewrite {
            None => Ok(start[..start.len() - self.data.len()].to_vec()),
            Some(rewrite) => {
                let kept = rewrite.mark.len() - self.data.len();
                rewrite.out.extend_from_slice(&rewrite.mark[..kept]);
                rewrite.mark = self.data;
                Ok(std::mem::take(&mut rewrite.out))
            }
        }
    }
}
