//! The `minecraft:profile` component of a stack, as the wire carries it: what a player head item
//! shows the skin of.

use super::dynamic::Resolvable;
use super::profile::resolvable;
use photon_core::binary::Uuid;
use photon_net::skins::ProfileProperty;
use photon_world::ClientItemStackState;

/// A reader over the component's bytes; every read gives `None` once they run out.
struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take(&mut self, count: usize) -> Option<&[u8]> {
        let (head, rest) = self.0.split_at_checked(count)?;
        self.0 = rest;
        Some(head)
    }

    fn boolean(&mut self) -> Option<bool> {
        Some(self.take(1)?[0] != 0)
    }

    fn varint(&mut self) -> Option<usize> {
        let mut value = 0usize;
        for shift in (0..35).step_by(7) {
            let byte = self.take(1)?[0];
            value |= usize::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }

    fn string(&mut self) -> Option<String> {
        let length = self.varint()?;
        Some(String::from_utf8_lossy(self.take(length)?).into_owned())
    }

    fn uuid(&mut self) -> Option<Uuid> {
        let bytes = self.take(16)?;
        let half = |bytes: &[u8]| u64::from_be_bytes(bytes.try_into().unwrap_or_default());
        Some(Uuid::new(half(&bytes[..8]), half(&bytes[8..])))
    }

    fn properties(&mut self) -> Option<Vec<ProfileProperty>> {
        (0..self.varint()?)
            .map(|_| {
                Some(ProfileProperty {
                    name: self.string()?,
                    value: self.string()?,
                    signature: if self.boolean()? {
                        Some(self.string()?)
                    } else {
                        None
                    },
                })
            })
            .collect()
    }
}

/// `ResolvableProfile.STREAM_CODEC`: a complete game profile (id, name, properties), or the
/// partial one of a name, an id and properties each of which may be missing.
fn read_profile(bytes: &[u8]) -> Option<Resolvable> {
    let mut reader = Reader(bytes);
    if reader.boolean()? {
        let id = reader.uuid()?;
        reader.string()?;
        return Some(Resolvable::Static(super::profile::StaticProfile {
            id,
            properties: reader.properties()?,
        }));
    }
    let name = if reader.boolean()? {
        Some(reader.string()?)
    } else {
        None
    };
    let id = if reader.boolean()? {
        Some(reader.uuid()?)
    } else {
        None
    };
    Some(resolvable(name, id, reader.properties()?))
}

/// A `ResolvableProfile` as an entity's data carries it (a mannequin's), from the start of
/// `bytes`.
pub(crate) fn wire_profile(bytes: &[u8]) -> Option<Resolvable> {
    read_profile(bytes)
}

/// The profile a stack carries when it names its skin itself.
pub(crate) fn stack_profile(stack: &ClientItemStackState) -> Option<Resolvable> {
    let values =
        photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
    read_profile(values.raw(photon_data::item::component_types::PROFILE)?)
}

#[cfg(test)]
mod tests {
    use super::super::dynamic::Dynamic;
    use super::*;

    fn string(out: &mut Vec<u8>, text: &str) {
        out.push(text.len() as u8);
        out.extend(text.as_bytes());
    }

    #[test]
    fn a_complete_profile_gives_its_id_and_properties() {
        let mut bytes = vec![1];
        bytes.extend(1u64.to_be_bytes());
        bytes.extend(2u64.to_be_bytes());
        string(&mut bytes, "Steve");
        bytes.push(1);
        string(&mut bytes, "textures");
        string(&mut bytes, "abc");
        bytes.push(0);
        let Some(Resolvable::Static(found)) = read_profile(&bytes) else {
            panic!("a complete profile is static");
        };
        assert_eq!(found.id, Uuid::new(1, 2));
        assert_eq!(found.properties[0].value, "abc");
        assert_eq!(found.properties[0].signature, None);
    }

    #[test]
    fn a_name_alone_is_looked_up() {
        let mut bytes = vec![0, 1];
        string(&mut bytes, "Steve");
        bytes.extend([0, 0]);
        assert_eq!(
            read_profile(&bytes),
            Some(Resolvable::Dynamic(Dynamic::Name("Steve".into())))
        );
    }

    #[test]
    fn an_id_alone_is_looked_up_and_both_are_not() {
        let mut bytes = vec![0, 0, 1];
        bytes.extend(3u64.to_be_bytes());
        bytes.extend(4u64.to_be_bytes());
        bytes.push(0);
        assert_eq!(
            read_profile(&bytes),
            Some(Resolvable::Dynamic(Dynamic::Id(Uuid::new(3, 4))))
        );
        let mut both = vec![0, 1];
        string(&mut both, "Steve");
        both.push(1);
        both.extend(3u64.to_be_bytes());
        both.extend(4u64.to_be_bytes());
        both.push(0);
        assert!(matches!(read_profile(&both), Some(Resolvable::Static(_))));
    }

    #[test]
    fn a_cut_off_value_reads_as_none() {
        assert_eq!(read_profile(&[1, 0, 0]), None);
    }
}
