//! What a display entity does on the client (`Display`): the data the server sends, the render
//! state made from it when it changes (a transformation, shadow and the rest that ease from the
//! last one to the new over the interpolation duration), and the progress of that easing.

use photon_core::math::{mth_inverse_lerp_f32, mth_lerp_f32, mth_lerp_int};

use crate::metadata::{
    find_metadata_field, metadata_byte_value, metadata_float_value, metadata_int_value,
};

pub(crate) mod data_ids;
pub use data_ids::*;
mod render_state;
pub use render_state::*;
mod state;
pub use state::*;
mod transformation;
pub use transformation::*;
