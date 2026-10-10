//! The `json` family: Gson in strict mode, as the reference reads models.
//!
//! The client reads every model, blockstate and pack file through a reader
//! with strict strictness, so the leniencies Gson allows elsewhere (single
//! quotes, comments, trailing commas, unquoted keys, NaN) are all rejections
//! here. The rows cover the accepted grammar, the number literals kept
//! verbatim, escape and surrogate handling, and the shapes that must fail.

use serde_json::{Value, json};

use crate::bits::hex;

const ACCEPTED: &[(&str, &str)] = &[
    ("empty-object", "{}"),
    ("empty-array", "[]"),
    ("flat-object", r#"{"a":1,"b":"two","c":true,"d":null}"#),
    ("nested", r#"{"a":{"b":[1,2,{"c":3}]}}"#),
    (
        "model-shaped",
        r#"{"parent":"minecraft:block/cube_all","textures":{"all":"minecraft:block/stone"}}"#,
    ),
    ("whitespace", " \t\r\n { \"a\" : [ 1 , 2 ] } \n "),
    // Key order is preserved, and a repeated key keeps the last value.
    ("key-order", r#"{"z":1,"a":2,"m":3}"#),
    ("duplicate-key", r#"{"a":1,"a":2}"#),
    // Does the repeated key keep its original slot or move to the end?
    ("duplicate-key-order", r#"{"a":1,"b":2,"a":3}"#),
    // Number literals are kept as written until a caller asks for a type.
    ("integers", "[0,-1,42,1000000]"),
    ("decimals", "[1.5,-0.25,0.0,100.125]"),
    ("exponents", "[1e3,1E3,1e+3,1e-3,1.5e10,-2.5E-7]"),
    ("large-number", "[9007199254740993,1e400]"),
    ("negative-zero", "[-0,-0.0]"),
];

const ACCEPTED_TEXT: &[(&str, &str)] = &[
    ("unicode-escape", r#"["\u0041\u00e7\u4e2d"]"#),
    ("surrogate-pair", r#"["\ud83d\ude00"]"#),
    ("lone-surrogate", r#"["\ud83d"]"#),
    ("escapes", r#"["\"\\\/\b\f\n\r\t"]"#),
    ("raw-utf8", "[\"\u{e7}\u{4e2d}\u{1f600}\"]"),
    ("empty-string-key", r#"{"":1}"#),
    ("string-with-space", r#"{"a b":"c d"}"#),
    ("top-level-string", r#""bare""#),
    ("top-level-number", "42"),
    ("top-level-true", "true"),
    ("top-level-null", "null"),
];

const REJECTED: &[(&str, &str)] = &[
    ("trailing-comma-object", r#"{"a":1,}"#),
    ("trailing-comma-array", "[1,2,]"),
    ("single-quotes", "{'a':1}"),
    ("unquoted-key", "{a:1}"),
    ("line-comment", r#"{"a":1} // note"#),
    ("block-comment", r#"{/* note */"a":1}"#),
    ("nan", "[NaN]"),
    ("infinity", "[Infinity]"),
    ("leading-plus", "[+1]"),
    ("leading-zero", "[01]"),
    ("hex-number", "[0x10]"),
    ("bare-decimal-point", "[.5]"),
    ("trailing-decimal-point", "[1.]"),
    ("unterminated-string", r#"{"a":"b}"#),
    ("unterminated-object", r#"{"a":1"#),
    ("bad-escape", r#"["\x41"]"#),
    ("raw-control-char", "[\"a\u{1}b\"]"),
    ("trailing-content", r#"{"a":1} {"b":2}"#),
    ("trailing-garbage", "[1] x"),
    ("empty-input", ""),
    ("only-whitespace", "   "),
    ("missing-colon", r#"{"a" 1}"#),
    ("missing-comma", r#"{"a":1 "b":2}"#),
    ("double-comma", "[1,,2]"),
];

const LITERALS: &[&str] = &[
    // Plain integers, including the boundaries of each width.
    "0",
    "-0",
    "1",
    "-1",
    "42",
    "2147483647",
    "-2147483648",
    "2147483648",
    "-2147483649",
    "9223372036854775807",
    "-9223372036854775808",
    "9223372036854775808",
    "9007199254740993",
    "123456789012345678901234567890",
    // Decimals, where an integer request has to drop the fraction.
    "1.0",
    "1.5",
    "-1.5",
    "0.0",
    "-0.0",
    "2.5",
    "-2.5",
    "0.1",
    "16.0",
    "0.0625",
    "-0.0625",
    "1.9999999",
    "2147483647.5",
    // Exponents, including values that overflow one width but not another.
    "1e0",
    "1E0",
    "1e1",
    "1e-1",
    "1e10",
    "1e38",
    "1e39",
    "1e308",
    "1e309",
    "1e400",
    "-1e400",
    "1e-45",
    "1e-46",
    "1e-324",
    "3.4028235e38",
    "3.4028236e38",
    "1.7976931348623157e308",
    // The smallest steps a float can still tell apart.
    "0.1000000014901161",
    "16777217",
    "16777216",
    // An exponent that moves the point back across the digits.
    "12345e-3",
    "12345e-5",
    "12345e-6",
    "0.0000001e10",
    "123e20",
    "1e18",
    "1e19",
    "1e20",
    "999999999999999999999e-3",
    // Long digit strings, where the low bits have to survive the wrap.
    "18446744073709551616",
    "18446744073709551617",
    "340282366920938463463374607431768211456",
    "-123456789012345678901234567890",
    "-9007199254740993",
];

pub fn cases() -> Vec<Value> {
    let numbers: Vec<Value> = LITERALS
        .iter()
        .map(|literal| json!({"label": literal, "literal": literal}))
        .collect();
    let deep = format!("{}1{}", "[".repeat(20), "]".repeat(20));
    let mut texts: Vec<(&str, &str)> = ACCEPTED.to_vec();
    texts.push(("deep-nesting", &deep));
    texts.extend(ACCEPTED_TEXT);
    texts.extend(REJECTED);
    let parses: Vec<Value> = texts
        .iter()
        .map(|(label, text)| json!({"label": label, "text": hex(text.as_bytes())}))
        .collect();
    vec![
        json!({"case_id": "json/number", "family": "json", "operation": "number", "rows": numbers}),
        json!({"case_id": "json/parse", "family": "json", "operation": "parse", "rows": parses}),
    ]
}
