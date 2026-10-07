"""The `json` family: original synthetic inputs."""

from cases.helpers import *


def json_cases():
    """Gson in strict mode: what the reference accepts as a model file.

    The client reads every model, blockstate and pack file through a reader
    with `Strictness.STRICT`, so the leniencies Gson allows elsewhere - single
    quotes, comments, trailing commas, unquoted keys, NaN - are all rejections
    here. The rows below cover the accepted grammar, the number literals kept
    verbatim, escape and surrogate handling, and the shapes that must fail.
    """
    accepted = [
        ("empty-object", "{}"),
        ("empty-array", "[]"),
        ("flat-object", '{"a":1,"b":"two","c":true,"d":null}'),
        ("nested", '{"a":{"b":[1,2,{"c":3}]}}'),
        ("model-shaped", '{"parent":"minecraft:block/cube_all",'
                         '"textures":{"all":"minecraft:block/stone"}}'),
        ("whitespace", ' \t\r\n { "a" : [ 1 , 2 ] } \n '),
        # Key order is preserved, and a repeated key keeps the last value.
        ("key-order", '{"z":1,"a":2,"m":3}'),
        ("duplicate-key", '{"a":1,"a":2}'),
        # Does the repeated key keep its original slot or move to the end?
        ("duplicate-key-order", '{"a":1,"b":2,"a":3}'),
        # Number literals are kept as written until a caller asks for a type.
        ("integers", '[0,-1,42,1000000]'),
        ("decimals", '[1.5,-0.25,0.0,100.125]'),
        ("exponents", '[1e3,1E3,1e+3,1e-3,1.5e10,-2.5E-7]'),
        ("large-number", '[9007199254740993,1e400]'),
        ("negative-zero", "[-0,-0.0]"),
        ("deep-nesting", "[" * 20 + "1" + "]" * 20),
        ("unicode-escape", '["\\u0041\\u00e7\\u4e2d"]'),
        ("surrogate-pair", '["\\ud83d\\ude00"]'),
        ("lone-surrogate", '["\\ud83d"]'),
        ("escapes", '["\\"\\\\\\/\\b\\f\\n\\r\\t"]'),
        ("raw-utf8", '["\u00e7\u4e2d\U0001f600"]'),
        ("empty-string-key", '{"":1}'),
        ("string-with-space", '{"a b":"c d"}'),
        ("top-level-string", '"bare"'),
        ("top-level-number", "42"),
        ("top-level-true", "true"),
        ("top-level-null", "null"),
    ]
    rejected = [
        ("trailing-comma-object", '{"a":1,}'),
        ("trailing-comma-array", "[1,2,]"),
        ("single-quotes", "{'a':1}"),
        ("unquoted-key", "{a:1}"),
        ("line-comment", '{"a":1} // note'),
        ("block-comment", '{/* note */"a":1}'),
        ("nan", "[NaN]"),
        ("infinity", "[Infinity]"),
        ("leading-plus", "[+1]"),
        ("leading-zero", "[01]"),
        ("hex-number", "[0x10]"),
        ("bare-decimal-point", "[.5]"),
        ("trailing-decimal-point", "[1.]"),
        ("unterminated-string", '{"a":"b}'),
        ("unterminated-object", '{"a":1'),
        ("bad-escape", '["\\x41"]'),
        ("raw-control-char", '["a\u0001b"]'),
        ("trailing-content", '{"a":1} {"b":2}'),
        ("trailing-garbage", "[1] x"),
        ("empty-input", ""),
        ("only-whitespace", "   "),
        ("missing-colon", '{"a" 1}'),
        ("missing-comma", '{"a":1 "b":2}'),
        ("double-comma", "[1,,2]"),
    ]

    literals = [
        # plain integers, including the boundaries of each width
        "0", "-0", "1", "-1", "42", "2147483647", "-2147483648",
        "2147483648", "-2147483649", "9223372036854775807", "-9223372036854775808",
        "9223372036854775808", "9007199254740993", "123456789012345678901234567890",
        # decimals, where an integer request has to drop the fraction
        "1.0", "1.5", "-1.5", "0.0", "-0.0", "2.5", "-2.5", "0.1",
        "16.0", "0.0625", "-0.0625", "1.9999999", "2147483647.5",
        # exponents, including values that overflow one width but not another
        "1e0", "1E0", "1e1", "1e-1", "1e10", "1e38", "1e39", "1e308", "1e309",
        "1e400", "-1e400", "1e-45", "1e-46", "1e-324", "3.4028235e38",
        "3.4028236e38", "1.7976931348623157e308",
        # the smallest steps a float can still tell apart
        "0.1000000014901161", "16777217", "16777216",
        # an exponent that moves the point back across the digits
        "12345e-3", "12345e-5", "12345e-6", "0.0000001e10", "123e20",
        "1e18", "1e19", "1e20", "999999999999999999999e-3",
        # long digit strings, where the low bits have to survive the wrap
        "18446744073709551616", "18446744073709551617",
        "340282366920938463463374607431768211456",
        "-123456789012345678901234567890", "-9007199254740993",
    ]
    yield {
        "case_id": "json/number",
        "family": "json",
        "operation": "number",
        "rows": [{"label": literal, "literal": literal} for literal in literals],
    }

    yield {
        "case_id": "json/parse",
        "family": "json",
        "operation": "parse",
        "rows": [
            {"label": label, "text": text.encode("utf-8").hex()}
            for label, text in accepted + rejected
        ],
    }
