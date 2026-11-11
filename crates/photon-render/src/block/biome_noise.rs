//! The noise the biome tints and the cold patches of a frozen biome read: the two-dimensional
//! simplex noise and the stack of three of them a frozen biome lays over itself.
//!
//! The two versions draw the same permutation tables from the same seeds, and differ in what the
//! noise hands back. 26.2 gives a double, and sums its octaves in doubles. 26.3 gives a float
//! (the double rounded once) and sums the octaves of the stack in floats, with the weights as
//! floats too. A threshold that a value sits just beside can fall on either side in the two.

use photon_core::random::RandomSource;

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

/// Whether the active version's noise returns floats (`biome_noise_is_float`).
fn floats() -> bool {
    photon_core::versions::rules().render.biome_noise_is_float
}

/// `SimplexNoise`, the two-dimensional half, without a noise start.
#[derive(Clone, Debug)]
pub(super) struct SimplexNoise {
    permutation: [i32; 512],
}

impl SimplexNoise {
    pub(super) fn new(random: &mut RandomSource) -> Self {
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

    /// The noise before the version rounds it.
    fn exact(&self, x_in: f64, y_in: f64) -> f64 {
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

    /// What the version's noise hands back at the point (a float widened in 26.3).
    pub(super) fn value(&self, x: f64, y: f64) -> f64 {
        let exact = self.exact(x, y);
        if floats() {
            f64::from(exact as f32)
        } else {
            exact
        }
    }
}

/// The octaves of the frozen temperature noise (`FROZEN_TEMPERATURE_NOISE`), highest frequency
/// first: weights of a seventh, two sevenths and four sevenths at one, a half and a quarter of
/// the frequency.
#[derive(Clone, Debug)]
pub(super) struct FrozenNoise([SimplexNoise; 3]);

impl FrozenNoise {
    /// The octaves drawn from the seed in turn, the highest frequency first.
    pub(super) fn new(random: &mut RandomSource) -> Self {
        Self([
            SimplexNoise::new(random),
            SimplexNoise::new(random),
            SimplexNoise::new(random),
        ])
    }

    /// The stack's value at the point, scaled by 7 as the frozen modifier does.
    pub(super) fn scaled(&self, x: f64, z: f64) -> f64 {
        if floats() {
            // `NoiseStack.get`: each layer's weight (a float) times its float noise, added up in
            // floats, then the float scale.
            let layers = [
                (1.0, 0.142_857_15_f32),
                (0.5, 0.285_714_3),
                (0.25, 0.571_428_6),
            ];
            let mut value = 0.0_f32;
            for (noise, (frequency, weight)) in self.0.iter().zip(layers) {
                value += weight * (noise.exact(x * frequency, z * frequency) as f32);
            }
            f64::from(value * 7.0_f32)
        } else {
            // `PerlinSimplexNoise.getValue` over the octaves -2 to 0.
            let mut value = 0.0;
            let mut factor = 1.0;
            let mut value_factor = 1.0 / 7.0;
            for level in &self.0 {
                value += level.exact(x * factor, z * factor) * value_factor;
                factor /= 2.0;
                value_factor *= 2.0;
            }
            value * 7.0
        }
    }
}
