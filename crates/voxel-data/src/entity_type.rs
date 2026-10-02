//! The entity type registry: each type's name, and its id (its place in the
//! list, which is what the wire carries).

use crate::generated::entity_picks::ENTITY_PICKS;
use crate::generated::entity_types::ENTITY_TYPES;

/// The name of the type with this id (`minecraft:pig`).
pub fn name(id: i32) -> Option<&'static str> {
    usize::try_from(id)
        .ok()
        .and_then(|index| ENTITY_TYPES.get(index))
        .copied()
}

/// The id of the type with this name.
pub fn id(name: &str) -> Option<i32> {
    ENTITY_TYPES
        .iter()
        .position(|candidate| *candidate == name)
        .map(|index| index as i32)
}

/// A pig or cow variant of the data pack: the textures (`entity/pig/pig_cold`
/// with its namespace) and the model family it asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityVariant {
    /// `pig` or `cow`.
    pub kind: &'static str,
    pub name: &'static str,
    pub asset: &'static str,
    pub baby_asset: &'static str,
    /// `normal`, `cold` or `warm`.
    pub model: &'static str,
}

/// The variant of that kind with this name (`minecraft:cold`).
pub fn variant(kind: &str, name: &str) -> Option<&'static EntityVariant> {
    crate::generated::entity_variants::ENTITY_VARIANTS
        .iter()
        .find(|variant| variant.kind == kind && variant.name == name)
}

/// Every variant of a kind.
pub fn variants(kind: &str) -> impl Iterator<Item = &'static EntityVariant> {
    let kind = kind.to_owned();
    crate::generated::entity_variants::ENTITY_VARIANTS
        .iter()
        .filter(move |variant| variant.kind == kind)
}

/// What a sheep's wool of this dye (`DyeColor.getId`) is multiplied by, as
/// ARGB (`ColorLerper.Type.SHEEP`).
pub fn sheep_colour(dye: usize) -> Option<u32> {
    crate::generated::sheep_colors::SHEEP_COLORS
        .get(dye)
        .copied()
}

/// How many dyes there are.
pub const DYE_COUNT: usize = crate::generated::sheep_colors::SHEEP_COLORS.len();

/// The name of the entity data serializer the wire numbers `id`
/// (`PIG_VARIANT`).
pub fn data_serializer(id: i32) -> Option<&'static str> {
    usize::try_from(id)
        .ok()
        .and_then(|index| crate::generated::entity_serializers::ENTITY_DATA_SERIALIZERS.get(index))
        .copied()
}

/// What the crosshair needs of an entity type, as a new entity of it answers
/// (`isPickable`, `getPickRadius`, `canBePickedFromInside`, and its size in
/// the standing pose with a baby's, each as width, height, eye height).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntityPick {
    pub pickable: bool,
    pub pick_radius: f32,
    pub from_inside: bool,
    pub size: (f32, f32, f32),
    /// Only the types a baby can be.
    pub baby: Option<(f32, f32, f32)>,
}

/// The crosshair's answers for the type with this id; the player has none.
pub fn pick(id: i32) -> Option<&'static EntityPick> {
    usize::try_from(id)
        .ok()
        .and_then(|index| ENTITY_PICKS.get(index))
        .and_then(Option::as_ref)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_ids_go_both_ways() {
        assert_eq!(name(156), Some("minecraft:player"));
        assert_eq!(id("minecraft:player"), Some(156));
        assert_eq!(id("minecraft:cow"), Some(30));
        assert_eq!(name(-1), None);
        assert_eq!(name(100_000), None);
        assert_eq!(id("minecraft:nope"), None);
    }
}
