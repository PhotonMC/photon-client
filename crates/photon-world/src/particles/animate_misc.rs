//! More of `Block.animateTick`: the nether portal, the spore blossom, mycelium, crying obsidian,
//! the respawn anchor and the end rod.

use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;
use photon_core::random::RandomSource;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::local_sound;
use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

/// `Direction.values()`: down, up, north, south, west, east, with their unit steps.
const DIRECTIONS: [(Direction, (i32, i32, i32)); 6] = [
    (Direction::Down, (0, -1, 0)),
    (Direction::Up, (0, 1, 0)),
    (Direction::North, (0, 0, -1)),
    (Direction::South, (0, 0, 1)),
    (Direction::West, (-1, 0, 0)),
    (Direction::East, (1, 0, 0)),
];

impl ParticleEngine {
    /// `Block.animateTick` of these blocks; false for another block.
    pub(super) fn misc_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, name): (BlockStateId, &str),
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) -> bool {
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        match name {
            "nether_portal" => self.nether_portal(pos, random, (context, sounds)),
            "spore_blossom" => self.spore_blossom(pos, random, context),
            // `MyceliumBlock.animateTick`.
            "mycelium" => {
                if random.next_int_bound(10) == Ok(0) {
                    let at = (x + random.next_double(), y + 1.1, z + random.next_double());
                    self.level_add(context, "mycelium", at, (0.0, 0.0, 0.0));
                }
            }
            "crying_obsidian" => self.crying_obsidian(pos, random, context),
            // `RespawnAnchorBlock.animateTick` of a charged one.
            "respawn_anchor" => {
                if crate::block::rules::property(state, "charges") == Some("0") {
                    return true;
                }
                if random.next_int_bound(100) == Ok(0) {
                    let event = (
                        "minecraft:block.respawn_anchor.ambient",
                        LocalSoundSource::Blocks,
                    );
                    let at = (x + 0.5, y + 0.5, z + 0.5);
                    sounds.push(local_sound(event, (1.0, 1.0), at, context));
                }
                let at = (
                    x + 0.5 + (0.5 - random.next_double()),
                    y + 1.0,
                    z + 0.5 + (0.5 - random.next_double()),
                );
                let rise = f64::from(random.next_float()) * 0.04;
                self.level_add(context, "reverse_portal", at, (0.0, rise, 0.0));
            }
            "end_rod" => self.end_rod(state, pos, random, context),
            _ => {
                let block = crate::block::state_or_air(state).block();
                return self.ambient_animate_tick((state, block), pos, random, (context, sounds));
            }
        }
        true
    }

    /// `NetherPortalBlock.animateTick`: its hum now and then, and four portal specks drawn out of
    /// its faces.
    fn nether_portal<F: FnMut() -> RandomSource>(
        &mut self,
        (bx, by, bz): (i32, i32, i32),
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        let (x, y, z) = (f64::from(bx), f64::from(by), f64::from(bz));
        if random.next_int_bound(100) == Ok(0) {
            let pitch = random.next_float() * 0.4 + 0.8;
            let event = ("minecraft:block.portal.ambient", LocalSoundSource::Blocks);
            sounds.push(local_sound(
                event,
                (0.5, pitch),
                (x + 0.5, y + 0.5, z + 0.5),
                context,
            ));
        }
        let portal = |dx: i32| {
            let state = context.world.block_state_or_air(bx + dx, by, bz);
            crate::block::state_or_air(state).block().index()
                == photon_data::block::ids::NETHER_PORTAL
        };
        let along_x = portal(-1) || portal(1);
        for _ in 0..4 {
            let mut at = (
                x + random.next_double(),
                y + random.next_double(),
                z + random.next_double(),
            );
            let mut push = (
                (f64::from(random.next_float()) - 0.5) * 0.5,
                (f64::from(random.next_float()) - 0.5) * 0.5,
                (f64::from(random.next_float()) - 0.5) * 0.5,
            );
            let flip = random.next_int_bound(2).unwrap_or(0) * 2 - 1;
            if along_x {
                at.2 = z + 0.5 + 0.25 * f64::from(flip);
                push.2 = f64::from(random.next_float() * 2.0 * flip as f32);
            } else {
                at.0 = x + 0.5 + 0.25 * f64::from(flip);
                push.0 = f64::from(random.next_float() * 2.0 * flip as f32);
            }
            self.level_add(context, "portal", at, push);
        }
    }

    /// `SporeBlossomBlock.animateTick`: a spore falls from it, and the air below it fills with
    /// specks where there is room.
    fn spore_blossom<F: FnMut() -> RandomSource>(
        &mut self,
        (bx, by, bz): (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let at = (
            f64::from(bx) + random.next_double(),
            f64::from(by) + 0.7,
            f64::from(bz) + random.next_double(),
        );
        self.level_add(context, "falling_spore_blossom", at, (0.0, 0.0, 0.0));
        for _ in 0..14 {
            // `Mth.nextInt(random, -10, 10)`.
            let spread = |random: &mut RandomSource| random.next_int_bound(21).unwrap_or(0) - 10;
            let ax = bx + spread(random);
            let ay = by - random.next_int_bound(10).unwrap_or(0);
            let az = bz + spread(random);
            let state = context.world.block_state_or_air(ax, ay, az);
            let full = crate::block::state(state).is_some_and(|info| info.collision_full_block);
            if !full {
                let at = (
                    f64::from(ax) + random.next_double(),
                    f64::from(ay) + random.next_double(),
                    f64::from(az) + random.next_double(),
                );
                self.level_add(context, "spore_blossom_air", at, (0.0, 0.0, 0.0));
            }
        }
    }

    /// `CryingObsidianBlock.animateTick`: a tear one time in five out of a side or the bottom whose
    /// neighbour does not cover it.
    fn crying_obsidian<F: FnMut() -> RandomSource>(
        &mut self,
        (bx, by, bz): (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        if random.next_int_bound(5) != Ok(0) {
            return;
        }
        let (direction, (dx, dy, dz)) = DIRECTIONS[random.next_int_bound(6).unwrap_or(0) as usize];
        if direction == Direction::Up {
            return;
        }
        let neighbour = context.world.block_state_or_air(bx + dx, by + dy, bz + dz);
        let covered = crate::block::state(neighbour)
            .is_some_and(|info| info.is_face_sturdy(direction.opposite()));
        if covered {
            return;
        }
        let mut offset = |step: i32| {
            if step == 0 {
                random.next_double()
            } else {
                0.5 + f64::from(step) * 0.6
            }
        };
        let (ox, oy, oz) = (offset(dx), offset(dy), offset(dz));
        let at = (f64::from(bx) + ox, f64::from(by) + oy, f64::from(bz) + oz);
        self.level_add(context, "dripping_obsidian_tear", at, (0.0, 0.0, 0.0));
    }

    /// `EndRodBlock.animateTick`: a glint one time in five somewhere along the rod.
    fn end_rod<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        (bx, by, bz): (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let step = match crate::block::rules::property(state, "facing") {
            Some("down") => (0.0, -1.0, 0.0),
            Some("north") => (0.0, 0.0, -1.0),
            Some("south") => (0.0, 0.0, 1.0),
            Some("west") => (-1.0, 0.0, 0.0),
            Some("east") => (1.0, 0.0, 0.0),
            _ => (0.0, 1.0, 0.0),
        };
        let tenth = 0.1_f32;
        let x = f64::from(bx) + 0.55 - f64::from(random.next_float() * tenth);
        let y = f64::from(by) + 0.55 - f64::from(random.next_float() * tenth);
        let z = f64::from(bz) + 0.55 - f64::from(random.next_float() * tenth);
        let along = f64::from(0.4 - (random.next_float() + random.next_float()) * 0.4);
        if random.next_int_bound(5) == Ok(0) {
            let at = (x + step.0 * along, y + step.1 * along, z + step.2 * along);
            let push = (
                random.next_gaussian() * 0.005,
                random.next_gaussian() * 0.005,
                random.next_gaussian() * 0.005,
            );
            self.level_add(context, "end_rod", at, push);
        }
    }
}
