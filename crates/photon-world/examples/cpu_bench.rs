//! The CPU parts of a frame that need no GPU and no assets: applying chunks and lighting them.
//! Prints one JSON report on stdout in the shape `cargo xtask bench --save` keeps; the rows of the
//! world are made here, so every machine builds the same terrain. Meshing is not part of it.

use std::time::Instant;

use photon_world::{ClientWorld, DimensionBounds, SectionInput};

/// The chunks on each side of the middle one: render distance 8 is a 17 by 17 square.
const RADIUS: i32 = 8;
const SECTIONS: i32 = 24;
const RUNS: usize = 7;

/// Stone up to the sixth section, then a bumpy layer with a shaft through it, so the light
/// has an open sky, a roof and walls to work against.
fn terrain(stone: i32, chunk: (i32, i32), section: i32) -> Vec<i32> {
    let mut blocks = vec![0; 4_096];
    for y in 0..16_i32 {
        for z in 0..16_i32 {
            for x in 0..16_i32 {
                let (wx, wz) = (chunk.0 * 16 + x, chunk.1 * 16 + z);
                let height = 6 * 16 + 4 + (wx * 3 + wz * 5).rem_euclid(7);
                let shaft = (wx.rem_euclid(24) < 3) && (wz.rem_euclid(24) < 3);
                let solid = section * 16 + y < height && !(shaft && section >= 3);
                if solid {
                    blocks[(y * 256 + z * 16 + x) as usize] = stone;
                }
            }
        }
    }
    blocks
}

fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * fraction).round() as usize]
}

/// One run: every chunk applied, then every chunk lit; the milliseconds each took.
fn one_run(stone: i32, biomes: &[i32]) -> (f64, f64) {
    let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
    let started = Instant::now();
    for chunk_x in -RADIUS..=RADIUS {
        for chunk_z in -RADIUS..=RADIUS {
            let columns: Vec<Vec<i32>> = (0..SECTIONS)
                .map(|section| terrain(stone, (chunk_x, chunk_z), section))
                .collect();
            let sections = columns.iter().map(|blocks| SectionInput {
                non_empty_block_count: blocks.iter().filter(|&&state| state != 0).count() as i16,
                fluid_count: 0,
                block_states: blocks,
                biomes,
            });
            assert!(world.replace_chunk(chunk_x, chunk_z, sections));
        }
    }
    let applied = started.elapsed().as_secs_f64() * 1_000.0;
    let lit = Instant::now();
    world.light_from_sources();
    world.flush_light();
    (applied, lit.elapsed().as_secs_f64() * 1_000.0)
}

fn main() {
    let stone = photon_data::block::registry::block("minecraft:stone")
        .expect("the block registry names stone")
        .default_state_id()
        .0;
    let biomes = vec![0; 64];
    let (mut totals, mut apply_max, mut light_max) = (Vec::new(), 0.0_f64, 0.0_f64);
    for _ in 0..RUNS {
        let (applied, light) = one_run(stone, &biomes);
        totals.push(applied + light);
        apply_max = apply_max.max(applied);
        light_max = light_max.max(light);
    }
    totals.sort_by(f64::total_cmp);
    println!(
        "{{\"kind\":\"cpu\",\"chunks\":{},\"work_ms\":{{\"p50\":{:.3},\"p95\":{:.3},\"p99\":{:.3},\"max\":{:.3}}},\
         \"parts_ms\":{{\"apply_chunks\":{{\"max\":{apply_max:.3}}},\"light\":{{\"max\":{light_max:.3}}}}}}}",
        (2 * RADIUS + 1) * (2 * RADIUS + 1),
        percentile(&totals, 0.5),
        percentile(&totals, 0.95),
        percentile(&totals, 0.99),
        percentile(&totals, 1.0),
    );
}
