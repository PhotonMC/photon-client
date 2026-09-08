//! Axis-aligned bounding boxes for the first P01.d geometry slice.
//!
//! This covers the constructor normalization and the contains/intersects,
//! expand-towards and deflate operations used by the current behavior corpus.

/// An axis-aligned box with normalized inclusive lower and exclusive upper
/// query boundaries, matching the Java reference's stored endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl Aabb {
    /// Construct a box, ordering each pair with Java `Math.min`/`Math.max`
    /// behavior for NaN and signed zero.
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            min_x: java_min(min_x, max_x),
            min_y: java_min(min_y, max_y),
            min_z: java_min(min_z, max_z),
            max_x: java_max(min_x, max_x),
            max_y: java_max(min_y, max_y),
            max_z: java_max(min_z, max_z),
        }
    }

    /// Test an inclusive lower/exclusive upper point membership rule.
    pub fn contains(self, x: f64, y: f64, z: f64) -> bool {
        x >= self.min_x
            && x < self.max_x
            && y >= self.min_y
            && y < self.max_y
            && z >= self.min_z
            && z < self.max_z
    }

    /// Test strict overlap on all three axes; touching faces do not overlap.
    pub fn intersects(self, other: Self) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
            && self.min_z < other.max_z
            && self.max_z > other.min_z
    }

    /// Extend only the endpoint in the direction of each nonzero delta.
    pub fn expand_towards(self, x: f64, y: f64, z: f64) -> Self {
        let (min_x, max_x) = expand_axis(self.min_x, self.max_x, x);
        let (min_y, max_y) = expand_axis(self.min_y, self.max_y, y);
        let (min_z, max_z) = expand_axis(self.min_z, self.max_z, z);
        Self::new(min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// Move every lower endpoint inward and every upper endpoint inward.
    pub fn deflate(self, amount: f64) -> Self {
        Self::new(
            self.min_x + amount,
            self.min_y + amount,
            self.min_z + amount,
            self.max_x - amount,
            self.max_y - amount,
            self.max_z - amount,
        )
    }
}

fn expand_axis(min: f64, max: f64, delta: f64) -> (f64, f64) {
    if delta < 0.0 {
        (min + delta, max)
    } else if delta > 0.0 {
        (min, max + delta)
    } else {
        (min, max)
    }
}

fn java_min(left: f64, right: f64) -> f64 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == right {
        if left == 0.0 && left.to_bits() & (1 << 63) == 0 {
            right
        } else {
            left
        }
    } else if left < right {
        left
    } else {
        right
    }
}

fn java_max(left: f64, right: f64) -> f64 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == right {
        if left == 0.0 && left.to_bits() & (1 << 63) != 0 {
            right
        } else {
            left
        }
    } else if left > right {
        left
    } else {
        right
    }
}
