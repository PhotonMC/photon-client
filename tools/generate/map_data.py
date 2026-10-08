"""Generate crates/photon-data/src/generated/map_data.rs from an entity-family capture.

    python3 tools/generate/map_data.py .data/captures/behavior/<id>/outputs.jsonl

The map's tables: the ARGB colour of each of the 256 packed colours (`MapColor.getColorFromPackedId`)
and the decoration types by the id the server numbers them with.
"""
from common import header, load_rows, run, rust_bool, rust_str, write


def generate(capture):
    rows = load_rows(capture, "entity/map-data")
    colors = next(row for row in rows if row["label"] == "colors")["colors"]
    types = sorted((row for row in rows if row["label"] != "colors"), key=lambda row: row["id"])
    out = header("The map's colours and decoration types.", "map_data.py") + [
        "use crate::map::MapDecorationType;",
        "",
        "/// The ARGB colour of each packed map colour (`id << 2 | brightness`).",
        "#[rustfmt::skip]",
        "pub static MAP_COLORS: [u32; 256] = [",
    ]
    for start in range(0, 256, 8):
        out.append("    " + ", ".join(f"0x{value & 0xFFFFFFFF:08X}" for value in colors[start:start + 8]) + ",")
    out += ["];", "", "#[rustfmt::skip]",
            f"pub static MAP_DECORATION_TYPES: [MapDecorationType; {len(types)}] = ["]
    for row in types:
        out.append(
            f"    MapDecorationType {{ name: {rust_str(row['label'])}, asset: {rust_str(row['asset'])}, "
            f"texture: {rust_str('map/decorations/' + row['asset'].split(':', 1)[1])}, "
            f"show_on_item_frame: {rust_bool(row['show_on_item_frame'])}, map_color: {row['map_color']}, "
            f"track_count: {rust_bool(row['track_count'])} }},")
    out.append("];")
    write("map_data.rs", out)
    print(f"map_decoration_types={len(types)}")


if __name__ == "__main__":
    run(generate)
