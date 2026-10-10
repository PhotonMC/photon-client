//! Status requests and pings.

use serde_json::{Value, json};

use super::codec::fixed_hex;
use crate::case::case;

fn wire(id: String, operation: &str, fields: Value) -> Value {
    case(id, "wire", operation, fields)
}

pub fn cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, prefix) in [("empty", ""), ("with-prefix", "deadbeef")] {
        let fields = json!({"direction": "write", "prefix_hex": prefix});
        cases.push(wire(format!("status/request/write/{label}"), "status_request", fields));
    }
    for (label, hex) in [("empty", ""), ("trailing", "aabbcc"), ("zero", "00")] {
        let fields = json!({"direction": "read", "hex": hex});
        cases.push(wire(format!("status/request/read/{label}"), "status_request", fields));
    }
    let writes = [
        ("zero", 0, "", "request"),
        ("negative-one", -1, "", "request"),
        ("minimum", i64::MIN, "dead", "response"),
        ("maximum", i64::MAX, "", "response"),
        ("pattern-request", 72623859790382856, "aa55", "request"),
        ("pattern-response", -723685415333072913, "", "response"),
    ];
    for (label, time, prefix, packet) in writes {
        let fields = json!({
            "direction": "write", "packet": packet, "time": time.to_string(), "prefix_hex": prefix,
        });
        cases.push(wire(format!("status/ping/write/{label}"), "ping", fields));
    }
    let reads = [
        ("zero", fixed_hex(0, 8)),
        ("negative-one", fixed_hex(-1, 8)),
        ("minimum", fixed_hex(i64::MIN, 8)),
        ("maximum", fixed_hex(i64::MAX, 8)),
        ("trailing", format!("{}aabb", fixed_hex(72623859790382856, 8))),
        ("underflow-seven", "00".repeat(7)),
        ("empty", String::new()),
    ];
    for packet in ["request", "response"] {
        for (label, hex) in &reads {
            let fields = json!({"direction": "read", "packet": packet, "hex": hex});
            cases.push(wire(format!("status/ping/{packet}/read/{label}"), "ping", fields));
        }
    }
    cases
}
