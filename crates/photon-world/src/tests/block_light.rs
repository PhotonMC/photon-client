use super::*;

/// `LightCoordsUtil.getLightCoords`: a block that renders emissively is lit fully, another by the
/// light around it or by its own emission where that is higher.
#[test]
fn a_block_is_lit_by_its_emission_or_fully_when_it_renders_emissively() {
    let id = |name: &str| {
        photon_data::block::registry::block(name)
            .expect(name)
            .default_state_id()
            .0
    };
    let mut blocks = vec![0; 4_096];
    blocks[0] = id("minecraft:glowstone");
    blocks[1] = id("minecraft:magma_block");
    blocks[2] = id("minecraft:stone");
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 3,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    // Glowstone gives off 15 of block light, magma renders emissively, stone gives nothing.
    assert_eq!(world.block_light_coords(0, 0, 0) >> 4 & 15, 15);
    assert_eq!(world.block_light_coords(1, 0, 0), 15_728_880);
    assert_eq!(world.block_light_coords(2, 0, 0) >> 4 & 15, 0);
}
