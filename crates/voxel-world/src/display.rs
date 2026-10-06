//! What a display entity does on the client (`Display`): the data the server sends, the render
//! state made from it when it changes (a transformation, shadow and the rest that ease from the
//! last one to the new over the interpolation duration), and the progress of that easing.

use std::cell::Cell;

use voxel_core::math::{mth_inverse_lerp_f32, mth_lerp_f32};

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

impl DisplayState {
    /// The state of a display of this kind before any data has come.
    pub(crate) fn new(kind: DisplayKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }

    /// `onSyncedDataUpdated` for each field the data holds: what it changes is noted for the tick.
    pub(crate) fn apply_metadata(&mut self, metadata: &[u8]) {
        if let Some(value) = metadata_int_value(metadata, START_DELTA_DATA_ID) {
            self.start_delta = value;
            self.update_start_tick = true;
        }
        if let Some(value) = metadata_int_value(metadata, INTERPOLATION_DURATION_DATA_ID) {
            self.interpolation_duration_data = value;
            self.update_interpolation_duration = true;
        }
        if let Some(value) = metadata_int_value(metadata, POS_ROT_DURATION_DATA_ID) {
            self.pos_rot_duration = value;
        }
        if let Some(value) = floats(metadata, TRANSLATION_DATA_ID, VECTOR3_SERIALIZER_ID) {
            self.translation = value;
            self.update_render_state = true;
        }
        if let Some(value) = floats(metadata, SCALE_DATA_ID, VECTOR3_SERIALIZER_ID) {
            self.scale = value;
            self.update_render_state = true;
        }
        if let Some(value) = floats(metadata, LEFT_ROTATION_DATA_ID, QUATERNION_SERIALIZER_ID) {
            self.left_rotation = value;
            self.update_render_state = true;
        }
        if let Some(value) = floats(metadata, RIGHT_ROTATION_DATA_ID, QUATERNION_SERIALIZER_ID) {
            self.right_rotation = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_byte_value(metadata, BILLBOARD_DATA_ID) {
            self.billboard = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_int_value(metadata, BRIGHTNESS_DATA_ID) {
            self.brightness = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_float_value(metadata, VIEW_RANGE_DATA_ID) {
            self.view_range = value;
        }
        if let Some(value) = metadata_float_value(metadata, SHADOW_RADIUS_DATA_ID) {
            self.shadow_radius = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_float_value(metadata, SHADOW_STRENGTH_DATA_ID) {
            self.shadow_strength = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_float_value(metadata, WIDTH_DATA_ID) {
            self.width = value;
        }
        if let Some(value) = metadata_float_value(metadata, HEIGHT_DATA_ID) {
            self.height = value;
        }
        if let Some(value) = metadata_int_value(metadata, GLOW_COLOR_DATA_ID) {
            self.glow_color = value;
        }
    }

    /// A block display's block state data (`DATA_BLOCK_STATE_ID`).
    pub(crate) fn apply_block_state(&mut self, metadata: &[u8]) {
        if let Some((BLOCK_STATE_SERIALIZER_ID, mut value)) =
            find_metadata_field(metadata, BLOCK_STATE_DATA_ID)
            && let Some(id) = voxel_core::wire::read_varint(&mut value)
                .ok()
                .and_then(|id| u32::try_from(id).ok())
        {
            self.block_state = id;
            self.update_render_state = true;
        }
    }

    /// An item display's context data (`DATA_ITEM_DISPLAY_ID`).
    pub(crate) fn apply_item_context(&mut self, metadata: &[u8]) {
        if let Some(value) = metadata_byte_value(metadata, ITEM_DISPLAY_DATA_ID) {
            self.item_display = value;
            self.update_render_state = true;
        }
    }

    /// An item display's item changed (`DATA_ITEM_STACK_ID`).
    pub(crate) fn item_changed(&mut self) {
        self.update_render_state = true;
    }

    /// `Display.createTransformation`.
    fn transformation(&self) -> Transformation {
        Transformation {
            translation: self.translation,
            left_rotation: self.left_rotation,
            scale: self.scale,
            right_rotation: self.right_rotation,
        }
    }

    /// The length of the position and turn interpolation (`getPosRotInterpolationDuration`).
    pub fn pos_rot_duration(&self) -> i32 {
        self.pos_rot_duration
    }

    /// `Display.tick`'s client half, but for the position's easing: the notes the data left are
    /// acted on. `tick_count` is the entity's.
    pub(crate) fn tick(&mut self, tick_count: i32) {
        if self.update_start_tick {
            self.update_start_tick = false;
            self.interpolation_start_tick = i64::from(tick_count) + i64::from(self.start_delta);
        }
        if self.update_interpolation_duration {
            self.update_interpolation_duration = false;
            self.interpolation_duration = self.interpolation_duration_data;
        }
        if self.update_render_state {
            self.update_render_state = false;
            let should_interpolate = self.interpolation_duration != 0;
            let progress = f32::from_bits(self.last_progress.get());
            self.render_state = Some(match self.render_state {
                Some(previous) if should_interpolate => {
                    self.interpolated_render_state(&previous, progress)
                }
                _ => self.fresh_render_state(),
            });
            // `updateRenderSubState`.
            match self.kind {
                DisplayKind::Block => self.block_render = Some(self.block_state),
                DisplayKind::Item => self.item_render = Some(self.item_display),
                DisplayKind::Text => {}
            }
        }
    }

    /// `createFreshRenderState`.
    fn fresh_render_state(&self) -> RenderState {
        RenderState {
            transformation: TransformationInterpolator::Constant(self.transformation()),
            billboard: Billboard::from_id(self.billboard),
            brightness_override: self.brightness,
            shadow_radius: FloatInterpolator::Constant(self.shadow_radius),
            shadow_strength: FloatInterpolator::Constant(self.shadow_strength),
            glow_color_override: self.glow_color,
        }
    }

    /// `createInterpolatedRenderState`: from where the last one had got to.
    fn interpolated_render_state(&self, previous: &RenderState, progress: f32) -> RenderState {
        RenderState {
            transformation: TransformationInterpolator::Slerp(
                previous.transformation.get(progress),
                self.transformation(),
            ),
            billboard: Billboard::from_id(self.billboard),
            brightness_override: self.brightness,
            shadow_radius: FloatInterpolator::Linear(
                previous.shadow_radius.get(progress),
                self.shadow_radius,
            ),
            shadow_strength: FloatInterpolator::Linear(
                previous.shadow_strength.get(progress),
                self.shadow_strength,
            ),
            glow_color_override: self.glow_color,
        }
    }

    /// `calculateInterpolationProgress`: how far the easing of the render state has gone at a
    /// part tick (and it is remembered for the next render state to ease from).
    pub fn interpolation_progress(&self, tick_count: i32, partial: f32) -> f32 {
        let duration = self.interpolation_duration;
        if duration <= 0 {
            return 1.0;
        }
        let since_update = (i64::from(tick_count) - self.interpolation_start_tick) as f32;
        let progress = mth_inverse_lerp_f32(since_update + partial, 0.0, duration as f32);
        let progress = if progress < 0.0 {
            0.0
        } else {
            progress.min(1.0)
        };
        self.last_progress.set(progress.to_bits());
        progress
    }

    /// `Display.renderState`: none until some data has changed.
    pub fn render_state(&self) -> Option<&RenderState> {
        self.render_state.as_ref()
    }

    /// `BlockDisplay.blockRenderState`.
    pub fn block_state(&self) -> Option<u32> {
        self.block_render
    }

    /// `ItemDisplay.itemRenderState`'s `ItemDisplayContext` id.
    pub fn item_context(&self) -> Option<u8> {
        self.item_render
    }

    /// `Display.getViewRange`.
    pub fn view_range(&self) -> f32 {
        self.view_range
    }

    /// `Display.getWidth` and `getHeight`.
    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }
}
