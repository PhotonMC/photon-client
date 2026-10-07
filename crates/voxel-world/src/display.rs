//! What a display entity does on the client (`Display`): the data the server sends, the render
//! state made from it when it changes (a transformation, shadow and the rest that ease from the
//! last one to the new over the interpolation duration), and the progress of that easing.

use std::cell::Cell;

use voxel_core::math::{mth_inverse_lerp_f32, mth_lerp_f32, mth_lerp_int};

use crate::metadata::{
    find_metadata_field, metadata_byte_value, metadata_float_value, metadata_int_value,
};

/// `Display`'s data ids (`Entity` has eight), in the order the reference defines them.
pub const START_DELTA_DATA_ID: u8 = 8;
pub const INTERPOLATION_DURATION_DATA_ID: u8 = 9;
pub const POS_ROT_DURATION_DATA_ID: u8 = 10;
pub const TRANSLATION_DATA_ID: u8 = 11;
pub const SCALE_DATA_ID: u8 = 12;
pub const LEFT_ROTATION_DATA_ID: u8 = 13;
pub const RIGHT_ROTATION_DATA_ID: u8 = 14;
pub const BILLBOARD_DATA_ID: u8 = 15;
pub const BRIGHTNESS_DATA_ID: u8 = 16;
pub const VIEW_RANGE_DATA_ID: u8 = 17;
pub const SHADOW_RADIUS_DATA_ID: u8 = 18;
pub const SHADOW_STRENGTH_DATA_ID: u8 = 19;
pub const WIDTH_DATA_ID: u8 = 20;
pub const HEIGHT_DATA_ID: u8 = 21;
pub const GLOW_COLOR_DATA_ID: u8 = 22;
/// A block display's block state; an item display's item (then its display context).
pub const BLOCK_STATE_DATA_ID: u8 = 23;
pub const ITEM_STACK_DATA_ID: u8 = 23;
pub const ITEM_DISPLAY_DATA_ID: u8 = 24;

/// A text display's data after the item display's: the text (a component), the line width, the
/// background colour, the text opacity (a byte) and the style flags (a byte).
pub const TEXT_DATA_ID: u8 = 23;
pub const LINE_WIDTH_DATA_ID: u8 = 24;
pub const BACKGROUND_DATA_ID: u8 = 25;
pub const TEXT_OPACITY_DATA_ID: u8 = 26;
pub const STYLE_FLAGS_DATA_ID: u8 = 27;

const COMPONENT_SERIALIZER_ID: i32 = 5;
const BLOCK_STATE_SERIALIZER_ID: i32 = 14;
const VECTOR3_SERIALIZER_ID: i32 = 39;
const QUATERNION_SERIALIZER_ID: i32 = 40;

/// `Display.NO_BRIGHTNESS_OVERRIDE`.
pub const NO_BRIGHTNESS_OVERRIDE: i32 = -1;

/// Which display it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayKind {
    #[default]
    Block,
    Item,
    Text,
}

/// `Display.BillboardConstraints`, by id (an id out of range is the first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Billboard {
    Fixed,
    Vertical,
    Horizontal,
    Center,
}

impl Billboard {
    fn from_id(id: u8) -> Self {
        match id {
            1 => Self::Vertical,
            2 => Self::Horizontal,
            3 => Self::Center,
            _ => Self::Fixed,
        }
    }
}

/// `com.mojang.math.Transformation`, as its four parts (the display's data makes it from them).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformation {
    pub translation: [f32; 3],
    /// `x, y, z, w`.
    pub left_rotation: [f32; 4],
    pub scale: [f32; 3],
    pub right_rotation: [f32; 4],
}

// A transformation is made of finite numbers: never NaN.
impl Eq for Transformation {}

impl Transformation {
    pub const IDENTITY: Self = Self {
        translation: [0.0; 3],
        left_rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0; 3],
        right_rotation: [0.0, 0.0, 0.0, 1.0],
    };

    /// `Transformation.slerp`: the translation and scale linearly, the turns spherically.
    pub fn slerp(&self, that: &Self, progress: f32) -> Self {
        Self {
            translation: lerp3(self.translation, that.translation, progress),
            left_rotation: slerp(self.left_rotation, that.left_rotation, progress),
            scale: lerp3(self.scale, that.scale, progress),
            right_rotation: slerp(self.right_rotation, that.right_rotation, progress),
        }
    }
}

/// `org.joml.Math.fma` as the reference runs it (not fused).
fn fma(a: f32, b: f32, c: f32) -> f32 {
    a * b + c
}

/// `Vector3f.lerp`.
fn lerp3(from: [f32; 3], to: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|axis| fma(to[axis] - from[axis], t, from[axis]))
}

/// `Quaternionf.slerp`.
fn slerp(from: [f32; 4], to: [f32; 4], alpha: f32) -> [f32; 4] {
    let [x, y, z, w] = from;
    let cosom = fma(x, to[0], fma(y, to[1], fma(z, to[2], w * to[3])));
    let abs_cosom = cosom.abs();
    let (scale0, mut scale1);
    if 1.0 - abs_cosom > 1.0e-6 {
        let sin_sqr = 1.0 - abs_cosom * abs_cosom;
        let sinom = 1.0 / sin_sqr.sqrt();
        let omega = f64::from(sin_sqr * sinom).atan2(f64::from(abs_cosom)) as f32;
        scale0 = (((1.0 - f64::from(alpha)) * f64::from(omega)).sin() * f64::from(sinom)) as f32;
        scale1 = f64::from(alpha * omega).sin() as f32 * sinom;
    } else {
        scale0 = 1.0 - alpha;
        scale1 = alpha;
    }
    if cosom < 0.0 {
        scale1 = -scale1;
    }
    [
        fma(scale0, x, scale1 * to[0]),
        fma(scale0, y, scale1 * to[1]),
        fma(scale0, z, scale1 * to[2]),
        fma(scale0, w, scale1 * to[3]),
    ]
}

/// `Display.GenericInterpolator<Transformation>`: a constant, or the last to the new.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformationInterpolator {
    Constant(Transformation),
    Slerp(Transformation, Transformation),
}

impl TransformationInterpolator {
    pub fn get(&self, progress: f32) -> Transformation {
        match self {
            Self::Constant(value) => *value,
            Self::Slerp(previous, current) => {
                if f64::from(progress) >= 1.0 {
                    *current
                } else {
                    previous.slerp(current, progress)
                }
            }
        }
    }
}

/// `Display.FloatInterpolator`: a constant, or the last to the new linearly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatInterpolator {
    Constant(f32),
    Linear(f32, f32),
}

// A float is made of finite numbers: never NaN.
impl Eq for FloatInterpolator {}

impl FloatInterpolator {
    pub fn get(&self, progress: f32) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Linear(previous, current) => mth_lerp_f32(progress, *previous, *current),
        }
    }
}

/// `Display.IntInterpolator`: a constant, the last to the new in whole numbers (`lerpInt`), or the
/// colours' channels one by one (`ARGB.srgbLerp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntInterpolator {
    Constant(i32),
    Linear(i32, i32),
    Color(i32, i32),
}

impl IntInterpolator {
    pub fn get(&self, progress: f32) -> i32 {
        match *self {
            Self::Constant(value) => value,
            Self::Linear(previous, current) => mth_lerp_int(progress, previous, current),
            Self::Color(previous, current) => {
                let channel = |shift: u32| {
                    mth_lerp_int(
                        progress,
                        ((previous as u32 >> shift) & 0xFF) as i32,
                        ((current as u32 >> shift) & 0xFF) as i32,
                    ) as u32
                };
                // `ARGB.color(a, r, g, b)`.
                ((channel(24) << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0)) as i32
            }
        }
    }
}

/// `Display.TextDisplay.TextRenderState`: the text (a component, as the network carried it), how
/// wide its lines are, how opaque and what background, and the style flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRenderState {
    pub text: Vec<u8>,
    pub line_width: i32,
    pub text_opacity: IntInterpolator,
    pub background: IntInterpolator,
    pub flags: i8,
}

/// `Display.RenderState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderState {
    pub transformation: TransformationInterpolator,
    pub billboard: Billboard,
    pub brightness_override: i32,
    pub shadow_radius: FloatInterpolator,
    pub shadow_strength: FloatInterpolator,
    pub glow_color_override: i32,
}

/// A display's data and what it has made of it.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayState {
    kind: DisplayKind,
    start_delta: i32,
    interpolation_duration_data: i32,
    pos_rot_duration: i32,
    translation: [f32; 3],
    scale: [f32; 3],
    left_rotation: [f32; 4],
    right_rotation: [f32; 4],
    billboard: u8,
    brightness: i32,
    view_range: f32,
    shadow_radius: f32,
    shadow_strength: f32,
    width: f32,
    height: f32,
    glow_color: i32,
    block_state: u32,
    item_display: u8,
    interpolation_start_tick: i64,
    interpolation_duration: i32,
    /// `lastProgress`, as its bits: asking for a progress remembers it.
    last_progress: Cell<u32>,
    update_render_state: bool,
    update_start_tick: bool,
    update_interpolation_duration: bool,
    render_state: Option<RenderState>,
    /// `BlockDisplay.blockRenderState`: the block it shows.
    block_render: Option<u32>,
    /// `ItemDisplay.itemRenderState`'s context (the stack is the entity's).
    item_render: Option<u8>,
    /// A text display's data, and the render state made of it, with how many times it was made
    /// (the lines split from its text are kept until then).
    text: TextData,
    text_render: Option<TextRenderState>,
    text_version: u32,
}

/// A text display's own data (`Display.TextDisplay`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct TextData {
    text: Vec<u8>,
    line_width: i32,
    background: i32,
    opacity: i8,
    flags: i8,
}

impl Default for TextData {
    fn default() -> Self {
        Self {
            // `Component.empty()`: a string tag with no text.
            text: vec![8, 0, 0],
            line_width: 200,
            background: 1_073_741_824,
            opacity: -1,
            flags: 0,
        }
    }
}

// The data is made of finite numbers: never NaN.
impl Eq for DisplayState {}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            kind: DisplayKind::Block,
            start_delta: 0,
            interpolation_duration_data: 0,
            pos_rot_duration: 0,
            translation: [0.0; 3],
            scale: [1.0; 3],
            left_rotation: [0.0, 0.0, 0.0, 1.0],
            right_rotation: [0.0, 0.0, 0.0, 1.0],
            billboard: 0,
            brightness: NO_BRIGHTNESS_OVERRIDE,
            view_range: 1.0,
            shadow_radius: 0.0,
            shadow_strength: 1.0,
            width: 0.0,
            height: 0.0,
            glow_color: -1,
            block_state: 0,
            item_display: 0,
            interpolation_start_tick: -2_147_483_648,
            interpolation_duration: 0,
            last_progress: Cell::new(0.0_f32.to_bits()),
            update_render_state: false,
            update_start_tick: false,
            update_interpolation_duration: false,
            render_state: None,
            block_render: None,
            item_render: None,
            text: TextData::default(),
            text_render: None,
            text_version: 0,
        }
    }
}

/// A vector of `N` floats a data value holds, if the field is one.
fn floats<const N: usize>(metadata: &[u8], data_id: u8, serializer: i32) -> Option<[f32; N]> {
    let (found, value) = find_metadata_field(metadata, data_id)?;
    if found != serializer {
        return None;
    }
    let mut out = [0.0; N];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = f32::from_be_bytes(value.get(index * 4..index * 4 + 4)?.try_into().ok()?);
    }
    Some(out)
}

mod display_state_state;
