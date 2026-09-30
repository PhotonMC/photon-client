//! Startup and meshing timings without a window or a server: the block
//! models baked from the assets, then sections of a made-up world meshed
//! the way the section workers mesh them.
//!
//! `VOXEL_ASSETS=.data/minecraft/assets/jar/assets cargo run --release --example bench`

#[path = "../src/assets.rs"]
mod assets;

use std::time::Instant;

use voxel_render::section_workers::SectionWorkers;
use voxel_render::world_mesh::BlockRenderer;
use voxel_world::{ClientWorld, DimensionBounds, SectionInput};

/// How many times each section is meshed for its average.
const REPEATS: u32 = 50;

fn state(name: &str) -> i32 {
    let block = voxel_data::block::registry::block(name).expect("a vanilla block");
    i32::try_from(block.default_state_id()).expect("state numbers fit an i32")
}

/// A 5 by 5 chunk overworld: stone with holes and ore under a dirt and
/// grass surface around y 60, water below sea level and scattered leaves
/// above, from a fixed xorshift so every run meshes the same blocks.
fn world() -> ClientWorld {
    let [stone, ore, dirt, grass, water, leaves] = [
        "minecraft:stone",
        "minecraft:iron_ore",
        "minecraft:dirt",
        "minecraft:grass_block",
        "minecraft:water",
        "minecraft:oak_leaves",
    ]
    .map(state);
    let mut seed = 12_345_u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
    let biomes = vec![0; 64];
    for chunk_x in -2..=2 {
        for chunk_z in -2..=2 {
            let sections: Vec<(Vec<i32>, i16)> = (0..24)
                .map(|section| {
                    let mut blocks = vec![0; 4096];
                    let mut filled = 0;
                    for (index, block) in blocks.iter_mut().enumerate() {
                        let y = -64 + section * 16 + (index / 256) as i32;
                        let surface = 60 + (next() % 8) as i32;
                        *block = if y < surface - 4 {
                            match next() % 60 {
                                0..=2 => 0,
                                3 | 4 => ore,
                                _ => stone,
                            }
                        } else if y < surface {
                            dirt
                        } else if y == surface {
                            grass
                        } else if y < 62 {
                            water
                        } else if y < 70 && next() % 9 == 0 {
                            leaves
                        } else {
                            0
                        };
                        filled += i16::from(*block != 0);
                    }
                    (blocks, filled)
                })
                .collect();
            world.replace_chunk(
                chunk_x,
                chunk_z,
                sections.iter().map(|(blocks, filled)| SectionInput {
                    non_empty_block_count: *filled,
                    fluid_count: 0,
                    block_states: blocks,
                    biomes: &biomes,
                }),
            );
        }
    }
    world
}

fn main() {
    let root = std::env::var("VOXEL_ASSETS").expect("VOXEL_ASSETS names the assets folder");
    let source = assets::FolderAssets { root: root.into() };
    let baking = Instant::now();
    let renderer = BlockRenderer::new(&source, 4).expect("the vanilla models bake");
    println!(
        "bake atlas={}x{} models={} in {:?}",
        renderer.atlas.width,
        renderer.atlas.height,
        renderer.model_count(),
        baking.elapsed()
    );

    let world = world();
    let tint = |_: voxel_render::biome_tint::TintKind, _: [i32; 3]| 0xff79_c05a_u32;
    for section_y in [-3, 0, 3, 4] {
        let key = (0, section_y, 0);
        let (min, size, quart_min, quart_size) = SectionWorkers::region(key);
        let copying = Instant::now();
        let copy = world.copy_region(min, size, quart_min, quart_size);
        let copied = copying.elapsed();
        let meshing = Instant::now();
        let mut quads = 0;
        for _ in 0..REPEATS {
            let region = renderer.mesh_region(
                &|x, y, z| copy.block_state_at(x, y, z),
                &|x, y, z| copy.light_at(x, y, z),
                &tint,
                [key.0 * 16, key.1 * 16, key.2 * 16],
                [16, 16, 16],
                [0, 0, 0],
            );
            quads = region.mesh.indices.len() / 6;
        }
        println!(
            "section y={section_y} quads={quads} copy={copied:?} mesh={:?}",
            meshing.elapsed() / REPEATS
        );
    }
}
