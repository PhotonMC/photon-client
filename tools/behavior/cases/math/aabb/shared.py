"""The `math` family: what several `aabb` cases share."""

from cases.math.common import *
from cases.math.shared import *


unit = {"min_x": float_bits(0.0, "f64"), "min_y": float_bits(0.0, "f64"),
        "min_z": float_bits(0.0, "f64"), "max_x": float_bits(1.0, "f64"),
        "max_y": float_bits(1.0, "f64"), "max_z": float_bits(1.0, "f64")}
