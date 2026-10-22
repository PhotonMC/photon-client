//! What fire needs of its place: `FireBlock.canBurn` and the survival of the fire that
//! `BaseFireBlock.getState` would place.

use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;

const ALL: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

/// `FireBlock.canBurn`: a block with ignite odds that is not under water.
pub fn can_burn(state: BlockStateId) -> bool {
    let name = super::state_or_air(state).block().name;
    super::rules::property(state, "waterlogged") != Some("true")
        && photon_data::fire::odds(name).is_some_and(|(ignite, _)| ignite > 0)
}

/// Whether the fire `BaseFireBlock.getState` picks for the empty place `at` survives there: soul
/// fire on a soul fire base block, else fire on a sturdy top or beside something that burns
/// (`SoulFireBlock.canSurviveOnBlock`, `FireBlock.canSurvive`, `isValidFireLocation`).
pub fn survives_at(
    state_at: impl Fn((i32, i32, i32)) -> BlockStateId,
    at: (i32, i32, i32),
) -> bool {
    let beside = |direction: Direction| {
        let (dx, dy, dz) = direction.step();
        state_at((at.0 + dx, at.1 + dy, at.2 + dz))
    };
    let below = beside(Direction::Down);
    if is_soul_fire_base(below) {
        return true;
    }
    super::state(below).is_some_and(|info| info.is_face_sturdy(Direction::Up))
        || ALL.into_iter().any(|direction| can_burn(beside(direction)))
}

/// The fire `BaseFireBlock.getState` places at an empty place: soul fire on a soul fire base
/// block, else fire, which with no burning or sturdy block below leans on the sides and the top
/// that burn (`FireBlock.getStateForPlacement`).
pub fn placement_state(
    state_at: impl Fn((i32, i32, i32)) -> BlockStateId,
    at: (i32, i32, i32),
) -> Option<BlockStateId> {
    let beside = |direction: Direction| {
        let (dx, dy, dz) = direction.step();
        state_at((at.0 + dx, at.1 + dy, at.2 + dz))
    };
    let below = beside(Direction::Down);
    let default_of = |name: &str| {
        let index = photon_data::block::registry::block_index(name)?;
        Some(photon_data::block::registry::BLOCKS[index].default_state_id())
    };
    if is_soul_fire_base(below) {
        return default_of("minecraft:soul_fire");
    }
    let mut fire = default_of("minecraft:fire")?;
    let sturdy = super::state(below).is_some_and(|info| info.is_face_sturdy(Direction::Up));
    if !can_burn(below) && !sturdy {
        for (name, direction) in [
            ("north", Direction::North),
            ("east", Direction::East),
            ("south", Direction::South),
            ("west", Direction::West),
            ("up", Direction::Up),
        ] {
            let value = if can_burn(beside(direction)) {
                "true"
            } else {
                "false"
            };
            fire = super::rules::with_property(fire, name, value)?;
        }
    }
    Some(fire)
}

/// `BlockTags.SOUL_FIRE_BASE_BLOCKS`.
fn is_soul_fire_base(state: BlockStateId) -> bool {
    matches!(
        super::state_or_air(state).block().name,
        "minecraft:soul_sand" | "minecraft:soul_soil"
    )
}

#[cfg(test)]
mod tests {
    use super::survives_at;
    use photon_core::ids::BlockStateId;

    fn named(name: &str) -> BlockStateId {
        let index = photon_data::block::registry::block_index(name).expect(name);
        photon_data::block::registry::BLOCKS[index].default_state_id()
    }

    #[test]
    fn fire_needs_a_floor_or_something_that_burns() {
        let (air, stone) = (named("minecraft:air"), named("minecraft:stone"));
        let planks = named("minecraft:oak_planks");
        let at = (0, 64, 0);
        let on = |below: BlockStateId, side: BlockStateId| {
            move |(x, y, z): (i32, i32, i32)| match (x, y, z) {
                (0, 63, 0) => below,
                (1, 64, 0) => side,
                _ => air,
            }
        };
        assert!(survives_at(on(stone, air), at));
        assert!(!survives_at(on(air, air), at));
        assert!(survives_at(on(air, planks), at));
        assert!(!survives_at(on(air, stone), at));
        assert!(survives_at(on(named("minecraft:soul_sand"), air), at));
    }

    #[test]
    fn fire_leans_on_what_burns_when_nothing_holds_it_up() {
        use super::placement_state;
        use crate::block::rules::property;
        let (air, planks) = (named("minecraft:air"), named("minecraft:oak_planks"));
        let at = (0, 64, 0);
        let state_at = |(x, y, z): (i32, i32, i32)| match (x, y, z) {
            (1, 64, 0) | (0, 65, 0) => planks,
            _ => air,
        };
        let fire = placement_state(state_at, at).unwrap();
        assert_eq!(property(fire, "east"), Some("true"));
        assert_eq!(property(fire, "up"), Some("true"));
        assert_eq!(property(fire, "west"), Some("false"));
        let on_stone = |(x, y, z): (i32, i32, i32)| match (x, y, z) {
            (0, 63, 0) => named("minecraft:stone"),
            (1, 64, 0) => planks,
            _ => air,
        };
        let plain = placement_state(on_stone, at).unwrap();
        assert_eq!(property(plain, "east"), Some("false"));
        let soul = placement_state(|_| named("minecraft:soul_soil"), at).unwrap();
        assert_eq!(
            crate::block::state_or_air(soul).block().name,
            "minecraft:soul_fire"
        );
    }
}
