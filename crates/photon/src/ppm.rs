//! Captured frames as binary PPM files.
//!
//! PPM keeps the readback dependency-free while the frame comparison format is
//! still undecided; P15.a will pin what captures are actually stored as.

use std::io::Write as _;

/// Write an RGBA image of this size as binary PPM, dropping the alpha.
pub fn write(
    path: &str,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(out, "P6\n{width} {height}\n255\n")?;
    for pixel in rgba.as_chunks::<4>().0 {
        out.write_all(&pixel[..3])?;
    }
    out.flush()?;
    Ok(())
}

/// Frames dumped while the game keeps running (`PHOTON_RECORD_DIR`, every `PHOTON_RECORD_EVERY`-th
/// frame): one numbered sequence across the menu and the game, so a version switch shows up as it
/// looks to the player. Unlike the `PHOTON_CAPTURE_*` variables it never stops the client.
pub mod record {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SETTINGS: OnceLock<Option<(String, u32)>> = OnceLock::new();
    static FRAME: AtomicU32 = AtomicU32::new(0);
    static SAVED: AtomicU32 = AtomicU32::new(0);

    fn settings() -> Option<&'static (String, u32)> {
        SETTINGS
            .get_or_init(|| {
                let dir = std::env::var("PHOTON_RECORD_DIR").ok()?;
                std::fs::create_dir_all(&dir).ok()?;
                let every = std::env::var("PHOTON_RECORD_EVERY")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1)
                    .max(1);
                Some((dir, every))
            })
            .as_ref()
    }

    /// Whether this frame is to be saved; counts the frame either way.
    pub fn due() -> bool {
        let Some((_, every)) = settings() else {
            return false;
        };
        FRAME.fetch_add(1, Ordering::Relaxed).is_multiple_of(*every)
    }

    /// Save one captured frame as the next file of the sequence.
    pub fn save(width: u32, height: u32, rgba: &[u8]) {
        let Some((dir, _)) = settings() else {
            return;
        };
        let number = SAVED.fetch_add(1, Ordering::Relaxed);
        let path = format!("{dir}/frame-{number:05}.ppm");
        if let Err(error) = super::write(&path, width, height, rgba) {
            eprintln!("record: {path}: {error}");
        }
    }
}
