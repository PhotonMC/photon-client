//! The pipeline cache kept on disk between runs, so the driver does not compile every pipeline's
//! shaders again at each start-up.

use std::path::{Path, PathBuf};

use super::*;

/// Who made a cache: the header of its data names the device and driver it is good for.
#[derive(Clone, Copy)]
pub(super) struct CacheIdentity {
    pub(super) vendor_id: u32,
    pub(super) device_id: u32,
    pub(super) uuid: [u8; vk::UUID_SIZE],
}

/// Size of `VkPipelineCacheHeaderVersionOne`: its length, version, vendor, device and UUID.
const HEADER_SIZE: usize = 16 + vk::UUID_SIZE;

impl CacheIdentity {
    /// Whether cache data was made by this device and driver. A driver checks this itself, but a
    /// file from another machine or driver is dropped here so it never reaches the driver.
    fn accepts(&self, data: &[u8]) -> bool {
        let word = |at: usize| {
            data.get(at..at + 4)
                .map(|bytes| u32::from_ne_bytes(bytes.try_into().expect("four bytes")))
        };
        data.len() >= HEADER_SIZE
            && word(0).is_some_and(|length| length as usize >= HEADER_SIZE)
            && word(4) == Some(vk::PipelineCacheHeaderVersion::ONE.as_raw() as u32)
            && word(8) == Some(self.vendor_id)
            && word(12) == Some(self.device_id)
            && data[16..HEADER_SIZE] == self.uuid
    }
}

impl VulkanBackend {
    /// Keep the pipeline cache in a file: start from what it holds, if it was made by this device
    /// and driver, and let [`Self::save_pipeline_cache`] write it back. Call it before the
    /// pipelines are built. Returns the bytes the cache started with.
    pub fn use_pipeline_cache(&mut self, path: PathBuf) -> Result<usize, VulkanError> {
        let initial = std::fs::read(&path)
            .ok()
            .filter(|data| self.cache_identity.accepts(data))
            .unwrap_or_default();
        // SAFETY: live handles; the old cache, if any, is destroyed before it is replaced, and no
        // pipeline is being built (this is called before the scene is loaded).
        unsafe {
            let cache = self
                .device
                .create_pipeline_cache(
                    &vk::PipelineCacheCreateInfo::default().initial_data(&initial),
                    None,
                )
                .map_err(|result| VulkanError::Call("vkCreatePipelineCache", result))?;
            self.device
                .destroy_pipeline_cache(self.pipeline_cache, None);
            self.pipeline_cache = cache;
        }
        self.pipeline_cache_path = Some(path);
        Ok(initial.len())
    }

    /// Write the pipeline cache to the file it was started from. A cache that cannot be written
    /// costs only the next start-up's time, so a failure is logged and nothing more.
    pub fn save_pipeline_cache(&self) {
        let Some(path) = &self.pipeline_cache_path else {
            return;
        };
        // SAFETY: live handles; the cache is owned by `self` and not being used by a build.
        let data = unsafe { self.device.get_pipeline_cache_data(self.pipeline_cache) };
        let written = data
            .map_err(|result| format!("{result}"))
            .and_then(|data| write_whole(path, &data).map_err(|error| error.to_string()));
        if let Err(error) = written {
            log::warn!("pipeline cache not saved to {}: {error}", path.display());
        }
    }
}

/// Write a file by way of a sibling and a rename, so a crash leaves the old file or the new one.
fn write_whole(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(directory) = path.parent() {
        std::fs::create_dir_all(directory)?;
    }
    let partial = path.with_extension("partial");
    std::fs::write(&partial, data)?;
    std::fs::rename(partial, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: CacheIdentity = CacheIdentity {
        vendor_id: 0x106b,
        device_id: 0x1234,
        uuid: [7; vk::UUID_SIZE],
    };

    fn header(vendor: u32, device: u32, uuid: [u8; vk::UUID_SIZE]) -> Vec<u8> {
        let mut data = Vec::new();
        for word in [HEADER_SIZE as u32, 1, vendor, device] {
            data.extend(word.to_ne_bytes());
        }
        data.extend(uuid);
        data.extend([0; 8]); // the driver's own data
        data
    }

    #[test]
    fn a_cache_of_this_device_is_accepted() {
        assert!(IDENTITY.accepts(&header(0x106b, 0x1234, [7; vk::UUID_SIZE])));
    }

    #[test]
    fn a_cache_of_another_device_or_driver_is_refused() {
        assert!(!IDENTITY.accepts(&header(0x10de, 0x1234, [7; vk::UUID_SIZE])));
        assert!(!IDENTITY.accepts(&header(0x106b, 0x9999, [7; vk::UUID_SIZE])));
        assert!(!IDENTITY.accepts(&header(0x106b, 0x1234, [8; vk::UUID_SIZE])));
    }

    #[test]
    fn a_short_or_empty_file_is_refused() {
        assert!(!IDENTITY.accepts(&[]));
        assert!(!IDENTITY.accepts(&header(0x106b, 0x1234, [7; vk::UUID_SIZE])[..20]));
    }
}
