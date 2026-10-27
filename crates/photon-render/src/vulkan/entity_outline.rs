//! The outlines of glowing entities (`LevelRenderer` with the `entity_outline` post chain): the
//! entities' silhouettes are drawn in their outline colours into an image of their own, with a
//! depth of their own, so nothing in the world hides them; the chain finds the edges, blurs them
//! along x and y, and the result is blended over the frame.

use super::blur::COLOUR_RANGE;
use super::shaders::{
    COPY_FRAGMENT_SHADER, OUTLINE_BLUR_FRAGMENT_SHADER, OUTLINE_SOBEL_FRAGMENT_SHADER,
    SCREENQUAD_VERTEX_SHADER,
};
use super::*;

const DEPTH_RANGE: vk::ImageSubresourceRange = vk::ImageSubresourceRange {
    aspect_mask: vk::ImageAspectFlags::DEPTH,
    base_mip_level: 0,
    level_count: 1,
    base_array_layer: 0,
    layer_count: 1,
};

/// The two images the chain ping-pongs over, and the silhouettes' depth.
struct Targets {
    extent: vk::Extent2D,
    a: Image,
    b: Image,
    depth: Image,
}

/// The chain's pipelines, samplers and sets, and its images once they exist.
pub(super) struct OutlineScene {
    /// The silhouettes' pipeline and the triangles it draws.
    silhouette: Pipeline,
    silhouettes: DrawList,
    /// The glowing items' silhouettes, cut out by the item atlas.
    item_silhouettes: DrawList,
    sobel: Pipeline,
    blur: Pipeline,
    composite: Pipeline,
    _nearest: Sampler,
    _linear: Sampler,
    /// The silhouettes, read without filtering by the edge pass.
    set_a_nearest: vk::DescriptorSet,
    /// The first image read smoothly, by the blur along y.
    set_a_linear: vk::DescriptorSet,
    /// The second image read smoothly, by the blur along x and by the final blend.
    set_b_linear: vk::DescriptorSet,
    targets: Option<Targets>,
}

/// The entities' or (`items`) the items' silhouettes.
fn outline_list(outline: &mut OutlineScene, items: bool) -> &mut DrawList {
    if items {
        &mut outline.item_silhouettes
    } else {
        &mut outline.silhouettes
    }
}

impl VulkanBackend {
    /// The chain's pipelines, samplers and three sets.
    ///
    /// # Safety
    ///
    /// Live handles; `layout` and `pool` belong to the scene's descriptors.
    unsafe fn create_outline_scene(
        &self,
        layout: vk::DescriptorSetLayout,
        pool: vk::DescriptorPool,
    ) -> Result<OutlineScene, VulkanError> {
        // SAFETY: the caller's contract; every create-info points at locals.
        unsafe {
            let silhouette = self.create_pipeline(layout, PipelineKind::EntityOutline)?;
            let fragment = vk::ShaderStageFlags::FRAGMENT;
            let sobel = self.create_plain_pipeline(
                layout,
                (&SCREENQUAD_VERTEX_SHADER, &OUTLINE_SOBEL_FRAGMENT_SHADER),
                (fragment, 8),
                (false, false),
            )?;
            let blur = self.create_plain_pipeline(
                layout,
                (&SCREENQUAD_VERTEX_SHADER, &OUTLINE_BLUR_FRAGMENT_SHADER),
                (fragment, 16),
                (false, false),
            )?;
            let composite = self.create_plain_pipeline(
                layout,
                (&SCREENQUAD_VERTEX_SHADER, &COPY_FRAGMENT_SHADER),
                (fragment, 0),
                (true, true),
            )?;
            let (nearest, linear) = (
                self.clamped_sampler(vk::Filter::NEAREST)?,
                self.clamped_sampler(vk::Filter::LINEAR)?,
            );
            let layouts = [layout; 3];
            let sets = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?;
            Ok(OutlineScene {
                silhouette,
                silhouettes: DrawList::default(),
                item_silhouettes: DrawList::default(),
                sobel,
                blur,
                composite,
                _nearest: nearest,
                _linear: linear,
                set_a_nearest: sets[0],
                set_a_linear: sets[1],
                set_b_linear: sets[2],
                targets: None,
            })
        }
    }

    /// A sampler that filters as `filter` says and clamps at the edges.
    ///
    /// # Safety
    ///
    /// Live handles.
    unsafe fn clamped_sampler(&self, filter: vk::Filter) -> Result<Sampler, VulkanError> {
        // SAFETY: the create-info points at locals.
        unsafe {
            self.device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(filter)
                        .min_filter(filter)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .max_lod(0.0),
                    None,
                )
                .map(|handle| Sampler::new(&self.device, handle))
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))
        }
    }

    /// Hand over the next frame's silhouettes (the items' when `items`). Nothing is kept for
    /// none, and the chain's images are made for the first ones.
    pub(super) fn set_silhouettes(
        &mut self,
        vertices: &[Vertex],
        items: bool,
    ) -> Result<(), VulkanError> {
        if vertices.is_empty() {
            if let Some(outline) = self
                .scene
                .as_mut()
                .and_then(|scene| scene.backdrop.outline.as_mut())
            {
                outline_list(outline, items).count = 0;
            }
            return Ok(());
        }
        self.ensure_outline_targets()?;
        let Some(mut list) = self
            .scene
            .as_mut()
            .and_then(|scene| scene.backdrop.outline.as_mut())
            .map(|outline| std::mem::take(outline_list(outline, items)))
        else {
            return Ok(());
        };
        let filled = self.fill_vertex_buffer(&mut list.buffer, vertices);
        list.count = vertices.len() as u32;
        if let Some(outline) = self
            .scene
            .as_mut()
            .and_then(|scene| scene.backdrop.outline.as_mut())
        {
            *outline_list(outline, items) = list;
        }
        filled
    }

    /// Make the chain's pipelines and sets once, and its images at the frame's size when there
    /// are none or it has changed.
    pub(super) fn ensure_outline_targets(&mut self) -> Result<(), VulkanError> {
        let extent = self.description.extent;
        let Some(scene) = self.scene.as_ref() else {
            return Ok(());
        };
        let current = scene
            .backdrop
            .outline
            .as_ref()
            .and_then(|outline| outline.targets.as_ref())
            .map(|targets| targets.extent);
        if current == Some(extent) {
            return Ok(());
        }
        let (layout, pool) = (scene.descriptors.layout, scene.descriptors.pool);
        let made = scene.backdrop.outline.is_none();
        // SAFETY: the wait leaves the GPU done with the old images and sets before they are
        // replaced; the handles are live.
        unsafe {
            self.wait_in_flight()?;
            if made {
                let outline = self.create_outline_scene(layout, pool)?;
                if let Some(scene) = self.scene.as_mut() {
                    scene.backdrop.outline = Some(outline);
                }
            }
            let targets = Targets {
                extent,
                a: self.colour_target(extent)?,
                b: self.colour_target(extent)?,
                depth: self.depth_image()?,
            };
            let Some(outline) = self
                .scene
                .as_mut()
                .and_then(|scene| scene.backdrop.outline.as_mut())
            else {
                return Ok(());
            };
            for (set, image, sampler) in [
                (outline.set_a_nearest, &targets.a, &outline._nearest),
                (outline.set_a_linear, &targets.a, &outline._linear),
                (outline.set_b_linear, &targets.b, &outline._linear),
            ] {
                let info = [vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(image.view)
                    .sampler(sampler.handle)];
                self.device.update_descriptor_sets(
                    &[vk::WriteDescriptorSet::default()
                        .dst_set(set)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(&info)],
                    &[],
                );
            }
            outline.targets = Some(targets);
        }
        Ok(())
    }

    /// The chain, when there are silhouettes to outline and the images exist at the frame's size.
    fn outline_chain<'a>(
        &self,
        scene: &'a SceneResources,
    ) -> Option<(&'a OutlineScene, &'a Targets)> {
        let outline = scene.backdrop.outline.as_ref()?;
        let targets = outline
            .targets
            .as_ref()
            .filter(|targets| targets.extent == self.description.extent)?;
        (!self.gui_only && outline.silhouettes.count + outline.item_silhouettes.count > 0)
            .then_some((outline, targets))
    }

    /// Draw the silhouettes and run the chain over them, before the frame's own rendering scope
    /// begins.
    ///
    /// # Safety
    ///
    /// Recording outside a rendering scope; `scene` is live and `base` are the frame's push
    /// constants.
    pub(super) unsafe fn record_outline_chain(&self, scene: &SceneResources, base: &[f32; 46]) {
        let Some((outline, targets)) = self.outline_chain(scene) else {
            return;
        };
        let extent = targets.extent;
        let size = [extent.width as f32, extent.height as f32];
        // SAFETY: recording outside a scope (see `# Safety`); the images, sets and pipelines are
        // live, and each pass writes a whole image the previous pass has finished with.
        unsafe {
            self.silhouette_pass((scene, outline), base);
            let fragment = vk::ShaderStageFlags::FRAGMENT;
            self.offscreen_pass(&targets.b, extent, |this| {
                this.bind_state(
                    &outline.sobel,
                    Some(outline.set_a_nearest),
                    &[(fragment, 0, &as_bytes(&size)[..8])],
                );
                this.device.cmd_draw(this.command_buffer, 3, 1, 0, 0);
            });
            for (direction, from, to) in [
                ([1.0, 0.0], outline.set_b_linear, &targets.a),
                ([0.0, 1.0], outline.set_a_linear, &targets.b),
            ] {
                let push = [direction[0], direction[1], size[0], size[1]];
                self.offscreen_pass(to, extent, |this| {
                    this.bind_state(
                        &outline.blur,
                        Some(from),
                        &[(fragment, 0, &as_bytes(&push)[..16])],
                    );
                    this.device.cmd_draw(this.command_buffer, 3, 1, 0, 0);
                });
            }
        }
    }

    /// Draw the silhouettes into the first image, cleared to nothing, over a depth of their own.
    ///
    /// # Safety
    ///
    /// Recording outside a rendering scope; the images are live.
    unsafe fn silhouette_pass(
        &self,
        (scene, outline): (&SceneResources, &OutlineScene),
        base: &[f32; 46],
    ) {
        let Some(targets) = outline.targets.as_ref() else {
            return;
        };
        // SAFETY: recording outside a scope (see `# Safety`).
        unsafe {
            self.begin_silhouettes(targets);
            self.draw_silhouettes(
                (scene.entity_set, &outline.silhouette, &outline.silhouettes),
                base,
            );
            let items = Some(scene.descriptor_set);
            self.draw_silhouettes(
                (items, &outline.silhouette, &outline.item_silhouettes),
                base,
            );
            self.dynamic_rendering
                .cmd_end_rendering(self.command_buffer);
            transition(
                &self.device,
                self.command_buffer,
                targets.a.image,
                COLOUR_RANGE,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            );
        }
    }

    /// Begin rendering into the first image and the depth, both cleared, and set the viewport and
    /// the scissor.
    ///
    /// # Safety
    ///
    /// Recording outside a rendering scope; the images are live.
    unsafe fn begin_silhouettes(&self, targets: &Targets) {
        let extent = targets.extent;
        // SAFETY: recording outside a scope (see `# Safety`).
        unsafe {
            for (image, range, layout) in [
                (
                    targets.a.image,
                    COLOUR_RANGE,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ),
                (
                    targets.depth.image,
                    DEPTH_RANGE,
                    vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                ),
            ] {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::UNDEFINED,
                    layout,
                );
            }
            let (colour, depth) = silhouette_attachments(targets);
            let rendering = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(extent))
                .layer_count(1)
                .color_attachments(&colour)
                .depth_attachment(&depth);
            self.dynamic_rendering
                .cmd_begin_rendering(self.command_buffer, &rendering);
            self.device.cmd_set_viewport(
                self.command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(extent.width as f32)
                    .height(extent.height as f32)
                    .max_depth(1.0)],
            );
            self.device.cmd_set_scissor(
                self.command_buffer,
                0,
                &[vk::Rect2D::default().extent(extent)],
            );
        }
    }

    /// Blend the chain's result over the frame.
    ///
    /// # Safety
    ///
    /// Recording inside the scene's rendering scope; `scene` is live.
    pub(super) unsafe fn draw_entity_outlines(&self, scene: &SceneResources) {
        let Some((outline, _)) = self.outline_chain(scene) else {
            return;
        };
        // SAFETY: recording inside the scope (see `# Safety`); the pipeline and set are live and
        // the image was written by `record_outline_chain` this frame.
        unsafe {
            self.bind_state(&outline.composite, Some(outline.set_b_linear), &[]);
            self.device.cmd_draw(self.command_buffer, 3, 1, 0, 0);
        }
    }
}

/// The cleared image and depth the silhouettes are drawn into.
fn silhouette_attachments(
    targets: &Targets,
) -> (
    [vk::RenderingAttachmentInfo<'static>; 1],
    vk::RenderingAttachmentInfo<'static>,
) {
    let colour = vk::RenderingAttachmentInfo::default()
        .image_view(targets.a.view)
        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0; 4] },
        });
    let depth = vk::RenderingAttachmentInfo::default()
        .image_view(targets.depth.view)
        .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .clear_value(vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 0.0,
                stencil: 0,
            },
        });
    ([colour], depth)
}
