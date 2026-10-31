//! Plain values viewed as the bytes the GPU reads.

use super::*;

/// A value whose bytes are all initialised, so a slice of it can go to the GPU as bytes.
///
/// # Safety
///
/// The type must be `repr(C)`, made only of numbers and arrays of them, and have no padding.
pub(super) unsafe trait Plain: Copy {}

// SAFETY: a number has no padding.
unsafe impl Plain for f32 {}
// SAFETY: a number has no padding.
unsafe impl Plain for u32 {}

/// Vertex types, each with the size its fields add up to, so a field that
/// brings padding fails the build instead of leaking uninitialised bytes.
macro_rules! plain_vertices {
    ($($vertex:ty = $size:expr),* $(,)?) => {$(
        const _: () = assert!(std::mem::size_of::<$vertex>() == $size);
        // SAFETY: `repr(C)` over `f32`/`u8` arrays, and the assertion above
        // shows the fields leave no padding.
        unsafe impl Plain for $vertex {}
    )*};
}

plain_vertices! {
    Vertex = 48,
    BlockVertex = 28,
    crate::packed_vertex::PackedVertex = 20,
    crate::block::outline::LineVertex = 44,
    crate::sky::clouds::CloudVertex = 16,
    crate::sky::SkyVertex = 44,
    crate::gui::GuiVertex = 32,
    crate::gui::items::GuiItemVertex = 36,
    crate::entity::instancing::Instance = 100,
}

/// The bytes of a slice of plain values.
pub(super) fn as_bytes<T: Plain>(values: &[T]) -> &[u8] {
    // SAFETY: `T: Plain` makes every byte of the slice initialised; the
    // pointer is valid for `size_of_val` bytes, `u8` needs no alignment, and
    // the result borrows `values`.
    unsafe {
        std::slice::from_raw_parts(values.as_ptr().cast::<u8>(), std::mem::size_of_val(values))
    }
}
