//! `BlockBehaviour.canSurvive`: may the state stand where it is placed.

use super::*;

/// `canSurvive` of the class the state's block runs.
pub(super) fn can_survive(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    match state_owner(state, Method::CanSurvive) {
        None => Rule::Known(true),
        Some(BlockClass::VegetationBlock) => vegetation(state, pos, ctx),
        Some(BlockClass::CarpetBlock) => Rule::Known(!is_air(ctx.state((pos.0, pos.1 - 1, pos.2)))),
        Some(BlockClass::FaceAttachedHorizontalDirectionalBlock) => {
            Rule::Known(face_attached_survives(state, pos, ctx))
        }
        _ => Rule::Unported,
    }
}

/// The block tag each class's `mayPlaceOn` asks the block below to be in.
const SUPPORT_TAGS: &[(BlockClass, &str)] = &[
    (BlockClass::VegetationBlock, "minecraft:supports_vegetation"),
    (
        BlockClass::DryVegetationBlock,
        "minecraft:supports_dry_vegetation",
    ),
    (BlockClass::AzaleaBlock, "minecraft:supports_azalea"),
    (
        BlockClass::NetherSproutsBlock,
        "minecraft:supports_nether_sprouts",
    ),
    (
        BlockClass::NetherWartBlock,
        "minecraft:supports_nether_wart",
    ),
    (
        BlockClass::WitherRoseBlock,
        "minecraft:supports_wither_rose",
    ),
    (BlockClass::CropBlock, "minecraft:supports_crops"),
];

/// The same for the classes whose tag is a field each block is constructed with.
const FIELD_SUPPORT_TAGS: &[(&str, &str)] = &[
    ("minecraft:pumpkin_stem", "minecraft:supports_pumpkin_stem"),
    ("minecraft:melon_stem", "minecraft:supports_melon_stem"),
    (
        "minecraft:crimson_fungus",
        "minecraft:supports_crimson_fungus",
    ),
    (
        "minecraft:warped_fungus",
        "minecraft:supports_warped_fungus",
    ),
    (
        "minecraft:crimson_roots",
        "minecraft:supports_crimson_roots",
    ),
    ("minecraft:warped_roots", "minecraft:supports_warped_roots"),
];

/// The tag the block below a plant must be in, or `None` where `mayPlaceOn` asks more than that.
fn support_tag(placed: &Block) -> Option<&'static str> {
    let owner = owner(placed, Method::MayPlaceOn)?;
    let class_tag = SUPPORT_TAGS
        .iter()
        .find(|(class, _)| *class == owner)
        .map(|(_, tag)| *tag);
    let field_tag = FIELD_SUPPORT_TAGS
        .iter()
        .find(|(name, _)| *name == placed.name)
        .map(|(_, tag)| *tag);
    class_tag.or(field_tag)
}

/// `VegetationBlock.canSurvive`: the block below must be one the plant's own `mayPlaceOn`
/// accepts.
fn vegetation(state: BlockStateId, pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> Rule<bool> {
    let Some(placed) = block(state) else {
        return Rule::Known(false);
    };
    let Some(tag) = support_tag(placed) else {
        return Rule::Unported;
    };
    let below = block_index(ctx.state((pos.0, pos.1 - 1, pos.2)));
    Rule::Known(below.is_some_and(|(index, _)| ctx.tags.contains(index, tag)))
}

/// `FaceAttachedHorizontalDirectionalBlock.canSurvive`: the block it hangs from has a full face
/// towards it.
pub(super) fn face_attached_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let connected = match property(state, "face") {
        Some("ceiling") => Direction::Down,
        Some("floor") => Direction::Up,
        _ => property(state, "facing")
            .and_then(Direction::from_name)
            .unwrap_or(Direction::North),
    };
    let (dx, dy, dz) = connected.opposite().step();
    let support = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
    crate::block::state(support).is_some_and(|info| info.is_face_sturdy(connected))
}
