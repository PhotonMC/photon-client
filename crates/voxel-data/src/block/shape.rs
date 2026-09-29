//! What the collision and outline tables share.

/// One box of a shape: min x, y, z then max x, y, z.
pub type ShapeBox = [f64; 6];

/// One block's shapes and how far to trust them.
pub struct BlockShapes {
    /// Indices into the table's `SHAPES`: one for every state, or one per
    /// state (see [`super::per_state`]).
    pub shapes: &'static [u16],
    /// The shape moves with the block's random offset.
    pub offset: bool,
    /// The reference recomputes the shape from context rather than caching it.
    pub dynamic: bool,
}
