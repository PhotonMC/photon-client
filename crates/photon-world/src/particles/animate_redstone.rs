//! More of `Block.animateTick`: the lever, redstone ore, the repeater and redstone dust, a wet
//! sponge, a bubble column, the ender chest and the sculk sensors.

use photon_core::direction::Direction;
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

/// The horizontal directions, in `Direction.Plane.HORIZONTAL` order.
const HORIZONTAL: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// `RedStoneWireBlock.COLORS`: the colour of a power, `ARGB.colorFromFloat(1, r, g, b)`.
fn wire_color(power: i32) -> u32 {
    let power = power as f32 / 15.0;
    let red = power * 0.6 + if power > 0.0 { 0.4 } else { 0.3 };
    let green = (power * power * 0.7 - 0.5).clamp(0.0, 1.0);
    let blue = (power * power * 0.6 - 0.7).clamp(0.0, 1.0);
    let channel = |value: f32| (value * 255.0).floor() as i32 as u32 & 0xFF;
    0xFF00_0000 | channel(red) << 16 | channel(green) << 8 | channel(blue)
}

fn direction_of(name: Option<&str>) -> Direction {
    name.and_then(Direction::from_name)
        .unwrap_or(Direction::North)
}

impl ParticleEngine {
    /// `Block.animateTick` of these blocks; false for another block.
    pub(super) fn redstone_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, name): (BlockStateId, &str),
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) -> bool {
        let property = |key: &str| crate::block::rules::property(state, key);
        match name {
            "lever" => {
                if property("powered") == Some("true") && random.next_float() < 0.25 {
                    self.lever_particle(state, pos, 0.5, context);
                }
            }
            "redstone_ore" | "deepslate_redstone_ore" => {
                if property("lit") == Some("true") {
                    self.redstone_ore_particles(pos, context);
                }
            }
            "repeater" => self.repeater(state, pos, random, context),
            "redstone_wire" => self.redstone_wire(state, pos, random, context),
            "wet_sponge" => self.wet_sponge(pos, random, context),
            "bubble_column" => self.bubble_column(state, pos, random, (context, sounds)),
            "ender_chest" => self.ender_chest(pos, random, context),
            "sculk_sensor" | "calibrated_sculk_sensor" => {
                self.sculk_sensor(state, pos, random, context);
            }
            _ => return false,
        }
        true
    }

    /// `RepeaterBlock.animateTick`: a speck of dust along its torches while it is powered.
    fn repeater<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let property = |key: &str| crate::block::rules::property(state, key);
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        if property("powered") == Some("true") {
            let facing = direction_of(property("facing")).step();
            let at = (
                x + 0.5 + (random.next_double() - 0.5) * 0.2,
                y + 0.4 + (random.next_double() - 0.5) * 0.2,
                z + 0.5 + (random.next_double() - 0.5) * 0.2,
            );
            let mut offset = -5.0_f32;
            if random.next_boolean() {
                let delay: i32 = property("delay").and_then(|d| d.parse().ok()).unwrap_or(1);
                offset = (delay * 2 - 1) as f32;
            }
            offset /= 16.0;
            let (xo, zo) = (
                f64::from(offset) * f64::from(facing.0),
                f64::from(offset) * f64::from(facing.2),
            );
            self.level_add_data(context, "dust", REDSTONE, (at.0 + xo, at.1, at.2 + zo));
        }
    }

    /// `BubbleColumnBlock.animateTick`: bubbles rising (or a current drawing down) and the sound.
    fn bubble_column<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        let property = |key: &str| crate::block::rules::property(state, key);
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        let drag_down = property("drag") == Some("true");
        if drag_down {
            self.level_add_always_visible(
                context,
                "current_down",
                (x + 0.5, y + 0.8, z),
                (0.0, 0.0, 0.0),
            );
        } else {
            self.level_add_always_visible(
                context,
                "bubble_column_up",
                (x + 0.5, y, z + 0.5),
                (0.0, 0.04, 0.0),
            );
            let at = (
                x + f64::from(random.next_float()),
                y + f64::from(random.next_float()),
                z + f64::from(random.next_float()),
            );
            self.level_add_always_visible(context, "bubble_column_up", at, (0.0, 0.04, 0.0));
        }
        if random.next_int_bound(200) == Ok(0) {
            let volume = 0.2 + random.next_float() * 0.2;
            let pitch = 0.9 + random.next_float() * 0.15;
            let event = if drag_down {
                "minecraft:block.bubble_column.whirlpool_ambient"
            } else {
                "minecraft:block.bubble_column.upwards_ambient"
            };
            let event = (event, LocalSoundSource::Blocks);
            sounds.push(local_sound(event, (volume, pitch), (x, y, z), context));
        }
    }

    /// `EnderChestBlock.animateTick`: three portal specks drawn into the chest.
    fn ender_chest<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let (x, z) = (f64::from(pos.0), f64::from(pos.2));
        for _ in 0..3 {
            let flip_x = random.next_int_bound(2).unwrap_or(0) * 2 - 1;
            let flip_z = random.next_int_bound(2).unwrap_or(0) * 2 - 1;
            let px = x + 0.5 + 0.25 * f64::from(flip_x);
            // `pos.getY() + random.nextFloat()` adds in `float`.
            let py = f64::from(pos.1 as f32 + random.next_float());
            let pz = z + 0.5 + 0.25 * f64::from(flip_z);
            let xa = f64::from(random.next_float() * flip_x as f32);
            let ya = (f64::from(random.next_float()) - 0.5) * 0.125;
            let za = f64::from(random.next_float() * flip_z as f32);
            self.level_add(context, "portal", (px, py, pz), (xa, ya, za));
        }
    }

    /// `SculkSensorBlock.animateTick`: a spark off a side of an active sensor.
    fn sculk_sensor<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let property = |key: &str| crate::block::rules::property(state, key);
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        if property("sculk_sensor_phase") == Some("active") {
            let direction = Direction::get_random(random);
            if direction != Direction::Up && direction != Direction::Down {
                let (sx, _, sz) = direction.step();
                let px = x
                    + 0.5
                    + if sx == 0 {
                        0.5 - random.next_double()
                    } else {
                        f64::from(sx) * 0.6
                    };
                let pz = z
                    + 0.5
                    + if sz == 0 {
                        0.5 - random.next_double()
                    } else {
                        f64::from(sz) * 0.6
                    };
                let ya = f64::from(random.next_float()) * 0.04;
                // `DustColorTransitionOptions.SCULK_TO_REDSTONE`.
                let data = ParticleData::DustTransition {
                    from: 3_790_560,
                    to: 0xFF_0000,
                    scale: 1.0,
                };
                self.level_add_data_pushed(
                    context,
                    "dust_color_transition",
                    data,
                    (px, y + 0.25, pz),
                    (0.0, ya, 0.0),
                );
            }
        }
    }

    /// `LeverBlock.makeParticle`: a red speck at the lever's handle.
    fn lever_particle<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: (i32, i32, i32),
        scale: f32,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let property = |key: &str| crate::block::rules::property(state, key);
        let facing = direction_of(property("facing"));
        let connected = match property("face") {
            Some("ceiling") => Direction::Down,
            Some("floor") => Direction::Up,
            _ => facing,
        };
        let (a, b) = (facing.opposite().step(), connected.opposite().step());
        let at = (
            f64::from(pos.0) + 0.5 + 0.1 * f64::from(a.0) + 0.2 * f64::from(b.0),
            f64::from(pos.1) + 0.5 + 0.1 * f64::from(a.1) + 0.2 * f64::from(b.1),
            f64::from(pos.2) + 0.5 + 0.1 * f64::from(a.2) + 0.2 * f64::from(b.2),
        );
        let data = ParticleData::Dust {
            color: 0xFF_0000,
            scale,
        };
        self.level_add_data(context, "dust", data, at);
    }

    /// `RedStoneOreBlock.spawnParticles`: dust on each face that is not against a solid block.
    fn redstone_ore_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        for direction in [
            Direction::Down,
            Direction::Up,
            Direction::North,
            Direction::South,
            Direction::West,
            Direction::East,
        ] {
            let (sx, sy, sz) = direction.step();
            let next = context
                .world
                .block_state_or_air(pos.0 + sx, pos.1 + sy, pos.2 + sz);
            if crate::block::state(next).is_some_and(|info| info.solid_render) {
                continue;
            }
            let random = &mut *context.level_random;
            let mut place = |step: i32, along: bool| {
                if along {
                    0.5 + 0.5625 * f64::from(step)
                } else {
                    f64::from(random.next_float())
                }
            };
            let dx = place(sx, sx != 0);
            let dy = place(sy, sy != 0);
            let dz = place(sz, sz != 0);
            let at = (
                f64::from(pos.0) + dx,
                f64::from(pos.1) + dy,
                f64::from(pos.2) + dz,
            );
            self.level_add_data(context, "dust", REDSTONE, at);
        }
    }

    /// `RedStoneWireBlock.animateTick`: dust along each of its arms.
    fn redstone_wire<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let power: i32 = crate::block::rules::property(state, "power")
            .and_then(|power| power.parse().ok())
            .unwrap_or(0);
        if power == 0 {
            return;
        }
        let color = wire_color(power);
        for horizontal in HORIZONTAL {
            let connection = crate::block::rules::property(state, horizontal.name());
            match connection {
                Some("up") => {
                    self.wire_particles(
                        pos,
                        color,
                        (horizontal, Direction::Up, -0.5, 0.5),
                        random,
                        context,
                    );
                    self.wire_particles(
                        pos,
                        color,
                        (Direction::Down, horizontal, 0.0, 0.5),
                        random,
                        context,
                    );
                }
                Some("side") => {
                    self.wire_particles(
                        pos,
                        color,
                        (Direction::Down, horizontal, 0.0, 0.5),
                        random,
                        context,
                    );
                }
                _ => {
                    self.wire_particles(
                        pos,
                        color,
                        (Direction::Down, horizontal, 0.0, 0.3),
                        random,
                        context,
                    );
                }
            }
        }
    }

    /// `RedStoneWireBlock.spawnParticlesAlongLine`.
    fn wire_particles<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        color: u32,
        (side, along, from, to): (Direction, Direction, f32, f32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let span = to - from;
        if random.next_float() >= 0.2 * span {
            return;
        }
        let on_line = from + span * random.next_float();
        let (sx, sy, sz) = side.step();
        let (ax, ay, az) = along.step();
        let side_of_block = f64::from(0.4375_f32);
        let place = |base: i32, side_step: i32, along_step: i32| {
            // `0.5 + side + along`, then the block's place added to it.
            let within = 0.5
                + side_of_block * f64::from(side_step)
                + f64::from(on_line) * f64::from(along_step);
            f64::from(base) + within
        };
        let at = (
            place(pos.0, sx, ax),
            place(pos.1, sy, ay),
            place(pos.2, sz, az),
        );
        let data = ParticleData::Dust { color, scale: 1.0 };
        self.level_add_data(context, "dust", data, at);
    }

    /// `WetSpongeBlock.animateTick`: a drop that forms on a side that is not against a block.
    fn wet_sponge<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let direction = Direction::get_random(random);
        if direction == Direction::Up {
            return;
        }
        let (sx, sy, sz) = direction.step();
        let next = context
            .world
            .block_state_or_air(pos.0 + sx, pos.1 + sy, pos.2 + sz);
        // The sponge occludes: the drop forms unless the neighbour is sturdy toward it.
        let sturdy =
            crate::block::state(next).is_some_and(|info| info.is_face_sturdy(direction.opposite()));
        if sturdy {
            return;
        }
        let (mut x, mut y, mut z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        if direction == Direction::Down {
            y -= 0.05;
            x += random.next_double();
            z += random.next_double();
        } else {
            y += random.next_double() * 0.8;
            if matches!(direction, Direction::West | Direction::East) {
                z += random.next_double();
                x += if direction == Direction::East {
                    1.1
                } else {
                    0.05
                };
            } else {
                x += random.next_double();
                z += if direction == Direction::South {
                    1.1
                } else {
                    0.05
                };
            }
        }
        self.level_add(context, "dripping_water", (x, y, z), (0.0, 0.0, 0.0));
    }

    /// `Level.addAlwaysVisibleParticle` of a plain type: past the limiter, shown whatever the
    /// option.
    fn level_add_always_visible<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        name: &str,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
    ) {
        let Some(type_id) = photon_data::particle_type::id(name) else {
            return;
        };
        let spawn = super::engine::Spawn {
            type_id,
            data: ParticleData::Plain,
            override_limiter: true,
            always_show: true,
            at,
            push,
        };
        self.add_particle(
            context.world,
            &spawn,
            context.view,
            context.level_random,
            &mut context.particle_random,
        );
    }
}
