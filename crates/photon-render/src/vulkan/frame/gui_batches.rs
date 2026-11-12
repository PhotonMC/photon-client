//! What a GUI batch is drawn with: its pipeline and the descriptor set it samples.

use super::*;

/// The pipeline of a GUI batch that is not translucent items: the plain one, or the one with the
/// blend its run asks for.
pub(super) fn plain_gui_pipeline(blend: crate::gui::GuiBlend, scene: &SceneResources) -> &Pipeline {
    match blend {
        crate::gui::GuiBlend::Invert => &scene.gui_pipelines.invert,
        crate::gui::GuiBlend::Highlight => &scene.gui_pipelines.highlight,
        crate::gui::GuiBlend::Vignette | crate::gui::GuiBlend::VignetteBlur => {
            &scene.gui_pipelines.vignette
        }
        crate::gui::GuiBlend::LogoAdd => &scene.gui_pipelines.logo,
        _ => &scene.gui_pipeline,
    }
}

/// The descriptor set a translucent GUI batch samples: the GUI atlas, or for a sprite of the block
/// atlas that atlas (the pipeline's layout is the same for both).
pub(super) fn translucent_set(
    blend: crate::gui::GuiBlend,
    gui: vk::DescriptorSet,
    scene: &SceneResources,
) -> vk::DescriptorSet {
    match blend {
        crate::gui::GuiBlend::BlockSprite => scene.descriptor_set,
        crate::gui::GuiBlend::Surface => scene.surface_set.unwrap_or(gui),
        crate::gui::GuiBlend::Map => scene.map_set.unwrap_or(gui),
        crate::gui::GuiBlend::MapDecoration => scene.entity_set.unwrap_or(gui),
        crate::gui::GuiBlend::TranslucentBlur
        | crate::gui::GuiBlend::VignetteBlur
        | crate::gui::GuiBlend::LogoAdd => scene.gui_blur.as_ref().map_or(gui, |(set, _)| *set),
        _ => gui,
    }
}
