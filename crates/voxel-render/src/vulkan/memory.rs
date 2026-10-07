//! GPU memory: host buffers, the section arena, staging chunks and the screenshot readback.

use super::*;

impl VulkanBackend {
    /// A buffer of `size` bytes bound to memory with `properties`. Nothing leaks if a step fails.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(super) unsafe fn bound_buffer(
        &self,
        size: u64,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<BoundBuffer, VulkanError> {
        // SAFETY: live handles; the buffer is owned by the returned value from its creation on
        // (a null memory frees as a no-op), and its memory is made from its own requirements.
        unsafe {
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(size)
                        .usage(usage)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let mut bound = BoundBuffer {
                device: self.device.clone(),
                buffer,
                memory: vk::DeviceMemory::null(),
            };
            let requirements = self.device.get_buffer_memory_requirements(buffer);
            bound.memory = self.allocate(requirements, properties)?;
            self.device
                .bind_buffer_memory(buffer, bound.memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;
            Ok(bound)
        }
    }

    /// Allocate a buffer and fill it from host memory.
    ///
    /// # Safety
    ///
    /// `bytes` must not be empty; an empty range cannot be mapped.
    pub(super) unsafe fn upload_buffer(
        &self,
        usage: vk::BufferUsageFlags,
        bytes: &[u8],
    ) -> Result<BoundBuffer, VulkanError> {
        // SAFETY: live handles. The buffer is bound to host-visible, coherent memory sized from
        // its own requirements, and `bytes` is not empty, so the mapping covers the `bytes.len()`
        // bytes the copy fills before the unmap.
        unsafe {
            let bound = self.bound_buffer(
                bytes.len() as u64,
                usage,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            let mapped = self
                .device
                .map_memory(
                    bound.memory,
                    0,
                    bytes.len() as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped, bytes.len());
            self.device.unmap_memory(bound.memory);
            Ok(bound)
        }
    }

    /// The buffer a screenshot of the current extent is copied into, made
    /// when one is taken and freed once it has been read.
    ///
    /// # Safety
    ///
    /// The GPU must not be using the old readback buffer; the caller has waited for the frames that may copy into it.
    pub(super) unsafe fn make_readback(&mut self) -> Result<(), VulkanError> {
        // SAFETY: live handles; the old buffer is dropped before the new one is stored.
        unsafe {
            self.readback = None;
            let extent = self.description.extent;
            self.readback = Some(self.bound_buffer(
                u64::from(extent.width) * u64::from(extent.height) * 4,
                vk::BufferUsageFlags::TRANSFER_DST,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?);
            Ok(())
        }
    }

    /// Allocate device memory of a type matching the requirements and properties.
    ///
    /// # Safety
    ///
    /// Nothing beyond the live handles.
    pub(super) unsafe fn allocate(
        &self,
        requirements: vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<vk::DeviceMemory, VulkanError> {
        // SAFETY: live handles; the memory type index comes from this physical device's own
        // properties.
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

    /// Give every block that holds nothing back to the driver but one, which
    /// stays for the next meshes.
    ///
    /// # Safety
    ///
    /// The GPU must be done with every empty block.
    pub(super) unsafe fn trim_arena(&mut self) {
        // SAFETY: a block is dropped only when its free list is the whole block, so no section
        // uses it; the caller guarantees the GPU is done with it.
        let mut spare = false;
        for slot in &mut self.arena {
            let empty = slot
                .as_ref()
                .is_some_and(|block| block.free.as_slice() == [(0, block.size)]);
            if !empty {
                continue;
            }
            if !spare {
                spare = true;
                continue;
            }
            *slot = None;
        }
    }

    /// Give the staging chunks past the spare ones back to the driver; none
    /// is in use when a frame starts.
    ///
    /// # Safety
    ///
    /// No chunk may be in flight past `staging_in_flight`: call it once the finished frames' chunks
    /// have been dropped from it.
    pub(super) unsafe fn trim_staging(&mut self) {
        // SAFETY: only chunks outside `staging_filling` and `staging_in_flight` are dropped, so no
        // frame copies from them.
        let mut idle = 0;
        for (index, slot) in self.staging.iter_mut().enumerate() {
            if slot.is_none()
                || self.staging_filling.contains(&index)
                || self
                    .staging_in_flight
                    .iter()
                    .any(|&(_, busy)| busy == index)
            {
                continue;
            }
            idle += 1;
            if idle <= STAGING_SPARE {
                continue;
            }
            *slot = None;
        }
    }

    /// Copy bytes into a free range of the arena, growing it by a block
    /// when none fits.
    ///
    /// # Safety
    ///
    /// Only the live handles; the copy is recorded later by `record_staged_copies`.
    pub(super) unsafe fn arena_upload(&mut self, bytes: &[u8]) -> Result<ArenaSlice, VulkanError> {
        // SAFETY: live handles. The destination range is reserved from the arena's free list, and
        // the copy stays inside the staging chunk: `size` is at least `bytes.len()` and the chunk
        // had `size` bytes free at `used`. Chunks are mapped for good.
        unsafe {
            let size = (bytes.len() as u64).div_ceil(16) * 16;
            let found = self.arena.iter().enumerate().find_map(|(block, arena)| {
                arena
                    .as_ref()?
                    .free
                    .iter()
                    .position(|&(_, length)| length >= size)
                    .map(|range| (block, range))
            });
            let (block, range) = match found {
                Some(found) => found,
                None => {
                    let block = self.create_arena_block(size.max(ARENA_BLOCK_SIZE))?;
                    let given_back = self.arena.iter().position(Option::is_none);
                    match given_back {
                        Some(slot) => {
                            self.arena[slot] = Some(block);
                            (slot, 0)
                        }
                        None => {
                            self.arena.push(Some(block));
                            (self.arena.len() - 1, 0)
                        }
                    }
                }
            };
            let arena = self.arena[block].as_mut().expect("found or made above");
            let (offset, length) = arena.free[range];
            if length == size {
                arena.free.remove(range);
            } else {
                arena.free[range] = (offset + size, length - size);
            }
            let target = ArenaSlice {
                block,
                offset,
                size,
            };
            let chunk = match self.staging_filling.iter().copied().find(|&chunk| {
                let held = self.staging[chunk]
                    .as_ref()
                    .expect("filling chunks are live");
                held.size - held.used >= size
            }) {
                Some(chunk) => chunk,
                None => {
                    let free = (0..self.staging.len()).find(|chunk| {
                        !self.staging_filling.contains(chunk)
                            && !self
                                .staging_in_flight
                                .iter()
                                .any(|&(_, busy)| busy == *chunk)
                            && self.staging[*chunk]
                                .as_ref()
                                .is_some_and(|held| held.size >= size)
                    });
                    let chunk = match free {
                        Some(chunk) => chunk,
                        None => {
                            let made = self.create_staging_chunk(size.max(STAGING_CHUNK_SIZE))?;
                            let given_back = self.staging.iter().position(Option::is_none);
                            match given_back {
                                Some(slot) => {
                                    self.staging[slot] = Some(made);
                                    slot
                                }
                                None => {
                                    self.staging.push(Some(made));
                                    self.staging.len() - 1
                                }
                            }
                        }
                    };
                    self.staging[chunk]
                        .as_mut()
                        .expect("chosen or made above")
                        .used = 0;
                    self.staging_filling.push(chunk);
                    chunk
                }
            };
            let staging = self.staging[chunk].as_mut().expect("chosen or made above");
            let source = staging.used;
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                staging.mapped.add(source as usize),
                bytes.len(),
            );
            staging.used += size;
            self.staged.push(StagedCopy {
                chunk,
                source,
                target,
                size,
            });
            Ok(target)
        }
    }

    pub(super) fn arena_free(&mut self, slice: ArenaSlice) {
        let free = &mut self.arena[slice.block]
            .as_mut()
            .expect("a section's block is not given back")
            .free;
        let at = free.partition_point(|&(offset, _)| offset < slice.offset);
        free.insert(at, (slice.offset, slice.size));
        // Join with the ranges on either side.
        if at + 1 < free.len() && free[at].0 + free[at].1 == free[at + 1].0 {
            free[at].1 += free[at + 1].1;
            free.remove(at + 1);
        }
        if at > 0 && free[at - 1].0 + free[at - 1].1 == free[at].0 {
            free[at - 1].1 += free[at].1;
            free.remove(at);
        }
    }

    /// A new arena block, in the GPU's own memory.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(super) unsafe fn create_arena_block(&self, size: u64) -> Result<ArenaBlock, VulkanError> {
        // SAFETY: live handles; the buffer is bound to memory made from its own requirements.
        unsafe {
            Ok(ArenaBlock {
                buffer: self.bound_buffer(
                    size,
                    vk::BufferUsageFlags::VERTEX_BUFFER
                        | vk::BufferUsageFlags::INDEX_BUFFER
                        | vk::BufferUsageFlags::TRANSFER_DST,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )?,
                size,
                free: vec![(0, size)],
            })
        }
    }

    /// A new staging chunk, mapped for good.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(super) unsafe fn create_staging_chunk(
        &self,
        size: u64,
    ) -> Result<StagingChunk, VulkanError> {
        // SAFETY: live handles; the memory is host-visible and coherent, and mapped over the
        // chunk's whole size.
        unsafe {
            let buffer = self.bound_buffer(
                size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            let mapped = self
                .device
                .map_memory(buffer.memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            Ok(StagingChunk {
                buffer,
                mapped,
                size,
                used: 0,
            })
        }
    }

    /// Record the staged section copies, and make the draws wait for them.
    ///
    /// # Safety
    ///
    /// `command_buffer` must be recording, outside a render pass.
    pub(super) unsafe fn record_staged_copies(&mut self) {
        // SAFETY: recording, outside a render pass. Each copy names a staging chunk held in
        // `staging_filling` until now and an arena range from `arena_upload`; the range's section
        // is retired only with its copy dropped (`free_retired`), so the range is still its own.
        unsafe {
            if self.staged.is_empty() {
                return;
            }
            for copy in std::mem::take(&mut self.staged) {
                self.device.cmd_copy_buffer(
                    self.command_buffer,
                    self.staging[copy.chunk]
                        .as_ref()
                        .expect("a copy's chunk is live")
                        .buffer
                        .buffer,
                    self.arena[copy.target.block]
                        .as_ref()
                        .expect("a copy's block is live")
                        .buffer
                        .buffer,
                    &[vk::BufferCopy {
                        src_offset: copy.source,
                        dst_offset: copy.target.offset,
                        size: copy.size,
                    }],
                );
            }
            self.device.cmd_pipeline_barrier(
                self.command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::VERTEX_INPUT,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(
                        vk::AccessFlags::VERTEX_ATTRIBUTE_READ | vk::AccessFlags::INDEX_READ,
                    )],
                &[],
                &[],
            );
            // These chunks are copied from until the frame being recorded has finished.
            let frame = self.frames_submitted + 1;
            self.staging_in_flight.extend(
                std::mem::take(&mut self.staging_filling)
                    .into_iter()
                    .map(|chunk| (frame, chunk)),
            );
        }
    }
}

impl HostBuffer {
    /// A zeroed buffer of `capacity` bytes, which must not be 0 (an empty range cannot be mapped).
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(super) unsafe fn new(
        backend: &VulkanBackend,
        capacity: usize,
    ) -> Result<Self, VulkanError> {
        // SAFETY: the caller's contract; `capacity` is not 0 (see above).
        let bound = unsafe {
            backend.upload_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vec![0_u8; capacity])
        }?;
        Ok(Self { bound, capacity })
    }

    /// Write the parts one after another from the start of the buffer. Nothing is mapped for no
    /// bytes.
    ///
    /// # Safety
    ///
    /// The GPU must not be reading the buffer.
    pub(super) unsafe fn write(&self, parts: &[&[u8]]) -> Result<(), VulkanError> {
        let size: usize = parts.iter().map(|part| part.len()).sum();
        if size == 0 {
            return Ok(());
        }
        assert!(
            size <= self.capacity,
            "{size} bytes in a {}-byte buffer",
            self.capacity
        );
        // SAFETY: the memory is host-visible and coherent and mapped over `size` bytes, which fit
        // the buffer (checked above); each part is copied inside that range, and the memory is
        // unmapped before the GPU may read it (see `# Safety`).
        unsafe {
            let mapped = self
                .device
                .map_memory(self.memory, 0, size as u64, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            let mut at = 0;
            for part in parts {
                std::ptr::copy_nonoverlapping(part.as_ptr(), mapped.add(at), part.len());
                at += part.len();
            }
            self.device.unmap_memory(self.memory);
        }
        Ok(())
    }
}

impl VulkanBackend {
    /// Write the parts into the buffer of `slot` that no frame in flight draws, and make it the one
    /// the next frame draws. Only the frames that may still draw that buffer are waited for, never
    /// the one drawing the other.
    ///
    /// # Safety
    ///
    /// Only the live handles.
    pub(super) unsafe fn fill_dynamic(
        &self,
        slot: &mut DynamicBuffer,
        parts: &[&[u8]],
        minimum: usize,
    ) -> Result<(), VulkanError> {
        let target = 1 - slot.active;
        let size: usize = parts.iter().map(|part| part.len()).sum();
        // SAFETY: live handles. The frames that may draw the target have finished, so it may be
        // replaced and written; it becomes the active buffer only after the write succeeded.
        unsafe {
            self.complete_through(slot.retired_at[target])?;
            self.grow_host_buffer(&mut slot.buffers[target], size, minimum)?
                .write(parts)?;
        }
        slot.retired_at[slot.active] = self.frames_submitted;
        slot.active = target;
        Ok(())
    }

    /// Make `slot` hold a buffer of at least `size` bytes, replacing a smaller one with one of at
    /// least `minimum`; the old buffer is freed first.
    ///
    /// # Safety
    ///
    /// The caller has waited for the frame in flight, so the old buffer is unused.
    pub(super) unsafe fn grow_host_buffer<'a>(
        &self,
        slot: &'a mut Option<HostBuffer>,
        size: usize,
        minimum: usize,
    ) -> Result<&'a HostBuffer, VulkanError> {
        if slot.as_ref().is_none_or(|buffer| buffer.capacity < size) {
            *slot = None;
            // SAFETY: live handles; `size.max(minimum)` is not 0 because `minimum` is not.
            *slot = Some(unsafe { HostBuffer::new(self, size.max(minimum))? });
        }
        Ok(slot.as_ref().expect("filled above"))
    }
}
