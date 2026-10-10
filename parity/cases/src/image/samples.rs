//! Synthetic pixel rows: original patterns, one function per kind.

type Rows = Vec<Vec<u8>>;

fn grid(width: u32, height: u32, texel: impl Fn(i64, i64) -> Vec<i64>) -> Rows {
    (0..i64::from(height))
        .map(|y| {
            (0..i64::from(width))
                .flat_map(|x| texel(x, y))
                .map(|value| value.rem_euclid(256) as u8)
                .collect()
        })
        .collect()
}

pub fn rgba(width: u32, height: u32) -> Rows {
    grid(width, height, |x, y| {
        vec![x * 17 + y, y * 23 + x, (x + y) * 5, 255 - x * 9]
    })
}

pub fn rgb(width: u32, height: u32) -> Rows {
    grid(width, height, |x, y| vec![x * 31 + y, y * 13, x * y])
}

pub fn gray(width: u32, height: u32) -> Rows {
    grid(width, height, |x, y| vec![x * 37 + y * 11])
}

pub fn gray_alpha(width: u32, height: u32) -> Rows {
    grid(width, height, |x, y| vec![x * 37 + y * 11, x * 40])
}

pub fn indexed(width: u32, height: u32, count: i64) -> Rows {
    grid(width, height, |x, y| vec![(x + y) % count])
}

pub fn rgba16(width: u32, height: u32) -> Rows {
    grid(width, height, |x, y| {
        let channels = [
            (x * 257 + y) % 65536,
            (y * 999) % 65536,
            (x * y * 13) % 65536,
            65535 - (x * 1234) % 65536,
        ];
        channels
            .iter()
            .flat_map(|channel| [channel >> 8, channel & 0xff])
            .collect()
    })
}

/// Samples of `bit_depth` bits packed most significant first, each row
/// padded to a whole byte.
pub fn packed(width: u32, height: u32, bit_depth: u32, value_at: fn(i64, i64) -> i64) -> Rows {
    (0..i64::from(height))
        .map(|y| {
            let mut bits = Vec::new();
            for x in 0..i64::from(width) {
                let value = value_at(x, y) & ((1 << bit_depth) - 1);
                bits.extend((0..bit_depth).map(|bit| (value >> (bit_depth - 1 - bit)) & 1));
            }
            bits.resize(bits.len().div_ceil(8) * 8, 0);
            bits.chunks(8)
                .map(|byte| byte.iter().fold(0u8, |acc, bit| (acc << 1) | *bit as u8))
                .collect()
        })
        .collect()
}

/// Four palette entries.
pub fn palette() -> Vec<u8> {
    (0..4)
        .flat_map(|index: i64| [index * 60 % 256, 255 - index * 40 % 256, index * 85 % 256])
        .map(|value| value as u8)
        .collect()
}

pub fn gradient(width: u32, height: u32) -> Rows {
    let (w, h) = (i64::from(width), i64::from(height));
    grid(width, height, |x, y| {
        vec![
            x * 255 / (w - 1).max(1),
            y * 255 / (h - 1).max(1),
            (x + y) * 255 / (w + h - 2).max(1),
            255,
        ]
    })
}

/// A fixed linear congruential sequence, restarted for each image.
pub fn noisy(width: u32, height: u32) -> Rows {
    let mut state: u64 = 12345;
    let mut step = move || {
        state = (state * 1103515245 + 12345) & 0x7fff_ffff;
        ((state >> 16) & 0xff) as u8
    };
    (0..height)
        .map(|_| (0..width * 4).map(|_| step()).collect())
        .collect()
}

/// A shape with hard transparent edges, like a plant or a leaf.
pub fn cutout(width: u32, height: u32) -> Rows {
    let (w, h) = (i64::from(width), i64::from(height));
    grid(width, height, |x, y| {
        let inside = (x - w / 2).abs() + (y - h / 2).abs() < w / 3;
        vec![x * 13, y * 29, 180, if inside { 255 } else { 0 }]
    })
}

/// Alpha that ramps rather than jumping, so the coverage search has to
/// converge on a scale instead of reproducing one exactly.
pub fn soft_alpha(width: u32, height: u32) -> Rows {
    let w = i64::from(width);
    grid(width, height, |x, _| {
        vec![200, 120, 60, x * 255 / (w - 1).max(1)]
    })
}

/// No opaque texel at all, which leaves the fill with no colour to spread
/// and the dark fill with no darkest colour to find.
pub fn clear(width: u32, height: u32) -> Rows {
    solid(width, height, [90, 140, 200, 0])
}

pub fn solid(width: u32, height: u32, texel: [u8; 4]) -> Rows {
    vec![texel.repeat(width as usize); height as usize]
}
