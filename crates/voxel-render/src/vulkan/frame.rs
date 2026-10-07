//! Recording and submitting one frame, and what it measures.

use super::*;

/// Insert the layout transition the clear and the present each need.
///
/// # Safety
///
/// `command_buffer` must be recording outside a render pass, and `image` must belong to `device`.
pub(super) unsafe fn transition(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    range: vk::ImageSubresourceRange,
    from: vk::ImageLayout,
    to: vk::ImageLayout,
) {
    // SAFETY: the caller's contract: `command_buffer` is recording and `image` belongs to
    // `device`.
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

mod vulkan_backend_frame;
mod vulkan_backend_record;
mod vulkan_backend_scene;
