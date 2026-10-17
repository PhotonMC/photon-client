//! More of `Block.animateTick`, for the blocks that read only the blocks about them, their tags and
//! the random: the wither rose's smoke, the end portal's, the dried ghast's, bookshelves' glyphs
//! over an enchanting table, honey dripping from a hive, dust under a brushable block, and the
//! idle sounds of moss, eyeblossoms, dead bushes and dry grass.

use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;
use photon_core::math::mth_lerp_f64;
use photon_core::random::RandomSource;
use photon_data::block::blocks::id;
use photon_data::block::classes::BlockClass;
use photon_data::block::registry::Block;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::{ambient, local_sound};
use super::animate_falling::{in_tag, is_free, state_at, world_surface};
use super::engine::{ParticleData, ParticleEngine};
use super::level_events::LevelEventContext;

type Pos = (i32, i32, i32);

/// `EnchantingTableBlock.BOOKSHELF_OFFSETS`: the ring two blocks off, at the table's height and the
/// one above, in the order `BlockPos.betweenClosed` goes (x fastest, then y, then z).
fn bookshelf_offsets() -> impl Iterator<Item = Pos> {
    (-2..=2_i32)
        .flat_map(|z| (0..2_i32).flat_map(move |y| (-2..=2_i32).map(move |x| (x, y, z))))
        .filter(|(x, _, z)| x.abs() == 2 || z.abs() == 2)
}

impl ParticleEngine {
    /// `Block.animateTick` of these blocks; false for another block.
    pub(super) fn ambient_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, block): (BlockStateId, &'static Block),
        pos: Pos,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) -> bool {
        self.ambient_particles_tick((state, block), pos, random, (context, sounds))
            || self.ambient_sounds_tick((state, block), pos, random, (context, sounds))
    }

    /// The blocks that show something.
    fn ambient_particles_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, block): (BlockStateId, &'static Block),
        pos: Pos,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) -> bool {
        let class = block.class;
        if class.is(BlockClass::WitherRoseBlock) {
            self.wither_rose(state, pos, random, context);
        } else if class.is(BlockClass::EndPortalBlock) {
            let at = (
                f64::from(pos.0) + random.next_double(),
                f64::from(pos.1) + 0.8,
                f64::from(pos.2) + random.next_double(),
            );
            self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
        } else if class.is(BlockClass::DriedGhastBlock) {
            self.dried_ghast(state, pos, random, (context, sounds));
        } else if class.is(BlockClass::EnchantingTableBlock) {
            self.enchanting_table(pos, random, context);
        } else if class.is(BlockClass::BrushableBlock) {
            self.brushable_block(state, pos, random, context);
        } else if class.is(BlockClass::BeehiveBlock) {
            self.beehive(state, pos, random, context);
        } else if class.is(BlockClass::EndGatewayBlock) {
            self.end_gateway(state, pos, random, context);
        } else if class.is(BlockClass::PointedDripstoneBlock) {
            self.dripstone_animate_tick(state, pos, random, context);
        } else if class.is(BlockClass::LightningRodBlock) {
            self.lightning_rod(state, pos, context);
        } else {
            return false;
        }
        true
    }

    /// `EndGatewayBlock.animateTick`: portal specks for each of its faces that shows.
    fn end_gateway<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        (px, py, pz): Pos,
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let count = Direction::stream()
            .filter(|direction| {
                let (dx, dy, dz) = direction.step();
                let next = context.world.block_state_at(px + dx, py + dy, pz + dz);
                crate::block::should_render_face(
                    state,
                    next,
                    *direction,
                    context.world.block_state_registry().tags(),
                )
            })
            .count();
        for _ in 0..count {
            let mut at = (
                f64::from(px) + random.next_double(),
                f64::from(py) + random.next_double(),
                f64::from(pz) + random.next_double(),
            );
            let mut push = (
                (random.next_double() - 0.5) * 0.5,
                (random.next_double() - 0.5) * 0.5,
                (random.next_double() - 0.5) * 0.5,
            );
            let flip = random.next_int_bound(2).unwrap_or(0) * 2 - 1;
            if random.next_boolean() {
                at.2 = f64::from(pz) + 0.5 + 0.25 * f64::from(flip);
                push.2 = f64::from(random.next_float() * 2.0 * flip as f32);
            } else {
                at.0 = f64::from(px) + 0.5 + 0.25 * f64::from(flip);
                push.0 = f64::from(random.next_float() * 2.0 * flip as f32);
            }
            self.level_add(context, "portal", at, push);
        }
    }

    /// `LightningRodBlock.animateTick`: in a thunderstorm, sparks along a rod that stands at the
    /// top of its column, more the later in the second it is.
    fn lightning_rod<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: Pos,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let world = context.world;
        if !world.is_thundering() {
            return;
        }
        let late = context.level_random.next_int_bound(200).unwrap_or(0);
        if i64::from(late) > (world.game_time() % 200) as i64
            || pos.1 != world_surface(world, pos.0, pos.2) - 1
        {
            return;
        }
        let facing = crate::block::rules::property(state, "facing").and_then(Direction::from_name);
        if let Some(axis) = facing.map(Direction::axis) {
            self.sparks_along_axis(axis, pos, (1, 2), context);
        }
    }

    /// `WitherRoseBlock.animateTick`: smoke in a column from its shape's middle.
    fn wither_rose<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        (px, py, pz): Pos,
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let shape = crate::block::registry::outline_shape(state, [px, py, pz]);
        let Some(centre) = shape.bounds().map(|bounds| bounds.center()) else {
            return;
        };
        let (x, z) = (f64::from(px) + centre.x, f64::from(pz) + centre.z);
        for _ in 0..3 {
            if random.next_boolean() {
                let at = (
                    x + random.next_double() / 5.0,
                    f64::from(py) + (0.5 - random.next_double()),
                    z + random.next_double() / 5.0,
                );
                self.level_add(context, "smoke", at, (0.0, 0.0, 0.0));
            }
        }
    }

    /// `DriedGhastBlock.animateTick`: its murmur and wisps of smoke, or in water its sound and
    /// happy specks.
    fn dried_ghast<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        (px, py, pz): Pos,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) {
        let (x, y, z) = (
            f64::from(px) + 0.5,
            f64::from(py) + 0.5,
            f64::from(pz) + 0.5,
        );
        let wet = crate::block::rules::property(state, "waterlogged") == Some("true");
        let sound = |event: &'static str, context: &mut LevelEventContext<'_, F>| {
            local_sound(
                (event, LocalSoundSource::Blocks),
                (1.0, 1.0),
                (x, y, z),
                context,
            )
        };
        if !wet {
            let below = state_at(context.world, (px, py - 1, pz));
            if random.next_int_bound(40) == Ok(0)
                && in_tag(
                    context.world,
                    below,
                    "minecraft:triggers_ambient_dried_ghast_block_sounds",
                )
            {
                sounds.push(sound("minecraft:block.dried_ghast.ambient", context));
            }
            if random.next_int_bound(6) == Ok(0) {
                self.level_add(context, "white_smoke", (x, y, z), (0.0, 0.02, 0.0));
            }
            return;
        }
        if random.next_int_bound(40) == Ok(0) {
            sounds.push(sound("minecraft:block.dried_ghast.ambient_water", context));
        }
        if random.next_int_bound(6) == Ok(0) {
            let spread =
                |random: &mut RandomSource| f64::from((random.next_float() * 2.0 - 1.0) / 3.0);
            let at = (x + spread(random), y + 0.4, z + spread(random));
            let rise = f64::from(random.next_float());
            self.level_add(context, "happy_villager", at, (0.0, rise, 0.0));
        }
    }

    /// `EnchantingTableBlock.animateTick`: a glyph flies from the book over the table to each
    /// bookshelf that reaches it, one time in sixteen.
    fn enchanting_table<F: FnMut() -> RandomSource>(
        &mut self,
        (px, py, pz): Pos,
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        for (ox, oy, oz) in bookshelf_offsets() {
            if random.next_int_bound(16) != Ok(0)
                || !valid_bookshelf(context, (px, py, pz), (ox, oy, oz))
            {
                continue;
            }
            let at = (
                f64::from(px) + 0.5,
                f64::from(py) + 2.0,
                f64::from(pz) + 0.5,
            );
            let push = (
                f64::from(ox as f32 + random.next_float()) - 0.5,
                f64::from(oy as f32 - random.next_float() - 1.0),
                f64::from(oz as f32 + random.next_float()) - 0.5,
            );
            self.level_add(context, "enchant", at, push);
        }
    }

    /// `BrushableBlock.animateTick`: dust sifts out of it where there is room below.
    fn brushable_block<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        (px, py, pz): Pos,
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        if random.next_int_bound(16) == Ok(0)
            && is_free(context.world, state_at(context.world, (px, py - 1, pz)))
        {
            let at = (
                f64::from(px) + random.next_double(),
                f64::from(py) - 0.05,
                f64::from(pz) + random.next_double(),
            );
            self.level_add_data(context, "falling_dust", ParticleData::Block(state), at);
        }
    }

    /// `BeehiveBlock.animateTick`: a full hive drips honey (`random.nextInt(1) + 1` is one try).
    fn beehive<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: Pos,
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let honey: i32 = crate::block::rules::property(state, "honey_level")
            .and_then(|level| level.parse().ok())
            .unwrap_or(0);
        if honey < 5 {
            return;
        }
        // The bound is asked again on every pass of the loop.
        let mut tries = 0;
        while tries < random.next_int_bound(1).unwrap_or(0) + 1 {
            self.hive_drip(state, pos, context);
            tries += 1;
        }
    }

    /// `BeehiveBlock.trySpawnDripParticles`: from the underside of a hive, or from one that stands
    /// clear above what lets the honey fall.
    fn hive_drip<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        (px, py, pz): Pos,
        context: &mut LevelEventContext<'_, F>,
    ) {
        use photon_core::direction::Axis;
        let info = crate::block::state_or_air(state);
        if !info.fluid.is_empty() || context.level_random.next_float() < 0.3 {
            return;
        }
        let shape = crate::block::registry::collision_shape_at(state, [px, py, pz]);
        let impermeable = in_tag(context.world, state, "minecraft:impermeable");
        if shape.max(Axis::Y) < 1.0 || impermeable {
            return;
        }
        let bottom = shape.min(Axis::Y);
        let height = if bottom > 0.0 {
            f64::from(py) + bottom - 0.05
        } else {
            let below_pos = (px, py - 1, pz);
            let below = state_at(context.world, below_pos);
            let below_info = crate::block::state_or_air(below);
            let below_shape = crate::block::registry::collision_shape_at(below, [px, py - 1, pz]);
            let full = below_info.collision_full_block;
            if !((below_shape.max(Axis::Y) < 1.0 || !full) && below_info.fluid.is_empty()) {
                return;
            }
            f64::from(py) - 0.05
        };
        let (fx, fz) = (f64::from(px), f64::from(pz));
        let x = mth_lerp_f64(
            context.level_random.next_double(),
            fx + shape.min(Axis::X),
            fx + shape.max(Axis::X),
        );
        let z = mth_lerp_f64(
            context.level_random.next_double(),
            fz + shape.min(Axis::Z),
            fz + shape.max(Axis::Z),
        );
        self.level_add(context, "dripping_honey", (x, height, z), (0.0, 0.0, 0.0));
    }

    /// The blocks that only make idle sounds.
    fn ambient_sounds_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, block): (BlockStateId, &'static Block),
        pos: Pos,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) -> bool {
        let _ = state;
        let class = block.class;
        let sound = if class.is(BlockClass::HangingMossBlock) {
            hanging_moss(pos, random, context)
        } else if class.is(BlockClass::EyeblossomBlock) {
            eyeblossom(block, pos, random, context)
        } else if class.is(BlockClass::DryVegetationBlock) {
            dead_bush(pos, random, context)
        } else if class.is(BlockClass::ShortDryGrassBlock)
            || class.is(BlockClass::TallDryGrassBlock)
        {
            dry_grass(pos, random, context)
        } else {
            return false;
        };
        sounds.extend(sound);
        true
    }
}

/// `EnchantingTableBlock.isValidBookShelf`: a bookshelf at the offset, with nothing but air (or
/// what lets power through) half way between it and the table.
fn valid_bookshelf<F: FnMut() -> RandomSource>(
    context: &LevelEventContext<'_, F>,
    (px, py, pz): Pos,
    (ox, oy, oz): Pos,
) -> bool {
    let world = context.world;
    let shelf = state_at(world, (px + ox, py + oy, pz + oz));
    let between = state_at(world, (px + ox / 2, py + oy, pz + oz / 2));
    in_tag(world, shelf, "minecraft:enchantment_power_provider")
        && in_tag(world, between, "minecraft:enchantment_power_transmitter")
}

/// `HangingMossBlock.animateTick`: now and then a creak, under pale oak.
fn hanging_moss<F: FnMut() -> RandomSource>(
    (px, py, pz): Pos,
    random: &mut RandomSource,
    context: &mut LevelEventContext<'_, F>,
) -> Option<LocalSound> {
    if random.next_int_bound(500) != Ok(0) {
        return None;
    }
    let above = state_at(context.world, (px, py + 1, pz));
    let held = in_tag(context.world, above, "minecraft:pale_oak_logs")
        || usize::from(crate::block::state_or_air(above).block) == id::PALE_OAK_LEAVES;
    held.then(|| {
        let at = (f64::from(px), f64::from(py), f64::from(pz));
        ambient(
            "minecraft:block.pale_hanging_moss.idle",
            (1.0, 1.0),
            at,
            context,
        )
    })
}

/// `EyeblossomBlock.animateTick`: an open one over pale moss whispers now and then.
fn eyeblossom<F: FnMut() -> RandomSource>(
    block: &'static Block,
    (px, py, pz): Pos,
    random: &mut RandomSource,
    context: &mut LevelEventContext<'_, F>,
) -> Option<LocalSound> {
    if block.name != "minecraft:open_eyeblossom" || random.next_int_bound(700) != Ok(0) {
        return None;
    }
    let below = state_at(context.world, (px, py - 1, pz));
    let pale_moss = usize::from(crate::block::state_or_air(below).block) == id::PALE_MOSS_BLOCK;
    pale_moss.then(|| {
        let at = (f64::from(px), f64::from(py), f64::from(pz));
        ambient("minecraft:block.eyeblossom.idle", (1.0, 1.0), at, context)
    })
}

/// `AmbientDesertBlockSoundsPlayer.shouldPlayDesertDryVegetationBlockSounds`: sand or the like two
/// blocks deep.
fn dry_vegetation_ground<F: FnMut() -> RandomSource>(
    context: &LevelEventContext<'_, F>,
    (x, y, z): Pos,
) -> bool {
    let tag = "minecraft:triggers_ambient_desert_dry_vegetation_block_sounds";
    in_tag(context.world, state_at(context.world, (x, y, z)), tag)
        && in_tag(context.world, state_at(context.world, (x, y - 1, z)), tag)
}

/// `AmbientDesertBlockSoundsPlayer.playAmbientDeadBushSounds`.
fn dead_bush<F: FnMut() -> RandomSource>(
    (px, py, pz): Pos,
    random: &mut RandomSource,
    context: &mut LevelEventContext<'_, F>,
) -> Option<LocalSound> {
    if random.next_int_bound(130) != Ok(0) {
        return None;
    }
    let below = state_at(context.world, (px, py - 1, pz));
    let red_sand = usize::from(crate::block::state_or_air(below).block) == id::RED_SAND;
    let terracotta = in_tag(context.world, below, "minecraft:terracotta");
    if (red_sand || terracotta) && random.next_int_bound(3) != Ok(0) {
        return None;
    }
    dry_vegetation_ground(context, (px, py - 1, pz)).then(|| {
        let at = (f64::from(px), f64::from(py), f64::from(pz));
        ambient("minecraft:block.deadbush.idle", (1.0, 1.0), at, context)
    })
}

/// `AmbientDesertBlockSoundsPlayer.playAmbientDryGrassSounds`: the sound is the player's own
/// (`Level.playPlayerSound`), made only where the client has a player.
fn dry_grass<F: FnMut() -> RandomSource>(
    (px, py, pz): Pos,
    random: &mut RandomSource,
    context: &mut LevelEventContext<'_, F>,
) -> Option<LocalSound> {
    if random.next_int_bound(200) != Ok(0) || !dry_vegetation_ground(context, (px, py - 1, pz)) {
        return None;
    }
    context.local_player?;
    context.level_random.next_long();
    let mut sound = LocalSound::at(
        "minecraft:block.dry_grass.ambient",
        LocalSoundSource::Ambient,
        (1.0, 1.0),
        (0.0, 0.0, 0.0),
    );
    sound.relative = true;
    Some(sound)
}
