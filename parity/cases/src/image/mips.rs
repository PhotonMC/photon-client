//! Mipmap generation and the transparency scan that resolves `AUTO`.

use serde_json::{Value, json};

use super::png;
use super::samples::{clear, cutout, gradient, noisy, soft_alpha, solid};
use crate::bits::f32_bits;
use crate::case::case;

/// One mip chain: texels, requested levels, strategy, alpha bias, sprite
/// path and the transparency the atlas reports.
struct Mip {
    label: &'static str,
    width: u32,
    rows: Vec<Vec<u8>>,
    levels: u32,
    strategy: &'static str,
    bias: f64,
    path: &'static str,
    transparency: (bool, bool),
}

fn mip(
    label: &'static str,
    width: u32,
    rows: Vec<Vec<u8>>,
    levels: u32,
    strategy: &'static str,
) -> Mip {
    Mip {
        label,
        width,
        rows,
        levels,
        strategy,
        bias: 0.0,
        path: "block/parity",
        transparency: (false, false),
    }
}

fn mean_rows() -> Vec<Mip> {
    vec![
        mip("gradient-16", 16, gradient(16, 16), 4, "MEAN"),
        mip("gradient-8", 8, gradient(8, 8), 3, "MEAN"),
        mip("noisy-16", 16, noisy(16, 16), 4, "MEAN"),
        mip("noisy-32", 32, noisy(32, 32), 4, "MEAN"),
        mip("flat-4", 4, solid(4, 4, [200, 100, 50, 255]), 2, "MEAN"),
        // A single level requested means the source comes back untouched.
        mip("no-mips", 16, gradient(16, 16), 0, "MEAN"),
    ]
}

fn cutout_rows() -> Vec<Mip> {
    let shape = || cutout(16, 16);
    vec![
        // The cutout strategies spread colour into the transparent area
        // before halving, then rescale alpha so the visible area keeps its
        // size.
        mip("cutout-16", 16, shape(), 4, "CUTOUT"),
        mip("cutout-32", 32, cutout(32, 32), 4, "CUTOUT"),
        // A stricter reference alpha moves the coverage the search aims at.
        mip("strict-cutout-16", 16, shape(), 4, "STRICT_CUTOUT"),
        Mip {
            bias: 0.1,
            ..mip("cutout-bias", 16, shape(), 4, "CUTOUT")
        },
        mip("cutout-soft", 16, soft_alpha(16, 16), 4, "CUTOUT"),
        mip("dark-cutout-16", 16, shape(), 4, "DARK_CUTOUT"),
        // Filling is skipped for item textures, so level zero keeps its holes.
        Mip {
            path: "item/parity",
            ..mip("cutout-item", 16, shape(), 4, "CUTOUT")
        },
        // Asking for no mips still fills, because that runs before the early
        // exit.
        mip("cutout-no-mips", 16, shape(), 0, "CUTOUT"),
        mip("dark-cutout-no-mips", 16, shape(), 0, "DARK_CUTOUT"),
        mip("cutout-clear", 8, clear(8, 8), 3, "CUTOUT"),
        mip("dark-cutout-clear", 8, clear(8, 8), 3, "DARK_CUTOUT"),
        // AUTO picks its strategy from the transparency it is told about.
        mip("auto-opaque", 16, shape(), 4, "AUTO"),
        Mip {
            transparency: (true, false),
            ..mip("auto-transparent", 16, shape(), 4, "AUTO")
        },
        Mip {
            transparency: (false, true),
            ..mip("auto-translucent", 16, shape(), 4, "AUTO")
        },
    ]
}

pub fn mip_levels() -> Value {
    let rows: Vec<Value> = mean_rows()
        .into_iter()
        .chain(cutout_rows())
        .map(|mip| {
            json!({
                "label": mip.label, "name": format!("minecraft:{}", mip.path),
                "levels": mip.levels, "strategy": mip.strategy, "bias": f32_bits(mip.bias),
                "transparent": mip.transparency.0, "translucent": mip.transparency.1,
                "png": png::rgba(mip.width, &mip.rows),
            })
        })
        .collect();
    case(
        "image/mip-levels",
        "image",
        "mip_levels",
        json!({ "rows": rows }),
    )
}

/// What the atlas measures from a sprite's pixels to resolve AUTO. The scan
/// separates fully transparent from partly transparent, and a texture can
/// be both at once.
pub fn compute_transparency() -> Value {
    let both: Vec<Vec<u8>> = (0..8)
        .map(|y| [10, 20, 30, if y % 2 == 1 { 0 } else { 128 }].repeat(8))
        .collect();
    let mut hole = solid(8, 8, [10, 20, 30, 255]);
    hole[5][3 * 4 + 3] = 0;
    let images = [
        ("opaque", 8, solid(8, 8, [10, 20, 30, 255])),
        ("transparent", 8, clear(8, 8)),
        ("cutout", 16, cutout(16, 16)),
        ("translucent", 8, solid(8, 8, [10, 20, 30, 128])),
        ("both", 8, both),
        // 254 counts as partly transparent; only exactly 255 is opaque.
        ("almost-opaque", 8, solid(8, 8, [10, 20, 30, 254])),
        ("single-hole", 8, hole),
    ];
    let rows: Vec<Value> = images
        .iter()
        .map(|(label, width, rows)| json!({"label": label, "png": png::rgba(*width, rows)}))
        .collect();
    case(
        "image/compute-transparency",
        "image",
        "compute_transparency",
        json!({ "rows": rows }),
    )
}
