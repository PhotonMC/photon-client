//! Cases for `vec3/rotate`, `vec3/direction-from-rotation`, `vec3/local-coordinates`, `vec3/rotation` and more.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::case::case;
use crate::math::values::*;

fn x_signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/rotate/x-signed-zero",
        "math",
        "vec3_rotate",
        json!({
            "axis": "x",
            "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"},
            "radians": "80000000",
        }),
    ));
    out.push(case(
        "vec3/rotate/y-signed-zero",
        "math",
        "vec3_rotate",
        json!({
            "axis": "y",
            "first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"},
            "radians": "00000000",
        }),
    ));
    out.push(case(
        "vec3/rotate/z-signed-zero",
        "math",
        "vec3_rotate",
        json!({
            "axis": "z",
            "first": {"x": "0000000000000000", "y": "8000000000000000", "z": "0000000000000000"},
            "radians": "80000000",
        }),
    ));
    out.push(case(
        "vec3/rotate/x-special-vector",
        "math",
        "vec3_rotate",
        json!({
            "axis": "x",
            "first": {"x": "7ff0000000000000", "y": "7ff8000000000001", "z": "fff0000000000000"},
            "radians": f32_bits(0.5),
        }),
    ));
    out
}

fn y_special_vector() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/rotate/y-special-vector",
        "math",
        "vec3_rotate",
        json!({
            "axis": "y",
            "first": {"x": "fff0000000000000", "y": "7ff0000000000000", "z": "7ff8000000000001"},
            "radians": f32_bits(-0.75),
        }),
    ));
    out.push(case(
        "vec3/rotate/z-special-vector",
        "math",
        "vec3_rotate",
        json!({
            "axis": "z",
            "first": {"x": "7ff8000000000001", "y": "7ff0000000000000", "z": "fff0000000000000"},
            "radians": f32_bits(1.25),
        }),
    ));
    out.push(case(
        "vec3/rotate/x-nan-angle",
        "math",
        "vec3_rotate",
        json!({"axis": "x", "first": vector_bits(1.25, -2.5, 3.75), "radians": "7fc00001"}),
    ));
    out.push(case(
        "vec3/rotate/y-positive-infinity-angle",
        "math",
        "vec3_rotate",
        json!({"axis": "y", "first": vector_bits(-4.5, 6.25, -8.75), "radians": "7f800000"}),
    ));
    out.push(case(
        "vec3/rotate/z-negative-infinity-angle",
        "math",
        "vec3_rotate",
        json!({"axis": "z", "first": vector_bits(2.0, -3.0, 4.0), "radians": "ff800000"}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/components-basic",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": f32_bits(30.0), "rot_y": f32_bits(45.0)}),
    ));
    out
}

fn vec2_basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/direction-from-rotation/vec2-basic",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "vec2", "rotation": vector2_bits(20.0, -35.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/components-zero",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": f32_bits(0.0), "rot_y": f32_bits(0.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/vec2-yaw-quarter",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "vec2", "rotation": vector2_bits(0.0, 90.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/components-yaw-negative",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": f32_bits(0.0), "rot_y": f32_bits(-90.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/vec2-yaw-half",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "vec2", "rotation": vector2_bits(0.0, 180.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/components-pitch-up",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": f32_bits(90.0), "rot_y": f32_bits(0.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/vec2-pitch-down",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "vec2", "rotation": vector2_bits(-90.0, 0.0)}),
    ));
    out
}

fn components_combined() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/direction-from-rotation/components-combined",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": f32_bits(37.5), "rot_y": f32_bits(-123.25)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/vec2-wrap",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "vec2", "rotation": vector2_bits(360.0, 720.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/components-small-quantized",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": f32_bits(0.0001), "rot_y": f32_bits(-0.0001)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/vec2-signed-zero",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "vec2", "rotation": {"x": "80000000", "y": "00000000"}}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/components-signed-zero",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": "00000000", "rot_y": "80000000"}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/components-nan-pitch",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": "7fc00001", "rot_y": f32_bits(45.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/vec2-nan-yaw",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "vec2", "rotation": {"x": f32_bits(30.0), "y": "7fc00001"}}),
    ));
    out
}

fn components_positive_infinity_pitch() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/direction-from-rotation/components-positive-infinity-pitch",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": "7f800000", "rot_y": f32_bits(45.0)}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/vec2-negative-infinity-yaw",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "vec2", "rotation": {"x": f32_bits(30.0), "y": "ff800000"}}),
    ));
    out.push(case(
        "vec3/direction-from-rotation/components-max-finite",
        "math",
        "vec3_direction_from_rotation",
        json!({"kind": "components", "rot_x": "7f7fffff", "rot_y": "7f7fffff"}),
    ));
    out.push(case(
        "vec3/local-coordinates/apply-basic",
        "math",
        "vec3_local_coordinates",
        json!({"kind": "apply", "rotation": vector2_bits(0.0, 0.0), "direction": vector_bits(1.25, -2.5, 3.75)}),
    ));
    out.push(case(
        "vec3/local-coordinates/apply-combined",
        "math",
        "vec3_local_coordinates",
        json!({"kind": "apply", "rotation": vector2_bits(20.0, -35.0), "direction": vector_bits(-4.5, 6.25, 8.75)}),
    ));
    out.push(case(
        "vec3/local-coordinates/apply-axis-turns",
        "math",
        "vec3_local_coordinates",
        json!({"kind": "apply", "rotation": vector2_bits(90.0, 180.0), "direction": vector_bits(1.0, 2.0, -3.0)}),
    ));
    out
}

fn apply_signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/local-coordinates/apply-signed-zero",
        "math",
        "vec3_local_coordinates",
        json!({
            "kind": "apply",
            "rotation": {"x": "80000000", "y": "00000000"},
            "direction": {"x": "8000000000000000", "y": "0000000000000001", "z": "8000000000000001"},
        }),
    ));
    out.push(case(
        "vec3/local-coordinates/add-axis",
        "math",
        "vec3_local_coordinates",
        json!({"kind": "add", "first": vector_bits(0.0, 0.0, 1.0), "direction": vector_bits(1.0, 2.0, 3.0)}),
    ));
    out.push(case(
        "vec3/local-coordinates/add-combined",
        "math",
        "vec3_local_coordinates",
        json!({"kind": "add", "first": vector_bits(-4.0, 5.0, -6.0), "direction": vector_bits(-0.5, 1.25, -2.75)}),
    ));
    out.push(case("vec3/rotation/basic", "math", "vec3_rotation", json!({"first": vector_bits(1.0, -2.0, 3.0)})));
    out.push(case("vec3/rotation/x-positive", "math", "vec3_rotation", json!({"first": vector_bits(1.0, 0.0, 0.0)})));
    out.push(case("vec3/rotation/x-negative", "math", "vec3_rotation", json!({"first": vector_bits(-1.0, 0.0, 0.0)})));
    out.push(case("vec3/rotation/z-positive", "math", "vec3_rotation", json!({"first": vector_bits(0.0, 0.0, 1.0)})));
    out.push(case("vec3/rotation/z-negative", "math", "vec3_rotation", json!({"first": vector_bits(0.0, 0.0, -1.0)})));
    out.push(case("vec3/rotation/y-positive", "math", "vec3_rotation", json!({"first": vector_bits(0.0, 1.0, 0.0)})));
    out.push(case("vec3/rotation/y-negative", "math", "vec3_rotation", json!({"first": vector_bits(0.0, -1.0, 0.0)})));
    out.push(case("vec3/rotation/combined", "math", "vec3_rotation", json!({"first": vector_bits(1.0, 2.0, 3.0)})));
    out.push(case(
        "vec3/rotation/negative-combined",
        "math",
        "vec3_rotation",
        json!({"first": vector_bits(-4.0, 5.0, -6.0)}),
    ));
    out.push(case("vec3/rotation/non-unit", "math", "vec3_rotation", json!({"first": vector_bits(0.5, -1.5, 2.25)})));
    out.push(case("vec3/rotation/zero", "math", "vec3_rotation", json!({"first": vector_bits(0.0, 0.0, 0.0)})));
    out.push(case(
        "vec3/rotation/signed-zero",
        "math",
        "vec3_rotation",
        json!({"first": {"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"}}),
    ));
    out
}

fn signed_zero_atan2() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/rotation/signed-zero-atan2",
        "math",
        "vec3_rotation",
        json!({"first": {"x": "0000000000000000", "y": "8000000000000000", "z": "8000000000000000"}}),
    ));
    out.push(case(
        "vec3/rotation/subnormal",
        "math",
        "vec3_rotation",
        json!({"first": {"x": "0000000000000001", "y": "0000000000000001", "z": "0000000000000001"}}),
    ));
    out.push(case(
        "vec3/rotation/subnormal-normal",
        "math",
        "vec3_rotation",
        json!({"first": {"x": "0000000000000001", "y": "3ff0000000000000", "z": "0000000000000000"}}),
    ));
    out.push(case(
        "vec3/rotation/max-finite",
        "math",
        "vec3_rotation",
        json!({"first": {"x": "7fefffffffffffff", "y": "7fefffffffffffff", "z": "7fefffffffffffff"}}),
    ));
    out.push(case(
        "vec3/rotation/infinities",
        "math",
        "vec3_rotation",
        json!({"first": {"x": "7ff0000000000000", "y": "3ff0000000000000", "z": "fff0000000000000"}}),
    ));
    out.push(case(
        "vec3/rotation/nan",
        "math",
        "vec3_rotation",
        json!({"first": {"x": "7ff8000000000001", "y": "3ff0000000000000", "z": "0000000000000000"}}),
    ));
    for name in ["zero", "x_axis", "y_axis", "z_axis"] {
        out.push(case(format!("vec3/constants/{name}"), "math", "vec3_constant", json!({"name": name})));
    }
    out
}

pub fn cases() -> Vec<Value> {
    [
        x_signed_zero(),
        y_special_vector(),
        vec2_basic(),
        components_combined(),
        components_positive_infinity_pitch(),
        apply_signed_zero(),
        signed_zero_atan2(),
    ]
    .concat()
}
