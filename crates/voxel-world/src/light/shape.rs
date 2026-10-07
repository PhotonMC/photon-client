use super::*;

pub(super) fn column(key: SectionKey) -> ColumnKey {
    (key.0, key.2)
}

pub(super) fn section_of(block: BlockKey) -> SectionKey {
    (block.0 >> 4, block.1 >> 4, block.2 >> 4)
}

pub(super) fn local(block: BlockKey) -> (i32, i32, i32) {
    (block.0 & 15, block.1 & 15, block.2 & 15)
}

pub(super) fn repeat_first_layer(layer: &DataLayer) -> DataLayer {
    if layer.is_definitely_homogenous() {
        return layer.clone();
    }
    let input = layer.data.as_ref().expect("not homogenous");
    let mut output = vec![0_u8; 2048];
    for i in 0..16 {
        output[i * 128..(i + 1) * 128].copy_from_slice(&input[..128]);
    }
    DataLayer::from_bytes(&output)
}

pub(super) fn light_traits(state: i32) -> &'static crate::block::StateInfo {
    crate::block::state_or_air(i64::from(state))
}

pub(super) fn emission(state: i32) -> u8 {
    light_traits(state).light_emission
}

pub(super) fn dampening(state: i32) -> u8 {
    light_traits(state).light_dampening
}

/// `LightEngine.isEmptyShape`: light ignores the shape of a block that
/// cannot occlude or does not use its shape for light.
pub(super) fn is_empty_shape(state: i32) -> bool {
    let traits = light_traits(state);
    !traits.can_occlude || !traits.use_shape_for_light_occlusion
}

/// `LightEngine.getOcclusionShape`, as a face number.
pub(super) fn occlusion_shape(state: i32, direction: Direction) -> crate::block::state::Face {
    if is_empty_shape(state) {
        crate::block::state::EMPTY_FACE
    } else {
        light_traits(state).face(direction)
    }
}

/// `Shapes.faceShapeOccludes`.
pub(super) fn face_shape_occludes(
    shape: crate::block::state::Face,
    occluder: crate::block::state::Face,
) -> bool {
    crate::block::states().covers(shape, occluder)
}

/// `LightEngine.hasDifferentLightProperties`.
pub fn has_different_light_properties(old: i32, new: i32) -> bool {
    if old == new {
        return false;
    }
    let (a, b) = (light_traits(old), light_traits(new));
    b.light_dampening != a.light_dampening
        || b.light_emission != a.light_emission
        || b.use_shape_for_light_occlusion
        || a.use_shape_for_light_occlusion
}

/// `ChunkSkyLightSources.isEdgeOccluded`.
pub(super) fn is_edge_occluded(top: i32, bottom: i32) -> bool {
    dampening(bottom) != 0
        || face_shape_occludes(
            occlusion_shape(top, Direction::Down),
            occlusion_shape(bottom, Direction::Up),
        )
}

pub(super) fn bedrock() -> i32 {
    static STATE: std::sync::OnceLock<i32> = std::sync::OnceLock::new();
    *STATE.get_or_init(|| {
        voxel_data::block::registry::block("minecraft:bedrock")
            .map_or(0, |block| block.default_state_id() as i32)
    })
}

pub(super) fn offset(block: BlockKey, direction: Direction) -> BlockKey {
    let (dx, dy, dz) = direction.step();
    (block.0 + dx, block.1 + dy, block.2 + dz)
}
