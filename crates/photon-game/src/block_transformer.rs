//! `BlockTransformer`, the `block_transformer` item component's registry entry (26.3): the
//! blocks a tool turns into others, read from the data pack or from the entries a server sends,
//! and what deciding a transform takes from the level.
//!
//! The entry's state provider and block predicates are the generic ones of world generation. The
//! client decides with the ones the tools' entries use (a state, `rule_based`, `copy_properties`;
//! `matching_blocks`, `matching_block_tag`, `all_of`, `any_of`, `not`, `true`); any other kind
//! reads as [`Unported`] when a transform reaches it, so no prediction is made for it.

mod parse;
#[cfg(test)]
mod tests;

pub use parse::{from_json, from_nbt};

use photon_core::ids::BlockStateId;
use photon_world::ClientWorld;
use photon_world::block::named_state::with_properties_of;

/// The registry the entries belong to, as the server names it.
pub const REGISTRY: &str = concat!("minecraft:", "block_transformer");

/// A provider or predicate of a kind the client does not decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unported;

/// A position in the level.
pub type Pos = (i32, i32, i32);

/// `BlockTransformer.TransformParticle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Particle {
    None,
    Scrape,
    WaxOn,
    WaxOff,
}

impl Particle {
    /// The level event the particle sends, none for no particle.
    pub fn level_event(self) -> Option<i32> {
        match self {
            Self::None => None,
            Self::Scrape => Some(3005),
            Self::WaxOn => Some(3003),
            Self::WaxOff => Some(3004),
        }
    }
}

/// The blocks a `matching_blocks` predicate names: a list, or a tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockSet {
    Names(Vec<String>),
    Tag(String),
}

/// `BlockPredicate`, as far as the client decides it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    True,
    AllOf(Vec<Predicate>),
    AnyOf(Vec<Predicate>),
    Not(Box<Predicate>),
    Blocks { offset: Pos, blocks: BlockSet },
    Tag { offset: Pos, tag: String },
    Unsupported,
}

/// `BlockStateProvider`, as far as the client decides it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    /// A state, offered whatever the level holds.
    State(BlockStateId),
    /// The source's state with the properties of the block it replaces.
    CopyProperties(Box<Provider>),
    /// The first rule whose predicate holds and whose provider offers a state, else the fallback.
    Rules {
        fallback: Option<Box<Provider>>,
        rules: Vec<(Predicate, Provider)>,
    },
    Unsupported,
}

/// One entry of the transformer: `BlockTransformer.BlockTransformData`, without the loot and
/// the item damage, which the server applies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transform {
    pub provider: Provider,
    /// The sound event, by its name; none for the empty event or one the client lacks.
    pub sound: Option<&'static str>,
    pub particle: Particle,
    /// Directions as the protocol numbers them (`Direction.get3DDataValue`).
    pub disallowed_faces: Vec<u8>,
    pub update_from_neighbors: bool,
    /// The `copper_chest` transform type: a double chest's other half is told too.
    pub copper_chest: bool,
    pub consume_on_use: bool,
}

/// A registry entry: its name, and its transforms unless the entry could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transformer {
    pub name: String,
    pub transforms: Option<Vec<Transform>>,
}

impl Transformer {
    /// The entry of the data pack at `data_root` with this name: its transforms are those of the
    /// file, none when the pack has no such file or it does not read.
    pub fn from_pack(data_root: &std::path::Path, name: String) -> Self {
        let file = crate::registries::entry_path(data_root, "block_transformer", &name);
        let transforms = std::fs::read_to_string(file)
            .ok()
            .and_then(|text| photon_core::json::parse(&text).ok())
            .and_then(|value| from_json(&value));
        Self { name, transforms }
    }
}

fn block_name(state: BlockStateId) -> Option<&'static str> {
    photon_data::block::registry::block_for_state(state).map(|(block, _)| block.name)
}

/// The block state `offset` away from `origin`.
fn state_at(world: &ClientWorld, origin: Pos, offset: Pos) -> BlockStateId {
    world.block_state_or_air(
        origin.0 + offset.0,
        origin.1 + offset.1,
        origin.2 + offset.2,
    )
}

impl BlockSet {
    fn holds(&self, world: &ClientWorld, state: BlockStateId) -> bool {
        match self {
            Self::Names(names) => {
                block_name(state).is_some_and(|own| names.iter().any(|name| name == own))
            }
            Self::Tag(tag) => world.block_state_registry().is_state_in_tag(state, tag),
        }
    }
}

impl Predicate {
    /// `BlockPredicate.test(level, origin)`.
    pub fn test(&self, world: &ClientWorld, origin: Pos) -> Result<bool, Unported> {
        match self {
            Self::True => Ok(true),
            Self::AllOf(all) => Self::all_hold(all, world, origin),
            Self::AnyOf(any) => Self::any_holds(any, world, origin),
            Self::Not(inner) => inner.test(world, origin).map(|held| !held),
            Self::Blocks { offset, blocks } => {
                Ok(blocks.holds(world, state_at(world, origin, *offset)))
            }
            Self::Tag { offset, tag } => Ok(world
                .block_state_registry()
                .is_state_in_tag(state_at(world, origin, *offset), tag)),
            Self::Unsupported => Err(Unported),
        }
    }

    /// `AllOfPredicate`: stops at the first that fails, so a later unsupported one is not asked.
    fn all_hold(all: &[Self], world: &ClientWorld, origin: Pos) -> Result<bool, Unported> {
        for predicate in all {
            if !predicate.test(world, origin)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// `AnyOfPredicate`: stops at the first that holds.
    fn any_holds(any: &[Self], world: &ClientWorld, origin: Pos) -> Result<bool, Unported> {
        for predicate in any {
            if predicate.test(world, origin)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Provider {
    /// `BlockStateProvider.getOptionalState`: the state offered at `pos`, `None` for none.
    pub fn optional_state(
        &self,
        world: &ClientWorld,
        pos: Pos,
    ) -> Result<Option<BlockStateId>, Unported> {
        match self {
            Self::State(state) => Ok(Some(*state)),
            Self::CopyProperties(source) => {
                // `getState` of the source: a rule provider with nothing to offer gives the
                // block that is there.
                let current = world.block_state_or_air(pos.0, pos.1, pos.2);
                let state = source.optional_state(world, pos)?.unwrap_or(current);
                Ok(Some(with_properties_of(state, current)))
            }
            Self::Rules { fallback, rules } => {
                for (predicate, then) in rules {
                    if predicate.test(world, pos)?
                        && let Some(state) = then.optional_state(world, pos)?
                    {
                        return Ok(Some(state));
                    }
                }
                match fallback {
                    Some(fallback) => fallback.optional_state(world, pos),
                    None => Ok(None),
                }
            }
            Self::Unsupported => Err(Unported),
        }
    }
}

impl Transform {
    /// Whether the clicked face is one the transform leaves alone.
    pub fn disallows(&self, face: u8) -> bool {
        self.disallowed_faces.contains(&face)
    }
}
