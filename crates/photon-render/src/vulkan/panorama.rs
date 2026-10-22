//! The title screen's panorama (`CubeMap`): six faces in one cube image, drawn as a cube round the
//! camera before the GUI.

use super::shaders::{PANORAMA_FRAGMENT_SHADER, PANORAMA_VERTEX_SHADER, Shader};
use super::*;

/// The cube image, the set that samples it and the pipeline that draws the cube.
pub(super) struct PanoramaScene {
    _texture: Texture,
    pub(super) set: vk::DescriptorSet,
    pipeline: Pipeline,
}

/// The cube's vertices: six quads of two triangles, generated in the vertex shader.
const CUBE_VERTICES: u32 = 36;

impl VulkanBackend {
    /// Load the six faces, in the cube's layer order (`panorama::layers`): one image of six
    /// layers, sampled linearly and clamped.
    pub fn load_panorama(&mut self, layers: &[DecodedImage; 6]) -> Result<(), VulkanError> {
        if self
            .scene
            .as_ref()
            .is_none_or(|scene| scene.backdrop.panorama.is_some())
        {
            return Ok(());
        }
        // SAFETY: live handles; this only creates objects and stores them in the scene. No frame
        // is recording while a texture is loaded.
        unsafe {
            let (layout, pool) = {
                let descriptors = &self.scene.as_ref().expect("checked above").descriptors;
                (descriptors.layout, descriptors.pool)
            };
            let image = self.upload_cube(layers)?;
            let (sampler, set) = self.cube_set(layout, pool, &image)?;
            let pipeline = self.create_panorama_pipeline(layout)?;
            self.ensure_blur_scene()?;
            let panorama = PanoramaScene {
                _texture: Texture { image, sampler },
                set,
                pipeline,
            };
            self.scene
                .as_mut()
                .expect("checked above")
                .backdrop
                .panorama = Some(panorama);
            Ok(())
        }
    }

    /// A linear, clamped sampler over the cube and a set that samples it.
    ///
    /// # Safety
    ///
    /// Live handles; `layout` and `pool` belong to the scene's descriptors.
    unsafe fn cube_set(
        &self,
        layout: vk::DescriptorSetLayout,
        pool: vk::DescriptorPool,
        image: &Image,
    ) -> Result<(Sampler, vk::DescriptorSet), VulkanError> {
        // SAFETY: the caller's contract; this only creates objects the caller stores.
        unsafe {
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::LINEAR)
                        .min_filter(vk::Filter::LINEAR)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .max_lod(0.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;
            let sampler = Sampler::new(&self.device, sampler);
            let layouts = [layout];
            let set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(image.view)
                .sampler(sampler.handle)];
            self.device.update_descriptor_sets(
                &[vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_info)],
                &[],
            );
            Ok((sampler, set))
        }
    }

    /// Hand over where the cube is seen from for the next frames, or `None` to draw none.
    pub fn set_panorama(&mut self, view_projection: Option<[[f32; 4]; 4]>) {
        self.panorama_view =
            view_projection.map(|matrix| std::array::from_fn(|i| matrix[i / 4][i % 4]));
    }

    /// Draw the cube over the cleared frame.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    pub(super) unsafe fn record_panorama(&self, scene: &SceneResources) {
        let (Some(panorama), Some(matrix)) = (
            scene.backdrop.panorama.as_ref(),
            self.panorama_view.as_ref(),
        ) else {
            return;
        };
        // SAFETY: recording (see `# Safety`); the pipeline and set are live and the cube needs no
        // vertex buffer.
        unsafe {
            if self.blur_active() {
                self.draw_blurred_panorama(scene);
                return;
            }
            self.bind_state(
                &panorama.pipeline,
                Some(panorama.set),
                &[(vk::ShaderStageFlags::VERTEX, 0, &as_bytes(matrix)[..64])],
            );
            self.device
                .cmd_draw(self.command_buffer, CUBE_VERTICES, 1, 0, 0);
        }
    }

    /// Upload six equal faces into a cube-compatible image of six layers.
    ///
    /// # Safety
    ///
    /// No frame may be recording: it submits setup work and waits for the queue.
    unsafe fn upload_cube(&self, layers: &[DecodedImage; 6]) -> Result<Image, VulkanError> {
        let (width, height) = (layers[0].width, layers[0].height);
        let staged: Vec<u8> = layers.iter().flat_map(|layer| layer.rgba.clone()).collect();
        // SAFETY: live handles; the staging buffer lives only until `run_once` has finished the
        // copy, and the image's memory is bound before it is viewed.
        unsafe {
            let staging = self.upload_buffer(vk::BufferUsageFlags::TRANSFER_SRC, &staged)?;
            let format = vk::Format::R8G8B8A8_UNORM;
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .flags(vk::ImageCreateFlags::CUBE_COMPATIBLE)
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width,
                            height,
                            depth: 1,
                        })
                        .mip_levels(1)
                        .array_layers(6)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .initial_layout(vk::ImageLayout::UNDEFINED),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImage", result))?;
            let requirements = self.device.get_image_memory_requirements(image);
            let memory = self.allocate(requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            self.device
                .bind_image_memory(image, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindImageMemory", result))?;
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(6);
            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::CUBE)
                        .format(format)
                        .subresource_range(range),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            let texture = Image::new(&self.device, (image, memory, view), format);
            let layer_bytes = u64::from(width) * u64::from(height) * 4;
            let regions: Vec<vk::BufferImageCopy> = (0..6)
                .map(|layer| {
                    vk::BufferImageCopy::default()
                        .buffer_offset(u64::from(layer) * layer_bytes)
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_array_layer(layer)
                                .layer_count(1),
                        )
                        .image_extent(vk::Extent3D {
                            width,
                            height,
                            depth: 1,
                        })
                })
                .collect();
            self.run_once(|command| {
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::UNDEFINED,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
                self.device.cmd_copy_buffer_to_image(
                    command,
                    staging.buffer,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &regions,
                );
                transition(
                    &self.device,
                    command,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
            })?;
            Ok(texture)
        }
    }

    /// `RenderPipelines.PANORAMA`: no vertex input, nothing blended, no depth.
    ///
    /// # Safety
    ///
    /// Live handles; `descriptor_layout` belongs to this device.
    unsafe fn create_panorama_pipeline(
        &self,
        descriptor_layout: vk::DescriptorSetLayout,
    ) -> Result<Pipeline, VulkanError> {
        // SAFETY: the caller's contract.
        unsafe {
            self.create_plain_pipeline(
                descriptor_layout,
                (&PANORAMA_VERTEX_SHADER, &PANORAMA_FRAGMENT_SHADER),
                (vk::ShaderStageFlags::VERTEX, 64),
                (true, false),
            )
        }
    }

    /// A pipeline with no vertex input and nothing blended or depth tested: the shaders, the
    /// stage and size of its push constants, and whether the target it draws to has a depth
    /// attachment.
    ///
    /// # Safety
    ///
    /// Live handles; `descriptor_layout` belongs to this device.
    pub(super) unsafe fn create_plain_pipeline(
        &self,
        descriptor_layout: vk::DescriptorSetLayout,
        (vertex, fragment): (&Shader, &Shader),
        (push_stage, push_size): (vk::ShaderStageFlags, u32),
        (with_depth, blended): (bool, bool),
    ) -> Result<Pipeline, VulkanError> {
        // SAFETY: the shader modules are made from the embedded SPIR-V and destroyed once the
        // pipeline is built, and every create-info points at locals that outlive its call.
        unsafe {
            let make_module = |code: Vec<u32>| -> Result<vk::ShaderModule, VulkanError> {
                self.device
                    .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&code), None)
                    .map_err(|result| VulkanError::Call("vkCreateShaderModule", result))
            };
            let vertex_module = make_module(vertex.words())?;
            let fragment_module = make_module(fragment.words())?;
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
            let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
            let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewport_count(1)
                .scissor_count(1);
            let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state =
                vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
            let raster = vk::PipelineRasterizationStateCreateInfo::default()
                .polygon_mode(vk::PolygonMode::FILL)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .line_width(1.0);
            let multisample = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let depth_state = vk::PipelineDepthStencilStateCreateInfo::default();
            let blends = [plain_blend(blended)];
            let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blends);
            let push_ranges = [vk::PushConstantRange::default()
                .stage_flags(push_stage)
                .size(push_size)];
            let set_layouts = [descriptor_layout];
            let pipeline_layout = self
                .device
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::default()
                        .set_layouts(&set_layouts)
                        .push_constant_ranges(&push_ranges),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreatePipelineLayout", result))?;
            let colour_formats = [self.description.surface_format];
            let mut rendering = vk::PipelineRenderingCreateInfo::default()
                .color_attachment_formats(&colour_formats)
                .depth_attachment_format(if with_depth {
                    self.depth
                        .as_ref()
                        .map_or(vk::Format::UNDEFINED, |d| d.format)
                } else {
                    vk::Format::UNDEFINED
                });
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

/// The one colour attachment of a plain pipeline: written as it is, or with the ordinary alpha
/// blend (`BlendFunction.TRANSLUCENT`).
fn plain_blend(blended: bool) -> vk::PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(blended)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD)
}
