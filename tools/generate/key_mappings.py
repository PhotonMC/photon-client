"""Generate crates/photon-data/src/generated/key_mappings.rs from a gui-family capture.

    python3 tools/generate/key_mappings.py .data/captures/behavior/<id>/outputs.jsonl

The key mappings of `Options` (`keyMappings` order: name, category, default key, order), the
categories in their sort order, and the keys `InputConstants` names.
"""
import json

from common import header, run, rust_str, write


def generate(capture):
    result = next(case["result"] for case in map(json.loads, open(capture, encoding="utf-8"))
                  if case["case_id"] == "gui/key-mappings")
    mappings, categories, keys = result["mappings"], result["categories"], result["keys"]
    out = header("The key mappings, their categories and the named keys.", "key_mappings.py") + [
        "/// Name, category, default key and order, in `Options.keyMappings` order.",
        "#[rustfmt::skip]",
        f"pub static KEY_MAPPINGS: [(&str, &str, &str, i32); {len(mappings)}] = [",
    ]
    for mapping in mappings:
        out.append(f"    ({rust_str(mapping['name'])}, {rust_str(mapping['category'])}, "
                   f"{rust_str(mapping['key'])}, {mapping['order']}),")
    out += ["];", "", "/// `KeyMapping.Category.SORT_ORDER`.", "#[rustfmt::skip]",
            f"pub static KEY_CATEGORIES: [&str; {len(categories)}] = ["]
    for category in categories:
        out.append(f"    {rust_str(category)},")
    out += ["];", "", "/// The keys `InputConstants` names: type, code, name.", "#[rustfmt::skip]",
            f"pub static NAMED_KEYS: [(&str, i32, &str); {len(keys)}] = ["]
    for key in keys:
        out.append(f"    ({rust_str(key['type'])}, {key['code']}, {rust_str(key['name'])}),")
    out.append("];")
    write("key_mappings.rs", out)
    print(f"key_mappings mappings={len(mappings)} categories={len(categories)} keys={len(keys)}")


if __name__ == "__main__":
    run(generate)
