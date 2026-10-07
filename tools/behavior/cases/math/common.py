"""Helpers of the `math` family inputs."""

from cases.helpers import *


def box_bits(min_x, min_y, min_z, max_x, max_y, max_z):
    return {"min_x": float_bits(min_x, "f64"), "min_y": float_bits(min_y, "f64"),
            "min_z": float_bits(min_z, "f64"), "max_x": float_bits(max_x, "f64"),
            "max_y": float_bits(max_y, "f64"), "max_z": float_bits(max_z, "f64")}


def vector2_bits(x, y, width="f32"):
    return {"x": float_bits(x, width), "y": float_bits(y, width)}


def raw_box(min_x, min_y, min_z, max_x, max_y, max_z):
    return {"min_x": min_x, "min_y": min_y, "min_z": min_z,
            "max_x": max_x, "max_y": max_y, "max_z": max_z}


def neighbors(value, width):
    bits = int(float_bits(value, width), 16)
    digits = 8 if width == "f32" else 16
    return [f"{bits + offset:0{digits}x}" for offset in (-1, 0, 1)]
