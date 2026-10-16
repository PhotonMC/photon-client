//! `Block.animateTick` of the blocks carried: torches, fire, campfires, candles, the furnaces and
//! the brewing stand. Each draws from the block random it is given.

use photon_core::ids::BlockStateId;
use photon_core::random::RandomSource;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::local_sound;
use super::engine::{ParticleData, ParticleEngine};
use super::level_events::LevelEventContext;

/// `DustParticleOptions.REDSTONE`: red, full size.
const REDSTONE: ParticleData = ParticleData::Dust {
    color: 0xFF_0000,
    scale: 1.0,
};

/// `CandleBlock.PARTICLE_OFFSETS`: where each wick of one to four candles is, in sixteenths.
const CANDLE_WICKS: [&[[f64; 3]]; 4] = [
    &[[8.0, 8.0, 8.0]],
    &[[6.0, 7.0, 8.0], [10.0, 8.0, 7.0]],
    &[[8.0, 5.0, 10.0], [6.0, 7.0, 8.0], [9.0, 8.0, 7.0]],
    &[
        [7.0, 5.0, 9.0],
        [10.0, 7.0, 9.0],
        [6.0, 7.0, 6.0],
        [9.0, 8.0, 6.0],
    ],
];

/// The unit step a `facing` points (`Direction.getStepX`, `getStepZ`).
fn step(facing: Option<&str>) -> (f64, f64) {
    match facing {
        Some("north") => (0.0, -1.0),
        Some("south") => (0.0, 1.0),
        Some("west") => (-1.0, 0.0),
        _ => (1.0, 0.0),
    }
}

/// `FireBlock.canBurn`: a block with ignite odds that is not under water.
fn can_burn(state: i32) -> bool {
    let name = crate::block::state_or_air(i64::from(state)).block().name;
    crate::block::rules::property(state, "waterlogged") != Some("true")
        && photon_data::fire::odds(name).is_some_and(|(ignite, _)| ignite > 0)
}

impl ParticleEngine {
    /// `Block.animateTick` for the state at a block: its particles go through the level, its
    /// sounds are added to `sounds`.
    pub(super) fn block_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        state: i32,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        let full = crate::block::state_or_air(i64::from(state)).block().name;
        let name = full.strip_prefix("minecraft:").unwrap_or(full);
        let lit = crate::block::rules::property(state, "lit") == Some("true");
        if photon_data::leaves::of(full).is_some() {
            let materials = self.terrain_materials();
            self.leaves_animate_tick((state, full), pos, random, (context, materials.as_deref()));
            return;
        }
        if photon_data::map::dust_color(full).is_some() {
            // `SandBlock` plays the desert's sounds after.
            let sand = matches!(name, "sand" | "red_sand");
            self.falling_block_animate_tick((state, sand), pos, random, (context, sounds));
            return;
        }
        match name {
            "fire" | "soul_fire" => self.fire(pos, name == "soul_fire", random, (context, sounds)),
            "campfire" | "soul_campfire" if lit => {
                self.campfire(pos, name == "campfire", random, (context, sounds));
            }
            "furnace" | "blast_furnace" | "smoker" if lit => {
                self.furnace(state, pos, name, random, (context, sounds));
            }
            "brewing_stand" => {
                let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
                let x = x + 0.4 + f64::from(random.next_float()) * 0.2;
                let y = y + 0.7 + f64::from(random.next_float()) * 0.3;
                let z = z + 0.4 + f64::from(random.next_float()) * 0.2;
                self.level_add(context, "smoke", (x, y, z), (0.0, 0.0, 0.0));
            }
            candle if lit && candle.ends_with("candle_cake") => {
                self.candle_wicks(pos, &[[8.0, 16.0, 8.0]], random, (context, sounds));
            }
            candle if lit && candle.ends_with("candle") => {
                let count: usize = crate::block::rules::property(state, "candles")
                    .and_then(|count| count.parse().ok())
                    .unwrap_or(1);
                let wicks = CANDLE_WICKS[count.clamp(1, 4) - 1];
                self.candle_wicks(pos, wicks, random, (context, sounds));
            }
            _ => {
                if !self.misc_animate_tick((state, name), pos, random, (context, sounds))
                    && !self.redstone_animate_tick((state, name), pos, random, (context, sounds))
                {
                    self.torch_animate_tick(state, name, pos, random, context);
                }
            }
        }
    }

    /// `TorchBlock`, `WallTorchBlock`, `RedstoneTorchBlock` and `RedstoneWallTorchBlock`.
    fn torch_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        state: i32,
        name: &str,
        (x, y, z): (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let (x, y, z) = (f64::from(x) + 0.5, f64::from(y) + 0.7, f64::from(z) + 0.5);
        // `FACING.getOpposite()`: against the way a wall torch faces.
        let away = || {
            let (sx, sz) = step(crate::block::rules::property(state, "facing"));
            (-sx, -sz)
        };
        let lit = || crate::block::rules::property(state, "lit") == Some("true");
        let flame = |torch: &str| match torch {
            "soul" => "soul_fire_flame",
            "copper" => "copper_fire_flame",
            _ => "flame",
        };
        match name {
            "torch" | "soul_torch" | "copper_torch" => {
                let torch = name.trim_end_matches("torch").trim_end_matches('_');
                self.level_add(context, "smoke", (x, y, z), (0.0, 0.0, 0.0));
                self.level_add(context, flame(torch), (x, y, z), (0.0, 0.0, 0.0));
            }
            "wall_torch" | "soul_wall_torch" | "copper_wall_torch" => {
                let torch = name.trim_end_matches("wall_torch").trim_end_matches('_');
                let (ax, az) = away();
                let at = (x + 0.27 * ax, y + 0.22, z + 0.27 * az);
                self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
                self.level_add(context, flame(torch), at, (0.0, 0.0, 0.0));
            }
            "redstone_torch" | "redstone_wall_torch" if lit() => {
                let (ax, az, up) = if name == "redstone_wall_torch" {
                    let (ax, az) = away();
                    (0.27 * ax, 0.27 * az, 0.22)
                } else {
                    (0.0, 0.0, 0.0)
                };
                let x = x + (random.next_double() - 0.5) * 0.2 + ax;
                let y = y + (random.next_double() - 0.5) * 0.2 + up;
                let z = z + (random.next_double() - 0.5) * 0.2 + az;
                self.level_add_data(context, "dust", REDSTONE, (x, y, z));
            }
            _ => {}
        }
    }

    /// `BaseFireBlock.animateTick`: its crackle now and then; smoke off the burning faces about a
    /// fire that floats, or rising through one that stands on something.
    fn fire<F: FnMut() -> RandomSource>(
        &mut self,
        (bx, by, bz): (i32, i32, i32),
        soul: bool,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        let (x, y, z) = (f64::from(bx), f64::from(by), f64::from(bz));
        if random.next_int_bound(24) == Ok(0) {
            let volume = 1.0 + random.next_float();
            let pitch = random.next_float() * 0.7 + 0.3;
            let at = (x + 0.5, y + 0.5, z + 0.5);
            let event = ("minecraft:block.fire.ambient", LocalSoundSource::Blocks);
            sounds.push(local_sound(event, (volume, pitch), at, context));
        }
        let state = |dx: i32, dy: i32, dz: i32| {
            context
                .world
                .block_state_at(bx + dx, by + dy, bz + dz)
                .unwrap_or(0)
        };
        // `SoulFireBlock.canBurn` is always true.
        let burns = |state: i32| soul || can_burn(state);
        let below = state(0, -1, 0);
        let below_sturdy = crate::block::state(BlockStateId(below))
            .is_some_and(|info| info.is_face_sturdy(photon_core::direction::Direction::Up));
        let tenth = f64::from(0.1_f32);
        if !burns(below) && !below_sturdy {
            let faces = [
                (burns(state(-1, 0, 0)), 0),
                (burns(state(1, 0, 0)), 1),
                (burns(state(0, 0, -1)), 2),
                (burns(state(0, 0, 1)), 3),
                (burns(state(0, 1, 0)), 4),
            ];
            for (burning, face) in faces {
                if !burning {
                    continue;
                }
                for _ in 0..2 {
                    let mut draw = || random.next_double();
                    let at = match face {
                        0 => (x + draw() * tenth, y + draw(), z + draw()),
                        1 => (x + 1.0 - draw() * tenth, y + draw(), z + draw()),
                        2 => (x + draw(), y + draw(), z + draw() * tenth),
                        3 => (x + draw(), y + draw(), z + 1.0 - draw() * tenth),
                        _ => (x + draw(), y + 1.0 - draw() * tenth, z + draw()),
                    };
                    self.level_add(context, "large_smoke", at, (0.0, 0.0, 0.0));
                }
            }
        } else {
            for _ in 0..3 {
                let at = (
                    x + random.next_double(),
                    y + random.next_double() * 0.5 + 0.5,
                    z + random.next_double(),
                );
                self.level_add(context, "large_smoke", at, (0.0, 0.0, 0.0));
            }
        }
    }

    /// `CampfireBlock.animateTick` of a lit one: its crackle, and a spark from a campfire (not a
    /// soul campfire's) now and then.
    fn campfire<F: FnMut() -> RandomSource>(
        &mut self,
        (x, y, z): (i32, i32, i32),
        sparks: bool,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        let at = (f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5);
        if random.next_int_bound(10) == Ok(0) {
            let volume = 0.5 + random.next_float();
            let pitch = random.next_float() * 0.7 + 0.6;
            let event = ("minecraft:block.campfire.crackle", LocalSoundSource::Blocks);
            sounds.push(local_sound(event, (volume, pitch), at, context));
        }
        if sparks && random.next_int_bound(5) == Ok(0) {
            // The loop's bound is drawn again before each pass.
            let mut made = 0;
            while made < random.next_int_bound(1).unwrap_or(0) + 1 {
                let push = (
                    f64::from(random.next_float() / 2.0),
                    5.0E-5,
                    f64::from(random.next_float() / 2.0),
                );
                self.level_add(context, "lava", at, push);
                made += 1;
            }
        }
    }

    /// `FurnaceBlock`, `BlastFurnaceBlock` and `SmokerBlock` lit: a crackle one time in ten, and
    /// smoke (and a furnace's flame) out of the front, or a smoker's out of the top.
    fn furnace<F: FnMut() -> RandomSource>(
        &mut self,
        state: i32,
        (bx, by, bz): (i32, i32, i32),
        name: &str,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        let (x, y, z) = (f64::from(bx) + 0.5, f64::from(by), f64::from(bz) + 0.5);
        if random.next_double() < 0.1 {
            let event = match name {
                "furnace" => "minecraft:block.furnace.fire_crackle",
                "blast_furnace" => "minecraft:block.blastfurnace.fire_crackle",
                _ => "minecraft:block.smoker.smoke",
            };
            let source = (event, LocalSoundSource::Blocks);
            sounds.push(local_sound(source, (1.0, 1.0), (x, y, z), context));
        }
        if name == "smoker" {
            self.level_add(context, "smoke", (x, y + 1.1, z), (0.0, 0.0, 0.0));
            return;
        }
        let facing = crate::block::rules::property(state, "facing");
        let (sx, sz) = step(facing);
        let along_x = matches!(facing, Some("east" | "west"));
        let side = random.next_double() * 0.6 - 0.3;
        let dx = if along_x { sx * 0.52 } else { side };
        let height = if name == "furnace" { 6.0 } else { 9.0 };
        let dy = random.next_double() * height / 16.0;
        let dz = if along_x { side } else { sz * 0.52 };
        let at = (x + dx, y + dy, z + dz);
        self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
        if name == "furnace" {
            self.level_add(context, "flame", at, (0.0, 0.0, 0.0));
        }
    }

    /// `AbstractCandleBlock.animateTick` of a lit one: at each wick, smoke three times in ten and
    /// its sound under one in five, then its small flame.
    fn candle_wicks<F: FnMut() -> RandomSource>(
        &mut self,
        (bx, by, bz): (i32, i32, i32),
        wicks: &[[f64; 3]],
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        for [wx, wy, wz] in wicks {
            let at = (
                wx * 0.0625 + f64::from(bx),
                wy * 0.0625 + f64::from(by),
                wz * 0.0625 + f64::from(bz),
            );
            let chance = random.next_float();
            if chance < 0.3 {
                self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
                if chance < 0.17 {
                    let volume = 1.0 + random.next_float();
                    let pitch = random.next_float() * 0.7 + 0.3;
                    let event = ("minecraft:block.candle.ambient", LocalSoundSource::Blocks);
                    let place = (at.0 + 0.5, at.1 + 0.5, at.2 + 0.5);
                    sounds.push(local_sound(event, (volume, pitch), place, context));
                }
            }
            self.level_add(context, "small_flame", at, (0.0, 0.0, 0.0));
        }
    }
}
