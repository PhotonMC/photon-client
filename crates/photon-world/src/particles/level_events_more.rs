//! More of `LevelEventHandler.levelEvent`: the composter filling, a splash potion breaking, an ender
//! eye shattering, a spawner's flames, bone meal's sparkles and a mace's smash.

use photon_core::direction::Axis;
use photon_core::ids::ItemId;
use photon_core::math::{fdlibm_cos, fdlibm_sin, mth_next_double};
use photon_core::random::RandomSource;
use photon_data::block::classes::BlockInterface;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::animate::{delayed_sound, local_sound};
use super::engine::{ParticleData, ParticleEngine};
use super::level_events::LevelEventContext;

/// `ARGB.colorFromFloat(1, r, g, b)`.
fn color_from_float([r, g, b]: [f32; 3]) -> u32 {
    let channel = |value: f32| (value * 255.0).floor() as i32 as u32 & 0xFF;
    0xFF00_0000 | channel(r) << 16 | channel(g) << 8 | channel(b)
}

impl ParticleEngine {
    /// The events of this file; false for another.
    pub(super) fn more_level_event<F: FnMut() -> RandomSource>(
        &mut self,
        (event, pos, data): (i32, (i32, i32, i32), i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) -> bool {
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        match event {
            1500 => self.composter_fill(pos, data > 0, context, sounds),
            2002 | 2007 => self.splash_potion(pos, (event == 2007, data), context, sounds),
            2003 => self.ender_eye(pos, context),
            2004 => {
                for _ in 0..20 {
                    let random = &mut *context.level_random;
                    let x = x + 0.5 + (random.next_double() - 0.5) * 2.0;
                    let y = y + 0.5 + (random.next_double() - 0.5) * 2.0;
                    let z = z + 0.5 + (random.next_double() - 0.5) * 2.0;
                    self.level_add(context, "smoke", (x, y, z), (0.0, 0.0, 0.0));
                    self.level_add(context, "flame", (x, y, z), (0.0, 0.0, 0.0));
                }
            }
            3016 => self.vault_deactivate(pos, data, context, sounds),
            1505 => self.bone_meal(pos, data, context, sounds),
            2011 | 2012 => self.particle_in_block(pos, data, context),
            2013 => self.smash_attack(pos, data, context),
            _ => return false,
        }
        true
    }

    /// `ComposterBlock.handleFill`: its sound, and ten specks over its contents.
    fn composter_fill<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        success: bool,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        let event = if success {
            "minecraft:block.composter.fill_success"
        } else {
            "minecraft:block.composter.fill"
        };
        let event = (event, LocalSoundSource::Blocks);
        sounds.push(local_sound(
            event,
            (1.0, 1.0),
            (x + 0.5, y + 0.5, z + 0.5),
            context,
        ));
        let state = context.world.block_state_or_air(pos.0, pos.1, pos.2);
        let shape = crate::block::registry::outline_shape(state, [pos.0, pos.1, pos.2]);
        let center = shape.max_at(Axis::Y, 0.5, 0.5) + 0.031_25;
        for _ in 0..10 {
            let random = &mut *context.level_random;
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            let at = (
                x + 0.1875 + 0.625 * f64::from(random.next_float()),
                y + center + f64::from(random.next_float()) * (1.0 - center),
                z + 0.1875 + 0.625 * f64::from(random.next_float()),
            );
            self.level_add(context, "composter", at, push);
        }
    }

    /// Events 2002 and 2007: the potion's shards, then a hundred specks of its colour flung out,
    /// then the sound of the bottle.
    fn splash_potion<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        (instant, color): (bool, i32),
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let (x, y, z) = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1),
            f64::from(pos.2) + 0.5,
        );
        let shard = photon_data::item::registry::index("minecraft:splash_potion")
            .map_or(ItemId(0), |id| ItemId(id as i32));
        for _ in 0..8 {
            let random = &mut *context.level_random;
            let push = (
                random.next_gaussian() * 0.15,
                random.next_double() * 0.2,
                random.next_gaussian() * 0.15,
            );
            self.level_add_data_pushed(context, "item", ParticleData::Item(shard), (x, y, z), push);
        }
        let channel = |shift: i32| ((color >> shift) & 0xFF) as f32 / 255.0;
        let rgb = [channel(16), channel(8), channel(0)];
        let name = if instant { "instant_effect" } else { "effect" };
        for _ in 0..100 {
            let random = &mut *context.level_random;
            let dist = random.next_double() * 4.0;
            let angle = random.next_double() * std::f64::consts::PI * 2.0;
            let (vx, vz) = (fdlibm_cos(angle) * dist, fdlibm_sin(angle) * dist);
            let vy = 0.01 + random.next_double() * 0.5;
            let brightness = 0.75 + random.next_float() * 0.25;
            let data = ParticleData::Spell {
                color: color_from_float(rgb.map(|channel| channel * brightness)),
                power: dist as f32,
            };
            let at = (x + vx * 0.1, y + 0.3, z + vz * 0.1);
            self.level_add_data_pushed(context, name, data, at, (vx, vy, vz));
        }
        let pitch = context.level_random.next_float() * 0.1 + 0.9;
        let event = (
            "minecraft:entity.splash_potion.break",
            LocalSoundSource::Neutral,
        );
        let place = (x, y + 0.5, z);
        sounds.push(local_sound(event, (1.0, pitch), place, context));
    }

    /// Event 2003: an ender eye's shards, and a ring of portal specks drawn in at two speeds.
    fn ender_eye<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        let (x, y, z) = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1),
            f64::from(pos.2) + 0.5,
        );
        let eye = photon_data::item::registry::index("minecraft:ender_eye")
            .map_or(ItemId(0), |id| ItemId(id as i32));
        for _ in 0..8 {
            let random = &mut *context.level_random;
            let push = (
                random.next_gaussian() * 0.15,
                random.next_double() * 0.2,
                random.next_gaussian() * 0.15,
            );
            self.level_add_data_pushed(context, "item", ParticleData::Item(eye), (x, y, z), push);
        }
        let mut angle = 0.0_f64;
        while angle < std::f64::consts::PI * 2.0 {
            let (cos, sin) = (fdlibm_cos(angle), fdlibm_sin(angle));
            let at = (x + cos * 5.0, y - 0.4, z + sin * 5.0);
            self.level_add(context, "portal", at, (cos * -5.0, 0.0, sin * -5.0));
            self.level_add(context, "portal", at, (cos * -7.0, 0.0, sin * -7.0));
            angle += std::f64::consts::PI / 20.0;
        }
    }

    /// `ParticleUtils.spawnParticleInBlock` of happy villager specks (bone meal, a bee's pollen).
    fn particle_in_block<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        count: i32,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let state = context.world.block_state_or_air(pos.0, pos.1, pos.2);
        let height = if crate::block::state_or_air(state).block().air {
            1.0
        } else {
            crate::block::registry::outline_shape(state, [pos.0, pos.1, pos.2]).max(Axis::Y)
        };
        self.happy_villagers(pos, count, (0.5, height, true), context);
    }

    /// `ParticleUtils.spawnParticles` of happy villager specks: `count` of them spread `width`
    /// either way of the block's centre and `height` up, the ones with air under them left out
    /// unless `floating`.
    fn happy_villagers<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        count: i32,
        (width, height, floating): (f64, f64, bool),
        context: &mut LevelEventContext<'_, F>,
    ) {
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        for _ in 0..count {
            let random = &mut *context.level_random;
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            let at = (
                x + (0.5 - width) + random.next_double() * width * 2.0,
                y + random.next_double() * height,
                z + (0.5 - width) + random.next_double() * width * 2.0,
            );
            let (bx, by, bz) = (
                at.0.floor() as i32,
                at.1.floor() as i32,
                at.2.floor() as i32,
            );
            let below = context.world.block_state_or_air(bx, by - 1, bz);
            if floating || !crate::block::state_or_air(below).block().air {
                self.level_add(context, "happy_villager", at, push);
            }
        }
    }

    /// Event 3016, `VaultBlockEntity.Client.emitDeactivationParticles` then the vault's sound:
    /// twenty flames about the middle of its cage (a soul flame for an ominous vault).
    fn vault_deactivate<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        data: i32,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let flame = if data == 0 {
            "small_flame"
        } else {
            "soul_fire_flame"
        };
        let (x, y, z) = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
        for _ in 0..20 {
            let random = &mut *context.level_random;
            let at = (
                x + mth_next_double(random, 0.4, 0.6),
                y + mth_next_double(random, 0.4, 0.6),
                z + mth_next_double(random, 0.4, 0.6),
            );
            let push = (
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
                random.next_gaussian() * 0.02,
            );
            self.level_add(context, flame, at, push);
        }
        let random = &mut *context.level_random;
        let pitch = (random.next_float() - random.next_float()) * 0.2 + 1.0;
        let centre = (x + 0.5, y + 0.5, z + 0.5);
        let event = ("minecraft:block.vault.deactivate", LocalSoundSource::Blocks);
        sounds.push(delayed_sound(event, (1.0, pitch), centre, context));
    }

    /// Event 1505, `BoneMealItem.addGrowthParticles` then the bone meal's sound: what grows is
    /// sparkled over, what spreads (and water) in a wider cloud about it.
    fn bone_meal<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        count: i32,
        context: &mut LevelEventContext<'_, F>,
        sounds: &mut Vec<LocalSound>,
    ) {
        let state = context.world.block_state_or_air(pos.0, pos.1, pos.2);
        let block = crate::block::state_or_air(state).block();
        let cloud = (3.0, 1.0, false);
        if block.class.implements(BlockInterface::BonemealableBlock) {
            let (x, y, z) = pos;
            let spreads = block.class.bone_meal_spreads();
            let at = match (block.class.particles_below(), spreads) {
                (true, _) => (x, y - 1, z),
                (false, true) => (x, y + 1, z),
                (false, false) => pos,
            };
            if spreads {
                self.happy_villagers(at, count.wrapping_mul(3), cloud, context);
            } else {
                self.particle_in_block(at, count, context);
            }
        } else if block.name == "minecraft:water" {
            self.happy_villagers(pos, count.wrapping_mul(3), cloud, context);
        }
        let center = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        let event = ("minecraft:item.bone_meal.use", LocalSoundSource::Blocks);
        sounds.push(local_sound(event, (1.0, 1.0), center, context));
    }

    /// `ParticleUtils.spawnSmashAttackParticles`: dust of the block struck, a third as many as
    /// asked, about its top.
    fn smash_attack<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        count: i32,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let state = context.world.block_state_or_air(pos.0, pos.1, pos.2);
        let (cx, cy, cz) = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 1.0,
            f64::from(pos.2) + 0.5,
        );
        let fifth = f64::from(0.2_f32);
        let mut made = 0;
        while (made as f32) < count as f32 / 3.0 {
            let random = &mut *context.level_random;
            let x = cx + random.next_gaussian() / 2.0;
            let z = cz + random.next_gaussian() / 2.0;
            let push = (
                random.next_gaussian() * fifth,
                random.next_gaussian() * fifth,
                random.next_gaussian() * fifth,
            );
            self.level_add_data_pushed(
                context,
                "dust_pillar",
                ParticleData::Block(state),
                (x, cy, z),
                push,
            );
            made += 1;
        }
    }
}
