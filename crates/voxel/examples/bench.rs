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

/// Blocks whose models turn, nudge, span two blocks or take colours, which
/// the made-up world scatters in a section above its surface.
fn variety() -> Vec<i32> {
    [
        ("minecraft:oak_log", &[("axis", "x")][..]),
        ("minecraft:oak_log", &[("axis", "z")]),
        (
            "minecraft:oak_stairs",
            &[("facing", "east"), ("half", "top")],
        ),
        (
            "minecraft:oak_stairs",
            &[("facing", "south"), ("shape", "outer_left")],
        ),
        ("minecraft:tall_grass", &[("half", "lower")]),
        ("minecraft:tall_grass", &[("half", "upper")]),
        ("minecraft:short_grass", &[]),
        (
            "minecraft:oak_door",
            &[("half", "upper"), ("facing", "west")],
        ),
        (
            "minecraft:red_bed",
            &[("part", "foot"), ("facing", "north")],
        ),
        ("minecraft:redstone_wire", &[("power", "15")]),
        ("minecraft:lily_pad", &[]),
        ("minecraft:oak_leaves", &[]),
        ("minecraft:glass", &[]),
        (
            "minecraft:oak_fence",
            &[("north", "true"), ("east", "true")],
        ),
        ("minecraft:pointed_dripstone", &[]),
        ("minecraft:grass_block", &[]),
    ]
    .iter()
    .map(|(name, values)| {
        let block = voxel_data::block::registry::block(name).expect("a vanilla block");
        let chosen = values
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        i32::try_from(block.state_id(&chosen).expect("valid properties")).expect("fits")
    })
    .collect()
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
    let variety = variety();
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
                        } else if (80..96).contains(&y) && next() % 3 == 0 {
                            variety[(next() % variety.len() as u64) as usize]
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
    let caching = Instant::now();
    let faces = voxel_world::block::states().faces().len();
    println!("state cache faces={faces} in {:?}", caching.elapsed());
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
    let (stored, plain) = world.block_state_bytes();
    println!("block states stored={stored} bytes plain={plain} bytes");
    let tint = |_: voxel_render::biome_tint::TintKind, _: [i32; 3]| 0xff79_c05a_u32;
    for section_y in [-3, 0, 3, 4, 5] {
        let key = (0, section_y, 0);
        let (min, size, quart_min, quart_size) = SectionWorkers::region(key);
        let copying = Instant::now();
        let copy = world.copy_region(min, size, quart_min, quart_size);
        let copied = copying.elapsed();
        let meshing = Instant::now();
        let mut quads = 0;
        let mut digest = 0;
        for _ in 0..REPEATS {
            let region = renderer.mesh_region(
                &voxel_render::world_mesh::MeshWorld {
                    read: &|x, y, z| copy.block_state_at(x, y, z),
                    light: &|x, y, z| copy.light_at(x, y, z),
                    biome_tint: &tint,
                    tags: copy.tags(),
                },
                [key.0 * 16, key.1 * 16, key.2 * 16],
                [16, 16, 16],
                [0, 0, 0],
            );
            quads = region.mesh.indices.len() / 6;
            digest = mesh_digest(&region.mesh);
        }
        println!(
            "section y={section_y} quads={quads} digest={digest:016x} copy={copied:?} mesh={:?}",
            meshing.elapsed() / REPEATS
        );
    }
}

/// A hash of every vertex's bits and every index, so two builds can be
/// checked for the same mesh.
fn mesh_digest(mesh: &voxel_render::Mesh) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for vertex in &mesh.vertices {
        for value in vertex
            .position
            .iter()
            .chain(&vertex.uv)
            .chain(std::iter::once(&vertex.shade))
            .chain(&vertex.tint)
            .chain(&vertex.light)
        {
            value.to_bits().hash(&mut hasher);
        }
    }
    mesh.indices.hash(&mut hasher);
    hasher.finish()
}
