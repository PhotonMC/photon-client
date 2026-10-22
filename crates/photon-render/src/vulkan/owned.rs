//! Vulkan objects that free themselves when dropped: buffers, images, samplers, pipelines and
//! descriptor pools.
//!
//! Each owner drops them only when the GPU is done with them: after a wait for the frames that
//! may use them, or once the device is idle (`VulkanBackend::drop` clears every owner before the
//! device goes). None may outlive the device.

use super::*;

/// A GPU image with its memory and view.
pub(super) struct Image {
    device: ash::Device,
    pub(super) image: vk::Image,
    pub(super) memory: vk::DeviceMemory,
    pub(super) view: vk::ImageView,
    pub(super) format: vk::Format,
}

impl Image {
    /// Take ownership of an image, its bound memory and its view.
    pub(super) fn new(
        device: &ash::Device,
        (image, memory, view): (vk::Image, vk::DeviceMemory, vk::ImageView),
        format: vk::Format,
    ) -> Self {
        Self {
            device: device.clone(),
            image,
            memory,
            view,
            format,
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        // SAFETY: this value owns the three handles and runs once, so each is destroyed once; the
        // owner keeps the contract above, so the device is live and the GPU is done with them.
        unsafe {
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.image, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

/// A sampler.
pub(super) struct Sampler {
    device: ash::Device,
    pub(super) handle: vk::Sampler,
}

impl Sampler {
    /// Take ownership of a sampler.
    pub(super) fn new(device: &ash::Device, handle: vk::Sampler) -> Self {
        Self {
            device: device.clone(),
            handle,
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        // SAFETY: this value owns the handle and runs once; see the module's contract.
        unsafe { self.device.destroy_sampler(self.handle, None) };
    }
}

/// An image and the sampler it is read with.
pub(super) struct Texture {
    pub(super) image: Image,
    pub(super) sampler: Sampler,
}

/// A graphics pipeline with its layout.
pub(super) struct Pipeline {
    device: ash::Device,
    pub(super) layout: vk::PipelineLayout,
    pub(super) handle: vk::Pipeline,
}

impl Pipeline {
    /// Take ownership of a pipeline and its layout.
    pub(super) fn new(
        device: &ash::Device,
        layout: vk::PipelineLayout,
        handle: vk::Pipeline,
    ) -> Self {
        Self {
            device: device.clone(),
            layout,
            handle,
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        // SAFETY: this value owns both handles and runs once; see the module's contract. The
        // pipeline goes before the layout it was made with.
        unsafe {
            self.device.destroy_pipeline(self.handle, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}

/// A descriptor pool with the layout of the sets it hands out.
pub(super) struct Descriptors {
    device: ash::Device,
    pub(super) layout: vk::DescriptorSetLayout,
    pub(super) pool: vk::DescriptorPool,
}

/// How many sets the scene's pool hands out: the atlas's, the rain's and the snow's, the skin's,
/// the GUI's, the cracks', the maps', the panorama's, the world border's, the water's over the view
/// and the end portal's two, and the entity outline chain's three.
const SET_COUNT: u32 = 20;

impl Descriptors {
    /// A layout of one combined image sampler for the fragment stage, and a pool for the
    /// scene's sets of it.
    ///
    /// # Safety
    ///
    /// `device` must be live.
    pub(super) unsafe fn create(device: &ash::Device) -> Result<Self, VulkanError> {
        // SAFETY: the caller's contract; the layout is destroyed here if the pool cannot be made,
        // and otherwise both are owned by the returned value.
        unsafe {
            let bindings = [vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
            let layout = device
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateDescriptorSetLayout", result))?;
            let sizes = [vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(SET_COUNT)];
            let pool = match device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(SET_COUNT)
                    .pool_sizes(&sizes),
                None,
            ) {
                Ok(pool) => pool,
                Err(result) => {
                    device.destroy_descriptor_set_layout(layout, None);
                    return Err(VulkanError::Call("vkCreateDescriptorPool", result));
                }
            };
            Ok(Self {
                device: device.clone(),
                layout,
                pool,
            })
        }
    }
}

impl Drop for Descriptors {
    fn drop(&mut self) {
        // SAFETY: this value owns both handles and runs once; see the module's contract. Freeing
        // the pool frees its sets, and the layout goes after it.
        unsafe {
            self.device.destroy_descriptor_pool(self.pool, None);
            self.device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

/// A buffer with the memory bound to it.
pub(super) struct BoundBuffer {
    pub(super) device: ash::Device,
    pub(super) buffer: vk::Buffer,
    pub(super) memory: vk::DeviceMemory,
}

impl Drop for BoundBuffer {
    fn drop(&mut self) {
        // SAFETY: this value owns both handles and runs once; see the module's contract. A null
        // memory (a buffer whose memory could not be made) is a no-op for `free_memory`.
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
