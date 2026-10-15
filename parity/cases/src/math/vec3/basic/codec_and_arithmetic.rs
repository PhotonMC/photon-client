//! Cases for `vec3/codec`, `vec3/normalize`, `vec3/chain`, `vec3/reverse` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;
use crate::math::values::*;

fn decode() -> Vec<Value> {
    let mut out = Vec::new();
    let codec_decodes = codec_decodes_rows();
    for (label, values) in codec_decodes.iter() {
        out.push(case(
            format!("vec3/codec/decode/{}", label),
            "math",
            "vec3_codec",
            json!({"mode": "decode", "values": values.clone()}),
        ));
    }
    let codec_encodes = [
        ("basic", vector_bits(1.25, -2.5, 3.75)),
        ("zero", vector_bits(0.0, 0.0, 0.0)),
        ("signed-zero", json!({"x": "8000000000000000", "y": "0000000000000000", "z": "8000000000000000"})),
        ("special", json!({"x": "7ff8000000000002", "y": "7ff0000000000000", "z": "fff0000000000000"})),
    ];
    for (label, values) in codec_encodes.iter() {
        out.push(case(
            format!("vec3/codec/encode/{}", label),
            "math",
            "vec3_codec",
            merged(vec![json!({"mode": "encode"}), values.clone()]),
        ));
    }
    let mut vectors3: Vec<(String, (String, String, String))> = Vec::new();
    for x in [-1, 0, 1] {
        for y in [-1, 0, 1] {
            for z in [-1, 0, 1] {
                vectors3.push((format!("direction-{x}-{y}-{z}"), (f64_bits(x), f64_bits(y), f64_bits(z))));
            }
        }
    }
    let named = [
        ("negative-zero", ["8000000000000000"; 3].map(str::to_owned)),
        ("subnormal", ["0000000000000001"; 3].map(str::to_owned)),
        ("max-finite", ["7fefffffffffffff"; 3].map(str::to_owned)),
        ("unequal", [f64_bits(3), f64_bits(-7), f64_bits(0.5)]),
    ];
    for (label, [x, y, z]) in named {
        vectors3.push((label.to_owned(), (x, y, z)));
    }
    for (index, bits) in neighbors(threshold(), "f64").into_iter().enumerate() {
        let zero = "0000000000000000".to_owned();
        vectors3.push((format!("normalization-boundary-{index}"), (bits, zero.clone(), zero)));
    }
    for (label, (x, y, z)) in vectors3 {
        out.push(case(
            format!("vec3/normalize/{}", label),
            "math",
            "vec3",
            json!({"x": x.clone(), "y": y.clone(), "z": z.clone(), "steps": [{"action": "normalize"}]}),
        ));
    }
    out
}

fn codec_decodes_rows() -> Vec<(&'static str, Vec<std::string::String>)> {
    vec![
        ("basic", vec![f64_bits(1.25), f64_bits(-2.5), f64_bits(3.75)]),
        ("zero", vec![f64_bits(0.0), f64_bits(0.0), f64_bits(0.0)]),
        (
            "signed-zero",
            vec!["8000000000000000".to_string(), "0000000000000000".to_string(), "8000000000000000".to_string()],
        ),
        (
            "subnormal-and-large",
            vec!["0000000000000001".to_string(), "7fefffffffffffff".to_string(), "8000000000000001".to_string()],
        ),
        (
            "special",
            vec!["7ff8000000000002".to_string(), "7ff0000000000000".to_string(), "fff0000000000000".to_string()],
        ),
        ("empty", vec![]),
        ("short", vec![f64_bits(1.0), f64_bits(2.0)]),
        ("long", vec![f64_bits(1.0), f64_bits(2.0), f64_bits(3.0), f64_bits(4.0)]),
    ]
}

fn chain() -> Vec<Value> {
    let mut out = Vec::new();
    for factor in [-2.0, -0.0, 0.0, 0.37, 0.98, 3.0] {
        out.push(case(
            format!("vec3/chain/{}", f64_bits(factor)),
            "math",
            "vec3",
            json!({
                "x": f64_bits(1),
                "y": f64_bits(-1),
                "z": f64_bits(0.5),
                "steps": [
                    {"action": "normalize"},
                    {"action": "scale", "factor": f64_bits(factor)},
                    {"action": "normalize"},
                ],
            }),
        ));
    }
    out.push(case(
        "vec3/reverse/basic",
        "math",
        "vec3",
        json!({"x": f64_bits(1.25), "y": f64_bits(-2.5), "z": f64_bits(3.75), "steps": [{"action": "reverse"}]}),
    ));
    out.push(case(
        "vec3/reverse/chain",
        "math",
        "vec3",
        json!({
            "x": f64_bits(-4.5),
            "y": f64_bits(6.25),
            "z": f64_bits(8.75),
            "steps": [
                {"action": "reverse"},
                {"action": "add_components", "x": f64_bits(0.5), "y": f64_bits(-1.5), "z": f64_bits(-0.75)},
            ],
        }),
    ));
    out.push(case(
        "vec3/reverse/signed-zero",
        "math",
        "vec3",
        json!({"x": f64_bits(-0.0), "y": f64_bits(0.0), "z": f64_bits(-0.0), "steps": [{"action": "reverse"}]}),
    ));
    out
}

fn subnormal_and_large() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/reverse/subnormal-and-large",
        "math",
        "vec3",
        json!({
            "x": "0000000000000001",
            "y": "7fefffffffffffff",
            "z": "8000000000000001",
            "steps": [{"action": "reverse"}],
        }),
    ));
    out.push(case(
        "vec3/reverse/infinities",
        "math",
        "vec3",
        json!({
            "x": f64_bits(f64::INFINITY),
            "y": f64_bits(-2.0),
            "z": f64_bits(f64::NEG_INFINITY),
            "steps": [{"action": "reverse"}],
        }),
    ));
    out.push(case(
        "vec3/horizontal/basic",
        "math",
        "vec3",
        json!({"x": f64_bits(1.25), "y": f64_bits(-2.5), "z": f64_bits(3.75), "steps": [{"action": "horizontal"}]}),
    ));
    out.push(case(
        "vec3/horizontal/chain",
        "math",
        "vec3",
        json!({
            "x": f64_bits(-4.5),
            "y": f64_bits(6.25),
            "z": f64_bits(8.75),
            "steps": [
                {"action": "horizontal"},
                {"action": "add_components", "x": f64_bits(0.5), "y": f64_bits(-1.5), "z": f64_bits(-0.75)},
            ],
        }),
    ));
    out
}

fn signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/horizontal/signed-zero",
        "math",
        "vec3",
        json!({"x": f64_bits(-0.0), "y": f64_bits(-0.0), "z": f64_bits(-0.0), "steps": [{"action": "horizontal"}]}),
    ));
    out.push(case(
        "vec3/horizontal/subnormal-and-large",
        "math",
        "vec3",
        json!({
            "x": "0000000000000001",
            "y": "7fefffffffffffff",
            "z": "8000000000000001",
            "steps": [{"action": "horizontal"}],
        }),
    ));
    out.push(case(
        "vec3/horizontal/infinities",
        "math",
        "vec3",
        json!({
            "x": f64_bits(f64::INFINITY),
            "y": f64_bits(f64::NEG_INFINITY),
            "z": f64_bits(f64::NEG_INFINITY),
            "steps": [{"action": "horizontal"}],
        }),
    ));
    out.push(case(
        "vec3/multiply/vector-basic",
        "math",
        "vec3",
        json!({
            "x": f64_bits(1.25),
            "y": f64_bits(-2.5),
            "z": f64_bits(3.75),
            "steps": [{"action": "multiply_vec3", "scale": vector_bits(-2.0, 0.5, 4.0)}],
        }),
    ));
    out
}

fn components_basic() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/multiply/components-basic",
        "math",
        "vec3",
        json!({
            "x": f64_bits(-1.5),
            "y": f64_bits(2.25),
            "z": f64_bits(0.75),
            "steps": [{"action": "multiply_components", "x": f64_bits(0.5), "y": f64_bits(-4.0), "z": f64_bits(8.0)}],
        }),
    ));
    out.push(case(
        "vec3/multiply/overload-chain",
        "math",
        "vec3",
        json!({
            "x": f64_bits(134217728.0),
            "y": f64_bits(-134217728.0),
            "z": f64_bits(1.1),
            "steps": [
                {"action": "multiply_components", "x": f64_bits(0.5), "y": f64_bits(-0.5), "z": f64_bits(2.0)},
                {"action": "multiply_vec3", "scale": vector_bits(1.0, -2.0, 0.25)},
            ],
        }),
    ));
    out.push(case(
        "vec3/multiply/signed-zero",
        "math",
        "vec3",
        json!({
            "x": f64_bits(-0.0),
            "y": f64_bits(0.0),
            "z": f64_bits(-0.0),
            "steps": [
                {"action": "multiply_components", "x": f64_bits(-0.0), "y": f64_bits(-0.0), "z": f64_bits(0.0)},
                {"action": "multiply_vec3", "scale": vector_bits(-1.0, 1.0, -1.0)},
            ],
        }),
    ));
    out
}

fn overflow_and_special_values() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/multiply/overflow-and-special-values",
        "math",
        "vec3",
        json!({
            "x": "7fefffffffffffff",
            "y": "ffefffffffffffff",
            "z": f64_bits(1.0),
            "steps": [
                {
                    "action": "multiply_vec3",
                    "scale": {"x": f64_bits(2.0), "y": f64_bits(2.0), "z": "7ff8000000000002"},
                },
                {
                    "action": "multiply_components",
                    "x": f64_bits(1.0),
                    "y": f64_bits(-1.0),
                    "z": f64_bits(f64::INFINITY),
                },
            ],
        }),
    ));
    out.push(case(
        "vec3/add-subtract/vector-chain",
        "math",
        "vec3",
        json!({
            "x": f64_bits(1.25),
            "y": f64_bits(-2.5),
            "z": f64_bits(3.75),
            "steps": [
                {"action": "add_vec3", "delta": vector_bits(-0.25, 4.0, -1.5)},
                {"action": "subtract_vec3", "delta": vector_bits(0.5, -1.0, 2.25)},
                {"action": "normalize"},
            ],
        }),
    ));
    out
}

fn scalar_chain() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/add-subtract/scalar-chain",
        "math",
        "vec3",
        json!({
            "x": f64_bits(-1.5),
            "y": f64_bits(2.25),
            "z": f64_bits(0.75),
            "steps": [
                {"action": "add_scalar", "value": f64_bits(0.5)},
                {"action": "subtract_scalar", "value": f64_bits(-1.25)},
                {"action": "add_scalar", "value": f64_bits(-0.0)},
            ],
        }),
    ));
    out.push(case(
        "vec3/add-subtract/component-chain",
        "math",
        "vec3",
        json!({
            "x": f64_bits(10.0),
            "y": f64_bits(-20.0),
            "z": f64_bits(30.0),
            "steps": [
                {"action": "add_components", "x": f64_bits(-1.0), "y": f64_bits(2.0), "z": f64_bits(-3.0)},
                {"action": "subtract_components", "x": f64_bits(4.0), "y": f64_bits(-5.0), "z": f64_bits(6.0)},
                {"action": "add_vec3", "delta": vector_bits(0.125, -0.25, 0.5)},
                {"action": "subtract_scalar", "value": f64_bits(0.125)},
            ],
        }),
    ));
    out
}

fn add_subtract_signed_zero() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/add-subtract/signed-zero",
        "math",
        "vec3",
        json!({
            "x": f64_bits(-0.0),
            "y": f64_bits(0.0),
            "z": f64_bits(-0.0),
            "steps": [
                {"action": "add_scalar", "value": f64_bits(-0.0)},
                {"action": "subtract_components", "x": f64_bits(0.0), "y": f64_bits(-0.0), "z": f64_bits(0.0)},
                {"action": "add_vec3", "delta": vector_bits(0.0, -0.0, 0.0)},
            ],
        }),
    ));
    out.push(case(
        "vec3/add-subtract/special-values",
        "math",
        "vec3",
        json!({
            "x": f64_bits(1.0),
            "y": f64_bits(-2.0),
            "z": f64_bits(0.0),
            "steps": [
                {
                    "action": "add_vec3",
                    "delta": {"x": f64_bits(f64::INFINITY), "y": f64_bits(f64::NEG_INFINITY), "z": f64_bits(1.0)},
                },
                {"action": "subtract_components", "x": f64_bits(1.0), "y": f64_bits(-2.0), "z": "7ff8000000000002"},
            ],
        }),
    ));
    out.push(case(
        "vec3/vector-to/basic",
        "math",
        "vec3",
        json!({
            "x": f64_bits(1.25),
            "y": f64_bits(-2.5),
            "z": f64_bits(3.75),
            "steps": [{"action": "vector_to", "target": vector_bits(4.5, -1.0, -0.25)}],
        }),
    ));
    out
}

fn vector_to_chain() -> Vec<Value> {
    let mut out = Vec::new();
    out.push(case(
        "vec3/vector-to/chain",
        "math",
        "vec3",
        json!({
            "x": f64_bits(2.0),
            "y": f64_bits(-3.0),
            "z": f64_bits(4.0),
            "steps": [
                {"action": "vector_to", "target": vector_bits(5.0, 1.0, -2.0)},
                {"action": "vector_to", "target": vector_bits(-1.0, 1.0, 3.0)},
                {"action": "add_components", "x": f64_bits(0.25), "y": f64_bits(-0.5), "z": f64_bits(0.75)},
                {"action": "normalize"},
            ],
        }),
    ));
    out.push(case(
        "vec3/vector-to/signed-zero",
        "math",
        "vec3",
        json!({
            "x": f64_bits(-0.0),
            "y": f64_bits(0.0),
            "z": f64_bits(-0.0),
            "steps": [
                {"action": "vector_to", "target": vector_bits(0.0, -0.0, 0.0)},
                {"action": "vector_to", "target": vector_bits(-0.0, 0.0, -0.0)},
            ],
        }),
    ));
    out
}

pub fn cases() -> Vec<Value> {
    [
        decode(),
        chain(),
        subnormal_and_large(),
        signed_zero(),
        components_basic(),
        overflow_and_special_values(),
        scalar_chain(),
        add_subtract_signed_zero(),
        vector_to_chain(),
    ]
    .concat()
}
