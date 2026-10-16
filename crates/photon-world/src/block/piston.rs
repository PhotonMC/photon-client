//! What a piston can move: `PistonBaseBlock.isPushable` and `PistonStructureResolver`, which
//! works out the blocks a push moves and the ones it breaks.

use photon_core::direction::Direction;
use photon_data::block::blocks::id;
use photon_data::block::classes::BlockInterface;
use photon_data::block::definition::PushReaction;

use super::{StateInfo, rules, state_or_air};
use crate::ClientWorld;

/// A block's place.
pub(crate) type Pos = (i32, i32, i32);

/// How many blocks a piston moves at most (`PistonStructureResolver.MAX_PUSH_DEPTH`).
const MAX_PUSH_DEPTH: usize = 12;

/// The place `n` blocks from `pos` towards `direction`.
pub(crate) fn offset(pos: Pos, direction: Direction, n: i32) -> Pos {
    let (x, y, z) = direction.step();
    (pos.0 + x * n, pos.1 + y * n, pos.2 + z * n)
}

fn state_at(world: &ClientWorld, pos: Pos) -> &'static StateInfo {
    state_or_air(world.block_state_or_air(pos.0, pos.1, pos.2))
}

fn is_block(state: &StateInfo, block: usize) -> bool {
    usize::from(state.block) == block
}

/// `WorldBorder.isWithinBounds(BlockPos)`.
fn within_border(world: &ClientWorld, pos: Pos) -> bool {
    let border = world.world_border();
    let (x, z) = (f64::from(pos.0), f64::from(pos.2));
    x >= border.min_x(0.0)
        && x < border.max_x(0.0)
        && z >= border.min_z(0.0)
        && z < border.max_z(0.0)
}

/// `PistonBaseBlock.isPushable`: whether a block can be moved towards `direction`, given the way
/// it is connected to what pushes it.
pub(crate) fn is_pushable(
    world: &ClientWorld,
    pos: Pos,
    (direction, connection): (Direction, Direction),
    allow_destroyable: bool,
) -> bool {
    let bounds = world.dimension();
    let (min_y, max_y) = (bounds.min_y, bounds.min_y + bounds.height - 1);
    if pos.1 < min_y || pos.1 > max_y || !within_border(world, pos) {
        return false;
    }
    let state = state_at(world, pos);
    if state.block().air {
        return true;
    }
    if never_moves(state, (pos, direction), (min_y, max_y)) {
        return false;
    }
    reaction_verdict(state, (direction, connection), allow_destroyable)
        .unwrap_or_else(|| !state.block().class.implements(BlockInterface::EntityBlock))
}

/// The blocks that stay put whatever pushes them, and the ends of the level.
fn never_moves(
    state: &StateInfo,
    (pos, direction): (Pos, Direction),
    (min_y, max_y): (i32, i32),
) -> bool {
    let unmovable = [
        id::OBSIDIAN,
        id::CRYING_OBSIDIAN,
        id::RESPAWN_ANCHOR,
        id::REINFORCED_DEEPSLATE,
    ];
    unmovable.iter().any(|own| is_block(state, *own))
        || direction == Direction::Down && pos.1 == min_y
        || direction == Direction::Up && pos.1 == max_y
}

/// What the block's own push reaction says, where it says anything: a piston that is out stays,
/// a block of no hardness at all stays, and the reaction decides the rest.
fn reaction_verdict(
    state: &StateInfo,
    (direction, connection): (Direction, Direction),
    allow_destroyable: bool,
) -> Option<bool> {
    let block = state.block();
    if is_block(state, id::PISTON) || is_block(state, id::STICKY_PISTON) {
        return extended(state).then_some(false);
    }
    if block.destroy_time == -1.0 {
        return Some(false);
    }
    match block.push_reaction {
        PushReaction::Block => Some(false),
        PushReaction::Destroy => Some(allow_destroyable),
        PushReaction::PushOnly => Some(direction == connection),
        PushReaction::Normal | PushReaction::Ignore => None,
    }
}

/// A piston's `EXTENDED`.
fn extended(state: &StateInfo) -> bool {
    let block = state.block();
    block
        .properties
        .iter()
        .find(|own| own.name == "extended")
        .is_some_and(|property| block.flag(u32::from(state.offset), property))
}

/// `PistonStructureResolver.isSticky`.
fn is_sticky(state: &StateInfo) -> bool {
    is_block(state, id::SLIME_BLOCK) || is_block(state, id::HONEY_BLOCK)
}

/// `PistonStructureResolver.canStickToEachOther`.
fn can_stick_to_each_other(first: &StateInfo, second: &StateInfo) -> bool {
    if is_block(first, id::HONEY_BLOCK) && is_block(second, id::SLIME_BLOCK)
        || is_block(first, id::SLIME_BLOCK) && is_block(second, id::HONEY_BLOCK)
    {
        return false;
    }
    is_sticky(first) || is_sticky(second)
}

/// What a push moves and what it breaks, in the order the reference lists them.
pub(crate) struct Resolved {
    pub to_push: Vec<Pos>,
    pub to_destroy: Vec<Pos>,
}

struct Resolver<'a> {
    world: &'a ClientWorld,
    piston: Pos,
    extending: bool,
    start: Pos,
    push: Direction,
    piston_direction: Direction,
    to_push: Vec<Pos>,
    to_destroy: Vec<Pos>,
}

/// `PistonStructureResolver.resolve`: `None` where the push is blocked.
pub(crate) fn resolve(
    world: &ClientWorld,
    piston: Pos,
    direction: Direction,
    extending: bool,
) -> Option<Resolved> {
    let (push, start) = if extending {
        (direction, offset(piston, direction, 1))
    } else {
        (direction.opposite(), offset(piston, direction, 2))
    };
    let mut resolver = Resolver {
        world,
        piston,
        extending,
        start,
        push,
        piston_direction: direction,
        to_push: Vec::new(),
        to_destroy: Vec::new(),
    };
    resolver.resolve().then_some(Resolved {
        to_push: resolver.to_push,
        to_destroy: resolver.to_destroy,
    })
}

impl Resolver<'_> {
    fn state(&self, pos: Pos) -> &'static StateInfo {
        state_at(self.world, pos)
    }

    fn pushable(&self, pos: Pos, connection: Direction, allow_destroyable: bool) -> bool {
        is_pushable(self.world, pos, (self.push, connection), allow_destroyable)
    }

    fn resolve(&mut self) -> bool {
        let next = self.state(self.start);
        if !self.pushable(self.start, self.piston_direction, false) {
            if self.extending && next.block().push_reaction == PushReaction::Destroy {
                self.to_destroy.push(self.start);
                return true;
            }
            return false;
        }
        if !self.add_block_line(self.start, self.push) {
            return false;
        }
        let mut index = 0;
        while index < self.to_push.len() {
            let pos = self.to_push[index];
            if is_sticky(self.state(pos)) && !self.add_branching_blocks(pos) {
                return false;
            }
            index += 1;
        }
        true
    }

    /// `addBlockLine`: the line of blocks from `start` on, and what the pushing runs into.
    fn add_block_line(&mut self, start: Pos, direction: Direction) -> bool {
        let next = self.state(start);
        if next.block().air || !self.pushable(start, direction, false) {
            return true;
        }
        if start == self.piston || self.to_push.contains(&start) {
            return true;
        }
        let Some(count) = self.sticky_run_behind(start, next) else {
            return false;
        };
        let mut added = 0;
        for i in (0..count).rev() {
            self.to_push
                .push(offset(start, self.push.opposite(), i as i32));
            added += 1;
        }
        self.extend_line_ahead(start, added)
    }

    /// How many blocks of a sticky run behind `start` go along with it, itself among them; none
    /// where that is more than a piston moves.
    fn sticky_run_behind(&self, start: Pos, first: &'static StateInfo) -> Option<usize> {
        let mut count = 1;
        let mut next = first;
        if count + self.to_push.len() > MAX_PUSH_DEPTH {
            return None;
        }
        while is_sticky(next) {
            let pos = offset(start, self.push.opposite(), count as i32);
            let previous = next;
            next = self.state(pos);
            if next.block().air
                || !can_stick_to_each_other(previous, next)
                || !self.pushable(pos, self.push.opposite(), false)
                || pos == self.piston
            {
                break;
            }
            count += 1;
            if count + self.to_push.len() > MAX_PUSH_DEPTH {
                return None;
            }
        }
        Some(count)
    }

    /// The blocks in front of the line, each pushed in turn until air, a block that breaks, or
    /// the line meets itself.
    fn extend_line_ahead(&mut self, start: Pos, mut added: usize) -> bool {
        let mut i = 1;
        loop {
            let pos = offset(start, self.push, i);
            if let Some(collision) = self.to_push.iter().position(|own| *own == pos) {
                return self.meet_own_line(added, collision);
            }
            let next = self.state(pos);
            if next.block().air {
                return true;
            }
            if !self.pushable(pos, self.push, true) || pos == self.piston {
                return false;
            }
            if next.block().push_reaction == PushReaction::Destroy {
                self.to_destroy.push(pos);
                return true;
            }
            if self.to_push.len() >= MAX_PUSH_DEPTH {
                return false;
            }
            self.to_push.push(pos);
            added += 1;
            i += 1;
        }
    }

    /// The line ran into blocks already to be pushed: it goes ahead of them, and whatever is
    /// sticky among the blocks up to there brings its neighbours.
    fn meet_own_line(&mut self, added: usize, collision: usize) -> bool {
        self.reorder_at_collision(added, collision);
        for j in 0..=collision + added {
            let block = self.to_push[j];
            if is_sticky(self.state(block)) && !self.add_branching_blocks(block) {
                return false;
            }
        }
        true
    }

    /// `reorderListAtCollision`: the line just added goes ahead of what it ran into.
    fn reorder_at_collision(&mut self, added: usize, collision: usize) {
        let len = self.to_push.len();
        let head = self.to_push[..collision].to_vec();
        let last_line = self.to_push[len - added..].to_vec();
        let collision_to_line = self.to_push[collision..len - added].to_vec();
        self.to_push = [head, last_line, collision_to_line].concat();
    }

    /// `addBranchingBlocks`.
    fn add_branching_blocks(&mut self, from: Pos) -> bool {
        let from_state = self.state(from);
        for direction in [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            if direction.axis() != self.push.axis() {
                let neighbour = offset(from, direction, 1);
                if can_stick_to_each_other(self.state(neighbour), from_state)
                    && !self.add_block_line(neighbour, direction)
                {
                    return false;
                }
            }
        }
        true
    }
}

/// The direction a piston or a moving piston faces, from its `facing`.
pub(crate) fn facing(state: photon_core::ids::BlockStateId) -> Option<Direction> {
    rules::property(state, "facing").and_then(Direction::from_name)
}
