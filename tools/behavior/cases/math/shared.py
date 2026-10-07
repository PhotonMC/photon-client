"""The `math` family: what several `all` cases share."""

import struct
from cases.math.common import *


threshold = struct.unpack(">d", struct.pack(">d", struct.unpack(">f", struct.pack(">f", 1e-5))[0]))[0]

def block_position(x, y, z):
    return {"x": str(x), "y": str(y), "z": str(z)}

def point(x, y, z):
    return {"x": float_bits(x, "f64"), "y": float_bits(y, "f64"), "z": float_bits(z, "f64")}
