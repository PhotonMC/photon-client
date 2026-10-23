//! The block item classes: what `getPlacementState` asks of the block, and the blocks the item
//! and the placed block's `setPlacedBy` write around it.

use super::*;

/// What a block item does beyond `BlockItem`'s own steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemPlacement {
    /// `BlockItem`: the block's own state, if it may stand there.
    Plain,
    /// `StandingAndWallBlockItem` (torches, heads, banners, signs, coral fans): the standing
    /// block or the wall block, whichever the look reaches first and can stand.
    StandingAndWall {
        wall: usize,
        /// The side the standing block hangs from: down, or up for a hanging sign.
        attachment: Direction,
        /// `HangingSignItem`, which also asks the wall sign for a side to hang from.
        hanging_sign: bool,
    },
    /// `DoubleHighBlockItem`: clears the block above first.
    DoubleHigh,
    /// `BedItem`: sets the foot without shape updates.
    Bed,
    /// `ScaffoldingBlockItem`: the block goes where the stack of scaffolding ends, and need not
    /// be able to stand there.
    Scaffolding,
    /// `GameMasterBlockItem`: only an operator in creative mode places it.
    GameMaster,
    /// `PlaceOnWaterBlockItem` (lily pad, frogspawn): placed by `use` on the water surface the
    /// look reaches; the placement itself is `BlockItem`'s.
    PlaceOnWater,
}

/// One block `BlockItem.place` sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Write {
    pub pos: (i32, i32, i32),
    pub state: BlockStateId,
    pub flags: u32,
}

/// The wall block an item places: the one other than its own block that stands for the item.
fn wall_block(item: &Item) -> Option<usize> {
    let own = item.block?;
    BLOCKS
        .iter()
        .position(|block| block.item_name() == item.name && block.name != own)
}

/// The class of a block item and what it adds to `BlockItem.place`; `None` for the classes that
/// are not written (scaffolding, items named after other blocks, and the rest).
pub fn item_placement(item_id: ItemId) -> Option<ItemPlacement> {
    let item = item(item_id)?;
    item.block?;
    match item.class {
        ItemClass::BlockItem => Some(ItemPlacement::Plain),
        ItemClass::DoubleHighBlockItem => Some(ItemPlacement::DoubleHigh),
        ItemClass::BedItem => Some(ItemPlacement::Bed),
        ItemClass::ScaffoldingBlockItem => Some(ItemPlacement::Scaffolding),
        ItemClass::GameMasterBlockItem => Some(ItemPlacement::GameMaster),
        ItemClass::PlaceOnWaterBlockItem => Some(ItemPlacement::PlaceOnWater),
        ItemClass::StandingAndWallBlockItem
        | ItemClass::SignItem
        | ItemClass::HangingSignItem
        | ItemClass::PlayerHeadItem
        | ItemClass::BannerItem => {
            let hanging_sign = item.class == ItemClass::HangingSignItem;
            Some(ItemPlacement::StandingAndWall {
                wall: wall_block(item)?,
                attachment: if hanging_sign {
                    Direction::Up
                } else {
                    Direction::Down
                },
                hanging_sign,
            })
        }
        _ => None,
    }
}

/// `BlockItem.getPlacementState` before its check that nothing stands in the way: the state the
/// item places, if the block gives one and it may stand where it goes.
pub fn placement_state(
    kind: ItemPlacement,
    block: usize,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    match kind {
        ItemPlacement::StandingAndWall {
            wall,
            attachment,
            hanging_sign,
        } => standing_or_wall(block, wall, attachment, hanging_sign, ctx),
        ItemPlacement::GameMaster if !ctx.game_master => Rule::Known(None),
        // `mustSurvive` is false: the scaffolding is checked by its own distance.
        ItemPlacement::Scaffolding => state_for_placement(block, ctx),
        _ => match state_for_placement(block, ctx) {
            Rule::Known(Some(state)) if hook_pairs(state, ctx) => Rule::Unported,
            Rule::Known(Some(state)) => survives(state, ctx).map(|stands| stands.then_some(state)),
            Rule::Known(None) => Rule::Known(None),
            Rule::Unported => Rule::Unported,
        },
    }
}

/// Whether the state is a tripwire hook that pairs with another along its line.
fn hook_pairs(state: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    block(state).is_some_and(|block| block.class.is(BlockClass::TripWireHookBlock))
        && super::chorus::hook_has_receiver(state, ctx.clicked_pos(), ctx)
}

/// `ScaffoldingBlockItem.updatePlacementContext`: on scaffolding already, the click moves along
/// the stack (up, or the way the player faces from the top; a sneaking player goes the other
/// way) to the first place that gives way; elsewhere it stands if it is not too far from
/// support. Every other item keeps the context. `None` is a refusal.
pub fn update_context<'a>(
    kind: ItemPlacement,
    ctx: &PlaceContext<'a>,
) -> Rule<Option<PlaceContext<'a>>> {
    if kind != ItemPlacement::Scaffolding {
        return Rule::Known(Some(*ctx));
    }
    let pos = ctx.clicked_pos();
    if !is_scaffolding(ctx.state(pos)) {
        if growth::dynamic_support(pos, ctx) {
            return Rule::Unported;
        }
        let too_far = growth::scaffolding_distance(pos, ctx) == 7;
        return Rule::Known((!too_far).then_some(*ctx));
    }
    extend_stack(ctx, pos)
}

/// The way a click on a stack of scaffolding goes: along the face for a sneaking player (out of
/// the opposite one from inside the block), up from the sides, the way the player faces from
/// the top.
fn stack_direction(ctx: &PlaceContext<'_>) -> Direction {
    match (ctx.secondary_use, ctx.inside, ctx.face) {
        (true, true, face) => face.opposite(),
        (true, false, face) => face,
        (false, _, Direction::Up) => ctx.horizontal_direction,
        (false, _, _) => Direction::Up,
    }
}

/// The first place along the stack that is not scaffolding, if it gives way, within seven
/// blocks sideways.
fn extend_stack<'a>(
    ctx: &PlaceContext<'a>,
    pos: (i32, i32, i32),
) -> Rule<Option<PlaceContext<'a>>> {
    let direction = stack_direction(ctx);
    let (dx, dy, dz) = direction.step();
    let mut at = (pos.0 + dx, pos.1 + dy, pos.2 + dz);
    let mut horizontal_distance = 0;
    while horizontal_distance < 7 {
        let found = ctx.state(at);
        if !is_scaffolding(found) {
            return match can_be_replaced(found, ctx) {
                Rule::Known(true) => at_position(ctx, at, direction),
                Rule::Known(false) => Rule::Known(None),
                Rule::Unported => Rule::Unported,
            };
        }
        at = (at.0 + dx, at.1 + dy, at.2 + dz);
        if direction.axis() != Axis::Y {
            horizontal_distance += 1;
        }
    }
    Rule::Known(None)
}

fn is_scaffolding(state: BlockStateId) -> bool {
    block(state).is_some_and(|block| block.class.is(BlockClass::ScaffoldingBlock))
}

/// `BlockPlaceContext.at`: a click on the middle of a face of the block at `pos`.
fn at_position<'a>(
    ctx: &PlaceContext<'a>,
    pos: (i32, i32, i32),
    face: Direction,
) -> Rule<Option<PlaceContext<'a>>> {
    let (dx, dy, dz) = face.step();
    let mut moved = PlaceContext {
        hit_pos: pos,
        face,
        location: (
            f64::from(pos.0) + 0.5 + f64::from(dx) * 0.5,
            f64::from(pos.1) + 0.5 + f64::from(dy) * 0.5,
            f64::from(pos.2) + 0.5 + f64::from(dz) * 0.5,
        ),
        inside: false,
        replace_clicked: true,
        ..*ctx
    };
    match can_be_replaced(ctx.state(pos), &moved) {
        Rule::Known(replaced) => {
            moved.replace_clicked = replaced;
            Rule::Known(Some(moved))
        }
        Rule::Unported => Rule::Unported,
    }
}

/// `canSurvive` where the item puts the block.
fn survives(state: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<bool> {
    can_survive(state, ctx.clicked_pos(), ctx)
}

/// `StandingAndWallBlockItem.getPlacementState`: each direction of the look in turn, the
/// standing block for the one it hangs from, the wall block for the sides.
fn standing_or_wall(
    block: usize,
    wall: usize,
    attachment: Direction,
    hanging_sign: bool,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    for direction in super::placement::nearest_looking_directions(ctx) {
        if direction == attachment.opposite() {
            continue;
        }
        let candidate =
            state_for_placement(if direction == attachment { block } else { wall }, ctx);
        let Rule::Known(Some(state)) = candidate else {
            if candidate == Rule::Unported {
                return Rule::Unported;
            }
            continue;
        };
        if hanging_sign && !hanging_sign_fits(state, ctx) {
            continue;
        }
        match survives(state, ctx) {
            Rule::Known(true) => return Rule::Known(Some(state)),
            Rule::Known(false) => {}
            Rule::Unported => return Rule::Unported,
        }
    }
    Rule::Known(None)
}

/// `HangingSignItem.canPlace`: a wall hanging sign needs a side to hang from.
fn hanging_sign_fits(state: BlockStateId, ctx: &PlaceContext<'_>) -> bool {
    !is_wall_hanging_sign(state) || mounted::hanging_can_place(state, ctx.clicked_pos(), ctx)
}

fn is_wall_hanging_sign(state: BlockStateId) -> bool {
    block(state).is_some_and(|block| block.class.is(BlockClass::WallHangingSignBlock))
}

/// The flags `placeBlock` sets the block with.
fn place_flags(kind: ItemPlacement) -> u32 {
    if kind == ItemPlacement::Bed { 26 } else { 11 }
}

/// The blocks `BlockItem.place` sets for a placed state, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceWrites {
    /// What the item sets first: the space above a tall block, cleared to air or water.
    pub before: Option<Write>,
    /// The block itself.
    pub block: Write,
    /// What the placed block's `setPlacedBy` adds: the other part.
    pub after: Option<Write>,
}

impl PlaceWrites {
    /// The writes in the order they are made.
    pub fn in_order(&self) -> impl Iterator<Item = Write> {
        self.before
            .into_iter()
            .chain([self.block])
            .chain(self.after)
    }
}

/// Every block `BlockItem.place` sets for a placed `state`.
pub fn place_writes(
    kind: ItemPlacement,
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> PlaceWrites {
    // `DoubleHighBlockItem.placeBlock`: the space above is cleared, to water where it was.
    let before = (kind == ItemPlacement::DoubleHigh).then(|| {
        let above = (pos.0, pos.1 + 1, pos.2);
        let cleared = if is_water(ctx.state(above)) {
            WATER_STATE
        } else {
            crate::constants::VANILLA_AIR_BLOCK_STATE_ID
        };
        Write {
            pos: above,
            state: cleared,
            flags: 27,
        }
    });
    PlaceWrites {
        before,
        block: Write {
            pos,
            state,
            flags: place_flags(kind),
        },
        after: placed_by(state, pos, ctx),
    }
}

/// `Blocks.WATER.defaultBlockState()`.
const WATER_STATE: BlockStateId = registry::default_state_named("minecraft:water");

/// `setPlacedBy` of the blocks that put down a second part (`setBlockAndUpdate`, flags 3).
pub fn placed_by(
    placed: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Option<Write> {
    let class = block(placed)?.class;
    let flags = 3;
    if class.is(BlockClass::DoorBlock) {
        let upper = with_property(placed, "half", "upper")?;
        return Some(Write {
            pos: (pos.0, pos.1 + 1, pos.2),
            state: upper,
            flags,
        });
    }
    // `TripWireHookBlock.setPlacedBy` with nothing to pair with sets the hook as it is.
    if class.is(BlockClass::TripWireHookBlock) {
        return Some(Write {
            pos,
            state: placed,
            flags,
        });
    }
    // A small dripleaf puts its upper half down only on a server, a pitcher crop never.
    let lone = class.is(BlockClass::SmallDripleafBlock) || class.is(BlockClass::PitcherCropBlock);
    if class.is(BlockClass::DoublePlantBlock) && !lone {
        let above = (pos.0, pos.1 + 1, pos.2);
        let upper = with_property(default_state_of(placed)?, "half", "upper")?;
        let upper = if property(upper, "waterlogged").is_some() {
            with_property(upper, "waterlogged", flag(is_water(ctx.state(above))))?
        } else {
            upper
        };
        return Some(Write {
            pos: above,
            state: upper,
            flags,
        });
    }
    if class.is(BlockClass::BedBlock) {
        let facing = facing(placed)?;
        let (dx, dy, dz) = facing.step();
        return Some(Write {
            pos: (pos.0 + dx, pos.1 + dy, pos.2 + dz),
            state: with_property(placed, "part", "head")?,
            flags,
        });
    }
    None
}

fn default_state_of(state: BlockStateId) -> Option<BlockStateId> {
    block_index(state).map(|(index, _)| default_state(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_of(name: &str) -> Option<ItemPlacement> {
        let index = photon_data::item::registry::index(name)?;
        item_placement(ItemId(i32::try_from(index).ok()?))
    }

    #[test]
    fn standing_and_wall_items_find_their_wall_block() {
        for (item, wall, attachment) in [
            ("minecraft:torch", "minecraft:wall_torch", Direction::Down),
            (
                "minecraft:soul_torch",
                "minecraft:soul_wall_torch",
                Direction::Down,
            ),
            (
                "minecraft:redstone_torch",
                "minecraft:redstone_wall_torch",
                Direction::Down,
            ),
            (
                "minecraft:tube_coral_fan",
                "minecraft:tube_coral_wall_fan",
                Direction::Down,
            ),
            (
                "minecraft:zombie_head",
                "minecraft:zombie_wall_head",
                Direction::Down,
            ),
            (
                "minecraft:red_banner",
                "minecraft:red_wall_banner",
                Direction::Down,
            ),
            (
                "minecraft:oak_sign",
                "minecraft:oak_wall_sign",
                Direction::Down,
            ),
            (
                "minecraft:oak_hanging_sign",
                "minecraft:oak_wall_hanging_sign",
                Direction::Up,
            ),
        ] {
            let Some(ItemPlacement::StandingAndWall {
                wall: found,
                attachment: side,
                ..
            }) = kind_of(item)
            else {
                panic!("{item} is not a standing and wall item");
            };
            assert_eq!(BLOCKS[found].name, wall, "{item}");
            assert_eq!(side, attachment, "{item}");
        }
    }

    #[test]
    fn plain_and_tall_items_are_told_apart() {
        assert_eq!(kind_of("minecraft:stone"), Some(ItemPlacement::Plain));
        assert_eq!(
            kind_of("minecraft:oak_door"),
            Some(ItemPlacement::DoubleHigh)
        );
        assert_eq!(
            kind_of("minecraft:tall_grass"),
            Some(ItemPlacement::DoubleHigh)
        );
        assert_eq!(kind_of("minecraft:red_bed"), Some(ItemPlacement::Bed));
        // Not an item that places a block, and one of a class that is not written.
        assert_eq!(kind_of("minecraft:stick"), None);
        assert_eq!(
            kind_of("minecraft:scaffolding"),
            Some(ItemPlacement::Scaffolding)
        );
        assert_eq!(
            kind_of("minecraft:lily_pad"),
            Some(ItemPlacement::PlaceOnWater)
        );
        assert_eq!(
            kind_of("minecraft:command_block"),
            Some(ItemPlacement::GameMaster)
        );
    }

    /// How many block items the rules predict on a click on the top of a stone block. The counts
    /// are floors: a written class may only raise them.
    #[test]
    fn the_block_items_that_are_predicted_are_counted() {
        let state_at = |_: i32, y: i32, _: i32| {
            if y < 64 {
                default_state(registry::block_index("minecraft:stone").expect("stone"))
            } else {
                crate::constants::VANILLA_AIR_BLOCK_STATE_ID
            }
        };
        let tags = crate::block::tags::BlockTags::default();
        let mut ctx = PlaceContext::new(
            &state_at,
            ((0, 63, 0), Direction::Up, (0.5, 64.0, 0.5)),
            None,
            (0.0, 0.0),
            &tags,
        );
        ctx.replace_clicked = false;
        ctx.brightness = Some(&|_, _, _| 15);
        // Plain block items, and those of the other written classes: (items, predicted).
        let (mut plain, mut other) = ((0, 0), (0, 0));
        for (index, item) in ITEMS.iter().enumerate() {
            let Some(block) = item.block.and_then(registry::block_index) else {
                continue;
            };
            let Some(kind) = item_placement(ItemId(i32::try_from(index).expect("an index"))) else {
                continue;
            };
            let known = placement_state(kind, block, &ctx) != Rule::Unported;
            let count = if kind == ItemPlacement::Plain {
                &mut plain
            } else {
                &mut other
            };
            count.0 += 1;
            count.1 += usize::from(known);
        }
        assert_eq!(plain.0, 938);
        assert!(plain.1 >= 937, "plain block items predicted: {}", plain.1);
        assert!(other.1 >= 115, "other block items predicted: {}", other.1);
    }
}
