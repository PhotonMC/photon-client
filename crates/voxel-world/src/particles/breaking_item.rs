//! `BreakingItemParticle`: a crumb of an item (a snowball's, a slime ball's, an eaten food's),
//! showing a quarter of the item's picture.

use std::sync::Arc;

use voxel_core::random::RandomSource;

use super::engine::ParticleData;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;
use super::terrain::TerrainMaterials;

impl Particle {
    /// The item crumbs' providers: `item` (the item its options carry, pushed as given), and the
    /// slime ball's, cobweb's and snowball's, and the sulfur cube's goo (its own first picture).
    /// Nothing for another type, or an item with no model.
    pub fn breaking_item_provider(
        name: &str,
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        (data, sprites): (ParticleData, &Arc<SpriteSet>),
        (materials, provider_random): (Option<&dyn TerrainMaterials>, &mut RandomSource),
    ) -> Option<Self> {
        let item = |name: &str| voxel_data::item::registry::index(name).map(|index| index as i32);
        let (item, pushed) = match (name, data) {
            ("item", ParticleData::Item(item)) => (Some(item), true),
            ("item_slime", _) => (item("minecraft:slime_ball"), false),
            ("item_cobweb", _) => (item("minecraft:cobweb"), false),
            ("item_snowball", _) => (item("minecraft:snowball"), false),
            ("sulfur_cube_goo", _) => (None, false),
            _ => return None,
        };
        let particle = match item {
            Some(item) => {
                // `ItemStackRenderState.pickParticleMaterial`: one of the model's layers, or the
                // missing picture for an item with none.
                let textures = materials
                    .map(|materials| materials.item_particle_textures(item))
                    .unwrap_or_default();
                let texture = if textures.is_empty() {
                    "minecraft:missingno".to_owned()
                } else {
                    let index = provider_random
                        .next_int_bound(textures.len() as i32)
                        .unwrap_or(0);
                    textures[index as usize].clone()
                };
                let set = Arc::new(SpriteSet::new(vec![texture]));
                Self::breaking_item(random, (at, push), (&set, 0), (true, pushed))
            }
            None => Self::breaking_item(
                random,
                (at, push),
                (sprites, sprites.first()),
                (false, false),
            ),
        };
        Some(particle)
    }

    /// `BreakingItemParticle(..)` of a picture: pushed as given (the eight-argument one, a tenth of
    /// its own spread kept) or not, in the block atlas or the particles' own.
    pub fn breaking_item(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprite: (&Arc<SpriteSet>, usize),
        (blocks, pushed): (bool, bool),
    ) -> Self {
        let mut particle = Self::crumb(random, at, sprite, blocks);
        if pushed {
            particle.xd = particle.xd * f64::from(0.1_f32) + push.0;
            particle.yd = particle.yd * f64::from(0.1_f32) + push.1;
            particle.zd = particle.zd * f64::from(0.1_f32) + push.2;
        }
        particle
    }

    /// `BreakingItemParticle(level, x, y, z, sprite)`: falls, half the size, a quarter of its
    /// picture at a place drawn at random.
    fn crumb(
        random: RandomSource,
        at: (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
        blocks: bool,
    ) -> Self {
        let placeholder = Kind::Terrain { uo: 0.0, vo: 0.0 };
        let mut particle = Self::thrown(random, at, (0.0, 0.0, 0.0), sprite, placeholder).quad();
        particle.gravity = 1.0;
        particle.quad_size /= 2.0;
        let uo = particle.random.next_float() * 3.0;
        let vo = particle.random.next_float() * 3.0;
        particle.kind = if blocks {
            Kind::Terrain { uo, vo }
        } else {
            Kind::Goo { uo, vo }
        };
        particle
    }
}
