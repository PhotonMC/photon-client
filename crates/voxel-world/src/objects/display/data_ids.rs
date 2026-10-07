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

pub(super) const COMPONENT_SERIALIZER_ID: i32 = 5;

pub(super) const BLOCK_STATE_SERIALIZER_ID: i32 = 14;

pub(super) const VECTOR3_SERIALIZER_ID: i32 = 39;

pub(super) const QUATERNION_SERIALIZER_ID: i32 = 40;

/// `Display.NO_BRIGHTNESS_OVERRIDE`.
pub const NO_BRIGHTNESS_OVERRIDE: i32 = -1;
