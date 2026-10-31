//! Pipeline states a few kinds set apart from the rest.

use ash::vk;

use super::buffer::PipelineKind;

/// What `uses_entity_shaders` says of a kind: the entity shaders draw it.
pub(super) fn uses_entity_shaders(kind: PipelineKind) -> bool {
    matches!(
        kind,
        PipelineKind::Entity
            | PipelineKind::EntityInstanced
            | PipelineKind::EntityOutline
            | PipelineKind::EntityMask
    )
}

/// The blend of the break cracks and of the water patch, which the others do not share.
pub(super) fn crumbling_or_mask_blend(
    kind: PipelineKind,
    crumbling: bool,
) -> Option<vk::PipelineColorBlendAttachmentState> {
    let blend = vk::PipelineColorBlendAttachmentState::default();
    if kind == PipelineKind::EntityMask {
        // `WATER_MASK` writes no colour: only the depth, which keeps the water out.
        return Some(blend.color_write_mask(vk::ColorComponentFlags::empty()));
    }
    // Source times destination, twice: darkens and lightens.
    crumbling.then(|| {
        blend
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::DST_COLOR)
            .dst_color_blend_factor(vk::BlendFactor::SRC_COLOR)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
    })
}
