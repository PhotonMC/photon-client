//! The vertex layouts of the pipelines: where each attribute of a vertex is in the buffer.

use super::*;

/// The attributes of the `sky` pipelines' vertices.
pub(super) fn sky_attributes() -> [vk::VertexInputAttributeDescription; 4] {
    [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(12),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(20),
        vk::VertexInputAttributeDescription::default()
            .location(3)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(36),
    ]
}

/// The attributes of the `terrain` pipelines' vertices.
pub(super) fn terrain_attributes() -> [vk::VertexInputAttributeDescription; 5] {
    [
        vk::VertexInputAttributeDescription::default()
            .location(4)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(40),
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(12),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R32_SFLOAT)
            .offset(20),
        vk::VertexInputAttributeDescription::default()
            .location(3)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(24),
    ]
}

/// The attributes of the `section` pipelines' vertices.
pub(super) fn section_attributes() -> [vk::VertexInputAttributeDescription; 4] {
    [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(12),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R8G8B8A8_UNORM)
            .offset(20),
        vk::VertexInputAttributeDescription::default()
            .location(3)
            .binding(0)
            .format(vk::Format::R8G8_UINT)
            .offset(24),
    ]
}

/// The attributes of the `cloud` pipelines' vertices.
pub(super) fn cloud_attributes() -> [vk::VertexInputAttributeDescription; 2] {
    [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32_SFLOAT)
            .offset(12),
    ]
}

/// The attributes of the `gui` pipelines' vertices.
pub(super) fn gui_attributes() -> [vk::VertexInputAttributeDescription; 3] {
    [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(8),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(16),
    ]
}

/// The attributes of the `line` pipelines' vertices.
pub(super) fn line_attributes() -> [vk::VertexInputAttributeDescription; 4] {
    [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(12),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(28),
        vk::VertexInputAttributeDescription::default()
            .location(3)
            .binding(0)
            .format(vk::Format::R32_SFLOAT)
            .offset(40),
    ]
}

/// The attributes of the `gui_item` pipelines' vertices.
pub(super) fn gui_item_attributes() -> [vk::VertexInputAttributeDescription; 3] {
    [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(12),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(20),
    ]
}

/// Which faces a pipeline leaves out: the back of each, except for the flat and doubled shapes (the
/// sky's fans, the entities, the GUI) that are built in either winding.
pub(super) fn cull_mode(kind: PipelineKind) -> vk::CullModeFlags {
    use PipelineKind::*;
    match kind {
        SkyTranslucent | SkyOverlay | Weather | Entity | Dissolve | Swirl | Glint | Rays
        | WorldBorder | Text | TextTranslucent | Shadow | TextSeeThrough | Gui | GuiInvert
        | GuiHighlight | GuiVignette | GuiItem | GuiGlint | Lines => vk::CullModeFlags::NONE,
        _ => vk::CullModeFlags::BACK,
    }
}
