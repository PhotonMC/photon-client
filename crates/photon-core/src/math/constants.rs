/// Java `Mth` public scalar constants, stored with their reference f32 bits.
pub const MTH_PI: f32 = f32::from_bits(0x4049_0fdb);

pub const MTH_HALF_PI: f32 = f32::from_bits(0x3fc9_0fdb);

pub const MTH_TWO_PI: f32 = f32::from_bits(0x40c9_0fdb);

pub const MTH_DEG_TO_RAD: f32 = f32::from_bits(0x3c8e_fa35);

pub const MTH_RAD_TO_DEG: f32 = f32::from_bits(0x4265_2ee0);

pub const MTH_EPSILON: f32 = f32::from_bits(0x3727_c5ac);

pub const MTH_SQRT_OF_TWO: f32 = f32::from_bits(0x3fb5_04f3);

pub(super) const MULTIPLY_DE_BRUIJN_BIT_POSITION: [i32; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26,
    12, 18, 6, 11, 5, 10, 9,
];

pub(super) const MTH_SIN_SCALE: f64 = 10430.378350470453;
