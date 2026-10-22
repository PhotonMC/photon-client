//! `PistonBaseBlock.triggerEvent` as the client runs it: the blocks a piston moves become moving
//! pistons the client ticks itself, and the ones it breaks go.

use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;
use photon_core::math::Vec3;
use photon_core::shape::{Shapes, VoxelShape};
use photon_data::block::blocks::id;
use photon_data::block::definition::PushReaction;
use photon_data::block::registry::BLOCKS;

use super::sounds::LevelEvent;
use crate::block::entity::MovingPiston;
use crate::block::piston::{self, Pos};
use crate::block::{self, rules};
use crate::*;

/// `Level.setBlock`'s reach for the shapes it updates in turn.
const UPDATE_LIMIT: i32 = 512;

/// The order `Block.updateFromNeighbourShapes` looks at the neighbours in.
const SHAPE_ORDER: [Direction; 6] = [
    Direction::West,
    Direction::East,
    Direction::North,
    Direction::South,
    Direction::Down,
    Direction::Up,
];

/// The state of a block named by its properties; air where it has no such state.
fn state_of(block: usize, properties: &[(&str, &str)]) -> BlockStateId {
    let chosen = properties
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    BLOCKS[block]
        .state_id(&chosen)
        .unwrap_or(VANILLA_AIR_BLOCK_STATE_ID)
}

/// A moving piston block of a facing, and of the kind of piston when it is a piston's.
fn moving_state(direction: Direction, kind: Option<&str>) -> BlockStateId {
    let mut properties = vec![("facing", direction.name())];
    properties.extend(kind.map(|kind| ("type", kind)));
    state_of(id::MOVING_PISTON, &properties)
}

/// A block on its way, from where it is.
fn moving(
    moved_state: BlockStateId,
    direction: Direction,
    (extending, source): (bool, bool),
) -> MovingPiston {
    MovingPiston {
        moved_state,
        direction,
        extending,
        source,
        progress: 0.0,
        progress_old: 0.0,
        death_ticks: 0,
    }
}

/// The collision shape of a state at a place.
fn shape_at(state: BlockStateId, pos: Pos) -> VoxelShape {
    block::registry::collision_shape_at(state, [pos.0, pos.1, pos.2])
}

/// The piston that draws back as an arm does leaves its base behind, in the shape of one that is out.
fn base_shape_of_retracting(moving: &MovingPiston, pos: Pos) -> VoxelShape {
    let carries_piston = block::state_or_air(moving.moved_state)
        .block()
        .class
        .is(photon_data::block::classes::BlockClass::PistonBaseBlock);
    if moving.extending || !moving.source || !carries_piston {
        return Shapes::empty();
    }
    rules::with_property(moving.moved_state, "extended", "true")
        .map_or_else(Shapes::empty, |state| shape_at(state, pos))
}

/// What the moving block is made of: the arm of a piston as the head, in the length it has by
/// now, or the block it carries.
fn shown_state(moving: &MovingPiston) -> BlockStateId {
    if !moving.source {
        return moving.moved_state;
    }
    let short = moving.extending != (1.0 - moving.progress < 0.25);
    let short = if short { "true" } else { "false" };
    state_of(
        id::PISTON_HEAD,
        &[("facing", moving.direction.name()), ("short", short)],
    )
}

impl ClientWorld {
    /// `PistonBaseBlock.triggerEvent` on the client: 0 extends, 1 and 2 retract.
    pub(crate) fn apply_piston_event(&mut self, pos: Pos, b0: i32, b1: i32) {
        let Some(state) = self.block_state_at(pos.0, pos.1, pos.2) else {
            return;
        };
        let Some(direction) = piston::facing(state) else {
            return;
        };
        let sticky = block::state_or_air(state).block == id::STICKY_PISTON as u16;
        if b0 == 0 {
            if !self.move_blocks(pos, direction, true) {
                return;
            }
            if let Some(extended) = rules::with_property(state, "extended", "true") {
                self.set_block_flagged(pos, extended, 67);
            }
        } else if b0 == 1 || b0 == 2 {
            self.retract_piston(pos, (direction, sticky), (b0, b1));
        } else {
            return;
        }
        self.level_events.0.push(LevelEvent::PistonSound);
    }

    /// The retracting half of `triggerEvent`: the piston turns into a moving one, and what the arm
    /// held on to is drawn back or left.
    fn retract_piston(
        &mut self,
        pos: Pos,
        (direction, sticky): (Direction, bool),
        (b0, b1): (i32, i32),
    ) {
        let arm = piston::offset(pos, direction, 1);
        self.final_tick(arm);
        let kind = if sticky { "sticky" } else { "normal" };
        let base = if sticky {
            id::STICKY_PISTON
        } else {
            id::PISTON
        };
        let facing = Direction::from_3d_data_value(b1 & 7);
        self.set_block_flagged(pos, moving_state(direction, Some(kind)), 276);
        let base_state = state_of(base, &[("facing", facing.name())]);
        self.set_moving_piston(pos, moving(base_state, direction, (false, true)));
        self.update_neighbour_shapes_of(pos, 2);
        if sticky {
            self.draw_back_or_release(pos, direction, b0);
        } else {
            self.remove_block(arm);
        }
    }

    /// A sticky piston's retraction past its arm: a block still on its way out is settled, one
    /// that can be drawn back is moved, and anything else is let go.
    fn draw_back_or_release(&mut self, pos: Pos, direction: Direction, b0: i32) {
        let arm = piston::offset(pos, direction, 1);
        let two = piston::offset(pos, direction, 2);
        let beyond = block::state_or_air(self.block_state_or_air(two.0, two.1, two.2));
        let on_its_way = self
            .moving_piston_at(two)
            .is_some_and(|entity| entity.direction == direction && entity.extending);
        if beyond.block == id::MOVING_PISTON as u16 && on_its_way {
            self.final_tick(two);
            return;
        }
        let block = beyond.block();
        let keeps = block.push_reaction == PushReaction::Normal
            || beyond.block == id::PISTON as u16
            || beyond.block == id::STICKY_PISTON as u16;
        let can_draw = b0 == 1
            && !block.air
            && piston::is_pushable(self, two, (direction.opposite(), direction), false)
            && keeps;
        if can_draw {
            self.move_blocks(pos, direction, false);
        } else {
            self.remove_block(arm);
        }
    }

    /// `PistonBaseBlock.moveBlocks`: false where the push is blocked.
    fn move_blocks(&mut self, piston_pos: Pos, direction: Direction, extending: bool) -> bool {
        let arm = piston::offset(piston_pos, direction, 1);
        let head_there = self
            .block_state_at(arm.0, arm.1, arm.2)
            .is_some_and(|state| block::state_or_air(state).block == id::PISTON_HEAD as u16);
        if !extending && head_there {
            self.set_block_flagged(arm, VANILLA_AIR_BLOCK_STATE_ID, 276);
        }
        let Some(resolved) = piston::resolve(self, piston_pos, direction, extending) else {
            return false;
        };
        let pushed: Vec<BlockStateId> = resolved
            .to_push
            .iter()
            .map(|pos| self.block_state_or_air(pos.0, pos.1, pos.2))
            .collect();
        let mut vacated = resolved.to_push.clone();
        self.break_blocks(&resolved.to_destroy);
        let push_direction = if extending {
            direction
        } else {
            direction.opposite()
        };
        for (pos, state) in resolved.to_push.iter().zip(&pushed).rev() {
            let target = piston::offset(*pos, push_direction, 1);
            vacated.retain(|own| *own != target);
            self.set_block_flagged(target, moving_state(direction, None), 324);
            self.set_moving_piston(target, moving(*state, direction, (extending, false)));
        }
        if extending {
            vacated.retain(|own| *own != arm);
            self.extend_arm(piston_pos, direction);
        }
        for pos in &vacated {
            self.set_block_flagged(*pos, VANILLA_AIR_BLOCK_STATE_ID, 82);
        }
        for pos in &vacated {
            // The old state's indirect shapes (redstone wire only) are not the client's concern;
            // the air's neighbours reshape to it.
            self.update_neighbour_shapes_of(*pos, 2);
        }
        true
    }

    /// The blocks a push breaks, from the far one back: a break event for each (but fire), then air.
    fn break_blocks(&mut self, to_destroy: &[Pos]) {
        for pos in to_destroy.iter().rev() {
            let state = self.block_state_or_air(pos.0, pos.1, pos.2);
            let burning = matches!(
                block::state_or_air(state).block().name,
                "minecraft:fire" | "minecraft:soul_fire"
            );
            if !burning {
                self.level_events.0.push(LevelEvent::Level {
                    event: 2001,
                    pos: *pos,
                    data: state.0,
                });
            }
            self.set_block_flagged(*pos, VANILLA_AIR_BLOCK_STATE_ID, 18);
        }
    }

    /// The piston's head, as a moving piston at the arm's place.
    fn extend_arm(&mut self, piston_pos: Pos, direction: Direction) {
        let arm = piston::offset(piston_pos, direction, 1);
        let kind = if self.piston_is_sticky(piston_pos) {
            "sticky"
        } else {
            "normal"
        };
        let head = state_of(
            id::PISTON_HEAD,
            &[("facing", direction.name()), ("type", kind)],
        );
        self.set_block_flagged(arm, moving_state(direction, Some(kind)), 324);
        self.set_moving_piston(arm, moving(head, direction, (true, true)));
    }

    /// Whether the piston block at a place is a sticky one.
    fn piston_is_sticky(&self, pos: Pos) -> bool {
        self.block_state_at(pos.0, pos.1, pos.2)
            .is_some_and(|state| block::state_or_air(state).block == id::STICKY_PISTON as u16)
    }

    /// `Level.setBlock(pos, state, flags)` as the client runs it.
    pub(crate) fn set_block_flagged(&mut self, pos: Pos, state: BlockStateId, flags: u32) -> bool {
        let at = Vec3::new(0.0, 0.0, 0.0);
        self.set_block_with_updates(pos, state, flags, UPDATE_LIMIT, at)
    }

    /// `Level.removeBlock(pos, false)`: the fluid's own block, which is air where there is none.
    fn remove_block(&mut self, pos: Pos) {
        let state = block::state_or_air(self.block_state_or_air(pos.0, pos.1, pos.2));
        let replacement = state.fluid.legacy_block();
        self.set_block_flagged(pos, replacement, 3);
    }

    /// `state.updateNeighbourShapes(level, pos, flags)` of the state at a place.
    fn update_neighbour_shapes_of(&mut self, pos: Pos, flags: u32) {
        let at = Vec3::new(0.0, 0.0, 0.0);
        self.update_neighbour_shapes(pos, flags, UPDATE_LIMIT, at);
    }

    /// `PistonMovingBlockEntity.getCollisionShape`: what a moving piston block fills at the
    /// progress it has made; with `no_clip` the direction an entity is being moved by the piston
    /// in, the block it carries lets that entity through while it is still moving that way.
    pub fn moving_piston_collision_shape(
        &self,
        pos: Pos,
        no_clip: Option<Direction>,
    ) -> VoxelShape {
        let Some(moving) = self.moving_piston_at(pos) else {
            return Shapes::empty();
        };
        let head_shape = base_shape_of_retracting(moving, pos);
        if moving.progress < 1.0 && no_clip == Some(moving.movement_direction()) {
            return head_shape;
        }
        let (dx, dy, dz) = moving.direction.step();
        let by = |step: i32| f64::from(step) * f64::from(moving.extended_progress());
        let carried = shape_at(shown_state(moving), pos).move_by(by(dx), by(dy), by(dz));
        Shapes::or(&head_shape, &carried)
    }

    /// Every moving piston block entity the level ticks, with where it is.
    pub fn moving_pistons(&self) -> Vec<(Pos, MovingPiston)> {
        self.block_tickers
            .positions()
            .filter_map(|pos| Some((pos, self.moving_piston_at(pos)?.clone())))
            .collect()
    }

    /// The moving piston a block entity holds at a place.
    pub fn moving_piston_at(&self, pos: Pos) -> Option<&MovingPiston> {
        self.block_entity_at(pos.0, pos.1, pos.2)?
            .events()
            .moving
            .as_ref()
    }

    /// `setBlockEntity(MovingPistonBlock.newMovingBlockEntity(...))`.
    fn set_moving_piston(&mut self, pos: Pos, moving: MovingPiston) {
        if let Some(entity) = self.block_entity_or_new(pos, "minecraft:piston") {
            entity.events.moving = Some(moving);
            self.block_tickers.register(pos);
        }
    }

    /// What `Minecraft.tick` does after the entities: the block entities tick, then the level's own
    /// tick runs its sky flash down.
    pub fn tick_after_entities(&mut self) {
        self.tick_block_entities();
        self.tick_sky_flash();
    }

    /// `Level.tickBlockEntities`: each block entity with a ticker ticks, in the order they were set
    /// up. The moving pistons, the end gateways, the potent sulfur blocks, the enchanting tables, the
    /// beacons, the conduits and the vaults tick.
    pub fn tick_block_entities(&mut self) {
        self.block_tickers.begin_pass();
        let mut index = 0;
        while let Some(pos) = self.block_tickers.next_to_tick(&mut index) {
            // `LevelChunk.isTicking`: not outside the world border, whichever the entity.
            if !self.border_holds(pos) {
                continue;
            }
            self.tick_block_entity(pos);
        }
        self.block_tickers.end_pass();
    }

    /// One block entity's ticker, by the block it belongs to.
    fn tick_block_entity(&mut self, pos: Pos) {
        if self.moving_piston_at(pos).is_some() {
            self.tick_moving_piston(pos);
            return;
        }
        let block =
            usize::from(block::state_or_air(self.block_state_or_air(pos.0, pos.1, pos.2)).block);
        match block {
            id::POTENT_SULFUR => self.tick_potent_sulfur(pos),
            id::ENCHANTING_TABLE => self.tick_enchanting_table(pos),
            id::BEACON => self.tick_beacon(pos),
            id::CONDUIT => self.tick_conduit(pos),
            id::VAULT => self.tick_vault(pos),
            id::SPAWNER => self.tick_spawner(pos),
            id::TRIAL_SPAWNER => self.tick_trial_spawner(pos),
            _ => self.tick_end_gateway(pos),
        }
    }

    /// `PistonMovingBlockEntity.tick`: a block on its way moves half the way on each tick; one that
    /// arrived waits a few more and settles as the block it is.
    fn tick_moving_piston(&mut self, pos: Pos) {
        let here = block::state_or_air(self.block_state_or_air(pos.0, pos.1, pos.2));
        let Some(mut moving) = self.moving_piston_at(pos).cloned() else {
            // The entity is gone (its chunk, or it was replaced by another's): no ticker for it.
            self.block_tickers.unregister(pos);
            return;
        };
        if here.block != id::MOVING_PISTON as u16 {
            return;
        }
        moving.progress_old = moving.progress;
        if moving.progress_old >= 1.0 {
            if moving.death_ticks < 5 {
                moving.death_ticks += 1;
                self.store_moving_piston(pos, moving);
            } else {
                self.settle_moving_piston(pos, &moving);
            }
        } else {
            moving.progress = (moving.progress + 0.5).min(1.0);
            self.store_moving_piston(pos, moving);
        }
    }

    /// `TheEndGatewayBlockEntity.beamAnimationTick`: the entity grows older, and its cooldown runs
    /// down. A ticker whose entity is gone goes with it.
    fn tick_end_gateway(&mut self, pos: Pos) {
        let here = block::state_or_air(self.block_state_or_air(pos.0, pos.1, pos.2));
        let Some(entity) = self.block_entity_mut_at(pos) else {
            self.block_tickers.unregister(pos);
            return;
        };
        if here.block != id::END_GATEWAY as u16 {
            return;
        }
        entity.events.age += 1;
        if entity.events.teleport_cooldown > 0 {
            entity.events.teleport_cooldown -= 1;
        }
    }

    /// Whether a place is inside the world border (`isTicking`).
    fn border_holds(&self, pos: Pos) -> bool {
        let border = self.world_border();
        let (x, z) = (f64::from(pos.0), f64::from(pos.2));
        x >= border.min_x(0.0)
            && x < border.max_x(0.0)
            && z >= border.min_z(0.0)
            && z < border.max_z(0.0)
    }

    fn store_moving_piston(&mut self, pos: Pos, moving: MovingPiston) {
        if let Some(entity) = self.block_entity_mut_at(pos) {
            entity.events.moving = Some(moving);
        }
    }

    /// The end of a move: the entity goes and the block it carried takes its place.
    fn settle_moving_piston(&mut self, pos: Pos, moving: &MovingPiston) {
        self.remove_block_entity(pos);
        let shaped = self.update_from_neighbour_shapes(moving.moved_state, pos);
        if rules::state_is_air(shaped) {
            self.set_block_flagged(pos, moving.moved_state, 340);
            return;
        }
        let dry = rules::with_property(shaped, "waterlogged", "false")
            .filter(|_| rules::property(shaped, "waterlogged") == Some("true"))
            .unwrap_or(shaped);
        self.set_block_flagged(pos, dry, 67);
    }

    /// `PistonMovingBlockEntity.finalTick`: the move is over at once, and the block settles.
    pub(crate) fn final_tick(&mut self, pos: Pos) {
        let Some(moving) = self.moving_piston_at(pos).cloned() else {
            return;
        };
        self.remove_block_entity(pos);
        let here = self.block_state_or_air(pos.0, pos.1, pos.2);
        if block::state_or_air(here).block != id::MOVING_PISTON as u16 {
            return;
        }
        let settled = if moving.source {
            VANILLA_AIR_BLOCK_STATE_ID
        } else {
            self.update_from_neighbour_shapes(moving.moved_state, pos)
        };
        self.set_block_flagged(pos, settled, 3);
    }

    /// `Block.updateFromNeighbourShapes`: the state as the blocks about it shape it.
    pub(crate) fn update_from_neighbour_shapes(
        &self,
        state: BlockStateId,
        pos: Pos,
    ) -> BlockStateId {
        let read = |x: i32, y: i32, z: i32| self.block_state_or_air(x, y, z);
        let mut shaped = state;
        for direction in SHAPE_ORDER {
            if let rules::Rule::Known(next) = rules::update_shape(shaped, &read, pos, direction) {
                shaped = next;
            }
        }
        shaped
    }
}
