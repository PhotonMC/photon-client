use super::*;
use photon_core::ids::BlockStateId;

pub(super) fn light_traits(state: BlockStateId) -> &'static crate::block::StateInfo {
    crate::block::state_or_air(state)
}

pub(super) fn emission(state: BlockStateId) -> u8 {
    light_traits(state).light_emission
}

pub(super) fn dampening(state: BlockStateId) -> u8 {
    light_traits(state).light_dampening
}

/// `LightEngine.isEmptyShape`: light ignores the shape of a block that
/// cannot occlude or does not use its shape for light.
pub(super) fn is_empty_shape(state: BlockStateId) -> bool {
    let traits = light_traits(state);
    !traits.can_occlude || !traits.use_shape_for_light_occlusion
}

/// `LightEngine.getOcclusionShape`, as a face number.
pub(super) fn occlusion_shape(
    state: BlockStateId,
    direction: Direction,
) -> crate::block::state::Face {
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
pub fn has_different_light_properties(old: BlockStateId, new: BlockStateId) -> bool {
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
pub(super) fn is_edge_occluded(top: BlockStateId, bottom: BlockStateId) -> bool {
    dampening(bottom) != 0
        || face_shape_occludes(
            occlusion_shape(top, Direction::Down),
            occlusion_shape(bottom, Direction::Up),
        )
}

pub(super) fn bedrock() -> BlockStateId {
    static STATE: std::sync::OnceLock<BlockStateId> = std::sync::OnceLock::new();
    *STATE.get_or_init(|| {
        photon_data::block::registry::block("minecraft:bedrock")
            .map_or(BlockStateId(0), |block| block.default_state_id())
    })
}

pub(super) struct Level<'a> {
    pub(super) chunks: &'a dyn LightChunks,
}

impl Engine {
    pub(super) fn opacity(state: BlockStateId) -> u8 {
        dampening(state).max(1)
    }

    pub(super) fn shape_occludes(
        from: BlockStateId,
        to: BlockStateId,
        direction: Direction,
    ) -> bool {
        face_shape_occludes(
            occlusion_shape(from, direction),
            occlusion_shape(to, direction.opposite()),
        )
    }
}

impl Level<'_> {
    pub(super) fn state(&self, block: BlockKey) -> BlockStateId {
        if self.chunks.has_chunk(block.0 >> 4, block.2 >> 4) {
            self.chunks.block_state(block.0, block.1, block.2)
        } else {
            bedrock()
        }
    }
}
