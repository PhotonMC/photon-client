use super::*;

/// Error returned when an AABB builder has not received a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AabbBuilderError {
    Undefined,
}

/// Accumulate the bounds of f32 points like Java's `AABB.Builder`.
#[derive(Debug, Clone, Copy)]
pub struct AabbBuilder {
    pub(super) min_x: f32,
    pub(super) min_y: f32,
    pub(super) min_z: f32,
    pub(super) max_x: f32,
    pub(super) max_y: f32,
    pub(super) max_z: f32,
    pub(super) defined: bool,
}

impl Default for AabbBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AabbBuilder {
    /// Create an undefined builder with Java's infinite sentinels.
    pub fn new() -> Self {
        Self {
            min_x: f32::INFINITY,
            min_y: f32::INFINITY,
            min_z: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            max_y: f32::NEG_INFINITY,
            max_z: f32::NEG_INFINITY,
            defined: false,
        }
    }

    /// Include one f32 point from a Java `Vector3fc`-compatible value.
    pub fn include(&mut self, x: f32, y: f32, z: f32) {
        self.min_x = java_min_f32(self.min_x, x);
        self.min_y = java_min_f32(self.min_y, y);
        self.min_z = java_min_f32(self.min_z, z);
        self.max_x = java_max_f32(self.max_x, x);
        self.max_y = java_max_f32(self.max_y, y);
        self.max_z = java_max_f32(self.max_z, z);
        self.defined = true;
    }

    /// Return whether at least one point has been included.
    pub fn is_defined(&self) -> bool {
        self.defined
    }

    /// Build the accumulated double-precision AABB.
    pub fn build(&self) -> Result<Aabb, AabbBuilderError> {
        if !self.defined {
            return Err(AabbBuilderError::Undefined);
        }
        Ok(Aabb::new(
            f64::from(self.min_x),
            f64::from(self.min_y),
            f64::from(self.min_z),
            f64::from(self.max_x),
            f64::from(self.max_y),
            f64::from(self.max_z),
        ))
    }
}
