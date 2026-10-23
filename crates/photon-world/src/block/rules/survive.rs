//! `BlockBehaviour.canSurvive`: may the state stand where it is placed.

use super::*;
use crate::block::SupportType;

/// `canSurvive` of the class the state's block runs.
pub(super) fn can_survive(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    let owner = state_owner(state, Method::CanSurvive);
    if owner.is_some_and(reads_support_shape) && has_dynamic_neighbour(pos, ctx) {
        return Rule::Unported;
    }
    match owner {
        None => Rule::Known(true),
        Some(BlockClass::VegetationBlock) => vegetation(state, pos, ctx),
        Some(BlockClass::CarpetBlock) => Rule::Known(!is_air(ctx.state((pos.0, pos.1 - 1, pos.2)))),
        Some(BlockClass::FaceAttachedHorizontalDirectionalBlock) => {
            Rule::Known(face_attached_survives(state, pos, ctx))
        }
        Some(class) => match super::mounted::survives(class, state, pos, ctx) {
            Rule::Unported => foot_survives(class, state, pos, ctx),
            known => known,
        },
    }
}

/// Whether the class decides by the shape of the block it stands on or hangs from.
fn reads_support_shape(class: BlockClass) -> bool {
    matches!(
        class,
        BlockClass::BasePressurePlateBlock
            | BlockClass::BaseRailBlock
            | BlockClass::DiodeBlock
            | BlockClass::LanternBlock
            | BlockClass::BellBlock
            | BlockClass::SporeBlossomBlock
            | BlockClass::CactusFlowerBlock
            | BlockClass::BaseTorchBlock
            | BlockClass::CeilingHangingSignBlock
            | BlockClass::CandleBlock
            | BlockClass::LeafLitterBlock
            | BlockClass::SeaPickleBlock
    )
}

/// Whether a block beside `pos` computes its support shape per position (bamboo, dripstone,
/// scaffolding, shulker boxes, powder snow), which the cached shapes here do not know.
pub(super) fn has_dynamic_neighbour(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    Direction::stream().any(|direction| {
        let (dx, dy, dz) = direction.step();
        block(ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz)))
            .is_some_and(|block| block.dynamic_shape)
    })
}

/// `Block.canSupportRigidBlock`.
fn can_support_rigid(ctx: &PlaceContext<'_>, below: (i32, i32, i32)) -> bool {
    crate::block::state(ctx.state(below))
        .is_some_and(|info| info.is_face_sturdy_for(Direction::Up, SupportType::Rigid))
}

/// `Block.canSupportCenter`.
pub(super) fn can_support_center(
    ctx: &PlaceContext<'_>,
    at: (i32, i32, i32),
    direction: Direction,
) -> bool {
    let state = ctx.state(at);
    if direction == Direction::Down
        && block_index(state)
            .is_some_and(|(index, _)| ctx.tags.contains(index, "minecraft:unstable_bottom_center"))
    {
        return false;
    }
    crate::block::state(state)
        .is_some_and(|info| info.is_face_sturdy_for(direction, SupportType::Center))
}

/// `LanternBlock.canSurvive`.
pub(super) fn lantern_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let toward = if property(state, "hanging") == Some("true") {
        Direction::Up
    } else {
        Direction::Down
    };
    let (dx, dy, dz) = toward.step();
    can_support_center(ctx, (pos.0 + dx, pos.1 + dy, pos.2 + dz), toward.opposite())
}

/// `BellBlock.canSurvive`: the way the bell hangs from, up for a ceiling bell.
pub(super) fn bell_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let toward = match property(state, "attachment") {
        Some("floor") => Direction::Down,
        Some("ceiling") => Direction::Up,
        _ => property(state, "facing")
            .and_then(Direction::from_name)
            .unwrap_or(Direction::North),
    };
    let (dx, dy, dz) = toward.step();
    let at = (pos.0 + dx, pos.1 + dy, pos.2 + dz);
    if toward == Direction::Up {
        can_support_center(ctx, at, Direction::Down)
    } else {
        crate::block::state(ctx.state(at))
            .is_some_and(|info| info.is_face_sturdy(toward.opposite()))
    }
}

/// `canSurvive` of the classes that stand on or hang from a block's centre or rigid face.
fn foot_survives(
    class: BlockClass,
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    match class {
        BlockClass::BasePressurePlateBlock => {
            let below = (pos.0, pos.1 - 1, pos.2);
            Rule::Known(
                can_support_rigid(ctx, below) || can_support_center(ctx, below, Direction::Up),
            )
        }
        BlockClass::BaseRailBlock | BlockClass::DiodeBlock => {
            Rule::Known(can_support_rigid(ctx, (pos.0, pos.1 - 1, pos.2)))
        }
        BlockClass::LanternBlock => Rule::Known(lantern_survives(state, pos, ctx)),
        BlockClass::BellBlock => Rule::Known(bell_survives(state, pos, ctx)),
        BlockClass::SporeBlossomBlock => Rule::Known(
            can_support_center(ctx, (pos.0, pos.1 + 1, pos.2), Direction::Down)
                && !is_water(ctx.state(pos)),
        ),
        _ => ground_survives(class, pos, ctx),
    }
}

/// `canSurvive` of the classes that stand on the block below.
fn ground_survives(class: BlockClass, pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> Rule<bool> {
    let below = (pos.0, pos.1 - 1, pos.2);
    match class {
        BlockClass::CandleBlock => Rule::Known(can_support_center(ctx, below, Direction::Up)),
        BlockClass::LeafLitterBlock => Rule::Known(
            crate::block::state(ctx.state(below))
                .is_some_and(|info| info.is_face_sturdy(Direction::Up)),
        ),
        BlockClass::SeaPickleBlock => Rule::Known(sea_pickle_survives(pos, ctx)),
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
pub(super) fn vegetation(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    let Some(placed) = block(state) else {
        return Rule::Known(false);
    };
    if owner(placed, Method::MayPlaceOn) == Some(BlockClass::CactusFlowerBlock) {
        // `CactusFlowerBlock.mayPlaceOn`: the override tag, or a sturdy centre.
        let below_pos = (pos.0, pos.1 - 1, pos.2);
        let overridden = block_index(ctx.state(below_pos)).is_some_and(|(index, _)| {
            ctx.tags
                .contains(index, "minecraft:support_override_cactus_flower")
        });
        return Rule::Known(overridden || can_support_center(ctx, below_pos, Direction::Up));
    }
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
    let connected = connected_direction(state);
    let (dx, dy, dz) = connected.opposite().step();
    let support = ctx.state((pos.0 + dx, pos.1 + dy, pos.2 + dz));
    crate::block::state(support).is_some_and(|info| info.is_face_sturdy(connected))
}

/// `FaceAttachedHorizontalDirectionalBlock.getConnectedDirection`: the way the block leans.
pub(super) fn connected_direction(state: BlockStateId) -> Direction {
    match property(state, "face") {
        Some("ceiling") => Direction::Down,
        Some("floor") => Direction::Up,
        _ => property(state, "facing")
            .and_then(Direction::from_name)
            .unwrap_or(Direction::North),
    }
}

/// `SeaPickleBlock.canSurvive`: the block below has a collision face up or a full one.
fn sea_pickle_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let info = crate::block::state_or_air(ctx.state((pos.0, pos.1 - 1, pos.2)));
    let face = crate::block::states()
        .collision(info)
        .face_shape(Direction::Up);
    !face.is_empty() || info.is_face_sturdy(Direction::Up)
}
