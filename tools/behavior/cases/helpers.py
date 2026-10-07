"""Original synthetic inputs only. Expected values always come from the Java JAR."""

import struct


def float_bits(value, width="f32"):
    return struct.pack(">f" if width == "f32" else ">d", value).hex()


def vector_bits(x, y, z, width="f64"):
    return {"x": float_bits(x, width), "y": float_bits(y, width),
            "z": float_bits(z, width)}
