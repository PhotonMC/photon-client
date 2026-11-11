//! The arithmetic on attribute colours: the blend between two colours, the three colour
//! modifiers, the blend towards grey and the alpha blend. 26.3 holds colours as float vectors and
//! works on those; 26.2 holds them as packed 8-bit ints and truncates every channel as integers
//! do. A colour here is always the float vector (`channel / 255` for a packed one), so the rest
//! of the sky reads one form, and the packed path packs it, works on the ints and unpacks.
//!
//! The set is chosen once ([`colour_ops`]); no function of the sky branches on the version.

use super::lerp::{lerp_int, srgb_lerp_vec};

/// The colour of an attribute: red, green, blue and alpha, each 0 to 1.
pub(super) type Colour = [f32; 4];

/// One way of doing the colour arithmetic.
pub(super) struct ColourOps {
    /// `ARGB.srgbLerp`.
    pub(super) lerp: fn(f32, Colour, Colour) -> Colour,
    /// `ColorModifier.MULTIPLY_RGB` and `MULTIPLY_ARGB`: every channel, alpha included.
    pub(super) multiply: fn(Colour, Colour) -> Colour,
    /// `ColorModifier.ADD`: the red, green and blue added, held to the top; the alpha is kept.
    pub(super) add: fn(Colour, Colour) -> Colour,
    /// `ColorModifier.SUBTRACT`: the red, green and blue taken off, held to nothing.
    pub(super) subtract: fn(Colour, Colour) -> Colour,
    /// `ColorModifier.BLEND_TO_GRAY` with a brightness and a factor.
    pub(super) blend_to_grey: fn(Colour, f32, f32) -> Colour,
    /// `ColorModifier.ALPHA_BLEND` of a source over a colour that has no alpha of its own.
    pub(super) alpha_blend: fn(Colour, Colour) -> Colour,
}

static FLOATS: ColourOps = ColourOps {
    lerp: srgb_lerp_vec,
    multiply: |s, a| [s[0] * a[0], s[1] * a[1], s[2] * a[2], s[3] * a[3]],
    add: |s, a| {
        [
            (s[0] + a[0]).min(1.0),
            (s[1] + a[1]).min(1.0),
            (s[2] + a[2]).min(1.0),
            s[3],
        ]
    },
    subtract: |s, a| {
        [
            (s[0] - a[0]).max(0.0),
            (s[1] - a[1]).max(0.0),
            (s[2] - a[2]).max(0.0),
            s[3],
        ]
    },
    blend_to_grey: |colour, brightness, factor| {
        let grey = colour[0] * 0.3 + colour[1] * 0.59 + colour[2] * 0.11;
        let scaled = (grey * brightness).clamp(0.0, 1.0);
        srgb_lerp_vec(factor, colour, [scaled, scaled, scaled, colour[3]])
    },
    alpha_blend: |colour, source| {
        if source[3] == 0.0 {
            return colour;
        }
        let blended = srgb_lerp_vec(source[3], colour, source);
        [blended[0], blended[1], blended[2], colour[3]]
    },
};

static PACKED: ColourOps = ColourOps {
    lerp: |alpha, from, to| unpack(lerp_packed(alpha, pack(from), pack(to))),
    multiply: |s, a| {
        let (lhs, rhs) = (pack(s), pack(a));
        // `ARGB.multiply` hands the other colour back when one is opaque white.
        if lhs == [255; 4] {
            return a;
        }
        if rhs == [255; 4] {
            return s;
        }
        unpack([0, 1, 2, 3].map(|i| lhs[i] * rhs[i] / 255))
    },
    add: |s, a| {
        let (lhs, rhs) = (pack(s), pack(a));
        unpack([
            (lhs[0] + rhs[0]).min(255),
            (lhs[1] + rhs[1]).min(255),
            (lhs[2] + rhs[2]).min(255),
            lhs[3],
        ])
    },
    subtract: |s, a| {
        let (lhs, rhs) = (pack(s), pack(a));
        unpack([
            (lhs[0] - rhs[0]).max(0),
            (lhs[1] - rhs[1]).max(0),
            (lhs[2] - rhs[2]).max(0),
            lhs[3],
        ])
    },
    blend_to_grey: |colour, brightness, factor| {
        let packed = pack(colour);
        // `ARGB.greyscale` truncates the weighted sum, `scaleRGB` truncates the scaled grey.
        let grey =
            (packed[0] as f32 * 0.3 + packed[1] as f32 * 0.59 + packed[2] as f32 * 0.11) as i32;
        let scaled = ((grey as f32 * brightness) as i32).clamp(0, 255);
        let target = [scaled, scaled, scaled, packed[3]];
        unpack(lerp_packed(factor, packed, target))
    },
    alpha_blend: |colour, source| {
        let (destination, source_packed) = (pack(colour), pack(source));
        let source_alpha = source_packed[3];
        if source_alpha == 255 {
            return source;
        }
        if source_alpha == 0 {
            return colour;
        }
        let alpha = source_alpha + destination[3] * (255 - source_alpha) / 255;
        let channel = |i: usize| {
            (source_packed[i] * source_alpha + destination[i] * (alpha - source_alpha)) / alpha
        };
        unpack([channel(0), channel(1), channel(2), alpha])
    },
};

/// The arithmetic of the version being played.
pub(super) fn colour_ops() -> &'static ColourOps {
    if photon_core::versions::rules()
        .render
        .attribute_colours_are_packed
    {
        &PACKED
    } else {
        &FLOATS
    }
}

/// The packed channels of a colour that came from packed ones (`channel / 255`), red first.
fn pack(colour: Colour) -> [i32; 4] {
    colour.map(|channel| (channel * 255.0).round() as i32)
}

fn unpack(channels: [i32; 4]) -> Colour {
    channels.map(|channel| channel as f32 / 255.0)
}

/// `ARGB.srgbLerp` on packed channels: each lerped as an int.
fn lerp_packed(alpha: f32, from: [i32; 4], to: [i32; 4]) -> [i32; 4] {
    [0, 1, 2, 3].map(|i| lerp_int(alpha, from[i], to[i]))
}
