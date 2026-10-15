//! Cases for `sliderable-enum/graphics-presets`, `cycleable-enum`, `alt-enum`, `lazy-enum` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;

/// Slider positions that cross every bucket edge and the non-finite values.
const EDGE_SLIDERS: [f64; 10] = [f64::NEG_INFINITY, -0.2, 0.0, 0.1, 0.5, 0.99999, 1.0, 1.5, f64::INFINITY, f64::NAN];

fn bits(values: &[f64]) -> Vec<String> {
    values.iter().copied().map(f64_bits).collect()
}

fn sliderable_graphics_presets() -> Value {
    let sliders = [
        f64::NEG_INFINITY,
        -0.0,
        0.0,
        0.124999999999,
        0.125,
        0.249999999999,
        0.25,
        0.5,
        0.75,
        0.99999,
        1.0,
        1.5,
        f64::INFINITY,
        f64::NAN,
    ];
    let probes = ["FAST", "FANCY", "FABULOUS", "CUSTOM"].map(|value| json!({"value": value}));
    case(
        "sliderable-enum/graphics-presets",
        "math",
        "sliderable_enum",
        json!({
            "probes": probes,
            "sliders": bits(&sliders),
        }),
    )
}

fn cycleable(label: &str, values: &[&str], probes: &[&str]) -> Value {
    case(format!("cycleable-enum/{label}"), "math", "cycleable_enum", json!({"values": values, "probes": probes}))
}

fn alternate(label: &str, values: &[&str], alt_values: &[&str], conditions: &[bool], probes: &[&str]) -> Value {
    case(
        format!("alt-enum/{label}"),
        "math",
        "alt_enum",
        json!({"values": values, "alt_values": alt_values, "conditions": conditions, "probes": probes}),
    )
}

fn lazy(label: &str, value_lists: &[&[&str]], valid_values: &[&str], probes: &[&str]) -> Value {
    case(
        format!("lazy-enum/{label}"),
        "math",
        "lazy_enum",
        json!({"value_lists": value_lists, "valid_values": valid_values, "probes": probes}),
    )
}

fn clamping_lazy_max_int_range() -> Value {
    case(
        "clamping-lazy-max-int-range/dynamic-upper-bound",
        "math",
        "clamping_lazy_max_int_range",
        json!({
            "min": -2,
            "max_values": [3, 5],
            "encodable_max": 7,
            "probes": [-3, -2, -1, 0, 3, 5, 6],
            "sliders": bits(&EDGE_SLIDERS),
        }),
    )
}

pub fn cases() -> Vec<Value> {
    vec![
        sliderable_graphics_presets(),
        cycleable("membership-and-order", &["off", "fast", "fancy", "fabulous"], &["off", "fabulous", "missing", ""]),
        cycleable("duplicate-values", &["first", "first", "last"], &["first", "last", "outside"]),
        alternate(
            "condition-switch",
            &["default", "shared", "only-default"],
            &["alternate", "shared", "only-alternate"],
            &[false, true, false, true],
            &["default", "alternate", "shared", "only-alternate", "missing"],
        ),
        alternate(
            "duplicates-and-empty-alternative",
            &["first", "first", "last"],
            &[],
            &[true, false, true, false],
            &["first", "last", "outside"],
        ),
        lazy(
            "dynamic-values",
            &[&[""], &["", "speakers"], &["", "headphones"]],
            &["", "speakers"],
            &["", "speakers", "headphones", "missing"],
        ),
        lazy(
            "validation-independent-of-list",
            &[&["default", "alternate"], &["default"]],
            &["default", "external"],
            &["default", "alternate", "external"],
        ),
        clamping_lazy_max_int_range(),
    ]
}
