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
