"""The `block` family: original synthetic inputs."""

from cases.helpers import *


def block_cases():
    """The block registry, read from the reference rather than rebuilt.

    The input is tiny because the oracle enumerates the registry itself; the
    expected output is the whole table, which is what the Rust side has to
    reproduce."""
    yield {
        "case_id": "block/state-definitions",
        "family": "block",
        "operation": "state_definitions",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/declarations",
        "family": "block",
        "operation": "block_declarations",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/item-declarations",
        "family": "block",
        "operation": "item_declarations",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/collision-shapes",
        "family": "block",
        "operation": "collision_shapes",
        "rows": [{"label": "registry"}],
    }

    yield {
        "case_id": "block/outline-shapes",
        "family": "block",
        "operation": "outline_shapes",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/movement-properties",
        "family": "block",
        "operation": "movement_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/offset-properties",
        "family": "block",
        "operation": "offset_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/fluid-overlay-blocks",
        "family": "block",
        "operation": "fluid_overlay_blocks",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/motion-blocking",
        "family": "block",
        "operation": "motion_blocking",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/fluid-flow-properties",
        "family": "block",
        "operation": "fluid_flow_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/light-properties",
        "family": "block",
        "operation": "light_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/face-occlusion",
        "family": "block",
        "operation": "face_occlusion",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/destroy-properties",
        "family": "block",
        "operation": "destroy_properties",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/item-tools",
        "family": "block",
        "operation": "item_tools",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/item-stacking",
        "family": "block",
        "operation": "item_stacking",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/item-components",
        "family": "block",
        "operation": "item_components",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/tooltip-registries",
        "family": "block",
        "operation": "tooltip_registries",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/potions",
        "family": "block",
        "operation": "potions",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/creative-tabs",
        "family": "block",
        "operation": "creative_tabs",
        "rows": [{"label": "registry"}],
    }
    # The search box's text: path text, a namespace and a path (trimmed),
    # nothing after '#', upper case (the tree lowercases, the tooltip tags do
    # not), an untrimmed path, and nothing found.
    yield {
        "case_id": "block/creative-tag-search",
        "family": "block",
        "operation": "creative_tag_search",
        "rows": [
            {"label": label, "text": text}
            for label, text in (
                ("path", "#logs"),
                ("namespace-path", "#minecraft:logs"),
                ("trimmed", "# mine : wool "),
                ("namespace-only", "#minecraft:"),
                ("empty", "#"),
                ("upper", "#Logs"),
                ("untrimmed", "#wool "),
                ("nested-path", "#enchantable/sw"),
                ("none", "#qqq"),
            )
        ],
    }
    yield {
        "case_id": "block/behaviours",
        "family": "block",
        "operation": "block_behaviours",
        "rows": [{"label": "registry"}],
    }
    yield {
        "case_id": "block/tints",
        "family": "block",
        "operation": "block_tints",
        "rows": [{"label": "registry"}],
    }
