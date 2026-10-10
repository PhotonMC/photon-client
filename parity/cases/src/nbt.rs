//! The `nbt` family: network NBT writing and reading.

use serde_json::{Value, json};

/// Tag type and value of each single-field compound.
const PRIMITIVES: &[(&str, &str)] = &[
    ("byte", "127"),
    ("short", "-12345"),
    ("int", "2147483647"),
    ("long", "-9223372036854775808"),
    ("float", "3f800000"),
    ("double", "bff0000000000000"),
    ("string", "hi"),
];

const READS: &[(&str, &str)] = &[
    ("empty", "0a0055aa"),
    ("null", "0055aa"),
    ("byte", "0a0100016b010055aa"),
    ("string", "0a08000161000268690055aa"),
    ("nested", "0a0a0001630100016202000055aa"),
    (
        "list-int",
        "0a0900016c03000000030000000100000002000000030055aa",
    ),
    ("empty-list", "0a0900016500000000000055aa"),
    ("non-compound", "030102030455aa"),
    ("truncated-tag", "0a01"),
    ("truncated-body", "0a0100016b"),
    ("truncated-nested", "0a0a000163"),
    ("short-nested", "0a0a000163010001620200"),
    ("missing-payload", "0a"),
    ("empty-input", ""),
];

fn tag(kind: &str, value: Value) -> Value {
    json!({"type": kind, "value": value})
}

fn compound(key: &str, value: Value) -> Value {
    tag("compound", json!({ key: value }))
}

fn list(kind: &str, values: &[&str]) -> Value {
    let items: Vec<Value> = values.iter().map(|value| tag(kind, json!(value))).collect();
    tag("list", Value::Array(items))
}

fn trees() -> Vec<(&'static str, Value)> {
    let point = |x: &str| compound("x", tag("int", json!(x)));
    vec![
        (
            "nested",
            compound("c", compound("b", tag("byte", json!("2")))),
        ),
        ("list-int", compound("l", list("int", &["1", "-2", "300"]))),
        ("list-string", compound("l", list("string", &["a", "bc"]))),
        ("empty-list", compound("e", tag("list", json!([])))),
        (
            "byte-array",
            compound("a", tag("byte-array", json!(["0", "127", "-128"]))),
        ),
        (
            "int-array",
            compound("a", tag("int-array", json!(["0", "-1", "2147483647"]))),
        ),
        (
            "long-array",
            compound("a", tag("long-array", json!(["0", "-9223372036854775808"]))),
        ),
        (
            "list-compound",
            compound("l", tag("list", json!([point("1"), point("2")]))),
        ),
    ]
}

fn case(id: String, operation: &str, fields: Value) -> Value {
    let mut case = json!({
        "case_id": id, "family": "nbt", "operation": operation, "prefix_hex": "a55a",
    });
    if let (Value::Object(case), Value::Object(fields)) = (&mut case, fields) {
        case.extend(fields);
    }
    case
}

pub fn cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (kind, value) in PRIMITIVES {
        let field = json!({"key": "k", "type": kind, "value": value});
        cases.push(case(
            format!("nbt/write/{kind}"),
            "write",
            json!({"fields": [field]}),
        ));
    }
    cases.push(case(
        "nbt/write/empty".to_owned(),
        "write",
        json!({"fields": []}),
    ));
    let null = json!({"null_root": true, "fields": []});
    cases.push(case("nbt/write/null".to_owned(), "write", null));
    for (label, tree) in trees() {
        let id = format!("nbt/write-tree/{label}");
        cases.push(case(id, "write", json!({ "tree": tree })));
    }
    for (label, framed) in READS {
        cases.push(case(
            format!("nbt/read/{label}"),
            "read",
            json!({ "hex": framed }),
        ));
    }
    cases
}
