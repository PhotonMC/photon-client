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
