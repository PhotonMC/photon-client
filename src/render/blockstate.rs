//! Blockstate files: which model a block state draws, and how it is turned.
//!
//! A block does not name a model directly. A blockstate file either lists
//! named selectors against models (`variants`) or lists pieces that each
//! apply when a condition holds (`multipart`), and a file may carry both.
//!
//! Only reading the file is here. Turning a selector or a condition into a
//! test against a real block state needs that block's state definition, which
//! needs the registry, and is not built yet.

use std::collections::BTreeMap;

use crate::identifier::Identifier;
use crate::json::{JsonObject, JsonValue};

/// A quarter turn, as a variant names it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Quadrant {
    #[default]
    R0,
    R90,
    R180,
    R270,
}

impl Quadrant {
    /// The quarter turn an angle in degrees names.
    ///
    /// The angle is taken modulo a full turn first, so `450` and `-270` both
    /// land on a quarter turn; anything that is not a multiple of ninety is
    /// refused rather than rounded.
    pub fn from_degrees(degrees: i32) -> Option<Self> {
        match crate::math::positive_modulo_i32(degrees, 360) {
            0 => Some(Self::R0),
            90 => Some(Self::R90),
            180 => Some(Self::R180),
            270 => Some(Self::R270),
            _ => None,
        }
    }

    /// The name the reference's enum carries.
    pub fn name(self) -> &'static str {
        match self {
            Self::R0 => "R0",
            Self::R90 => "R90",
            Self::R180 => "R180",
            Self::R270 => "R270",
        }
    }

    /// The angle in degrees.
    pub fn degrees(self) -> i32 {
        match self {
            Self::R0 => 0,
            Self::R90 => 90,
            Self::R180 => 180,
            Self::R270 => 270,
        }
    }
}

/// One model a state may draw, with the turn applied to it.
#[derive(Clone, PartialEq, Debug)]
pub struct Variant {
    /// The model to draw.
    pub model: Identifier,
    /// Quarter turn about each axis.
    pub x: Quadrant,
    pub y: Quadrant,
    pub z: Quadrant,
    /// Whether the texture stays put while the model turns.
    pub uvlock: bool,
    /// How often this one is picked out of the list it belongs to.
    pub weight: i32,
    /// Whether it was written inside a list. A list, even of one, is drawn
    /// from with the block's random source; a lone variant is not.
    pub listed: bool,
}

/// What a multipart piece tests before it applies.
#[derive(Clone, PartialEq, Debug)]
pub enum Condition {
    /// Every listed property must hold one of its listed values.
    KeyValue(BTreeMap<String, Vec<Term>>),
    /// Every term, or any term, depending on the operation.
    Combined {
        /// `AND` or `OR`.
        operation: Operation,
        /// The terms, which may be empty.
        terms: Vec<Condition>,
    },
}

/// How a combined condition folds its terms.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operation {
    And,
    Or,
}

impl Operation {
    /// The name the file spells, which is upper case.
    pub fn name(self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
        }
    }
}

/// One accepted value for a property, possibly negated.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Term {
    /// The value's text.
    pub value: String,
    /// Whether the property must *not* hold it.
    pub negated: bool,
}

/// One multipart piece.
#[derive(Clone, PartialEq, Debug)]
pub struct Selector {
    /// What must hold, or nothing for a piece that always applies.
    pub condition: Option<Condition>,
    /// The models the piece draws.
    pub apply: Vec<Variant>,
}

/// A whole blockstate file.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct BlockStateFile {
    /// Selector string to the models it draws.
    pub variants: Option<BTreeMap<String, Vec<Variant>>>,
    /// Pieces that each apply when their condition holds.
    pub multipart: Option<Vec<Selector>>,
}

/// Read one blockstate file.
///
/// Returns nothing when the file is refused. A file with neither section is
/// refused, and so is an empty `variants` map or an empty `multipart` list —
/// a file that draws nothing is treated as a mistake rather than as a block
/// with no appearance.
pub fn parse_block_state(text: &str) -> Option<BlockStateFile> {
    let JsonValue::Object(root) = crate::json::parse(text).ok()? else {
        return None;
    };
    let mut file = BlockStateFile::default();
    if root.has("variants") {
        let object = root.object("variants")?;
        let mut variants = BTreeMap::new();
        for (key, value) in object.entries() {
            variants.insert(String::from_utf16(key).ok()?, parse_variants(value)?);
        }
        if variants.is_empty() {
            return None;
        }
        file.variants = Some(variants);
    }
    if root.has("multipart") {
        let JsonValue::Array(items) = root.get_str("multipart")? else {
            return None;
        };
        if items.is_empty() {
            return None;
        }
        let mut selectors = Vec::with_capacity(items.len());
        for item in items {
            let JsonValue::Object(piece) = item else {
                return None;
            };
            let condition = if piece.has("when") {
                Some(parse_condition(piece.get_str("when")?)?)
            } else {
                None
            };
            selectors.push(Selector {
                condition,
                apply: parse_variants(piece.get_str("apply")?)?,
            });
        }
        file.multipart = Some(selectors);
    }
    if file.variants.is_none() && file.multipart.is_none() {
        return None;
    }
    Some(file)
}

/// One variant, or a non-empty list of weighted ones.
fn parse_variants(value: &JsonValue) -> Option<Vec<Variant>> {
    match value {
        JsonValue::Object(object) => Some(vec![parse_variant(object, 1, false)?]),
        JsonValue::Array(items) => {
            if items.is_empty() {
                return None;
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                let JsonValue::Object(object) = item else {
                    return None;
                };
                // A weight has to be at least one; zero would make the entry
                // unreachable and the reference refuses it outright.
                let weight = if object.has("weight") {
                    let weight = object.i32("weight")?;
                    if weight < 1 {
                        return None;
                    }
                    weight
                } else {
                    1
                };
                out.push(parse_variant(object, weight, true)?);
            }
            Some(out)
        }
        _ => None,
    }
}

fn parse_variant(object: &JsonObject, weight: i32, listed: bool) -> Option<Variant> {
    let quadrant = |name: &str| -> Option<Quadrant> {
        if object.has(name) {
            Quadrant::from_degrees(object.i32(name)?)
        } else {
            Some(Quadrant::R0)
        }
    };
    Some(Variant {
        model: Identifier::parse(&object.text("model")?).ok()?,
        x: quadrant("x")?,
        y: quadrant("y")?,
        z: quadrant("z")?,
        uvlock: if object.has("uvlock") {
            object.bool("uvlock")?
        } else {
            false
        },
        weight,
        listed,
    })
}

/// A condition is either a combiner or a set of property tests.
///
/// The reference tries the combiner first and falls back. A combiner must
/// hold exactly one operation key and nothing else: an operation next to a
/// property is refused rather than having the property quietly dropped.
fn parse_condition(value: &JsonValue) -> Option<Condition> {
    let JsonValue::Object(object) = value else {
        return None;
    };
    if object.is_empty() {
        return None;
    }
    let operations: Vec<Operation> = object
        .entries()
        .filter_map(|(key, _)| match String::from_utf16(key).ok()?.as_str() {
            "AND" => Some(Operation::And),
            "OR" => Some(Operation::Or),
            _ => None,
        })
        .collect();
    if !operations.is_empty() {
        // Anything beside the one operation makes this neither a valid
        // combiner nor a valid set of property tests.
        if operations.len() != 1 || object.entries().count() != 1 {
            return None;
        }
        let operation = operations[0];
        let JsonValue::Array(items) = object.get_str(operation.name())? else {
            return None;
        };
        let mut terms = Vec::with_capacity(items.len());
        for item in items {
            terms.push(parse_condition(item)?);
        }
        return Some(Condition::Combined { operation, terms });
    }
    let mut tests = BTreeMap::new();
    for (key, value) in object.entries() {
        tests.insert(String::from_utf16(key).ok()?, parse_terms(value)?);
    }
    Some(Condition::KeyValue(tests))
}

/// The accepted values for one property, split on the bar.
///
/// A value may be given as text, as a number or as a boolean; the last two
/// are taken as their text, and a number is truncated on the way. An empty
/// piece anywhere in the bar-separated list refuses the whole term.
fn parse_terms(value: &JsonValue) -> Option<Vec<Term>> {
    let text = match value {
        JsonValue::String(units) => String::from_utf16(units).ok()?,
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(_) => value.as_i32()?.to_string(),
        _ => return None,
    };
    let terms: Vec<Term> = text
        .split('|')
        .map(|piece| match piece.strip_prefix('!') {
            Some(rest) => Term {
                value: rest.to_owned(),
                negated: true,
            },
            None => Term {
                value: piece.to_owned(),
                negated: false,
            },
        })
        .collect();
    if terms.iter().any(|term| term.value.is_empty()) {
        return None;
    }
    Some(terms)
}

/// Read a `key=value,key=value` property string into a map.
///
/// The reference splits on the comma and then on the first equals sign, drops
/// an empty key, and refuses a key it does not recognise or a value the
/// property cannot hold. Those two refusals need the block's state
/// definition, so they are not done here — this is the shape of the
/// reference's parse without the part that needs the registry.
pub fn parse_properties(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for pair in text.split(',') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        out.insert(key.to_owned(), value.to_owned());
    }
    out
}

/// Whether a state holds every property a selector string names.
///
/// See [`parse_properties`] for what this does not check.
pub fn selector_matches(selector: &str, state: &BTreeMap<String, String>) -> bool {
    parse_properties(selector)
        .iter()
        .all(|(key, value)| state.get(key) == Some(value))
}

impl Condition {
    /// Whether the condition holds for a state.
    ///
    /// A property the state does not carry fails the test rather than
    /// throwing, which is where this departs from the reference: there an
    /// unknown property is a load error, caught and logged per selector.
    pub fn matches(&self, state: &BTreeMap<String, String>) -> bool {
        match self {
            Self::KeyValue(tests) => tests.iter().all(|(property, terms)| {
                let Some(actual) = state.get(property) else {
                    return false;
                };
                terms
                    .iter()
                    .any(|term| (term.value == *actual) != term.negated)
            }),
            Self::Combined { operation, terms } => match operation {
                Operation::And => terms.iter().all(|term| term.matches(state)),
                Operation::Or => terms.iter().any(|term| term.matches(state)),
            },
        }
    }
}

impl BlockStateFile {
    /// The weighted lists a state draws from, in the order the reference
    /// asks them: the first matching `variants` selector, then every
    /// `multipart` piece whose condition holds.
    pub fn matched_lists(&self, state: &BTreeMap<String, String>) -> Vec<Vec<Variant>> {
        let mut out = Vec::new();
        if let Some((_, chosen)) = self.variants.as_ref().and_then(|variants| {
            variants
                .iter()
                .find(|(selector, _)| selector_matches(selector, state))
        }) {
            out.push(chosen.clone());
        }
        if let Some(multipart) = &self.multipart {
            for selector in multipart {
                let holds = selector
                    .condition
                    .as_ref()
                    .is_none_or(|condition| condition.matches(state));
                if holds {
                    out.push(selector.apply.clone());
                }
            }
        }
        out
    }

    /// Pick one variant from each list with the block's random source: a
    /// lone variant written outside a list takes no draw, anything else
    /// draws by weight.
    pub fn pick_variants(
        lists: &[Vec<Variant>],
        random: &mut crate::random::RandomSource,
    ) -> Vec<Variant> {
        let mut out = Vec::with_capacity(lists.len());
        for list in lists {
            let picked = match list.as_slice() {
                [] => None,
                [only] if !only.listed => Some(only.clone()),
                _ => {
                    let total: i32 = list.iter().map(|variant| variant.weight).sum();
                    random.next_int_bound(total).ok().and_then(|mut selection| {
                        list.iter()
                            .find(|variant| {
                                selection -= variant.weight;
                                selection < 0
                            })
                            .cloned()
                    })
                }
            };
            out.extend(picked);
        }
        out
    }

    /// Every variant a state draws, picked as the reference picks them: the
    /// block's random source, seeded from its position, draws once from each
    /// weighted list in turn.
    pub fn choose_variants(
        &self,
        state: &BTreeMap<String, String>,
        random: &mut crate::random::RandomSource,
    ) -> Vec<Variant> {
        Self::pick_variants(&self.matched_lists(state), random)
    }

    /// Every variant a state draws.
    ///
    /// The `variants` section contributes the first selector that matches,
    /// and the `multipart` section contributes every piece whose condition
    /// holds. Each contributes the first entry of its weighted list, because
    /// picking among them needs the reference's own random source.
    pub fn variants_for(&self, state: &BTreeMap<String, String>) -> Vec<Variant> {
        let mut out = Vec::new();
        if let Some((_, chosen)) = self.variants.as_ref().and_then(|variants| {
            variants
                .iter()
                .find(|(selector, _)| selector_matches(selector, state))
        }) {
            out.extend(chosen.first().cloned());
        }
        if let Some(multipart) = &self.multipart {
            for selector in multipart {
                let holds = selector
                    .condition
                    .as_ref()
                    .is_none_or(|condition| condition.matches(state));
                if holds {
                    out.extend(selector.apply.first().cloned());
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod choice_tests {
    use super::*;

    #[test]
    fn a_list_draws_from_the_random_source_and_a_lone_variant_does_not() {
        let file = parse_block_state(
            r#"{"variants": {"": [{"model": "a"}, {"model": "b", "weight": 3}]}}"#,
        )
        .expect("a list");
        let state = BTreeMap::new();
        let mut random = crate::random::RandomSource::create(7);
        let mut expected = crate::random::RandomSource::create(7);
        let chosen = file.choose_variants(&state, &mut random);
        let selection = expected.next_int_bound(4).expect("bound");
        let wanted = if selection < 1 {
            "minecraft:a"
        } else {
            "minecraft:b"
        };
        assert_eq!(chosen[0].model.as_string(), wanted);
        assert_eq!(random.next_int(), expected.next_int());

        let lone = parse_block_state(r#"{"variants": {"": {"model": "c"}}}"#).expect("one");
        let mut untouched = crate::random::RandomSource::create(7);
        lone.choose_variants(&state, &mut untouched);
        assert_eq!(
            untouched.next_int(),
            crate::random::RandomSource::create(7).next_int()
        );
    }
}
