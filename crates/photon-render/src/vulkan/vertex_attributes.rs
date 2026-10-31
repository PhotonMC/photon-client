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

/// The attributes of the `section` pipelines' vertices ([`crate::packed_vertex::PackedVertex`]):
/// the position and the light share the first four 16-bit words.
pub(super) fn section_attributes() -> [vk::VertexInputAttributeDescription; 3] {
    [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R16G16B16A16_UINT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(8),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R8G8B8A8_UNORM)
            .offset(16),
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
        SkyTranslucent | SkyOverlay | Weather | Entity | EntityInstanced | EntityMask
        | Dissolve | Swirl | Glint | Rays | WorldBorder | Text | TextOffset | TextTranslucent
        | Shadow | TextSeeThrough | Gui | GuiInvert | GuiHighlight | GuiVignette | GuiLogo
        | GuiItem | GuiGlint | Lines => vk::CullModeFlags::NONE,
        _ => vk::CullModeFlags::BACK,
    }
}

/// The binding of the per-instance data of [`PipelineKind::EntityInstanced`].
pub(super) fn instance_binding() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription::default()
        .binding(1)
        .stride(std::mem::size_of::<crate::entity::instancing::Instance>() as u32)
        .input_rate(vk::VertexInputRate::INSTANCE)
}

/// The attributes of the instanced entity pipeline: the mesh's vertex (as the entity pipeline
/// reads it) at binding 0, then the instance at binding 1: the position matrix's four columns,
/// the normal matrix's three, the shade and colour, and the light.
pub(super) fn instanced_attributes() -> [vk::VertexInputAttributeDescription; 14] {
    let vertex = terrain_attributes();
    let instance = |location: u32, format: vk::Format, offset: u32| {
        vk::VertexInputAttributeDescription::default()
            .location(location)
            .binding(1)
            .format(format)
            .offset(offset)
    };
    let vec3 = vk::Format::R32G32B32_SFLOAT;
    [
        vertex[1],
        vertex[2],
        vertex[3],
        vertex[4],
        vertex[0],
        instance(5, vec3, 0),
        instance(6, vec3, 12),
        instance(7, vec3, 24),
        instance(8, vec3, 36),
        instance(9, vec3, 48),
        instance(10, vec3, 60),
        instance(11, vec3, 72),
        instance(12, vk::Format::R32G32_SFLOAT, 84),
        instance(13, vk::Format::R32G32_SFLOAT, 92),
    ]
}

/// The vertex buffers' bindings and the attributes the pipelines of a kind read.
pub(super) fn vertex_layout(
    kind: PipelineKind,
) -> (
    Vec<vk::VertexInputBindingDescription>,
    Vec<vk::VertexInputAttributeDescription>,
) {
    use PipelineKind::*;
    let (stride, attributes) = match kind {
        EntityInstanced => (
            std::mem::size_of::<Vertex>(),
            instanced_attributes().to_vec(),
        ),
        SkyTranslucent | SkyOverlay => (
            std::mem::size_of::<crate::sky::SkyVertex>(),
            sky_attributes().to_vec(),
        ),
        Clouds => (
            std::mem::size_of::<crate::sky::clouds::CloudVertex>(),
            cloud_attributes().to_vec(),
        ),
        Gui | GuiInvert | GuiHighlight | GuiVignette | GuiLogo => (
            std::mem::size_of::<crate::gui::GuiVertex>(),
            gui_attributes().to_vec(),
        ),
        GuiItem | GuiGlint => (
            std::mem::size_of::<crate::gui::items::GuiItemVertex>(),
            gui_item_attributes().to_vec(),
        ),
        Lines | LinesBacking => (
            std::mem::size_of::<crate::block::outline::LineVertex>(),
            line_attributes().to_vec(),
        ),
        Terrain | Translucent => (
            std::mem::size_of::<crate::packed_vertex::PackedVertex>(),
            section_attributes().to_vec(),
        ),
        _ => (std::mem::size_of::<Vertex>(), terrain_attributes().to_vec()),
    };
    let mut bindings = vec![
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(stride as u32)
            .input_rate(vk::VertexInputRate::VERTEX),
    ];
    if kind == EntityInstanced {
        bindings.push(instance_binding());
    }
    (bindings, attributes)
}
