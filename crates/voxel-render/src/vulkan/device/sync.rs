use super::*;

/// What the synchronisation stage makes: the command pool, the frames in flight, the semaphores
/// presentation waits on and the timestamp queries.
pub(super) struct SyncParts {
    pub(super) command_pool: vk::CommandPool,
    pub(super) frames: Vec<FrameSlot>,
    pub(super) render_finished: Vec<vk::Semaphore>,
    pub(super) timestamps: Option<TimestampQueries>,
}

/// The command buffers and sync objects of each frame in flight, one present semaphore for each
/// of `image_count` swapchain images, and the timestamp pool when the queue supports it.
///
/// # Safety
///
/// Both parts must be live.
pub(super) unsafe fn create_sync(
    instance_parts: &InstanceParts,
    device_parts: &DeviceParts,
    image_count: usize,
) -> Result<SyncParts, VulkanError> {
    let instance = &instance_parts.instance;
    let DeviceParts {
        physical_device,
        queue_family,
        properties,
        device,
        ..
    } = device_parts;
    let (physical_device, queue_family) = (*physical_device, *queue_family);
    // SAFETY: the caller's contract; everything is made on this device.
    unsafe {
        let command_pool = device
            .create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(queue_family)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
            .map_err(|result| VulkanError::Call("vkCreateCommandPool", result))?;
        let command_buffers = device
            .allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(FRAMES_IN_FLIGHT as u32),
            )
            .map_err(|result| VulkanError::Call("vkAllocateCommandBuffers", result))?;
        let mut frames = Vec::with_capacity(FRAMES_IN_FLIGHT);
        for command_buffer in command_buffers {
            frames.push(FrameSlot {
                command_buffer,
                in_flight: device
                    .create_fence(
                        &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .map_err(|result| VulkanError::Call("vkCreateFence", result))?,
                image_available: device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                    .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?,
                stamp: 0,
            });
        }
        // Present waits are per image: reusing one semaphore across images
        // is what trips validation on a FIFO swapchain.
        let mut render_finished = Vec::with_capacity(image_count);
        for _ in 0..image_count {
            render_finished.push(
                device
                    .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                    .map_err(|result| VulkanError::Call("vkCreateSemaphore", result))?,
            );
        }
        let valid_bits = instance.get_physical_device_queue_family_properties(physical_device)
            [queue_family as usize]
            .timestamp_valid_bits;
        let timestamps =
            if valid_bits > 0 && properties.limits.timestamp_compute_and_graphics == vk::TRUE {
                let pool = device
                    .create_query_pool(
                        &vk::QueryPoolCreateInfo::default()
                            .query_type(vk::QueryType::TIMESTAMP)
                            .query_count(TIMESTAMP_COUNT * FRAMES_IN_FLIGHT as u32),
                        None,
                    )
                    .map_err(|result| VulkanError::Call("vkCreateQueryPool", result))?;
                Some(TimestampQueries {
                    pool,
                    period_ns: f64::from(properties.limits.timestamp_period),
                    valid_bits,
                    pending: [false; FRAMES_IN_FLIGHT],
                    last: None,
                })
            } else {
                None
            };
        Ok(SyncParts {
            command_pool,
            frames,
            render_finished,
            timestamps,
        })
    }
}
