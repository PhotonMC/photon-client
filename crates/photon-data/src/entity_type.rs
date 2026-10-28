//! The entity type registry: each type's name, and its id (its place in the
//! list, which is what the wire carries).

use photon_core::ids::EntityTypeId;

use crate::generated::entity_picks::ENTITY_PICKS;
use crate::generated::entity_types::{ENTITY_TYPES, NOT_IN_PEACEFUL};

/// Every type's name, in id order.
pub fn names() -> &'static [&'static str] {
    &ENTITY_TYPES
}

/// The name of the type with this id (`minecraft:pig`).
pub fn name(id: EntityTypeId) -> Option<&'static str> {
    usize::try_from(id.0)
        .ok()
        .and_then(|index| ENTITY_TYPES.get(index))
        .copied()
}

/// `EntityType.isAllowedInPeaceful`: false for the types built with `notInPeaceful`.
pub fn allowed_in_peaceful(name: &str) -> bool {
    !NOT_IN_PEACEFUL.contains(&name)
}

/// The id of the type with this name.
pub fn id(name: &str) -> Option<EntityTypeId> {
    ENTITY_TYPES
        .iter()
        .position(|candidate| *candidate == name)
        .map(|index| EntityTypeId(index as i32))
}

/// The id of the type with this name, resolved while compiling: a name the registry lacks
/// stops the build.
pub const fn named(name: &str) -> EntityTypeId {
    let mut index = 0;
    while index < ENTITY_TYPES.len() {
        if crate::name::same(ENTITY_TYPES[index], name) {
            return EntityTypeId(index as i32);
        }
        index += 1;
    }
    panic!("the entity type registry has no such name");
}

/// The id of the entity data serializer with this name (`ITEM_STACK`), resolved while
/// compiling: a name the registry lacks stops the build.
pub const fn serializer_named(name: &str) -> i32 {
    let table = &crate::generated::entity_serializers::ENTITY_DATA_SERIALIZERS;
    let mut id = 0;
    while id < table.len() {
        if crate::name::same(table[id], name) {
            return id as i32;
        }
        id += 1;
    }
    panic!("the entity data serializer registry has no such name");
}

/// The types registered with `fireImmune` (`EntityType.fireImmune`): they never burn.
const FIRE_IMMUNE: [&str; 15] = [
    "minecraft:area_effect_cloud",
    "minecraft:blaze",
    "minecraft:ender_dragon",
    "minecraft:end_crystal",
    "minecraft:ghast",
    "minecraft:magma_cube",
    "minecraft:shulker",
    "minecraft:strider",
    "minecraft:tnt",
    "minecraft:vex",
    "minecraft:warden",
    "minecraft:wither",
    "minecraft:wither_skeleton",
    "minecraft:zoglin",
    "minecraft:zombified_piglin",
];

/// `EntityType.fireImmune` of the type with this id.
pub fn fire_immune(id: EntityTypeId) -> bool {
    name(id).is_some_and(|name| FIRE_IMMUNE.contains(&name))
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

/// A wolf variant of the data pack: the textures (with their namespace) a
/// grown wolf and a baby wear, each wild, tame and angry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WolfVariant {
    pub name: &'static str,
    /// Wild, tame, angry; then the baby's wild, tame, angry.
    pub assets: [&'static str; 6],
}

impl WolfVariant {
    /// The texture of a wolf in this state (`Wolf.getTexture`): a tame wolf is
    /// tame whether or not it is angry.
    pub fn texture(&self, baby: bool, tame: bool, angry: bool) -> &'static str {
        let state = if tame {
            1
        } else if angry {
            2
        } else {
            0
        };
        self.assets[if baby { 3 + state } else { state }]
    }
}

/// The texture (under `textures/`) a fox wears (`FoxRenderer.getTextureLocation`):
/// by variant (red or snow), whether it sleeps, and whether it is a baby. Every
/// path is compared to the reference's own table on each parity run.
pub fn fox_texture(variant: i32, sleeping: bool, baby: bool) -> &'static str {
    const TEXTURES: [[[&str; 2]; 2]; 2] = [
        [
            ["entity/fox/fox", "entity/fox/fox_baby"],
            ["entity/fox/fox_sleep", "entity/fox/fox_sleep_baby"],
        ],
        [
            ["entity/fox/fox_snow", "entity/fox/fox_snow_baby"],
            [
                "entity/fox/fox_snow_sleep",
                "entity/fox/fox_snow_sleep_baby",
            ],
        ],
    ];
    TEXTURES[usize::from(variant == 1)][usize::from(sleeping)][usize::from(baby)]
}

/// The texture (under `textures/`) a panda of this gene wears
/// (`PandaRenderer.getTextureLocation`), grown or baby. Every path is compared
/// to the reference's own tables on each parity run.
pub fn panda_texture(gene: u8, baby: bool) -> &'static str {
    const TEXTURES: [[&str; 2]; 7] = [
        ["entity/panda/panda", "entity/panda/panda_baby"],
        ["entity/panda/panda_lazy", "entity/panda/lazy_panda_baby"],
        [
            "entity/panda/panda_worried",
            "entity/panda/worried_panda_baby",
        ],
        [
            "entity/panda/panda_playful",
            "entity/panda/playful_panda_baby",
        ],
        ["entity/panda/panda_brown", "entity/panda/brown_panda_baby"],
        ["entity/panda/panda_weak", "entity/panda/weak_panda_baby"],
        [
            "entity/panda/panda_aggressive",
            "entity/panda/aggressive_panda_baby",
        ],
    ];
    TEXTURES[usize::from(gene).min(6)][usize::from(baby)]
}

/// A painting variant of the data pack: the texture it wears (`minecraft:kebab`, under
/// `textures/painting`) and its size in blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaintingVariant {
    pub name: &'static str,
    pub asset: &'static str,
    /// The texture's path under `textures/` (`painting/kebab`).
    pub texture: &'static str,
    pub width: i32,
    pub height: i32,
}

/// Every painting variant, by name.
pub fn painting_variants() -> impl Iterator<Item = &'static PaintingVariant> {
    crate::generated::painting_variants::PAINTING_VARIANTS.iter()
}

/// The painting variant with this name (`minecraft:kebab`).
pub fn painting_variant(name: &str) -> Option<&'static PaintingVariant> {
    crate::generated::painting_variants::PAINTING_VARIANTS
        .iter()
        .find(|variant| variant.name == name)
}

/// Every wolf variant, by name.
pub fn wolf_variants() -> impl Iterator<Item = &'static WolfVariant> {
    crate::generated::wolf_variants::WOLF_VARIANTS.iter()
}

/// The wolf variant with this name (`minecraft:ashen`).
pub fn wolf_variant(name: &str) -> Option<&'static WolfVariant> {
    crate::generated::wolf_variants::WOLF_VARIANTS
        .iter()
        .find(|variant| variant.name == name)
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

/// An attribute's default and, when it has one, its range
/// (`Attribute.getDefaultValue`, `RangedAttribute`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeRange {
    pub name: &'static str,
    pub default: f64,
    pub range: Option<(f64, f64)>,
}

/// Every attribute, in registry order.
pub fn attribute_ranges() -> &'static [AttributeRange] {
    &crate::generated::entity_attributes::ATTRIBUTE_RANGES
}

/// What each living entity type starts with: its name and the base value of
/// each attribute it has, in registry order of entity types.
pub fn default_attributes() -> &'static [(&'static str, &'static [(&'static str, f64)])] {
    &crate::generated::entity_attributes::ENTITY_ATTRIBUTES
}

/// The attribute with this name: its place in the registry and its range.
pub fn attribute(name: &str) -> Option<(usize, &'static AttributeRange)> {
    crate::generated::entity_attributes::ATTRIBUTE_RANGES
        .iter()
        .enumerate()
        .find(|(_, attribute)| attribute.name == name)
}

/// The attribute with this place in the registry.
pub fn attribute_by_id(id: i32) -> Option<&'static AttributeRange> {
    usize::try_from(id)
        .ok()
        .and_then(|index| crate::generated::entity_attributes::ATTRIBUTE_RANGES.get(index))
}

/// The base value a new entity of this type gives an attribute
/// (`DefaultAttributes`), if the type has the attribute.
pub fn default_attribute(entity_type: &str, attribute: &str) -> Option<f64> {
    crate::generated::entity_attributes::ENTITY_ATTRIBUTES
        .iter()
        .find(|(name, _)| *name == entity_type)?
        .1
        .iter()
        .find(|(name, _)| *name == attribute)
        .map(|(_, base)| *base)
}

/// What a sheep's wool of this dye (`DyeColor.getId`) is multiplied by, as
/// ARGB (`ColorLerper.Type.SHEEP`).
pub fn sheep_colour(dye: usize) -> Option<u32> {
    crate::generated::sheep_colors::SHEEP_COLORS
        .get(dye)
        .copied()
}

/// `DyeColor.getTextureDiffuseColor` of this dye (`DyeColor.getId`), as the
/// reference's int (a wolf's collar).
pub fn dye_texture_colour(dye: usize) -> Option<u32> {
    crate::generated::sheep_colors::DYE_TEXTURE_COLORS
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
    /// Whether the type is a `Monster`, which ticks its own swing.
    pub monster: bool,
    pub size: (f32, f32, f32),
    /// Only the types a baby can be.
    pub baby: Option<(f32, f32, f32)>,
    /// Whether the type can be leashed (`Leashable`), and whether it is tied by four lines
    /// (`supportQuadLeash`).
    pub leash: Option<Leash>,
    /// `supportQuadLeashAsHolder`: a holder that four lines tie to.
    pub quad_holder: bool,
    /// Where each passenger sits (`EntityAttachment.PASSENGER`), before the entity's turn.
    pub passengers: &'static [(f64, f64, f64)],
    /// Where the name tag hangs (`EntityAttachment.NAME_TAG`), before the entity's turn, for a
    /// grown one and a baby; none for a type without the attachment.
    pub name_tag: Option<(f64, f64, f64)>,
    pub baby_name_tag: Option<(f64, f64, f64)>,
}

/// How a leashable type takes a leash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leash {
    pub quad: bool,
}

/// The crosshair's answers for the type with this id; the player has none.
pub fn pick(id: EntityTypeId) -> Option<&'static EntityPick> {
    usize::try_from(id.0)
        .ok()
        .and_then(|index| ENTITY_PICKS.get(index))
        .and_then(Option::as_ref)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_ids_go_both_ways() {
        assert_eq!(name(EntityTypeId(156)), Some("minecraft:player"));
        assert_eq!(id("minecraft:player"), Some(EntityTypeId(156)));
        assert_eq!(id("minecraft:cow"), Some(EntityTypeId(30)));
        assert_eq!(name(EntityTypeId(-1)), None);
        assert_eq!(name(EntityTypeId(100_000)), None);
        assert_eq!(id("minecraft:nope"), None);
    }
}
