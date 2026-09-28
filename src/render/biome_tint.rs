//! The colours a world gives grass, leaves and water, from the biomes around
//! each block.
//!
//! A tinted block asks its tint source for a colour at its position. The
//! reference answers by averaging a five by five square of columns around the
//! block, each column's colour coming from the biome the block position falls
//! in. Which biome that is uses the biome manager's zoom: the position is
//! nudged towards one of the eight surrounding quarter-scale samples by a
//! seeded jitter, so biome edges are ragged rather than square. The colour of
//! a biome comes from its data: an override, or its temperature and rainfall
//! read off a colour map, then for grass a dark forest or swamp adjustment.

use std::path::Path;

use crate::json::{JsonObject, JsonValue};
use crate::random::RandomSource;
use crate::render::tint::{ColourMap, colour_map_lookup};

/// Which biome colour a tint layer reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TintKind {
    Grass,
    Foliage,
    DryFoliage,
    Water,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum GrassModifier {
    None,
    DarkForest,
    Swamp,
}

#[derive(Clone, Debug)]
struct BiomeColours {
    temperature: f32,
    downfall: f32,
    grass: Option<u32>,
    foliage: Option<u32>,
    dry_foliage: Option<u32>,
    water: u32,
    modifier: GrassModifier,
}

/// `SimplexNoise`, the two-dimensional half, as one octave of the biome
/// information noise uses it.
#[derive(Clone, Debug)]
struct SimplexNoise {
    permutation: [i32; 512],
}

const GRADIENT: [[i32; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

impl SimplexNoise {
    fn new(random: &mut RandomSource) -> Self {
        // The origin offsets are drawn but unused by the two-dimensional
        // lookup without a noise start.
        for _ in 0..3 {
            random.next_double();
        }
        let mut permutation = [0_i32; 512];
        for (index, slot) in permutation.iter_mut().take(256).enumerate() {
            *slot = index as i32;
        }
        for index in 0..256 {
            let offset = random
                .next_int_bound(256 - index as i32)
                .expect("a positive bound") as usize;
            permutation.swap(index, offset + index);
        }
        Self { permutation }
    }

    fn p(&self, index: i32) -> i32 {
        self.permutation[(index & 0xFF) as usize]
    }

    fn corner(index: i32, x: f64, y: f64) -> f64 {
        let t = 0.5 - x * x - y * y;
        if t < 0.0 {
            0.0
        } else {
            let t = t * t;
            let g = GRADIENT[index as usize];
            t * t * (f64::from(g[0]) * x + f64::from(g[1]) * y)
        }
    }

    fn value(&self, x_in: f64, y_in: f64) -> f64 {
        let sqrt3 = 3.0_f64.sqrt();
        let f2 = 0.5 * (sqrt3 - 1.0);
        let g2 = (3.0 - sqrt3) / 6.0;
        let s = (x_in + y_in) * f2;
        let i = (x_in + s).floor() as i32;
        let j = (y_in + s).floor() as i32;
        let t = f64::from(i + j) * g2;
        let x0 = x_in - (f64::from(i) - t);
        let y0 = y_in - (f64::from(j) - t);
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
        let x1 = x0 - f64::from(i1) + g2;
        let y1 = y0 - f64::from(j1) + g2;
        let x2 = x0 - 1.0 + 2.0 * g2;
        let y2 = y0 - 1.0 + 2.0 * g2;
        let ii = i & 0xFF;
        let jj = j & 0xFF;
        let gi0 = self.p(ii + self.p(jj)) % 12;
        let gi1 = self.p(ii + i1 + self.p(jj + j1)) % 12;
        let gi2 = self.p(ii + 1 + self.p(jj + 1)) % 12;
        70.0 * (Self::corner(gi0, x0, y0) + Self::corner(gi1, x1, y1) + Self::corner(gi2, x2, y2))
    }
}

/// Biome colours for a world, by the server's biome numbering.
#[derive(Clone, Debug)]
pub struct BiomeTints {
    biomes: Vec<BiomeColours>,
    plains: usize,
    grass: Vec<u32>,
    foliage: Vec<u32>,
    dry_foliage: Vec<u32>,
    zoom_seed: i64,
    swamp_noise: SimplexNoise,
}

fn read_json(path: &Path) -> Option<JsonObject> {
    let text = std::fs::read_to_string(path).ok()?;
    match crate::json::parse(&text).ok()? {
        JsonValue::Object(object) => Some(object),
        _ => None,
    }
}

fn hex_colour(object: &JsonObject, key: &str) -> Option<u32> {
    let text = object.text(key)?;
    let hex = text.strip_prefix('#')?;
    Some(0xFF00_0000 | u32::from_str_radix(hex, 16).ok()?)
}

fn colour_map(assets_root: &Path, name: &str) -> Option<Vec<u32>> {
    let bytes =
        std::fs::read(assets_root.join(format!("minecraft/textures/colormap/{name}.png"))).ok()?;
    let image = crate::render::png::decode_rgba(&bytes).ok()?;
    Some(
        image
            .rgba
            .chunks_exact(4)
            .map(|texel| {
                (u32::from(texel[3]) << 24)
                    | (u32::from(texel[0]) << 16)
                    | (u32::from(texel[1]) << 8)
                    | u32::from(texel[2])
            })
            .collect(),
    )
}

/// `LinearCongruentialGenerator.next`.
fn lcg(value: i64, step: i64) -> i64 {
    value
        .wrapping_mul(
            value
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407),
        )
        .wrapping_add(step)
}

fn fiddle(value: i64) -> f64 {
    let uniform = (value >> 24).rem_euclid(1024) as f64 / 1024.0;
    (uniform - 0.5) * 0.9
}

impl BiomeTints {
    /// Read every biome's colours from a data pack folder and the colour maps
    /// from an asset folder. `zoom_seed` is the hashed seed the login gives.
    pub fn load(
        data_root: &Path,
        assets_root: &Path,
        biome_names: &[String],
        zoom_seed: i64,
    ) -> Option<Self> {
        let biomes = biome_names
            .iter()
            .map(|name| {
                let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
                let biome = read_json(
                    &data_root
                        .join(namespace)
                        .join("worldgen/biome")
                        .join(format!("{path}.json")),
                );
                let effects = biome.as_ref().and_then(|biome| biome.object("effects"));
                BiomeColours {
                    temperature: biome
                        .as_ref()
                        .and_then(|b| b.f32("temperature"))
                        .unwrap_or(0.8),
                    downfall: biome
                        .as_ref()
                        .and_then(|b| b.f32("downfall"))
                        .unwrap_or(0.4),
                    grass: effects.and_then(|e| hex_colour(e, "grass_color")),
                    foliage: effects.and_then(|e| hex_colour(e, "foliage_color")),
                    dry_foliage: effects.and_then(|e| hex_colour(e, "dry_foliage_color")),
                    water: effects
                        .and_then(|e| hex_colour(e, "water_color"))
                        .unwrap_or(0xFF3F_76E4),
                    modifier: match effects
                        .and_then(|e| e.text("grass_color_modifier"))
                        .as_deref()
                    {
                        Some("dark_forest") => GrassModifier::DarkForest,
                        Some("swamp") => GrassModifier::Swamp,
                        _ => GrassModifier::None,
                    },
                }
            })
            .collect();
        let plains = biome_names
            .iter()
            .position(|name| name == "minecraft:plains")
            .unwrap_or(0);
        let mut random = RandomSource::create(2345);
        Some(Self {
            biomes,
            plains,
            grass: colour_map(assets_root, "grass")?,
            foliage: colour_map(assets_root, "foliage")?,
            dry_foliage: colour_map(assets_root, "dry_foliage")?,
            zoom_seed,
            swamp_noise: SimplexNoise::new(&mut random),
        })
    }

    /// `BiomeManager.getBiome`: the zoomed biome at a block position.
    fn biome_at(
        &self,
        x: i32,
        y: i32,
        z: i32,
        noise_biome: &impl Fn(i32, i32, i32) -> Option<i32>,
    ) -> usize {
        let (abs_x, abs_y, abs_z) = (x - 2, y - 2, z - 2);
        let (parent_x, parent_y, parent_z) = (abs_x >> 2, abs_y >> 2, abs_z >> 2);
        let fract = |v: i32| f64::from(v & 3) / 4.0;
        let (fract_x, fract_y, fract_z) = (fract(abs_x), fract(abs_y), fract(abs_z));
        let mut best = 0;
        let mut best_distance = f64::INFINITY;
        for i in 0..8 {
            let (x_even, y_even, z_even) = (i & 4 == 0, i & 2 == 0, i & 1 == 0);
            let corner = [
                if x_even { parent_x } else { parent_x + 1 },
                if y_even { parent_y } else { parent_y + 1 },
                if z_even { parent_z } else { parent_z + 1 },
            ];
            let distance = [
                if x_even { fract_x } else { fract_x - 1.0 },
                if y_even { fract_y } else { fract_y - 1.0 },
                if z_even { fract_z } else { fract_z - 1.0 },
            ];
            let mut value = self.zoom_seed;
            for step in [
                corner[0], corner[1], corner[2], corner[0], corner[1], corner[2],
            ] {
                value = lcg(value, i64::from(step));
            }
            let fiddle_x = fiddle(value);
            value = lcg(value, self.zoom_seed);
            let fiddle_y = fiddle(value);
            value = lcg(value, self.zoom_seed);
            let fiddle_z = fiddle(value);
            let fiddled = (distance[2] + fiddle_z).powi(2)
                + (distance[1] + fiddle_y).powi(2)
                + (distance[0] + fiddle_x).powi(2);
            if best_distance > fiddled {
                best = i;
                best_distance = fiddled;
            }
        }
        let biome_x = if best & 4 == 0 {
            parent_x
        } else {
            parent_x + 1
        };
        let biome_y = if best & 2 == 0 {
            parent_y
        } else {
            parent_y + 1
        };
        let biome_z = if best & 1 == 0 {
            parent_z
        } else {
            parent_z + 1
        };
        noise_biome(biome_x, biome_y, biome_z)
            .and_then(|id| usize::try_from(id).ok())
            .filter(|id| *id < self.biomes.len())
            .unwrap_or(self.plains)
    }

    fn colour(&self, kind: TintKind, biome: usize, x: i32, z: i32) -> u32 {
        let biome = &self.biomes[biome];
        let temperature = f64::from(biome.temperature.clamp(0.0, 1.0));
        let downfall = f64::from(biome.downfall.clamp(0.0, 1.0));
        let from_map = |map: ColourMap, pixels: &[u32]| {
            colour_map_lookup(map, pixels, temperature, downfall).unwrap_or(map.default_colour())
        };
        match kind {
            TintKind::Grass => {
                let base = biome
                    .grass
                    .unwrap_or_else(|| from_map(ColourMap::Grass, &self.grass));
                match biome.modifier {
                    GrassModifier::None => base,
                    GrassModifier::DarkForest => {
                        0xFF00_0000 | (((base & 0xFE_FEFE) + 0x28_340A) >> 1)
                    }
                    GrassModifier::Swamp => {
                        let ground = self
                            .swamp_noise
                            .value(f64::from(x) * 0.0225, f64::from(z) * 0.0225);
                        if ground < -0.1 {
                            -11_766_212_i32 as u32
                        } else {
                            -9_801_671_i32 as u32
                        }
                    }
                }
            }
            TintKind::Foliage => biome
                .foliage
                .unwrap_or_else(|| from_map(ColourMap::Foliage, &self.foliage)),
            TintKind::DryFoliage => biome
                .dry_foliage
                .unwrap_or_else(|| from_map(ColourMap::DryFoliage, &self.dry_foliage)),
            TintKind::Water => biome.water,
        }
    }

    /// `ClientLevel.calculateBlockTint` with the default blend radius of two:
    /// the channel average of the 25 columns around the block, at its height.
    pub fn tint(
        &self,
        kind: TintKind,
        position: [i32; 3],
        noise_biome: &impl Fn(i32, i32, i32) -> Option<i32>,
    ) -> u32 {
        const RADIUS: i32 = 2;
        let count = (RADIUS * 2 + 1) * (RADIUS * 2 + 1);
        let (mut red, mut green, mut blue) = (0, 0, 0);
        for z in position[2] - RADIUS..=position[2] + RADIUS {
            for x in position[0] - RADIUS..=position[0] + RADIUS {
                let biome = self.biome_at(x, position[1], z, noise_biome);
                let colour = self.colour(kind, biome, x, z);
                red += ((colour >> 16) & 0xFF) as i32;
                green += ((colour >> 8) & 0xFF) as i32;
                blue += (colour & 0xFF) as i32;
            }
        }
        0xFF00_0000
            | (((red / count) as u32) << 16)
            | (((green / count) as u32) << 8)
            | (blue / count) as u32
    }
}

/// Which biome colour each tint layer of a block reads in a world, or
/// nothing where the layer is the same everywhere.
pub fn tint_layers(block: &str) -> Option<&'static [Option<TintKind>]> {
    use TintKind as T;
    Some(match block {
        "minecraft:grass_block"
        | "minecraft:fern"
        | "minecraft:short_grass"
        | "minecraft:potted_fern"
        | "minecraft:bush"
        | "minecraft:sugar_cane"
        | "minecraft:large_fern"
        | "minecraft:tall_grass" => &[Some(T::Grass)],
        "minecraft:pink_petals" | "minecraft:wildflowers" => &[None, Some(T::Grass)],
        "minecraft:oak_leaves"
        | "minecraft:jungle_leaves"
        | "minecraft:acacia_leaves"
        | "minecraft:dark_oak_leaves"
        | "minecraft:vine"
        | "minecraft:mangrove_leaves" => &[Some(T::Foliage)],
        "minecraft:leaf_litter" => &[Some(T::DryFoliage)],
        "minecraft:water_cauldron" => &[Some(T::Water)],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lcg_matches_its_reference_steps() {
        // One step from zero adds only the increment's product with zero.
        assert_eq!(lcg(0, 5), 5);
        assert_eq!(
            lcg(1, 0),
            6_364_136_223_846_793_005_i64.wrapping_add(1_442_695_040_888_963_407)
        );
    }

    #[test]
    fn dark_forest_darkens_towards_its_fixed_green() {
        let base = 0xFF79_C05A_u32;
        let modified = 0xFF00_0000 | (((base & 0xFE_FEFE) + 0x28_340A) >> 1);
        assert_eq!(modified, 0xFF50_7A32);
    }
}
