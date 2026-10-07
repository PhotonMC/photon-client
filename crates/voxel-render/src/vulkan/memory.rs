//! GPU memory: host buffers, the section arena, staging chunks and the screenshot readback.

use super::*;

impl VulkanBackend {
    /// Allocate a buffer and fill it from host memory.
    ///
    /// # Safety
    ///
    /// `bytes` must not be empty; an empty range cannot be mapped.
    pub(super) unsafe fn upload_buffer(
        &self,
        usage: vk::BufferUsageFlags,
        bytes: &[u8],
    ) -> Result<(vk::Buffer, vk::DeviceMemory), VulkanError> {
        // SAFETY: live handles. The buffer is bound to host-visible, coherent memory sized from
        // its own requirements, and `bytes` is not empty, so the mapping covers the `bytes.len()`
        // bytes the copy fills before the unmap.
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

    /// The buffer a screenshot of the current extent is copied into, made
    /// when one is taken and freed once it has been read.
    ///
    /// # Safety
    ///
    /// The GPU must not be using the old readback buffer; the caller has waited for the frames that may copy into it.
    pub(super) unsafe fn make_readback(&mut self) -> Result<(), VulkanError> {
        // SAFETY: live handles; `free_readback` runs first, so the old buffer is gone before the
        // new pair is stored.
        unsafe {
            self.free_readback();
            let extent = self.description.extent;
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(u64::from(extent.width) * u64::from(extent.height) * 4)
                        .usage(vk::BufferUsageFlags::TRANSFER_DST)
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
            self.readback = Some((buffer, memory));
            Ok(())
        }
    }

    /// Free the screenshot buffer, if there is one.
    ///
    /// # Safety
    ///
    /// The GPU must not be using the readback buffer.
    pub(super) unsafe fn free_readback(&mut self) {
        // SAFETY: `take` removes the pair from `self`, so it is destroyed once; the caller
        // guarantees no pending work uses it.
        unsafe {
            if let Some((buffer, memory)) = self.readback.take() {
                self.device.destroy_buffer(buffer, None);
                self.device.free_memory(memory, None);
            }
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
        // SAFETY: a block is destroyed only when its free list is the whole block, so no section
        // uses it, and it is nulled afterwards, so it is destroyed once; the caller guarantees the
        // GPU is done with it.
        unsafe {
            let mut spare = false;
            for block in &mut self.arena {
                let empty = block.buffer != vk::Buffer::null()
                    && block.free.as_slice() == [(0, block.size)];
                if !empty {
                    continue;
                }
                if !spare {
                    spare = true;
                    continue;
                }
                self.device.destroy_buffer(block.buffer, None);
                self.device.free_memory(block.memory, None);
                block.buffer = vk::Buffer::null();
                block.memory = vk::DeviceMemory::null();
                block.free = Vec::new();
            }
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
        // SAFETY: only chunks outside `staging_filling` and `staging_in_flight` are unmapped and
        // destroyed, so no frame copies from them; each is nulled afterwards.
        unsafe {
            let mut idle = 0;
            for (index, chunk) in self.staging.iter_mut().enumerate() {
                if chunk.buffer == vk::Buffer::null()
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
                self.device.unmap_memory(chunk.memory);
                self.device.destroy_buffer(chunk.buffer, None);
                self.device.free_memory(chunk.memory, None);
                chunk.buffer = vk::Buffer::null();
                chunk.memory = vk::DeviceMemory::null();
                chunk.mapped = std::ptr::null_mut();
                chunk.size = 0;
                chunk.used = 0;
            }
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
                    .free
                    .iter()
                    .position(|&(_, length)| length >= size)
                    .map(|range| (block, range))
            });
            let (block, range) = match found {
                Some(found) => found,
                None => {
                    let block = self.create_arena_block(size.max(ARENA_BLOCK_SIZE))?;
                    let given_back = self
                        .arena
                        .iter()
                        .position(|own| own.buffer == vk::Buffer::null());
                    match given_back {
                        Some(slot) => {
                            self.arena[slot] = block;
                            (slot, 0)
                        }
                        None => {
                            self.arena.push(block);
                            (self.arena.len() - 1, 0)
                        }
                    }
                }
            };
            let arena = &mut self.arena[block];
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
            let chunk = match self
                .staging_filling
                .iter()
                .copied()
                .find(|&chunk| self.staging[chunk].size - self.staging[chunk].used >= size)
            {
                Some(chunk) => chunk,
                None => {
                    let free = (0..self.staging.len()).find(|chunk| {
                        !self.staging_filling.contains(chunk)
                            && !self
                                .staging_in_flight
                                .iter()
                                .any(|&(_, busy)| busy == *chunk)
                            && self.staging[*chunk].size >= size
                    });
                    let chunk = match free {
                        Some(chunk) => chunk,
                        None => {
                            let made = self.create_staging_chunk(size.max(STAGING_CHUNK_SIZE))?;
                            let given_back = self
                                .staging
                                .iter()
                                .position(|own| own.buffer == vk::Buffer::null());
                            match given_back {
                                Some(slot) => {
                                    self.staging[slot] = made;
                                    slot
                                }
                                None => {
                                    self.staging.push(made);
                                    self.staging.len() - 1
                                }
                            }
                        }
                    };
                    self.staging[chunk].used = 0;
                    self.staging_filling.push(chunk);
                    chunk
                }
            };
            let staging = &mut self.staging[chunk];
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
        let free = &mut self.arena[slice.block].free;
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
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(size)
                        .usage(
                            vk::BufferUsageFlags::VERTEX_BUFFER
                                | vk::BufferUsageFlags::INDEX_BUFFER
                                | vk::BufferUsageFlags::TRANSFER_DST,
                        )
                        .sharing_mode(vk::SharingMode::EXCLUSIVE),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreateBuffer", result))?;
            let requirements = self.device.get_buffer_memory_requirements(buffer);
            let memory = self.allocate(requirements, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|result| VulkanError::Call("vkBindBufferMemory", result))?;
            Ok(ArenaBlock {
                buffer,
                memory,
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
            let buffer = self
                .device
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(size)
                        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
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
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|result| VulkanError::Call("vkMapMemory", result))?
                .cast::<u8>();
            Ok(StagingChunk {
                buffer,
                memory,
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
                    self.staging[copy.chunk].buffer,
                    self.arena[copy.target.block].buffer,
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
        let (buffer, memory) = unsafe {
            backend.upload_buffer(vk::BufferUsageFlags::VERTEX_BUFFER, &vec![0_u8; capacity])
        }?;
        Ok(Self {
            device: backend.device.clone(),
            buffer,
            memory,
            capacity,
        })
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
