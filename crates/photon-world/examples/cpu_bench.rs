//! The CPU parts of a frame that need no GPU and no assets: applying chunks, lighting them, the
//! light a server sends and the light work of changed blocks.
//! Prints one JSON report on stdout in the shape `cargo xtask bench --save` keeps; the rows of the
//! world are made here, so every machine builds the same terrain. Meshing is not part of it.

use std::time::Instant;

use photon_core::binary::BitSet;
use photon_core::ids::BlockStateId;
use photon_world::{
    ClientWorld, DimensionBounds, LightLayerInput, LightUpdateInput, SectionInput,
    VANILLA_AIR_BLOCK_STATE_ID,
};

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

/// A world with every chunk applied.
fn loaded_world(stone: i32, biomes: &[i32]) -> ClientWorld {
    let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
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
    world
}

/// The light a server sends with each chunk, applied as the client does: queued with the chunk,
/// then run by the frames. The sky light is full above the terrain and empty below it, and
/// there is no block light, which is what an unlit surface world sends.
fn server_light(world: &mut ClientWorld) {
    // Light sections are the world's sections with one more below and one above.
    let first_full = 8_usize;
    let all = (1_u64 << (SECTIONS + 2)) - 1;
    let full_mask = all & !((1_u64 << first_full) - 1);
    let layers: Vec<Vec<u8>> = (first_full..(SECTIONS as usize + 2))
        .map(|_| vec![0xff; 2_048])
        .collect();
    for chunk_x in -RADIUS..=RADIUS {
        for chunk_z in -RADIUS..=RADIUS {
            world.apply_chunk_light(
                chunk_x,
                chunk_z,
                LightUpdateInput {
                    sky: LightLayerInput {
                        mask: &BitSet::from_longs(&[full_mask as i64]),
                        empty_mask: &BitSet::from_longs(&[(all & !full_mask) as i64]),
                        updates: &layers,
                    },
                    block: LightLayerInput {
                        mask: &BitSet::from_longs(&[0]),
                        empty_mask: &BitSet::from_longs(&[all as i64]),
                        updates: &[],
                    },
                },
            );
        }
    }
    world.flush_light();
}

/// Blocks dug out of the surface and put back, a few at a time with a frame's light work after
/// each batch: what the light engine does when the server changes blocks.
fn block_changes(world: &mut ClientWorld, stone: i32) {
    let stone = BlockStateId(stone);
    for batch in 0..20_i32 {
        for i in 0..15_i32 {
            let n = batch * 15 + i;
            let (chunk_x, chunk_z) = (n % 17 - RADIUS, n / 17 % 17 - RADIUS);
            let (x, z) = (chunk_x * 16 + n * 5 % 16, chunk_z * 16 + n * 7 % 16);
            // Down from the top of the stone, where the light reaches in from the open sky.
            let top = 99 + (x * 3 + z * 5).rem_euclid(7);
            world.update_block_state(x, top, z, VANILLA_AIR_BLOCK_STATE_ID);
            world.update_block_state(x, top - 1, z, VANILLA_AIR_BLOCK_STATE_ID);
            world.update_block_state(x, top - 1, z, stone);
        }
        world.flush_light();
    }
}

/// One run; the milliseconds that applying the chunks, lighting them from their own sources and
/// applying a server's light took.
fn one_run(stone: i32, biomes: &[i32]) -> [f64; 4] {
    let started = Instant::now();
    let mut world = loaded_world(stone, biomes);
    let applied = started.elapsed().as_secs_f64() * 1_000.0;
    let lit = Instant::now();
    world.light_from_sources();
    world.flush_light();
    let light = lit.elapsed().as_secs_f64() * 1_000.0;
    let mut served = loaded_world(stone, biomes);
    let sent = Instant::now();
    server_light(&mut served);
    let sent = sent.elapsed().as_secs_f64() * 1_000.0;
    assert_eq!(
        served.sky_light_at(0, 300, 0),
        Some(15),
        "the server's light was applied"
    );
    let changed = Instant::now();
    block_changes(&mut served, stone);
    [
        applied,
        light,
        sent,
        changed.elapsed().as_secs_f64() * 1_000.0,
    ]
}

fn main() {
    let stone = photon_data::block::registry::block("minecraft:stone")
        .expect("the block registry names stone")
        .default_state_id()
        .0;
    let biomes = vec![0; 64];
    let (mut totals, mut parts) = (Vec::new(), [0.0_f64; 4]);
    for _ in 0..RUNS {
        let run = one_run(stone, &biomes);
        totals.push(run.iter().sum::<f64>());
        for (part, took) in parts.iter_mut().zip(run) {
            *part = part.max(took);
        }
    }
    totals.sort_by(f64::total_cmp);
    println!(
        "{{\"kind\":\"cpu\",\"chunks\":{},\"work_ms\":{{\"p50\":{:.3},\"p95\":{:.3},\"p99\":{:.3},\"max\":{:.3}}},\
         \"parts_ms\":{{\"apply_chunks\":{{\"max\":{:.3}}},\"light\":{{\"max\":{:.3}}},\"server_light\":{{\"max\":{:.3}}},\"block_changes\":{{\"max\":{:.3}}}}}}}",
        (2 * RADIUS + 1) * (2 * RADIUS + 1),
        percentile(&totals, 0.5),
        percentile(&totals, 0.95),
        percentile(&totals, 0.99),
        percentile(&totals, 1.0),
        parts[0],
        parts[1],
        parts[2],
        parts[3],
    );
}
