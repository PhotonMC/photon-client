use super::*;
use photon_core::map_values::{MapItemDataInput, MapPatch};

/// `handleMapItemData`: a patch of colours lands at its place in the map's 128 by 128 grid.
#[test]
fn a_map_packet_patch_lands_in_the_maps_colours() {
    let mut world = ClientWorld::new(DimensionBounds::default());
    world.apply_map_item_data(&MapItemDataInput {
        map_id: 7,
        scale: 2,
        locked: true,
        decorations: None,
        patch: Some(MapPatch {
            start_x: 10,
            start_y: 20,
            width: 2,
            height: 2,
            colors: vec![1, 2, 3, 4],
        }),
    });
    let map = world.map_data(7).expect("the map");
    assert_eq!(map.colors()[10 + 20 * 128], 1);
    assert_eq!(map.colors()[11 + 21 * 128], 4);
}
