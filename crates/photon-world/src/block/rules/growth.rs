//! Plants that grow from a block and the connecting blocks: sugar cane, cactus, bamboo, kelp and
//! the vines that hang or climb, vines and multiface blocks, scaffolding, rails and redstone wire.

use super::directional::in_tag;
use super::*;

/// `Direction.Plane.HORIZONTAL`, in the reference's order.
pub(super) const HORIZONTAL: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

pub(super) fn offset(pos: (i32, i32, i32), direction: Direction) -> (i32, i32, i32) {
    let (dx, dy, dz) = direction.step();
    (pos.0 + dx, pos.1 + dy, pos.2 + dz)
}

pub(super) fn is_named(state: BlockStateId, name: &str) -> bool {
    block(state).is_some_and(|block| block.name == name)
}

pub(super) fn same_block(state: BlockStateId, block_index_of: usize) -> bool {
    block_index(state).map(|(index, _)| index) == Some(block_index_of)
}

fn number(state: BlockStateId, name: &str) -> Option<i32> {
    property(state, name).and_then(|value| value.parse().ok())
}

/// `canSurvive` of the classes in this file.
pub(super) fn survives(
    class: BlockClass,
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> Rule<bool> {
    Rule::Known(match class {
        BlockClass::SugarCaneBlock => sugar_cane(state, pos, ctx),
        BlockClass::CactusBlock => cactus(state, pos, ctx),
        BlockClass::BambooStalkBlock | BlockClass::BambooSaplingBlock => bamboo_survives(pos, ctx),
        BlockClass::GrowingPlantBlock => growing_plant_survives(state, pos, ctx),
        BlockClass::MultifaceBlock => multiface_survives(pos, state, ctx),
        BlockClass::VineBlock => vine_survives(state, pos, ctx),
        BlockClass::ScaffoldingBlock => scaffolding_distance(pos, ctx) < 7,
        BlockClass::RedStoneWireBlock => wire_survives(pos, ctx),
        _ => return Rule::Unported,
    })
}

/// `SugarCaneBlock.canSurvive`: on more cane, or on a block of the tag that has water (or ice)
/// beside it.
pub(super) fn sugar_cane(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let below_pos = offset(pos, Direction::Down);
    let below = ctx.state(below_pos);
    let Some((own, _)) = block_index(state) else {
        return false;
    };
    if same_block(below, own) {
        return true;
    }
    if !in_tag(below, ctx, "minecraft:supports_sugar_cane") {
        return false;
    }
    HORIZONTAL.iter().any(|direction| {
        let beside = ctx.state(offset(below_pos, *direction));
        is_water(beside) || in_tag(beside, ctx, "minecraft:supports_sugar_cane_adjacently")
    })
}

/// `CactusBlock.canSurvive`: no solid block or lava beside it, standing on cactus or sand, and
/// no liquid above.
pub(super) fn cactus(state: BlockStateId, pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    use photon_data::block::definition::Fluid;
    for direction in HORIZONTAL {
        let beside = ctx.state(offset(pos, direction));
        let info = crate::block::state_or_air(beside);
        if info.solid || matches!(info.fluid.fluid, Fluid::Lava | Fluid::FlowingLava) {
            return false;
        }
    }
    let below = ctx.state(offset(pos, Direction::Down));
    let own = block_index(state).map(|(index, _)| index);
    let supported = own.is_some_and(|own| same_block(below, own))
        || in_tag(below, ctx, "minecraft:supports_cactus");
    supported && !block(ctx.state(offset(pos, Direction::Up))).is_some_and(|above| above.liquid)
}

/// `BambooStalkBlock` and `BambooSaplingBlock.canSurvive`.
pub(super) fn bamboo_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    in_tag(
        ctx.state(offset(pos, Direction::Down)),
        ctx,
        "minecraft:supports_bamboo",
    )
}

/// `BambooStalkBlock.getStateForPlacement`: a stalk on bamboo, a sapling on the ground, and no
/// fluid.
pub(super) fn bamboo(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    if !crate::block::state_or_air(ctx.state(pos)).fluid.is_empty() {
        return None;
    }
    let below = ctx.state(offset(pos, Direction::Down));
    if !in_tag(below, ctx, "minecraft:supports_bamboo") {
        return None;
    }
    if is_named(below, "minecraft:bamboo_sapling") {
        return with_property(default, "age", "0");
    }
    if is_named(below, "minecraft:bamboo") {
        let age = if number(below, "age")? > 0 { "1" } else { "0" };
        return with_property(default, "age", age);
    }
    let above = ctx.state(offset(pos, Direction::Up));
    if is_named(above, "minecraft:bamboo") {
        return with_property(default, "age", property(above, "age")?);
    }
    Some(registry::default_state_named("minecraft:bamboo_sapling"))
}

/// The growth direction and the head and body blocks of a growing plant.
pub(super) fn plant_parts(name: &str) -> Option<(Direction, &'static str, &'static str)> {
    const PLANTS: [(&str, &str, Direction); 4] = [
        ("minecraft:kelp", "minecraft:kelp_plant", Direction::Up),
        (
            "minecraft:weeping_vines",
            "minecraft:weeping_vines_plant",
            Direction::Down,
        ),
        (
            "minecraft:twisting_vines",
            "minecraft:twisting_vines_plant",
            Direction::Up,
        ),
        (
            "minecraft:cave_vines",
            "minecraft:cave_vines_plant",
            Direction::Down,
        ),
    ];
    PLANTS
        .iter()
        .find(|(head, body, _)| *head == name || *body == name)
        .map(|(head, body, direction)| (*direction, *head, *body))
}

/// `GrowingPlantBlock.canSurvive`: attached to a block that can hold it behind, which is the
/// plant itself or has a full face towards it.
pub(super) fn growing_plant_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let Some((direction, head, body)) = block(state).and_then(|b| plant_parts(b.name)) else {
        return false;
    };
    let attached = ctx.state(offset(pos, direction.opposite()));
    // `KelpBlock.canAttachTo`; the other plants attach to anything.
    let is_kelp = head == "minecraft:kelp";
    if is_kelp && in_tag(attached, ctx, "minecraft:cannot_support_kelp") {
        return false;
    }
    is_named(attached, head)
        || is_named(attached, body)
        || crate::block::state_or_air(attached).is_face_sturdy(direction)
}

/// `GrowingPlantBlock.getStateForPlacement`: the body where the plant already continues, else a
/// head. A head's random age is not reproduced: the head keeps the default age.
pub(super) fn growing_plant(
    state: BlockStateId,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    let (direction, head, body) = block(state).and_then(|b| plant_parts(b.name))?;
    let ahead = ctx.state(offset(ctx.clicked_pos(), direction));
    if is_named(ahead, head) || is_named(ahead, body) {
        return registry::block_index(body).map(default_state);
    }
    Some(default)
}

/// `KelpBlock.getStateForPlacement`: only in a full block of water.
pub(super) fn kelp(
    state: BlockStateId,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Option<BlockStateId> {
    if !is_water_source(ctx.state(ctx.clicked_pos())) {
        return None;
    }
    growing_plant(state, default, ctx)
}

/// `MultifaceBlock.canAttachTo`: the neighbour has a full face towards the block, as support or
/// as collision.
pub(super) fn can_attach_to(toward: Direction, neighbour: BlockStateId) -> bool {
    let info = crate::block::state_or_air(neighbour);
    let face = toward.opposite();
    info.is_face_sturdy(face)
        || crate::block::state::is_shape_full_block(
            &crate::block::states().collision(info).face_shape(face),
        )
}

fn face_property(direction: Direction) -> &'static str {
    direction.name()
}

pub(super) fn has_face(state: BlockStateId, direction: Direction) -> bool {
    property(state, face_property(direction)) == Some("true")
}

/// `MultifaceBlock.canSurvive`: every face it has is on a block it can attach to, and it has one.
pub(super) fn multiface_survives(
    pos: (i32, i32, i32),
    state: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> bool {
    let mut any = false;
    for direction in Direction::stream() {
        if has_face(state, direction) {
            if !can_attach_to(direction, ctx.state(offset(pos, direction))) {
                return false;
            }
            any = true;
        }
    }
    any
}

/// `MultifaceBlock.getStateForPlacement`: the first direction the look reaches whose neighbour
/// the block can attach to and whose face it does not have yet.
pub(super) fn multiface(
    block_at: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    if super::survive::has_dynamic_neighbour(pos, ctx) {
        return Rule::Unported;
    }
    let old = ctx.state(pos);
    let old_is_own = same_block(old, block_at);
    for direction in super::placement::nearest_looking_directions(ctx) {
        let supported = property(default, face_property(direction)).is_some();
        if !supported || old_is_own && has_face(old, direction) {
            continue;
        }
        if !can_attach_to(direction, ctx.state(offset(pos, direction))) {
            continue;
        }
        let base = if old_is_own {
            Some(old)
        } else if is_water_source(old) {
            with_property(default, "waterlogged", "true")
        } else {
            Some(default)
        };
        return Rule::Known(
            base.and_then(|state| with_property(state, face_property(direction), "true")),
        );
    }
    Rule::Known(None)
}

/// `VineBlock.canSupportAtFace`.
pub(super) fn vine_supports(
    block_at: usize,
    pos: (i32, i32, i32),
    direction: Direction,
    ctx: &PlaceContext<'_>,
) -> bool {
    if direction == Direction::Down {
        return false;
    }
    if can_attach_to(direction, ctx.state(offset(pos, direction))) {
        return true;
    }
    if direction.axis() == Axis::Y {
        return false;
    }
    let above = ctx.state(offset(pos, Direction::Up));
    same_block(above, block_at) && has_face(above, direction)
}

/// `VineBlock.canSurvive`: some face still has something to hold on to, or a vine above to hang
/// from.
pub(super) fn vine_survives(
    state: BlockStateId,
    pos: (i32, i32, i32),
    ctx: &PlaceContext<'_>,
) -> bool {
    let Some((own, _)) = block_index(state) else {
        return false;
    };
    // The reference asks for the upper neighbour's top face here, not its underside.
    let up = has_face(state, Direction::Up)
        && can_attach_to(Direction::Down, ctx.state(offset(pos, Direction::Up)));
    up || HORIZONTAL
        .iter()
        .any(|direction| has_face(state, *direction) && vine_supports(own, pos, *direction, ctx))
}

/// `VineBlock.getStateForPlacement`: another face on the vine that is clicked, or a first face
/// in the first direction the look reaches that can hold it.
pub(super) fn vine(
    block_at: usize,
    default: BlockStateId,
    ctx: &PlaceContext<'_>,
) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    if super::survive::has_dynamic_neighbour(pos, ctx) {
        return Rule::Unported;
    }
    let clicked = ctx.state(pos);
    let clicked_vine = same_block(clicked, block_at);
    let result = if clicked_vine { clicked } else { default };
    for direction in super::placement::nearest_looking_directions(ctx) {
        if direction == Direction::Down {
            continue;
        }
        let occupied = clicked_vine && has_face(clicked, direction);
        if !occupied && vine_supports(block_at, pos, direction, ctx) {
            return Rule::Known(with_property(result, face_property(direction), "true"));
        }
    }
    Rule::Known(clicked_vine.then_some(result))
}

/// Whether the block under `pos` has a shape that depends on its position (and is not
/// scaffolding, which `getDistance` reads by name), which the cached shapes here do not know.
pub(super) fn dynamic_support(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    let below = ctx.state(offset(pos, Direction::Down));
    !is_named(below, "minecraft:scaffolding")
        && block(below).is_some_and(|block| block.dynamic_shape)
}

/// `ScaffoldingBlock.getDistance`.
pub(super) fn scaffolding_distance(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> i32 {
    let below_pos = offset(pos, Direction::Down);
    let below = ctx.state(below_pos);
    let mut distance = 7;
    if is_named(below, "minecraft:scaffolding") {
        distance = number(below, "distance").unwrap_or(7);
    } else if crate::block::state_or_air(below).is_face_sturdy(Direction::Up) {
        return 0;
    }
    for direction in HORIZONTAL {
        let beside = ctx.state(offset(pos, direction));
        if is_named(beside, "minecraft:scaffolding") {
            distance = distance.min(number(beside, "distance").unwrap_or(7) + 1);
            if distance == 1 {
                break;
            }
        }
    }
    distance
}

/// `ScaffoldingBlock.getStateForPlacement`.
pub(super) fn scaffolding(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let pos = ctx.clicked_pos();
    let distance = scaffolding_distance(pos, ctx);
    let bottom = distance > 0
        && !is_named(
            ctx.state(offset(pos, Direction::Down)),
            "minecraft:scaffolding",
        );
    with_property(
        default,
        "waterlogged",
        flag(is_water_source(ctx.state(pos))),
    )
    .and_then(|state| with_property(state, "distance", &distance.to_string()))
    .and_then(|state| with_property(state, "bottom", flag(bottom)))
}

/// `BaseRailBlock.getStateForPlacement`: straight along the way the player faces.
pub(super) fn rail(default: BlockStateId, ctx: &PlaceContext<'_>) -> Option<BlockStateId> {
    let east_west = matches!(ctx.horizontal_direction, Direction::East | Direction::West);
    let shape = if east_west {
        "east_west"
    } else {
        "north_south"
    };
    with_property(default, "shape", shape).and_then(|state| {
        with_property(
            state,
            "waterlogged",
            flag(is_water_source(ctx.state(ctx.clicked_pos()))),
        )
    })
}

/// Whether a block is one that gives a redstone signal (`isSignalSource`).
fn is_signal_source(state: BlockStateId) -> bool {
    const SOURCES: [BlockClass; 17] = [
        BlockClass::BasePressurePlateBlock,
        BlockClass::DiodeBlock,
        BlockClass::DetectorRailBlock,
        BlockClass::ButtonBlock,
        BlockClass::LecternBlock,
        BlockClass::PoweredBlock,
        BlockClass::DaylightDetectorBlock,
        BlockClass::JukeboxBlock,
        BlockClass::RedStoneWireBlock,
        BlockClass::SculkSensorBlock,
        BlockClass::LeverBlock,
        BlockClass::RedstoneTorchBlock,
        BlockClass::LightningRodBlock,
        BlockClass::ObserverBlock,
        BlockClass::TrappedChestBlock,
        BlockClass::TripWireHookBlock,
        BlockClass::TargetBlock,
    ];
    block(state).is_some_and(|block| SOURCES.iter().any(|class| block.class.is(*class)))
}

/// `RedStoneWireBlock.shouldConnectTo`.
fn wire_connects_to(state: BlockStateId, direction: Option<Direction>) -> bool {
    if is_named(state, "minecraft:redstone_wire") {
        true
    } else if is_named(state, "minecraft:repeater") {
        let facing = facing(state);
        direction.is_some() && (facing == direction || facing.map(Direction::opposite) == direction)
    } else if is_named(state, "minecraft:observer") {
        direction.is_some() && direction == facing(state)
    } else {
        is_signal_source(state) && direction.is_some()
    }
}

fn is_conductor(state: BlockStateId) -> bool {
    crate::block::state_or_air(state).redstone_conductor
}

/// `RedStoneWireBlock.canSurviveOn`.
fn wire_stands_on(state: BlockStateId) -> bool {
    crate::block::state_or_air(state).is_face_sturdy(Direction::Up)
        || is_named(state, "minecraft:hopper")
}

/// `RedStoneWireBlock.canSurvive`.
pub(super) fn wire_survives(pos: (i32, i32, i32), ctx: &PlaceContext<'_>) -> bool {
    wire_stands_on(ctx.state(offset(pos, Direction::Down)))
}

/// `RedStoneWireBlock.getConnectingSide`: `up`, `side` or `none`.
fn wire_side(
    pos: (i32, i32, i32),
    direction: Direction,
    can_connect_up: bool,
    ctx: &PlaceContext<'_>,
) -> &'static str {
    let relative_pos = offset(pos, direction);
    let relative = ctx.state(relative_pos);
    if can_connect_up {
        let is_trapdoor = block(relative).is_some_and(|b| b.class.is(BlockClass::TrapDoorBlock));
        let placeable_above = is_trapdoor || wire_stands_on(relative);
        let above = ctx.state(offset(relative_pos, Direction::Up));
        if placeable_above && wire_connects_to(above, None) {
            return if crate::block::state_or_air(relative).is_face_sturdy(direction.opposite()) {
                "up"
            } else {
                "side"
            };
        }
    }
    let below = ctx.state(offset(relative_pos, Direction::Down));
    if !wire_connects_to(relative, Some(direction))
        && (is_conductor(relative) || !wire_connects_to(below, None))
    {
        "none"
    } else {
        "side"
    }
}

/// A wire with no side on an axis connects both ends of the other axis, so that it is a line.
fn fill_lone_sides(mut sides: [&'static str; 4]) -> [&'static str; 4] {
    let connected = sides.map(|side| side != "none");
    let [north, east, south, west] = connected;
    let axis_empty = [!north && !south, !east && !west];
    // The side to fill, and the axis (0 north-south, 1 east-west) that must be empty for it.
    for (side, axis) in [(3, 0), (1, 0), (0, 1), (2, 1)] {
        if !connected[side] && axis_empty[axis] {
            sides[side] = "side";
        }
    }
    sides
}

/// `RedStoneWireBlock.getStateForPlacement`: connected to what is beside it, a lone wire a cross
/// and a wire with only one side the line through it.
pub(super) fn wire(default: BlockStateId, ctx: &PlaceContext<'_>) -> Rule<Option<BlockStateId>> {
    let pos = ctx.clicked_pos();
    if super::survive::has_dynamic_neighbour(pos, ctx) {
        return Rule::Unported;
    }
    let can_connect_up = !is_conductor(ctx.state(offset(pos, Direction::Up)));
    let sides =
        fill_lone_sides(HORIZONTAL.map(|direction| wire_side(pos, direction, can_connect_up, ctx)));
    let mut state = Some(default);
    for (direction, side) in HORIZONTAL.iter().zip(sides) {
        state = state.and_then(|state| with_property(state, direction.name(), side));
    }
    Rule::Known(state)
}
