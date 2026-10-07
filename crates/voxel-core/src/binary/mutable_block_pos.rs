use super::*;

/// Mutable BlockPos state used by Java's cursor-based helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    /// Create mutable BlockPos state from integer coordinates.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Create mutable BlockPos state using Java floor conversion.
    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self::new(math::floor_f64(x), math::floor_f64(y), math::floor_f64(z))
    }

    /// Return an immutable snapshot of the current mutable coordinates.
    pub const fn immutable(self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    /// Set all three coordinates.
    pub const fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    /// Set all three coordinates after Java floor conversion.
    pub fn set_f64(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.set(math::floor_f64(x), math::floor_f64(y), math::floor_f64(z))
    }

    /// Set all three coordinates from an immutable BlockPos.
    pub const fn set_block_pos(&mut self, value: BlockPos) -> &mut Self {
        self.set(value.x, value.y, value.z)
    }

    /// Set all three coordinates from a packed BlockPos long.
    pub const fn set_packed(&mut self, value: u64) -> &mut Self {
        self.set_block_pos(BlockPos::of(value))
    }

    /// Set coordinates after applying a Java AxisCycle to each output axis.
    pub fn set_axis_cycle(&mut self, cycle: AxisCycle, x: i32, y: i32, z: i32) -> &mut Self {
        self.set(
            cycle.cycle_i32(x, y, z, Axis::X),
            cycle.cycle_i32(x, y, z, Axis::Y),
            cycle.cycle_i32(x, y, z, Axis::Z),
        )
    }

    /// Set to a position plus one cardinal direction step.
    pub fn set_with_offset_direction(
        &mut self,
        position: BlockPos,
        direction: Direction,
    ) -> &mut Self {
        let (x, y, z) = direction.step();
        self.set(
            position.x.wrapping_add(x),
            position.y.wrapping_add(y),
            position.z.wrapping_add(z),
        )
    }

    /// Set to a position plus explicit integer offsets.
    pub const fn set_with_offset_components(
        &mut self,
        position: BlockPos,
        x: i32,
        y: i32,
        z: i32,
    ) -> &mut Self {
        self.set(
            position.x.wrapping_add(x),
            position.y.wrapping_add(y),
            position.z.wrapping_add(z),
        )
    }

    /// Set to a position plus an immutable BlockPos offset.
    pub const fn set_with_offset_block_pos(
        &mut self,
        position: BlockPos,
        offset: BlockPos,
    ) -> &mut Self {
        self.set_with_offset_components(position, offset.x, offset.y, offset.z)
    }

    /// Move by a cardinal direction and signed number of steps.
    pub fn move_direction(&mut self, direction: Direction, steps: i32) -> &mut Self {
        let (x, y, z) = direction.step();
        self.move_components(
            x.wrapping_mul(steps),
            y.wrapping_mul(steps),
            z.wrapping_mul(steps),
        )
    }

    /// Move by explicit integer offsets.
    pub const fn move_components(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = self.x.wrapping_add(x);
        self.y = self.y.wrapping_add(y);
        self.z = self.z.wrapping_add(z);
        self
    }

    /// Move by an immutable BlockPos offset.
    pub const fn move_block_pos(&mut self, value: BlockPos) -> &mut Self {
        self.move_components(value.x, value.y, value.z)
    }

    /// Clamp one selected axis and preserve the other two.
    pub fn clamp(&mut self, axis: Axis, minimum: i32, maximum: i32) -> &mut Self {
        match axis {
            Axis::X => self.x = math::clamp_i32(self.x, minimum, maximum),
            Axis::Y => self.y = math::clamp_i32(self.y, minimum, maximum),
            Axis::Z => self.z = math::clamp_i32(self.z, minimum, maximum),
        }
        self
    }

    /// Set the x coordinate.
    pub const fn set_x(&mut self, value: i32) -> &mut Self {
        self.x = value;
        self
    }

    /// Set the y coordinate.
    pub const fn set_y(&mut self, value: i32) -> &mut Self {
        self.y = value;
        self
    }

    /// Set the z coordinate.
    pub const fn set_z(&mut self, value: i32) -> &mut Self {
        self.z = value;
        self
    }

    /// Return an immutable offset result without changing this mutable state.
    pub fn offset(self, x: i32, y: i32, z: i32) -> BlockPos {
        self.immutable().offset(x, y, z)
    }

    /// Return an immutable scaled result without changing this mutable state.
    pub fn multiply(self, scale: i32) -> BlockPos {
        self.immutable().multiply(scale)
    }

    /// Return an immutable cardinal-relative result without changing this mutable state.
    pub fn relative_direction(self, direction: Direction, steps: i32) -> BlockPos {
        self.immutable().relative_direction(direction, steps)
    }

    /// Return an immutable axis-relative result without changing this mutable state.
    pub fn relative_axis(self, axis: Axis, steps: i32) -> BlockPos {
        self.immutable().relative_axis(axis, steps)
    }

    /// Return an immutable rotated result without changing this mutable state.
    pub fn rotate(self, rotation: BlockRotation) -> BlockPos {
        self.immutable().rotate(rotation)
    }
}
