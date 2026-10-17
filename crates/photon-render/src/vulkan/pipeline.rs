//! Building a graphics pipeline.

use super::shaders::*;
use super::vertex_attributes::*;
use super::*;

impl VulkanBackend {
    /// Build the graphics pipeline from its compiled shaders.
    ///
    /// Dynamic rendering is used rather than a render pass object, which is
    /// the extension the reference already requires of every device it picks.
    ///
    /// # Safety
    ///
    /// `descriptor_layout` must be a live layout of this device.
    pub(super) unsafe fn create_pipeline(
        &self,
        descriptor_layout: vk::DescriptorSetLayout,
        kind: PipelineKind,
    ) -> Result<Pipeline, VulkanError> {
        let sky = matches!(
            kind,
            PipelineKind::SkyTranslucent | PipelineKind::SkyOverlay
        );
        let translucent = kind == PipelineKind::Translucent;
        let clouds = kind == PipelineKind::Clouds;
        let weather = kind == PipelineKind::Weather;
        let dissolve = kind == PipelineKind::Dissolve;
        let rays = kind == PipelineKind::Rays;
        let portal = kind == PipelineKind::Portal;
        let world_border = kind == PipelineKind::WorldBorder;
        let entity = kind == PipelineKind::Entity || dissolve;
        let shadow = kind == PipelineKind::Shadow;
        let swirl = kind == PipelineKind::Swirl;
        let glint = kind == PipelineKind::Glint;
        let text = matches!(kind, PipelineKind::Text | PipelineKind::TextTranslucent) || shadow;
        let text_see_through = kind == PipelineKind::TextSeeThrough;
        let gui = matches!(
            kind,
            PipelineKind::Gui
                | PipelineKind::GuiInvert
                | PipelineKind::GuiHighlight
                | PipelineKind::GuiVignette
        );
        let gui_item = kind == PipelineKind::GuiItem;
        let lines = kind == PipelineKind::Lines;
        let crumbling = kind == PipelineKind::Crumbling;
        let screen_effect = kind == PipelineKind::ScreenEffect;
        let section = matches!(kind, PipelineKind::Terrain | PipelineKind::Translucent);
        // SAFETY: live handles; the shader modules are made from the embedded SPIR-V and
        // destroyed once the pipeline is built, and every create-info points at locals that
        // outlive its call.
        unsafe {
            let (vertex_source, fragment_source) = if sky {
                (SKY_VERTEX_SHADER, SKY_FRAGMENT_SHADER)
            } else if clouds {
                (CLOUD_VERTEX_SHADER, CLOUD_FRAGMENT_SHADER)
            } else if weather {
                (VERTEX_SHADER, WEATHER_FRAGMENT_SHADER)
            } else if dissolve {
                (ENTITY_VERTEX_SHADER, DISSOLVE_FRAGMENT_SHADER)
            } else if rays {
                (RAYS_VERTEX_SHADER, RAYS_FRAGMENT_SHADER)
            } else if portal {
                (PORTAL_VERTEX_SHADER, PORTAL_FRAGMENT_SHADER)
            } else if world_border {
                (WORLD_BORDER_VERTEX_SHADER, WORLD_BORDER_FRAGMENT_SHADER)
            } else if entity {
                (ENTITY_VERTEX_SHADER, ENTITY_FRAGMENT_SHADER)
            } else if swirl {
                (SWIRL_VERTEX_SHADER, SWIRL_FRAGMENT_SHADER)
            } else if glint {
                (GLINT_VERTEX_SHADER, GLINT_FRAGMENT_SHADER)
            } else if shadow {
                (SHADOW_VERTEX_SHADER, SHADOW_FRAGMENT_SHADER)
            } else if text {
                (TEXT_VERTEX_SHADER, TEXT_FRAGMENT_SHADER)
            } else if text_see_through {
                (
                    TEXT_SEE_THROUGH_VERTEX_SHADER,
                    TEXT_SEE_THROUGH_FRAGMENT_SHADER,
                )
            } else if gui {
                (GUI_VERTEX_SHADER, GUI_FRAGMENT_SHADER)
            } else if gui_item {
                (GUI_ITEM_VERTEX_SHADER, GUI_ITEM_FRAGMENT_SHADER)
            } else if lines {
                (LINE_VERTEX_SHADER, LINE_FRAGMENT_SHADER)
            } else if crumbling {
                (VERTEX_SHADER, CRUMBLING_FRAGMENT_SHADER)
            } else if section {
                (SECTION_VERTEX_SHADER, FRAGMENT_SHADER)
            } else {
                (HELD_VERTEX_SHADER, FRAGMENT_SHADER)
            };
            let (vertex_code, fragment_code) = (vertex_source.words(), fragment_source.words());
            let make_module = |code: &[u32]| -> Result<vk::ShaderModule, VulkanError> {
                self.device
                    .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None)
                    .map_err(|result| VulkanError::Call("vkCreateShaderModule", result))
            };
            let vertex_module = make_module(&vertex_code)?;
            let fragment_module = make_module(&fragment_code)?;

            let entry = CString::new("main").expect("static name");
            let stages = [
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::VERTEX)
                    .module(vertex_module)
                    .name(&entry),
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::FRAGMENT)
                    .module(fragment_module)
                    .name(&entry),
            ];

            let stride = if sky {
                std::mem::size_of::<crate::sky::SkyVertex>()
            } else if clouds {
                std::mem::size_of::<crate::sky::clouds::CloudVertex>()
            } else if gui {
                std::mem::size_of::<crate::gui::GuiVertex>()
            } else if gui_item {
                std::mem::size_of::<crate::gui::items::GuiItemVertex>()
            } else if lines {
                std::mem::size_of::<crate::block::outline::LineVertex>()
            } else if section {
                std::mem::size_of::<BlockVertex>()
            } else {
                std::mem::size_of::<Vertex>()
            };
            let bindings = [vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(stride as u32)
                .input_rate(vk::VertexInputRate::VERTEX)];
            let sky_attributes = sky_attributes();
            let terrain_attributes = terrain_attributes();
            let section_attributes = section_attributes();
            let cloud_attributes = cloud_attributes();
            let gui_attributes = gui_attributes();
            let line_attributes = line_attributes();
            let gui_item_attributes = gui_item_attributes();
            let attributes: &[vk::VertexInputAttributeDescription] = if sky {
                &sky_attributes
            } else if gui_item {
                &gui_item_attributes
            } else if clouds {
                &cloud_attributes
            } else if gui {
                &gui_attributes
            } else if lines {
                &line_attributes
            } else if section {
                &section_attributes
            } else {
                &terrain_attributes
            };
            let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(&bindings)
                .vertex_attribute_descriptions(attributes);
            let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
            // The viewport follows the window, so it is set per frame.
            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewport_count(1)
                .scissor_count(1);
            let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state =
                vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
            let raster = vk::PipelineRasterizationStateCreateInfo::default()
                .polygon_mode(vk::PolygonMode::FILL)
                // The sky's fans are built in either winding.
                .cull_mode(cull_mode(kind))
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .line_width(1.0)
                // The crack is pulled towards the camera so it wins over the
                // block's own faces: the reference's polygon offset. The
                // world border overlay gets its own, smaller offset
                // (`DepthStencilState`'s own bias factors).
                .depth_bias_enable(crumbling || world_border)
                .depth_bias_constant_factor(if crumbling {
                    10.0
                } else if world_border {
                    3.0
                } else {
                    0.0
                })
                .depth_bias_slope_factor(if crumbling {
                    1.0
                } else if world_border {
                    3.0
                } else {
                    0.0
                });
            let multisample = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let depth_state = vk::PipelineDepthStencilStateCreateInfo::default()
                // The sky is behind everything and tests nothing. Translucent
                // faces are tested against the opaque depth but leave it
                // alone, so what is behind them still shows. Depth is
                // reversed and equal depth passes, as in the reference, so
                // a model's later face over an earlier one at the same place
                // (the grass side overlay) is the one seen.
                .depth_test_enable(!sky && !gui && !text_see_through && !screen_effect)
                .depth_write_enable(matches!(
                    kind,
                    PipelineKind::Terrain
                        | PipelineKind::Clouds
                        | PipelineKind::Entity
                        | PipelineKind::Dissolve
                        | PipelineKind::Rays
                        | PipelineKind::Portal
                        | PipelineKind::Swirl
                        | PipelineKind::Text
                        | PipelineKind::Lines
                        | PipelineKind::GuiItem
                        | PipelineKind::WorldBorder
                ))
                .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);
            let blends = [if crumbling {
                // Source times destination, twice: darkens and lightens.
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::DST_COLOR)
                    .dst_color_blend_factor(vk::BlendFactor::SRC_COLOR)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if kind == PipelineKind::GuiInvert {
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::ONE_MINUS_DST_COLOR)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_COLOR)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if kind == PipelineKind::GuiVignette {
                // The credits' `VIGNETTE`: the destination darkened by the source colour.
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::ZERO)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_COLOR)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if kind == PipelineKind::GuiHighlight {
                // `BlendFunction.ADDITIVE`.
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::ONE)
                    .dst_color_blend_factor(vk::BlendFactor::ONE)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if kind == PipelineKind::SkyOverlay || rays || world_border {
                // Added by its alpha (`BlendFunction.LIGHTNING`/`BlendFunction.OVERLAY`:
                // the same factors).
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if glint {
                // `BlendFunction.GLINT`: added by its own colour, the alpha kept.
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_COLOR)
                    .dst_color_blend_factor(vk::BlendFactor::ONE)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ZERO)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if swirl {
                // Added to what is behind (`BlendFunction.ADDITIVE`).
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::ONE)
                    .dst_color_blend_factor(vk::BlendFactor::ONE)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else if translucent
                || sky
                || clouds
                || weather
                || entity
                || text
                || text_see_through
                || gui
                || gui_item
                || lines
                || screen_effect
            {
                vk::PipelineColorBlendAttachmentState::default()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            } else {
                vk::PipelineColorBlendAttachmentState::default()
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
            }];
            let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blends);

            let push_ranges = [
                if sky {
                    vk::PushConstantRange::default()
                        .stage_flags(vk::ShaderStageFlags::VERTEX)
                        .size(64)
                } else if clouds {
                    vk::PushConstantRange::default()
                        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                        .size(96)
                } else {
                    vk::PushConstantRange::default()
                        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                        .size(192)
                },
                vk::PushConstantRange::default()
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                    .offset(64)
                    .size(32),
            ];
            let push_ranges = if sky {
                &push_ranges[..]
            } else {
                &push_ranges[..1]
            };
            // The portal samples two textures, each in a set of the one layout.
            let both = [descriptor_layout, descriptor_layout];
            let set_layouts = if portal { &both[..] } else { &both[..1] };
            let pipeline_layout = self
                .device
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::default()
                        .set_layouts(set_layouts)
                        .push_constant_ranges(push_ranges),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreatePipelineLayout", result))?;

            let colour_formats = [self.description.surface_format];
            let mut rendering = vk::PipelineRenderingCreateInfo::default()
                .color_attachment_formats(&colour_formats)
                .depth_attachment_format(
                    self.depth
                        .as_ref()
                        .map_or(vk::Format::UNDEFINED, |d| d.format),
                );
            let create = vk::GraphicsPipelineCreateInfo::default()
                .stages(&stages)
                .vertex_input_state(&vertex_input)
                .input_assembly_state(&assembly)
                .viewport_state(&viewport_state)
                .dynamic_state(&dynamic_state)
                .rasterization_state(&raster)
                .multisample_state(&multisample)
                .depth_stencil_state(&depth_state)
                .color_blend_state(&blend)
                .layout(pipeline_layout)
                .push_next(&mut rendering);
            let created =
                self.device
                    .create_graphics_pipelines(self.pipeline_cache, &[create], None);
            // The modules are only needed while the pipeline is made.
            self.device.destroy_shader_module(vertex_module, None);
            self.device.destroy_shader_module(fragment_module, None);
            match created {
                Ok(pipelines) => Ok(Pipeline::new(&self.device, pipeline_layout, pipelines[0])),
                Err((_, result)) => {
                    self.device.destroy_pipeline_layout(pipeline_layout, None);
                    Err(VulkanError::Call("vkCreateGraphicsPipelines", result))
                }
            }
        }
    }
}
