//! `AdventureModePredicate` (`can_break`, `can_place_on`): the blocks an adventure player's stack
//! may break or be placed on.
//!
//! A predicate that tests block entity data (`nbt`, data components) never matches here: the
//! client keeps no block entity data to test it against.

use super::*;
use photon_core::ids::BlockStateId;

/// The blocks a predicate names: a tag, or registry ids.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Blocks {
    Tag(String),
    Ids(Vec<i32>),
}

/// `StatePropertiesPredicate.ValueMatcher`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Matcher {
    Exact(String),
    Range(Option<String>, Option<String>),
}

/// One `BlockPredicate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPredicate {
    blocks: Option<Blocks>,
    properties: Vec<(String, Matcher)>,
    /// It tests block entity data, which this does not have.
    needs_entity: bool,
}

impl Cursor<'_> {
    fn text(&mut self) -> R<String> {
        let length = self.count()?;
        let bytes = self.data.get(..length).ok_or(Malformed)?;
        let text = String::from_utf8_lossy(bytes).into_owned();
        self.skip(length)?;
        Ok(text)
    }

    fn block_set(&mut self) -> R<Blocks> {
        match self.varint()? {
            0 => Ok(Blocks::Tag(self.text()?)),
            count if count > 0 => (0..count - 1)
                .map(|_| self.varint())
                .collect::<R<Vec<_>>>()
                .map(Blocks::Ids),
            _ => Err(Malformed),
        }
    }

    fn state_matcher(&mut self) -> R<(String, Matcher)> {
        let name = self.text()?;
        let matcher = if self.boolean()? {
            Matcher::Exact(self.text()?)
        } else {
            let bound = |cursor: &mut Self| -> R<Option<String>> {
                if cursor.boolean()? {
                    cursor.text().map(Some)
                } else {
                    Ok(None)
                }
            };
            Matcher::Range(bound(self)?, bound(self)?)
        };
        Ok((name, matcher))
    }

    fn adventure_block_predicate(&mut self) -> R<BlockPredicate> {
        let blocks = if self.boolean()? {
            Some(self.block_set()?)
        } else {
            None
        };
        let mut properties = Vec::new();
        if self.boolean()? {
            for _ in 0..self.count()? {
                properties.push(self.state_matcher()?);
            }
        }
        let mut needs_entity = false;
        if self.boolean()? {
            needs_entity = true;
            self.compound_tag()?;
        }
        let exact = self.count()?;
        for _ in 0..exact {
            let id = self.varint()?;
            self.component(id)?;
        }
        needs_entity |= exact != 0;
        if self.count()? != 0 {
            // The partial component predicates are not read, so what follows is lost.
            return Err(Malformed);
        }
        Ok(BlockPredicate {
            blocks,
            properties,
            needs_entity,
        })
    }
}

/// The predicates of a `can_break` or `can_place_on` value; none when it cannot be read.
pub fn adventure_predicate(value: &[u8]) -> Option<Vec<BlockPredicate>> {
    let mut cursor = Cursor { data: value };
    let count = cursor.count().ok()?;
    (0..count)
        .map(|_| cursor.adventure_block_predicate())
        .collect::<R<Vec<_>>>()
        .ok()
}

/// What `AdventureModePredicate.addToTooltip` lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdventureTooltip {
    /// A predicate names no blocks: `item.canUse.unknown`.
    Unknown,
    /// The distinct block registry ids, in order.
    Blocks(Vec<i32>),
    /// A predicate names a tag, whose members this does not list.
    Tagged,
}

/// The tooltip of a `can_break` or `can_place_on` value; none when it cannot be read.
pub fn adventure_tooltip(value: &[u8]) -> Option<AdventureTooltip> {
    let predicates = adventure_predicate(value)?;
    let mut ids = Vec::new();
    let mut tagged = false;
    for predicate in &predicates {
        match &predicate.blocks {
            None => return Some(AdventureTooltip::Unknown),
            Some(Blocks::Tag(_)) => tagged = true,
            Some(Blocks::Ids(found)) => {
                for id in found {
                    if !ids.contains(id) {
                        ids.push(*id);
                    }
                }
            }
        }
    }
    Some(if tagged {
        AdventureTooltip::Tagged
    } else {
        AdventureTooltip::Blocks(ids)
    })
}

impl BlockPredicate {
    /// `BlockPredicate.matches(BlockInWorld)` for a block with no entity data.
    fn matches(&self, state: BlockStateId, in_tag: &impl Fn(&str) -> bool) -> bool {
        let Some((block, offset)) = photon_data::block::registry::block_for_state(state) else {
            return false;
        };
        let Some((index, _)) = photon_data::block::registry::block_index_for_state(state) else {
            return false;
        };
        if self.needs_entity {
            return false;
        }
        let named = match &self.blocks {
            None => true,
            Some(Blocks::Tag(tag)) => in_tag(tag),
            Some(Blocks::Ids(ids)) => ids.contains(&(index as i32)),
        };
        named
            && self.properties.iter().all(|(name, matcher)| {
                let Some(property) = block
                    .properties
                    .iter()
                    .find(|property| property.name == *name)
                else {
                    return false;
                };
                let current = block
                    .properties_at(offset)
                    .and_then(|all| all.into_iter().find(|(key, _)| key == name))
                    .map(|(_, value)| value);
                current.is_some_and(|value| matcher.accepts(property.values, value))
            })
    }
}

impl Matcher {
    /// Whether the value passes, by its place among the property's values.
    fn accepts(&self, values: &[&str], current: &str) -> bool {
        let place = |text: &str| values.iter().position(|value| *value == text);
        match self {
            Self::Exact(wanted) => place(wanted).is_some() && wanted == current,
            Self::Range(min, max) => {
                let Some(at) = place(current) else {
                    return false;
                };
                let low = min
                    .as_deref()
                    .is_none_or(|text| place(text).is_some_and(|edge| at >= edge));
                let high = max
                    .as_deref()
                    .is_none_or(|text| place(text).is_some_and(|edge| at <= edge));
                low && high
            }
        }
    }
}

/// `AdventureModePredicate.test`: any predicate matches the block state.
pub fn adventure_matches(
    predicates: &[BlockPredicate],
    state: BlockStateId,
    in_tag: impl Fn(&str) -> bool,
) -> bool {
    predicates
        .iter()
        .any(|predicate| predicate.matches(state, &in_tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A predicate list with one entry naming the given block ids and state properties.
    fn encoded(ids: &[i32], properties: &[(&str, &[u8])]) -> Vec<u8> {
        fn text(out: &mut Vec<u8>, value: &str) {
            out.push(value.len() as u8);
            out.extend_from_slice(value.as_bytes());
        }
        let mut out = vec![1, 1, ids.len() as u8 + 1];
        for id in ids {
            photon_core::wire::write_varint(*id, &mut out);
        }
        if properties.is_empty() {
            out.push(0);
        } else {
            out.extend([1, properties.len() as u8]);
            for (name, matcher) in properties {
                text(&mut out, name);
                out.extend_from_slice(matcher);
            }
        }
        // No nbt, no exact components, no partial ones.
        out.extend([0, 0, 0]);
        out
    }

    fn state(name: &str) -> BlockStateId {
        photon_data::block::registry::BLOCKS
            .iter()
            .find(|block| block.name == name)
            .unwrap()
            .default_state_id()
    }

    fn index(name: &str) -> i32 {
        photon_data::block::registry::BLOCKS
            .iter()
            .position(|block| block.name == name)
            .unwrap() as i32
    }

    #[test]
    fn a_predicate_matches_the_blocks_it_names() {
        let value = encoded(&[index("minecraft:stone")], &[]);
        let all = adventure_predicate(&value).unwrap();
        assert!(adventure_matches(&all, state("minecraft:stone"), |_| false));
        assert!(!adventure_matches(&all, state("minecraft:dirt"), |_| false));
    }

    #[test]
    fn a_predicate_on_a_state_property_reads_the_block_state() {
        let mut exact = vec![1, 4];
        exact.extend_from_slice(b"true");
        let value = encoded(&[index("minecraft:lever")], &[("powered", &exact)]);
        let all = adventure_predicate(&value).unwrap();
        // The default lever is not powered.
        assert!(!adventure_matches(&all, state("minecraft:lever"), |_| {
            false
        }));
        let mut off = vec![1, 5];
        off.extend_from_slice(b"false");
        let value = encoded(&[index("minecraft:lever")], &[("powered", &off)]);
        let all = adventure_predicate(&value).unwrap();
        assert!(adventure_matches(&all, state("minecraft:lever"), |_| false));
        // A range from 1 up leaves out a cake with no bite taken; from 0 up it holds it.
        for (range, matches) in [([0, 1, 1, b'1', 0], false), ([0, 1, 1, b'0', 0], true)] {
            let value = encoded(&[index("minecraft:cake")], &[("bites", &range)]);
            let all = adventure_predicate(&value).unwrap();
            let state = state("minecraft:cake");
            assert_eq!(adventure_matches(&all, state, |_| false), matches);
        }
    }

    #[test]
    fn nbt_predicates_never_match_without_a_block_entity() {
        let mut value = vec![1, 1, 2, index("minecraft:stone") as u8, 0, 1];
        value.extend_from_slice(&[10, 0]);
        value.extend([0, 0]);
        let all = adventure_predicate(&value).unwrap();
        assert!(!adventure_matches(&all, state("minecraft:stone"), |_| {
            false
        }));
    }
}
