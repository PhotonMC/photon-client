use super::*;

/// The seeded star field, as quads around the origin, built once.
pub fn build_stars() -> Vec<[[f32; 3]; 4]> {
    let mut random = RandomSource::create(10842);
    let mut stars = Vec::new();
    for _ in 0..1500 {
        let x = random.next_float() * 2.0 - 1.0;
        let y = random.next_float() * 2.0 - 1.0;
        let z = random.next_float() * 2.0 - 1.0;
        let size = 0.15 + random.next_float() * 0.1;
        let length_squared = x * x + y * y + z * z;
        if length_squared <= 0.010_000_001 || length_squared >= 1.0 {
            continue;
        }
        let length = length_squared.sqrt();
        let centre = [x / length * 100.0, y / length * 100.0, z / length * 100.0];
        let z_rotation = (random.next_double() * f64::from(std::f32::consts::PI) * 2.0) as f32;
        // A basis whose +Z faces the origin, then turned about that axis.
        let forward = {
            let f = [-centre[0], -centre[1], -centre[2]];
            let l = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
            [f[0] / l, f[1] / l, f[2] / l]
        };
        let up = [0.0_f32, 1.0, 0.0];
        let left = {
            let c = [
                up[1] * forward[2] - up[2] * forward[1],
                up[2] * forward[0] - up[0] * forward[2],
                up[0] * forward[1] - up[1] * forward[0],
            ];
            let l = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
            [c[0] / l, c[1] / l, c[2] / l]
        };
        let upn = [
            forward[1] * left[2] - forward[2] * left[1],
            forward[2] * left[0] - forward[0] * left[2],
            forward[0] * left[1] - forward[1] * left[0],
        ];
        let (s, c) = ((-z_rotation).sin(), (-z_rotation).cos());
        let corner = |a: f32, b: f32| {
            let (u, v) = (a * c - b * s, a * s + b * c);
            [
                left[0] * u + upn[0] * v + centre[0],
                left[1] * u + upn[1] * v + centre[1],
                left[2] * u + upn[2] * v + centre[2],
            ]
        };
        stars.push([
            corner(size, -size),
            corner(size, size),
            corner(-size, size),
            corner(-size, -size),
        ]);
    }
    stars
}
