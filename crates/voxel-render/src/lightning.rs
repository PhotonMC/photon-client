//! A lightning bolt (`LightningBoltRenderer`): the jagged column of translucent quads the
//! entity's seed draws, three passes of it (the bolt and two branches) at four widths.

use voxel_core::random::RandomSource;

use crate::dragon_rays::{RayVertex, channel};
use crate::joml::Pose;

/// `ARGB.colorFromFloat(0.3, 0.45, 0.45, 0.5)`: the bolt's colour.
fn colour() -> u32 {
    (channel(0.3) << 24) | (channel(0.45) << 16) | (channel(0.45) << 8) | channel(0.5)
}

/// `Random.nextInt(bound) - offset`, as a float.
fn jitter(random: &mut RandomSource, bound: i32, offset: i32) -> f32 {
    (random.next_int_bound(bound).unwrap_or_default() - offset) as f32
}

/// `LightningBoltRenderer.quad`: one side of a segment, four vertices.
#[allow(clippy::too_many_arguments)]
fn quad(
    pose: &Pose,
    out: &mut Vec<RayVertex>,
    ((xo0, zo0), (xo1, zo1), h): ((f32, f32), (f32, f32), i32),
    (rr1, rr2): (f32, f32),
    (px1, pz1, px2, pz2): (bool, bool, bool, bool),
) {
    let signed = |positive: bool, value: f32| if positive { value } else { -value };
    let colour = colour();
    for (x, y, z) in [
        (
            xo0 + signed(px1, rr2),
            (h * 16) as f32,
            zo0 + signed(pz1, rr2),
        ),
        (
            xo1 + signed(px1, rr1),
            ((h + 1) * 16) as f32,
            zo1 + signed(pz1, rr1),
        ),
        (
            xo1 + signed(px2, rr1),
            ((h + 1) * 16) as f32,
            zo1 + signed(pz2, rr1),
        ),
        (
            xo0 + signed(px2, rr2),
            (h * 16) as f32,
            zo0 + signed(pz2, rr2),
        ),
    ] {
        out.push(RayVertex {
            position: pose.pose.transform_position(x, y, z),
            colour,
        });
    }
}

/// `LightningBoltRenderer.submit` from the pose the entity's place leaves, in the order the
/// reference's vertex consumer receives the vertices (four to a quad).
pub fn lightning_bolt(pose: &Pose, seed: i64) -> Vec<RayVertex> {
    let mut x_offs = [0.0_f32; 8];
    let mut z_offs = [0.0_f32; 8];
    let (mut x_off, mut z_off) = (0.0_f32, 0.0_f32);
    let mut random = RandomSource::create(seed);
    for h in (0..8).rev() {
        x_offs[h] = x_off;
        z_offs[h] = z_off;
        x_off += jitter(&mut random, 11, 5);
        z_off += jitter(&mut random, 11, 5);
    }
    let mut out = Vec::new();
    for r in 0..4 {
        let mut random = RandomSource::create(seed);
        for p in 0..3_i32 {
            let (hs, ht) = if p > 0 { (7 - p, 7 - p - 2) } else { (7, 0) };
            let mut xo0 = x_offs[hs as usize] - x_off;
            let mut zo0 = z_offs[hs as usize] - z_off;
            for h in (ht..=hs).rev() {
                let (xo1, zo1) = (xo0, zo0);
                if p == 0 {
                    xo0 += jitter(&mut random, 11, 5);
                    zo0 += jitter(&mut random, 11, 5);
                } else {
                    xo0 += jitter(&mut random, 31, 15);
                    zo0 += jitter(&mut random, 31, 15);
                }
                let mut rr1 = 0.1 + r as f32 * 0.2;
                if p == 0 {
                    rr1 *= h as f32 * 0.1 + 1.0;
                }
                let mut rr2 = 0.1 + r as f32 * 0.2;
                if p == 0 {
                    rr2 *= (h as f32 - 1.0) * 0.1 + 1.0;
                }
                let segment = ((xo0, zo0), (xo1, zo1), h);
                for sides in [
                    (false, false, true, false),
                    (true, false, true, true),
                    (true, true, false, true),
                    (false, true, false, false),
                ] {
                    quad(pose, &mut out, segment, (rr1, rr2), sides);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bolt_is_four_widths_of_a_trunk_and_two_branches_in_quads() {
        let vertices = lightning_bolt(&Pose::default(), 12_345);
        // Per width: eight trunk segments, two and then two more branch ones, four quads each.
        assert_eq!(vertices.len(), 4 * (8 + 3 + 3) * 4 * 4);
        assert!(vertices.iter().all(|vertex| vertex.colour >> 24 == 76));
    }

    #[test]
    fn the_same_seed_draws_the_same_bolt() {
        let a = lightning_bolt(&Pose::default(), 99);
        assert_eq!(a, lightning_bolt(&Pose::default(), 99));
        assert_ne!(a, lightning_bolt(&Pose::default(), 100));
    }
}
