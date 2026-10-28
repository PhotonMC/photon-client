//! `MultifaceBlock` and `MultifaceSpreader` as a client asks them: which faces a block has, what a
//! face can attach to, and where the lichen could spread.

use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;

use super::registry;
use super::rules::property;
use super::state::is_shape_full_block;

type Pos = (i32, i32, i32);

fn beside(pos: Pos, direction: Direction) -> Pos {
    let (x, y, z) = direction.step();
    (pos.0 + x, pos.1 + y, pos.2 + z)
}

fn is_named(state: BlockStateId, name: &str) -> bool {
    super::state_or_air(state).block().name == name
}

fn is_lichen(state: BlockStateId) -> bool {
    is_named(state, "minecraft:glow_lichen")
}

/// `MultifaceBlock.hasFace`.
pub fn has_face(state: BlockStateId, face: Direction) -> bool {
    property(state, face.name()) == Some("true")
}

/// `MultifaceBlock.canAttachTo`: the neighbour's face towards `pos` is full in its support shape
/// or its collision shape.
pub fn can_attach_to(state_at: &dyn Fn(Pos) -> BlockStateId, pos: Pos, towards: Direction) -> bool {
    let neighbour = beside(pos, towards);
    let state = state_at(neighbour);
    let face = towards.opposite();
    super::state(state).is_some_and(|info| info.is_face_sturdy(face))
        || is_shape_full_block(
            &registry::collision_shape_at(state, [neighbour.0, neighbour.1, neighbour.2])
                .face_shape(face),
        )
}

/// `DefaultSpreaderConfig.canSpreadInto` for glow lichen: the position holds air, lichen or a water
/// source, and the lichen could put the face there.
fn can_spread_into(state_at: &dyn Fn(Pos) -> BlockStateId, (pos, face): (Pos, Direction)) -> bool {
    let existing = state_at(pos);
    let replaceable = super::state_or_air(existing).block().air
        || is_lichen(existing)
        || (is_named(existing, "minecraft:water") && property(existing, "level") == Some("0"));
    replaceable
        && !(is_lichen(existing) && has_face(existing, face))
        && can_attach_to(state_at, pos, face)
}

/// `GlowLichenBlock.isValidBonemealTarget`: some face of the lichen can spread to a neighbour.
pub fn lichen_can_spread(state_at: &dyn Fn(Pos) -> BlockStateId, pos: Pos) -> bool {
    let state = state_at(pos);
    Direction::stream().any(|from| {
        has_face(state, from)
            && Direction::stream().any(|toward| {
                toward.axis() != from.axis()
                    && !has_face(state, toward)
                    && [
                        (pos, toward),
                        (beside(pos, toward), from),
                        (beside(beside(pos, toward), from), toward.opposite()),
                    ]
                    .into_iter()
                    .any(|spread| can_spread_into(state_at, spread))
            })
    })
}

const HORIZONTAL: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// `MossyCarpetBlock.isValidBonemealTarget`: the carpet has its base and a topper could be made
/// above it (`createTopperWithSideChance` when every side survives its chance).
pub fn moss_carpet_can_grow_topper(state_at: &dyn Fn(Pos) -> BlockStateId, pos: Pos) -> bool {
    const CARPET: &str = "minecraft:pale_moss_carpet";
    if property(state_at(pos), "bottom") != Some("true") {
        return false;
    }
    let above = beside(pos, Direction::Up);
    let previous = state_at(above);
    let carpet_above = is_named(previous, CARPET);
    let replaceable = carpet_above || super::state_or_air(previous).block().replaceable;
    if (carpet_above && property(previous, "bottom") == Some("true")) || !replaceable {
        return false;
    }
    let sides = topper_sides(state_at, pos);
    let unchanged = carpet_above
        && property(previous, "bottom") == Some("false")
        && HORIZONTAL
            .iter()
            .zip(sides)
            .all(|(dir, side)| property(previous, dir.name()) == Some(side));
    sides.iter().any(|side| *side != "none") && !unchanged
}

/// `MossyCarpetBlock.getUpdatedState` for a topper without a base above `pos`: a side is low
/// where the neighbour can carry it, tall under a carpet side of the block over it, and none
/// over a side the carpet at `pos` lacks.
fn topper_sides(state_at: &dyn Fn(Pos) -> BlockStateId, pos: Pos) -> [&'static str; 4] {
    const CARPET: &str = "minecraft:pale_moss_carpet";
    let above = beside(pos, Direction::Up);
    let over = state_at(beside(above, Direction::Up));
    let below = state_at(pos);
    let has_side = |state: BlockStateId, dir: Direction| {
        is_named(state, CARPET) && property(state, dir.name()).is_some_and(|side| side != "none")
    };
    HORIZONTAL.map(|dir| {
        if !can_attach_to(state_at, above, dir)
            || (is_named(below, CARPET) && !has_side(below, dir))
        {
            "none"
        } else if has_side(over, dir) && property(over, "bottom") != Some("true") {
            "tall"
        } else {
            "low"
        }
    })
}
