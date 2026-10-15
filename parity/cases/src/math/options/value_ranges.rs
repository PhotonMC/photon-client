//! Cases for `option-values`, `mapped-int-range`, `mapped-unit-double`, `scroll-wheel` and more.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;

const EDGE_SLIDERS: [f64; 10] = [f64::NEG_INFINITY, -0.2, 0.0, 0.1, 0.5, 0.99999, 1.0, 1.5, f64::INFINITY, f64::NAN];

fn bits(values: &[f64]) -> Vec<String> {
    values.iter().copied().map(f64_bits).collect()
}

/// One `option_values` scenario: the integer range, its probes and the slider positions.
struct OptionValues {
    label: &'static str,
    range: (i64, i64),
    apply_value_immediately: bool,
    int_values: &'static [i64],
    sliders: &'static [f64],
    unit_values: &'static [f64],
}

const OPTION_VALUES: [OptionValues; 3] = [
    OptionValues {
        label: "zero-to-ten",
        range: (0, 10),
        apply_value_immediately: true,
        int_values: &[-1, 0, 1, 5, 10, 11],
        sliders: &[f64::NEG_INFINITY, -0.2, 0.0, 0.05, 0.5, 0.99999, 1.0, 1.5, f64::INFINITY, f64::NAN],
        unit_values: &[-0.1, -0.0, 0.0, 0.5, 1.0, 1.1, f64::INFINITY, f64::NAN],
    },
    OptionValues {
        label: "negative-range",
        range: (-3, 3),
        apply_value_immediately: false,
        int_values: &[-4, -3, -2, 0, 3, 4],
        sliders: &[-0.5, 0.0, 0.25, 0.5, 0.75, 1.0],
        unit_values: &[0.0, 0.25, 1.0],
    },
    OptionValues {
        label: "integer-edge",
        range: (2147483646, 2147483647),
        apply_value_immediately: true,
        int_values: &[2147483646, 2147483647],
        sliders: &[0.0, 0.5, 0.99999, 1.0],
        unit_values: &[0.0, 1.0],
    },
];

fn option_values() -> Vec<Value> {
    OPTION_VALUES
        .iter()
        .map(|scenario| {
            case(
                format!("option-values/{}", scenario.label),
                "math",
                "option_values",
                json!({
                    "min": scenario.range.0,
                    "max": scenario.range.1,
                    "apply_value_immediately": scenario.apply_value_immediately,
                    "int_values": scenario.int_values,
                    "sliders": bits(scenario.sliders),
                    "unit_values": bits(scenario.unit_values),
                }),
            )
        })
        .collect()
}

fn mapped_int_range(label: &str, range: (i64, i64), discrete: bool, values: &[&str], sliders: &[f64]) -> Value {
    case(
        format!("mapped-int-range/{label}"),
        "math",
        "mapped_int_range",
        json!({
            "min": range.0,
            "max": range.1,
            "discrete": discrete,
            "values": values,
            "sliders": bits(sliders),
        }),
    )
}

fn mapped_unit_double(label: &str, values: &[f64], sliders: &[f64]) -> Value {
    case(
        format!("mapped-unit-double/{label}"),
        "math",
        "mapped_unit_double",
        json!({"values": bits(values), "sliders": bits(sliders)}),
    )
}

fn scroll_wheel(label: &str, scrolls: &[(f64, f64)], selections: &[(f64, i64, i64)]) -> Value {
    case(
        format!("scroll-wheel/{label}"),
        "math",
        "scroll_wheel",
        json!({
            "scrolls": scrolls
                .iter()
                .map(|(x, y)| json!({"x": f64_bits(*x), "y": f64_bits(*y)}))
                .collect::<Vec<_>>(),
            "selections": selections
                .iter()
                .map(|(wheel, current, limit)| {
                    json!({"wheel": f64_bits(*wheel), "current": current, "limit": limit})
                })
                .collect::<Vec<_>>(),
        }),
    )
}

/// A smoothing step: `Some((target_delta, time))` advances it, `None` resets it.
fn smooth_double(label: &str, steps: &[Option<(f64, f64)>]) -> Value {
    let steps: Vec<Value> = steps
        .iter()
        .map(|step| match step {
            Some((target_delta, time)) => {
                json!({"op": "delta", "target_delta": f64_bits(*target_delta), "time": f64_bits(*time)})
            }
            None => json!({"op": "reset"}),
        })
        .collect();
    case(format!("smooth-double/{label}"), "math", "smooth_double", json!({"steps": steps}))
}

fn mapped_ranges() -> Vec<Value> {
    let unit_edges = [f64::NEG_INFINITY, -1.0, -0.0, 0.0, 0.25, 0.5, 1.0, 1.0000000001, f64::INFINITY, f64::NAN];
    let slider_edges = [f64::NEG_INFINITY, -0.25, -0.0, 0.0, 0.25, 0.5, 1.0, 1.0000000001, f64::INFINITY, f64::NAN];
    let rounding = [5e-324, 0.1, 0.3333333333333333, 0.9999999999999999];
    vec![
        mapped_int_range("discrete", (-2, 3), true, &["-3", "-2", "-1", "0", "2", "3", "4"], &EDGE_SLIDERS),
        mapped_int_range("continuous", (0, 4), false, &["-1", "0", "2", "4", "5"], &[-0.25, 0.0, 0.2, 0.5, 1.0]),
        mapped_unit_double("square-root", &unit_edges, &slider_edges),
        mapped_unit_double("rounding-and-subnormal", &rounding, &rounding),
    ]
}

fn scroll_wheels() -> Vec<Value> {
    vec![
        scroll_wheel(
            "fractional-and-both-axes",
            &[(0.4, 0.0), (0.4, 0.6), (0.4, 0.6), (-0.2, -0.6), (-1.0, -1.0)],
            &[
                (1.0, 0, 9),
                (-1.0, 0, 9),
                (0.0, 4, 9),
                (f64::NAN, 4, 9),
                (f64::INFINITY, 0, 9),
                (f64::NEG_INFINITY, 0, 9),
            ],
        ),
        scroll_wheel(
            "negative-and-selection-wrap",
            &[(-0.6, -0.6), (-0.6, -0.6), (-0.8, -0.8), (0.1, 0.1), (1.0, 0.0)],
            &[(1.0, 0, 9), (-1.0, 0, 9), (1.0, 8, 9), (-1.0, 0, 9), (f64::NAN, -1, 9), (f64::INFINITY, 8, 9)],
        ),
    ]
}

fn smooth_doubles() -> Vec<Value> {
    vec![
        smooth_double(
            "positive-negative",
            &[Some((1.0, 1.0)), Some((1.0, 0.5)), Some((-0.25, 1.0)), Some((0.0, 2.0)), Some((-2.0, 0.25))],
        ),
        smooth_double("reset", &[Some((4.0, 0.25)), None, Some((-1.0, 1.0)), Some((0.5, 0.0)), None, Some((0.0, 1.0))]),
        smooth_double("signed-zero", &[Some((-0.0, 1.0)), Some((0.0, -0.0)), Some((-1.0, -0.5))]),
        smooth_double(
            "special-values",
            &[
                Some((f64::NAN, 1.0)),
                Some((1.0, 1.0)),
                None,
                Some((f64::INFINITY, 0.5)),
                Some((f64::NEG_INFINITY, 1.0)),
            ],
        ),
    ]
}

pub fn cases() -> Vec<Value> {
    [option_values(), mapped_ranges(), scroll_wheels(), smooth_doubles()].concat()
}
