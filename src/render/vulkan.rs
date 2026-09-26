//! Vulkan device and swapchain set up the way the reference's Vulkan backend
//! does it.
//!
//! Nothing here draws world content yet: the backend comes up, acquires a
//! swapchain image, clears it and presents. That is enough to prove the surface
//! is live, and it deliberately makes no parity claim about pixels — those
//! start once the shader pipeline exists and P15.a has measured what the Java
//! reference's own repeats vary by.
//!
//! The choices that *are* taken from the reference are the ones that decide
//! which GPU and which image the client ends up with: API version, required
//! device extensions and features, discrete-GPU preference, surface format
//! filter, and the swapchain parameters.

use std::ffi::{CStr, CString};

use ash::vk;

use crate::render::png::DecodedImage;
use crate::render::scene::{Mesh, Vertex};

/// Scaffolding shaders. These are this project's own, not the reference's:
/// they exist to prove the pipeline works, and carry no parity claim.
const VERTEX_SHADER: &str = r"#version 450
layout(push_constant) uniform Push { mat4 view_projection; } push;
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in float in_shade;
layout(location = 0) out vec2 out_uv;
layout(location = 1) out float out_shade;
void main() {
    gl_Position = push.view_projection * vec4(in_position, 1.0);
    out_uv = in_uv;
    out_shade = in_shade;
}
";

const FRAGMENT_SHADER: &str = r"#version 450
layout(set = 0, binding = 0) uniform sampler2D atlas;
layout(location = 0) in vec2 in_uv;
layout(location = 1) in float in_shade;
layout(location = 0) out vec4 out_colour;
void main() {
    vec4 texel = texture(atlas, in_uv);
    if (texel.a < 0.1) { discard; }
    out_colour = vec4(texel.rgb * in_shade, texel.a);
}
";

/// Instance API version the reference asks for.
const API_VERSION: u32 = vk::API_VERSION_1_2;
/// Lowest device API version the reference's feature query accepts.
const MINIMUM_DEVICE_API_VERSION: u32 = vk::API_VERSION_1_1;
/// No transform, for the frame paths that draw nothing.
const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Swapchain images the reference requests, clamped up by surface capabilities.
const REQUESTED_IMAGE_COUNT: u32 = 3;

/// Device extensions the reference requires before a GPU is considered usable.
const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 5] = [
    c"VK_KHR_dynamic_rendering",
    c"VK_KHR_push_descriptor",
    c"VK_KHR_synchronization2",
    c"VK_EXT_vertex_attribute_divisor",
    c"VK_KHR_swapchain",
];

/// Surface formats the reference accepts, paired with an sRGB non-linear space.
const ACCEPTED_SURFACE_FORMATS: [vk::Format; 2] =
    [vk::Format::R8G8B8A8_UNORM, vk::Format::B8G8R8A8_UNORM];

/// Everything that can stop the backend from coming up.
#[derive(Debug)]
pub enum VulkanError {
    /// The loader or a driver entry point was missing.
    Loader(String),
    /// A Vulkan call returned a failure code.
    Call(&'static str, vk::Result),
    /// No physical device met the reference's requirements.
    NoSuitableDevice,
    /// The surface offered no format the reference would accept.
    NoSuitableSurfaceFormat,
    /// The scene was handed an atlas with no levels at all.
    NoAtlasLevels,
}

impl std::fmt::Display for VulkanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loader(detail) => write!(formatter, "Vulkan loader unavailable: {detail}"),
            Self::Call(what, result) => write!(formatter, "{what} failed: {result:?}"),
            Self::NoSuitableDevice => {
                formatter.write_str("no physical device meets the required extensions and features")
            }
            Self::NoSuitableSurfaceFormat => {
                formatter.write_str("surface offers no accepted format in sRGB non-linear space")
            }
            Self::NoAtlasLevels => formatter.write_str("atlas has no mip levels to upload"),
        }
    }
}

impl std::error::Error for VulkanError {}

/// What the backend selected, for reporting without exposing raw handles.
#[derive(Debug, Clone)]
pub struct BackendDescription {
    /// Device name as the driver reports it.
    pub device_name: String,
    /// Whether the selected device is discrete, which the reference prefers.
    pub discrete: bool,
    /// Device API version, encoded as Vulkan does.
    pub api_version: u32,
    /// Chosen swapchain image format.
    pub surface_format: vk::Format,
    /// Number of swapchain images actually created.
    pub image_count: u32,
    /// Swapchain extent in pixels.
    pub extent: vk::Extent2D,
}

/// Depth attachment matching the swapchain extent.
struct DepthTarget {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    format: vk::Format,
}

/// Everything needed to draw one textured mesh.
struct SceneResources {
    vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory,
    index_count: u32,
    texture: vk::Image,
    texture_memory: vk::DeviceMemory,
    texture_view: vk::ImageView,
    sampler: vk::Sampler,
    descriptor_pool: vk::DescriptorPool,
    descriptor_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

/// One frame read back from the swapchain, as tightly packed RGBA8 rows.
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Row-major RGBA8 samples.
    pub rgba: Vec<u8>,
}

/// A live Vulkan instance, device and swapchain for one window surface.
pub struct VulkanBackend {
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_instance: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    swapchain_device: ash::khr::swapchain::Device,
    // The instance targets Vulkan 1.2, where dynamic rendering is still an
    // extension, so its entry points come from the KHR loader rather than core.
    dynamic_rendering: ash::khr::dynamic_rendering::Device,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    readback_buffer: vk::Buffer,
    readback_memory: vk::DeviceMemory,
    depth: Option<DepthTarget>,
    scene: Option<SceneResources>,
    image_available: vk::Semaphore,
    render_finished: Vec<vk::Semaphore>,
    in_flight: vk::Fence,
    description: BackendDescription,
}

impl VulkanBackend {
    /// Bring the backend up against an existing window surface.
    ///
    /// # Safety
    ///
    /// `display_handle` and `window_handle` must refer to a window that outlives
    /// this backend; the surface borrows them.
    pub unsafe fn create(
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, VulkanError> {
        unsafe {
            let entry =
                ash::Entry::load().map_err(|error| VulkanError::Loader(error.to_string()))?;

            // The reference takes its instance extensions from GLFW and names
            // the application the same way on every run.
            let mut extensions = ash_window::enumerate_required_extensions(display_handle)
                .map_err(|result| VulkanError::Call("enumerate surface extensions", result))?
                .to_vec();
            extensions.retain(|name| !name.is_null());

            let application_name = CString::new("Minecraft Java Edition").expect("static name");
            let engine_name = CString::new("MinecraftJE").expect("static name");
            let application_info = vk::ApplicationInfo::default()
                .application_name(&application_name)
                .engine_name(&engine_name)
                .engine_version(0)
                .api_version(API_VERSION);
            let instance_info = vk::InstanceCreateInfo::default()
                .application_info(&application_info)
                .enabled_extension_names(&extensions);
            let instance = entry
                .create_instance(&instance_info, None)
                .map_err(|result| VulkanError::Call("vkCreateInstance", result))?;

            let surface_instance = ash::khr::surface::Instance::new(&entry, &instance);
            let surface =
                ash_window::create_surface(&entry, &instance, display_handle, window_handle, None)
                    .map_err(|result| VulkanError::Call("vkCreateSurfaceKHR", result))?;

            let (physical_device, queue_family, properties) =
                select_physical_device(&instance, &surface_instance, surface)?;

            let queue_priorities = [1.0_f32];
            let queue_info = [vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family)
                .queue_priorities(&queue_priorities)];
            let extension_names: Vec<*const i8> = REQUIRED_DEVICE_EXTENSIONS
                .iter()
                .map(|name| name.as_ptr())
                .collect();

            // The reference enables these by name; the ones that gate rendering
            // are requested here so device selection matches what it would pick.
            let mut features_11 =
                vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);
            let mut features_12 = vk::PhysicalDeviceVulkan12Features::default()
                .timeline_semaphore(true)
                .host_query_reset(true);
            let mut features_sync2 =
                vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);
            let mut features_dynamic =
                vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);
            let mut features_divisor =
                vk::PhysicalDeviceVertexAttributeDivisorFeaturesEXT::default()
                    .vertex_attribute_instance_rate_divisor(true);
            let features_10 = vk::PhysicalDeviceFeatures::default()
                .multi_draw_indirect(true)
                .fill_mode_non_solid(true)
                .sampler_anisotropy(true);

            let device_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&queue_info)
                .enabled_extension_names(&extension_names)
                .enabled_features(&features_10)
                .push_next(&mut features_11)
                .push_next(&mut features_12)
                .push_next(&mut features_sync2)
                .push_next(&mut features_dynamic)
                .push_next(&mut features_divisor);
            let device = instance
                .create_device(physical_device, &device_info, None)
                .map_err(|result| VulkanError::Call("vkCreateDevice", result))?;
            let queue = device.get_device_queue(queue_family, 0);

            let surface_format = pick_surface_format(&surface_instance, physical_device, surface)?;
            let capabilities = surface_instance
                .get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|result| {
                    VulkanError::Call("vkGetPhysicalDeviceSurfaceCapabilities", result)
                })?;
            let extent = clamp_extent(&capabilities, width, height);

            let swapchain_device = ash::khr::swapchain::Device::new(&instance, &device);
            let dynamic_rendering = ash::khr::dynamic_rendering::Device::new(&instance, &device);
            let swapchain_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(REQUESTED_IMAGE_COUNT.max(capabilities.min_image_count))
                .image_format(surface_format)
                .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
                )
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
                .clipped(true);
            let swapchain = swapchain_device
                .create_swapchain(&swapchain_info, None)
                .map_err(|result| VulkanError::Call("vkCreateSwapchainKHR", result))?;
            let images = swapchain_device
                .get_swapchain_images(swapchain)
                .map_err(|result| VulkanError::Call("vkGetSwapchainImagesKHR", result))?;

            let mut image_views = Vec::with_capacity(images.len());
            for image in &images {
                image_views.push(
                    device
                        .create_image_view(
                            &vk::ImageViewCreateInfo::default()
                                .image(*image)
                                .view_type(vk::ImageViewType::TYPE_2D)
                                .format(surface_format)
                                .subresource_range(
                                    vk::ImageSubresourceRange::default()
                                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                                        .level_count(1)
                                        .layer_count(1),
                                ),
                            None,
                        )
                        .map_err(|result| VulkanError::Call("vkCreateImageView", result))?,
                );
            }

            let command_pool = device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .queue_family_index(queue_family)
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateCommandPool", result))?;
            let command_buffer = device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1),
                )
                .map_err(|result| VulkanError::Call("vkAllocateCommandBuffers", result))?[0];

            let byte_count = u64::from(extent.width) * u64::from(extent.height) * 4;
            let readback_buffer = device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(byte_count)
                        .usage(vk::BufferUsageFlags::TRANSFER_DST)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let requirements = device.get_buffer_memory_requirements(readback_buffer);
            let memory_properties = instance.get_physical_device_memory_properties(physical_device);
            let memory_type = (0..memory_properties.memory_type_count)
                .find(|index| {
                    let supported = requirements.memory_type_bits & (1 << index) != 0;
                    let flags = memory_properties.memory_types[*index as usize].property_flags;
                    supported
                        && flags.contains(
                            vk::MemoryPropertyFlags::HOST_VISIBLE
                                | vk::MemoryPropertyFlags::HOST_COHERENT,
                        )
                })
                .ok_or(VulkanError::NoSuitableDevice)?;
            let readback_memory = device
                .allocate_memory(
                    &vk::MemoryAllocateInfo::default()
                        .allocation_size(requirements.size)
                        .memory_type_index(memory_type),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkAllocateMemory", result))?;
            device
                .bind_buffer_memory(readback_buffer, readback_memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;

            let image_available = device
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?;
            // Present waits are per image: reusing one semaphore across images
            // is what trips validation on a FIFO swapchain.
            let mut render_finished = Vec::with_capacity(images.len());
            for _ in 0..images.len() {
                render_finished.push(
                    device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                        .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?,
                );
            }
            let in_flight = device
                .create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateFence", result))?;

            let device_name = CStr::from_ptr(properties.device_name.as_ptr())
                .to_string_lossy()
                .into_owned();
            let description = BackendDescription {
                device_name,
                discrete: properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU,
                api_version: properties.api_version,
                surface_format,
                image_count: images.len() as u32,
                extent,
            };

            Ok(Self {
                _entry: entry,
                instance,
                surface_instance,
                surface,
                physical_device,
                device,
                queue,
                swapchain_device,
                dynamic_rendering,
                swapchain,
                images,
                image_views,
                command_pool,
                command_buffer,
                readback_buffer,
                readback_memory,
                depth: None,
                scene: None,
                image_available,
                render_finished,
                in_flight,
                description,
            })
        }
    }

    /// Allocate a buffer and fill it from host memory.
    unsafe fn upload_buffer(
        &self,
        usage: vk::BufferUsageFlags,
        bytes: &[u8],
    ) -> Result<(vk::Buffer, vk::DeviceMemory), VulkanError> {
        unsafe {
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(bytes.len() as u64)
                        .usage(usage)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let requirements = self.device.get_buffer_memory_requirements(buffer);
            let memory = self.allocate(
                requirements,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;
            let mapped = self
                .device
                .map_memory(memory, 0, bytes.len() as u64, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped, bytes.len());
            self.device.unmap_memory(memory);
            Ok((buffer, memory))
        }
    }

    /// Find a memory type matching the requirements and properties.
    unsafe fn allocate(
        &self,
        requirements: vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<vk::DeviceMemory, VulkanError> {
        unsafe {
            let available = self
                .instance
                .get_physical_device_memory_properties(self.physical_device);
            let index = (0..available.memory_type_count)
                .find(|index| {
                    requirements.memory_type_bits & (1 << index) != 0
                        && available.memory_types[*index as usize]
                            .property_flags
                            .contains(properties)
                })
                .ok_or(VulkanError::NoSuitableDevice)?;
            self.device
                .allocate_memory(
                    &vk::MemoryAllocateInfo::default()
                        .allocation_size(requirements.size)
                        .memory_type_index(index),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkAllocateMemory", result))
        }
    }

    /// Run one command buffer to completion, for setup work.
    unsafe fn run_once(&self, record: impl FnOnce(vk::CommandBuffer)) -> Result<(), VulkanError> {
        unsafe {
            let buffer = self
                .device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(self.command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1),
                )
                .map_err(|result| VulkanError::Call("vkAllocateCommandBuffers", result))?[0];
            self.device
                .begin_command_buffer(
                    buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|result| VulkanError::Call("vkBeginCommandBuffer", result))?;
            record(buffer);
            self.device
                .end_command_buffer(buffer)
                .map_err(|result| VulkanError::Call("vkEndCommandBuffer", result))?;
            let buffers = [buffer];
            let submit = vk::SubmitInfo::default().command_buffers(&buffers);
            self.device
                .queue_submit(self.queue, &[submit], vk::Fence::null())
                .map_err(|result| VulkanError::Call("vkQueueSubmit", result))?;
            self.device
                .queue_wait_idle(self.queue)
                .map_err(|result| VulkanError::Call("vkQueueWaitIdle", result))?;
            self.device
                .free_command_buffers(self.command_pool, &buffers);
            Ok(())
        }
    }

    /// Record the draw for one swapchain image.
    unsafe fn record_scene(&self, index: usize, view_projection: [[f32; 4]; 4]) {
        unsafe {
            let (Some(scene), Some(depth)) = (self.scene.as_ref(), self.depth.as_ref()) else {
                return;
            };
            let colour_attachment = [vk::RenderingAttachmentInfo::default()
                .image_view(self.image_views[index])
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                // The clear already happened, so the colour target is loaded.
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_view(depth.view)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                });
            let rendering = vk::RenderingInfo::default()
                .render_area(vk::Rect2D::default().extent(self.description.extent))
                .layer_count(1)
                .color_attachments(&colour_attachment)
                .depth_attachment(&depth_attachment);
            self.dynamic_rendering
                .cmd_begin_rendering(self.command_buffer, &rendering);
            self.device.cmd_bind_pipeline(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                scene.pipeline,
            );
            self.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                scene.pipeline_layout,
                0,
                &[scene.descriptor_set],
                &[],
            );
            let matrix: [f32; 16] = std::array::from_fn(|i| view_projection[i / 4][i % 4]);
            self.device.cmd_push_constants(
                self.command_buffer,
                scene.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                std::slice::from_raw_parts(matrix.as_ptr().cast::<u8>(), 64),
            );
            self.device.cmd_bind_vertex_buffers(
                self.command_buffer,
                0,
                &[scene.vertex_buffer],
                &[0],
            );
            self.device.cmd_bind_index_buffer(
                self.command_buffer,
                scene.index_buffer,
                0,
                vk::IndexType::UINT32,
            );
            self.device
                .cmd_draw_indexed(self.command_buffer, scene.index_count, 1, 0, 0, 0);
            self.dynamic_rendering
                .cmd_end_rendering(self.command_buffer);
        }
    }

    /// Compile the shaders and build the graphics pipeline.
    ///
    /// Dynamic rendering is used rather than a render pass object, which is
    /// the extension the reference already requires of every device it picks.
    unsafe fn create_pipeline(
        &self,
        descriptor_layout: vk::DescriptorSetLayout,
    ) -> Result<(vk::PipelineLayout, vk::Pipeline), VulkanError> {
        unsafe {
            let compiler =
                shaderc::Compiler::new().map_err(|error| VulkanError::Loader(error.to_string()))?;
            let compile = |source: &str, kind, name: &str| -> Result<Vec<u32>, VulkanError> {
                compiler
                    .compile_into_spirv(source, kind, name, "main", None)
                    .map(|artifact| artifact.as_binary().to_vec())
                    .map_err(|error| VulkanError::Loader(error.to_string()))
            };
            let vertex_code = compile(VERTEX_SHADER, shaderc::ShaderKind::Vertex, "cube.vert")?;
            let fragment_code =
                compile(FRAGMENT_SHADER, shaderc::ShaderKind::Fragment, "cube.frag")?;
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

            let bindings = [vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(std::mem::size_of::<Vertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)];
            let attributes = [
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
            ];
            let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(&bindings)
                .vertex_attribute_descriptions(&attributes);
            let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
            let viewports = [vk::Viewport::default()
                .width(self.description.extent.width as f32)
                .height(self.description.extent.height as f32)
                .max_depth(1.0)];
            let scissors = [vk::Rect2D::default().extent(self.description.extent)];
            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewports(&viewports)
                .scissors(&scissors);
            let raster = vk::PipelineRasterizationStateCreateInfo::default()
                .polygon_mode(vk::PolygonMode::FILL)
                .cull_mode(vk::CullModeFlags::BACK)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .line_width(1.0);
            let multisample = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let depth_state = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS);
            let blends = [vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)];
            let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blends);

            let push_ranges = [vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .size(64)];
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
                .rasterization_state(&raster)
                .multisample_state(&multisample)
                .depth_stencil_state(&depth_state)
                .color_blend_state(&blend)
                .layout(pipeline_layout)
                .push_next(&mut rendering);
            let pipeline = self
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create], None)
                .map_err(|(_, result)| VulkanError::Call("vkCreateGraphicsPipelines", result))?[0];

            self.device.destroy_shader_module(vertex_module, None);
            self.device.destroy_shader_module(fragment_module, None);
            Ok((pipeline_layout, pipeline))
        }
    }

    /// Build the depth target, texture, buffers and pipeline for one mesh.
    ///
    /// Shaders are compiled here with shaderc, which is the library the
    /// reference's Vulkan backend uses; the GLSL itself is this project's own.
    pub fn load_scene(
        &mut self,
        mesh: &Mesh,
        texture_levels: &[DecodedImage],
    ) -> Result<(), VulkanError> {
        unsafe {
            self.destroy_scene();
            self.depth = Some(self.create_depth_target()?);

            let vertex_bytes = std::slice::from_raw_parts(
                mesh.vertices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(mesh.vertices.as_slice()),
            );
            let (vertex_buffer, vertex_memory) =
                self.upload_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, vertex_bytes)?;
            let index_bytes = std::slice::from_raw_parts(
                mesh.indices.as_ptr().cast::<u8>(),
                std::mem::size_of_val(mesh.indices.as_slice()),
            );
            let (index_buffer, index_memory) =
                self.upload_buffer(vk::BufferUsageFlags::INDEX_BUFFER, index_bytes)?;

            let (texture, texture_memory, texture_view) = self.upload_texture(texture_levels)?;
            // These are the reference's own values for the terrain sampler,
            // read from `LevelRenderer`'s chunk layer sampler and from the
            // Vulkan sampler it builds. Both filters are linear, which reads
            // as a surprise for a game of pixel art but is consistent with the
            // atlas reserving padding around every sprite: with nearest
            // sampling that padding would have nothing to do. At a one to one
            // texel ratio linear sampling lands on the texel centre anyway, so
            // the blocky look survives; it softens only under magnification.
            let sampler = self
                .device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::LINEAR)
                        .min_filter(vk::Filter::LINEAR)
                        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        // The reference leaves the third axis at the default,
                        // which is repeat, and never samples it.
                        .address_mode_w(vk::SamplerAddressMode::REPEAT)
                        .mip_lod_bias(0.0)
                        .min_lod(0.0)
                        // An unbounded maximum, which the reference spells as
                        // one thousand.
                        .max_lod(1000.0)
                        .anisotropy_enable(false)
                        .max_anisotropy(1.0),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateSampler", result))?;

            let bindings = [vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
            let descriptor_layout = self
                .device
                .create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateDescriptorSetLayout", result))?;
            let sizes = [vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)];
            let descriptor_pool = self
                .device
                .create_descriptor_pool(
                    &vk::DescriptorPoolCreateInfo::default()
                        .max_sets(1)
                        .pool_sizes(&sizes),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateDescriptorPool", result))?;
            let layouts = [descriptor_layout];
            let descriptor_set = self
                .device
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(descriptor_pool)
                        .set_layouts(&layouts),
                )
                .map_err(|result| VulkanError::Call("vkAllocateDescriptorSets", result))?[0];
            let image_info = [vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_view)
                .sampler(sampler)];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_info);
            self.device.update_descriptor_sets(&[write], &[]);

            let (pipeline_layout, pipeline) = self.create_pipeline(descriptor_layout)?;

            self.scene = Some(SceneResources {
                vertex_buffer,
                vertex_memory,
                index_buffer,
                index_memory,
                index_count: mesh.indices.len() as u32,
                texture,
                texture_memory,
                texture_view,
                sampler,
                descriptor_pool,
                descriptor_layout,
                descriptor_set,
                pipeline_layout,
                pipeline,
            });
            Ok(())
        }
    }

    unsafe fn create_depth_target(&self) -> Result<DepthTarget, VulkanError> {
        unsafe {
            let format = vk::Format::D32_SFLOAT;
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: self.description.extent.width,
                            height: self.description.extent.height,
                            depth: 1,
                        })
                        .mip_levels(1)
                        .array_layers(1)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .tiling(vk::ImageTiling::OPTIMAL)
                        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
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
            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format)
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .level_count(1)
                                .layer_count(1),
                        ),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            Ok(DepthTarget {
                image,
                memory,
                view,
                format,
            })
        }
    }

    unsafe fn upload_texture(
        &self,
        levels: &[DecodedImage],
    ) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), VulkanError> {
        unsafe {
            let base = levels.first().ok_or(VulkanError::NoAtlasLevels)?;
            let mut staged = Vec::new();
            let mut offsets = Vec::with_capacity(levels.len());
            for level in levels {
                offsets.push(staged.len() as u64);
                staged.extend_from_slice(&level.rgba);
            }
            let (staging, staging_memory) =
                self.upload_buffer(vk::BufferUsageFlags::TRANSFER_SRC, &staged)?;
            let format = vk::Format::R8G8B8A8_UNORM;
            let image = self
                .device
                .create_image(
                    &vk::ImageCreateInfo::default()
                        .image_type(vk::ImageType::TYPE_2D)
                        .format(format)
                        .extent(vk::Extent3D {
                            width: base.width,
                            height: base.height,
                            depth: 1,
                        })
                        .mip_levels(levels.len() as u32)
                        .array_layers(1)
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
                .level_count(levels.len() as u32)
                .layer_count(1);
            // Every level is already built on the processor, so nothing is
            // blitted here: each one is its own copy out of the staging buffer.
            let regions: Vec<vk::BufferImageCopy> = levels
                .iter()
                .zip(&offsets)
                .enumerate()
                .map(|(level, (image, offset))| {
                    vk::BufferImageCopy::default()
                        .buffer_offset(*offset)
                        .image_subresource(
                            vk::ImageSubresourceLayers::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .mip_level(level as u32)
                                .layer_count(1),
                        )
                        .image_extent(vk::Extent3D {
                            width: image.width,
                            height: image.height,
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
                    staging,
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
            self.device.destroy_buffer(staging, None);
            self.device.free_memory(staging_memory, None);

            let view = self
                .device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format)
                        .subresource_range(range),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateImageView", result))?;
            Ok((image, memory, view))
        }
    }

    unsafe fn destroy_scene(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            if let Some(scene) = self.scene.take() {
                self.device.destroy_pipeline(scene.pipeline, None);
                self.device
                    .destroy_pipeline_layout(scene.pipeline_layout, None);
                self.device
                    .destroy_descriptor_pool(scene.descriptor_pool, None);
                self.device
                    .destroy_descriptor_set_layout(scene.descriptor_layout, None);
                self.device.destroy_sampler(scene.sampler, None);
                self.device.destroy_image_view(scene.texture_view, None);
                self.device.destroy_image(scene.texture, None);
                self.device.free_memory(scene.texture_memory, None);
                self.device.destroy_buffer(scene.index_buffer, None);
                self.device.free_memory(scene.index_memory, None);
                self.device.destroy_buffer(scene.vertex_buffer, None);
                self.device.free_memory(scene.vertex_memory, None);
            }
            if let Some(depth) = self.depth.take() {
                self.device.destroy_image_view(depth.view, None);
                self.device.destroy_image(depth.image, None);
                self.device.free_memory(depth.memory, None);
            }
        }
    }

    /// What the backend selected.
    pub fn description(&self) -> &BackendDescription {
        &self.description
    }

    /// Acquire, clear to one colour, and present a single frame.
    ///
    /// The colour is a placeholder: no reference pixel has been compared yet.
    pub fn draw_clear_frame(&mut self, colour: [f32; 4]) -> Result<(), VulkanError> {
        self.render_frame(colour, false, IDENTITY).map(|_| ())
    }

    /// Draw the loaded scene under one view-projection matrix.
    pub fn draw_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<(), VulkanError> {
        self.render_frame(colour, false, view_projection)
            .map(|_| ())
    }

    /// Draw the loaded scene and read the presented image back.
    pub fn capture_scene(
        &mut self,
        colour: [f32; 4],
        view_projection: [[f32; 4]; 4],
    ) -> Result<CapturedFrame, VulkanError> {
        let pixels = self.render_frame(colour, true, view_projection)?;
        Ok(CapturedFrame {
            width: self.description.extent.width,
            height: self.description.extent.height,
            rgba: pixels.unwrap_or_default(),
        })
    }

    /// Render one frame and read the presented image back as RGBA8 rows.
    ///
    /// Readback is the same path frame comparison will need later, so it copies
    /// the image the compositor receives rather than re-rendering into an
    /// offscreen target.
    pub fn capture_frame(&mut self, colour: [f32; 4]) -> Result<CapturedFrame, VulkanError> {
        let pixels = self.render_frame(colour, true, IDENTITY)?;
        Ok(CapturedFrame {
            width: self.description.extent.width,
            height: self.description.extent.height,
            rgba: pixels.unwrap_or_default(),
        })
    }

    fn render_frame(
        &mut self,
        colour: [f32; 4],
        capture: bool,
        view_projection: [[f32; 4]; 4],
    ) -> Result<Option<Vec<u8>>, VulkanError> {
        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;

            let (index, _) = match self.swapchain_device.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available,
                vk::Fence::null(),
            ) {
                Ok(value) => value,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return Ok(None),
                Err(result) => return Err(VulkanError::Call("vkAcquireNextImageKHR", result)),
            };
            self.device
                .reset_fences(&[self.in_flight])
                .map_err(|result| VulkanError::Call("vkResetFences", result))?;

            let image = self.images[index as usize];
            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(1);

            self.device
                .begin_command_buffer(
                    self.command_buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|result| VulkanError::Call("vkBeginCommandBuffer", result))?;

            transition(
                &self.device,
                self.command_buffer,
                image,
                range,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            );
            self.device.cmd_clear_color_image(
                self.command_buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearColorValue { float32: colour },
                &[range],
            );

            if self.scene.is_some() {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                );
                self.record_scene(index as usize, view_projection);
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                );
            }
            if capture {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                );
                let region = vk::BufferImageCopy::default()
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .image_extent(vk::Extent3D {
                        width: self.description.extent.width,
                        height: self.description.extent.height,
                        depth: 1,
                    });
                self.device.cmd_copy_image_to_buffer(
                    self.command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    self.readback_buffer,
                    &[region],
                );
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
            } else {
                transition(
                    &self.device,
                    self.command_buffer,
                    image,
                    range,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::PRESENT_SRC_KHR,
                );
            }

            self.device
                .end_command_buffer(self.command_buffer)
                .map_err(|result| VulkanError::Call("vkEndCommandBuffer", result))?;

            let wait_semaphores = [self.image_available];
            let wait_stages = [vk::PipelineStageFlags::TRANSFER];
            let command_buffers = [self.command_buffer];
            let signal_semaphores = [self.render_finished[index as usize]];
            let submit = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
            self.device
                .queue_submit(self.queue, &[submit], self.in_flight)
                .map_err(|result| VulkanError::Call("vkQueueSubmit", result))?;

            let swapchains = [self.swapchain];
            let indices = [index];
            let present = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);
            match self.swapchain_device.queue_present(self.queue, &present) {
                Ok(_)
                | Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
                | Err(vk::Result::SUBOPTIMAL_KHR) => {}
                Err(result) => return Err(VulkanError::Call("vkQueuePresentKHR", result)),
            }

            if !capture {
                return Ok(None);
            }
            self.device
                .wait_for_fences(&[self.in_flight], true, u64::MAX)
                .map_err(|result| VulkanError::Call("vkWaitForFences", result))?;
            let byte_count =
                (self.description.extent.width * self.description.extent.height * 4) as usize;
            let mapped = self
                .device
                .map_memory(
                    self.readback_memory,
                    0,
                    byte_count as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            let mut rgba = vec![0_u8; byte_count];
            std::ptr::copy_nonoverlapping(mapped, rgba.as_mut_ptr(), byte_count);
            self.device.unmap_memory(self.readback_memory);
            if self.description.surface_format == vk::Format::B8G8R8A8_UNORM {
                for pixel in rgba.as_chunks_mut::<4>().0 {
                    pixel.swap(0, 2);
                }
            }
            Ok(Some(rgba))
        }
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            self.destroy_scene();
            for view in self.image_views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            for semaphore in self.render_finished.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.device.destroy_semaphore(self.image_available, None);
            self.device.destroy_fence(self.in_flight, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_buffer(self.readback_buffer, None);
            self.device.free_memory(self.readback_memory, None);
            self.swapchain_device
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_instance.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

/// Insert the layout transition the clear and the present each need.
unsafe fn transition(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    range: vk::ImageSubresourceRange,
    from: vk::ImageLayout,
    to: vk::ImageLayout,
) {
    unsafe {
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(from)
            .new_layout(to)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(range)
            // Scaffolding barrier: make every prior write available and every
            // later read see it. Once the real pipeline exists these become the
            // specific stage/access pairs each pass needs.
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE);
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }
}

/// Pick a device the way the reference does: skip anything that cannot meet the
/// required extensions or the minimum API version, take the first that can, and
/// upgrade to a discrete GPU if a later one turns out to be discrete.
unsafe fn select_physical_device(
    instance: &ash::Instance,
    surface_instance: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32, vk::PhysicalDeviceProperties), VulkanError> {
    unsafe {
        let devices = instance
            .enumerate_physical_devices()
            .map_err(|result| VulkanError::Call("vkEnumeratePhysicalDevices", result))?;
        let mut selected: Option<(vk::PhysicalDevice, u32, vk::PhysicalDeviceProperties)> = None;

        for device in devices {
            let properties = instance.get_physical_device_properties(device);
            if properties.api_version < MINIMUM_DEVICE_API_VERSION {
                continue;
            }
            let Ok(extensions) = instance.enumerate_device_extension_properties(device) else {
                continue;
            };
            let has_all = REQUIRED_DEVICE_EXTENSIONS.iter().all(|required| {
                extensions
                    .iter()
                    .any(|available| CStr::from_ptr(available.extension_name.as_ptr()) == *required)
            });
            if !has_all {
                continue;
            }
            let Some(queue_family) =
                graphics_present_family(instance, surface_instance, surface, device)
            else {
                continue;
            };

            let discrete = properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU;
            match &selected {
                None => selected = Some((device, queue_family, properties)),
                Some((_, _, chosen)) => {
                    if discrete && chosen.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
                        selected = Some((device, queue_family, properties));
                    }
                }
            }
        }

        selected.ok_or(VulkanError::NoSuitableDevice)
    }
}

/// First queue family that can both render and present to this surface.
unsafe fn graphics_present_family(
    instance: &ash::Instance,
    surface_instance: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    device: vk::PhysicalDevice,
) -> Option<u32> {
    unsafe {
        instance
            .get_physical_device_queue_family_properties(device)
            .iter()
            .enumerate()
            .find_map(|(index, family)| {
                let index = index as u32;
                let graphics = family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                let present = surface_instance
                    .get_physical_device_surface_support(device, index, surface)
                    .unwrap_or(false);
                (graphics && present).then_some(index)
            })
    }
}

/// The reference accepts only these two formats, and only in sRGB non-linear.
unsafe fn pick_surface_format(
    surface_instance: &ash::khr::surface::Instance,
    device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> Result<vk::Format, VulkanError> {
    unsafe {
        let formats = surface_instance
            .get_physical_device_surface_formats(device, surface)
            .map_err(|result| VulkanError::Call("vkGetPhysicalDeviceSurfaceFormats", result))?;
        formats
            .iter()
            .find(|format| {
                format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    && ACCEPTED_SURFACE_FORMATS.contains(&format.format)
            })
            .map(|format| format.format)
            .ok_or(VulkanError::NoSuitableSurfaceFormat)
    }
}

/// Keep the requested size inside what the surface reports it can present.
fn clamp_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    width: u32,
    height: u32,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }
    vk::Extent2D {
        width: width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ),
        height: height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ),
    }
}
