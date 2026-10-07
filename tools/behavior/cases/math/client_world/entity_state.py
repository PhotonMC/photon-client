"""The `math` family: the `client_world/entity_state` cases."""

from cases.math.common import *
from cases.math.shared import *
from cases.math.client_world.shared import *


def cases():
    yield {
        "case_id": "client-world/entity-shared-flags",
        "family": "math",
        "operation": "client_world_entity_shared_flags",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "all-defined-flags",
                "metadata_hex": "0000fbff",
            },
            {
                "label": "air-supply-only-preserves-flags",
                "metadata_hex": "010107ff",
            },
            {
                "label": "clear-shared-flags",
                "metadata_hex": "000000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-air-supply",
        "family": "math",
        "operation": "client_world_entity_air_supply",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "set-positive-air-supply",
                "metadata_hex": "0101fa01ff",
            },
            {
                "label": "combined-signed-air-supply",
                "metadata_hex": "0000400101ffffffff0fff",
            },
            {
                "label": "flags-only-preserves-air-supply",
                "metadata_hex": "000000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-pose",
        "family": "math",
        "operation": "client_world_entity_pose",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "set-crouching",
                "metadata_hex": "061405ff",
            },
            {
                "label": "set-sitting-with-glowing",
                "metadata_hex": "00004006140aff",
            },
            {
                "label": "unknown-pose-falls-back",
                "metadata_hex": "061463ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-ticks-frozen",
        "family": "math",
        "operation": "client_world_entity_ticks_frozen",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "set-positive-freeze",
                "metadata_hex": "070146ff",
            },
            {
                "label": "set-full-freeze",
                "metadata_hex": "07018c01ff",
            },
            {
                "label": "combined-shared-flags-and-negative-freeze",
                "metadata_hex": "0000400701ffffffff0fff",
            },
            {
                "label": "clear-freeze",
                "metadata_hex": "070100ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-custom-name",
        "family": "math",
        "operation": "client_world_entity_custom_name",
        "entity_id": 37,
        "entity_type_id": 120,
        "operations": [
            {
                "label": "set-custom-name",
                "metadata_hex": "02060108000c446973706c6179206e616d65ff",
            },
            {
                "label": "set-visible-only",
                "metadata_hex": "030801ff",
            },
            {
                "label": "clear-custom-name",
                "metadata_hex": "020600ff",
            },
            {
                "label": "restore-hidden-custom-name",
                "metadata_hex": "02060108000c446973706c6179206e616d65030800ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-living-flags",
        "family": "math",
        "operation": "client_world_entity_living_flags",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-all-living-flags",
                "metadata_hex": "080007ff",
            },
            {
                "label": "combined-shared-air-and-living",
                "metadata_hex": "000040010107080001ff",
            },
            {
                "label": "clear-living-flags",
                "metadata_hex": "080000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-health",
        "family": "math",
        "operation": "client_world_entity_health",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-health-after-living-flags",
                "metadata_hex": "080000090340f00000ff",
            },
            {
                "label": "set-zero-health",
                "metadata_hex": "090300000000ff",
            },
            {
                "label": "no-gravity-preserves-health",
                "metadata_hex": "050801ff",
            },
            {
                "label": "set-negative-health",
                "metadata_hex": "0903bf800000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-mob-flags",
        "family": "math",
        "operation": "client_world_entity_mob_flags",
        "entity_id": 37,
        "entity_type_id": 70,
        "operations": [
            {
                "label": "set-all-mob-flags",
                "metadata_hex": "0f0007ff",
            },
            {
                "label": "combined-shared-air-and-mob",
                "metadata_hex": "0000400101070f0002ff",
            },
            {
                "label": "no-gravity-preserves-mob",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-mob-flags",
                "metadata_hex": "0f0000ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-arrow-count",
        "family": "math",
        "operation": "client_world_entity_arrow_count",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-positive-arrow-count",
                "metadata_hex": "0c0107ff",
            },
            {
                "label": "combined-living-and-negative-arrow-count",
                "metadata_hex": "0800010c01ffffffff0fff",
            },
            {
                "label": "no-gravity-preserves-arrow-count",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-arrow-count",
                "metadata_hex": "0c0100ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-effect-ambience",
        "family": "math",
        "operation": "client_world_entity_effect_ambience",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-effect-ambience",
                "metadata_hex": "0b0801ff",
            },
            {
                "label": "combined-living-and-effect-ambience",
                "metadata_hex": "0800010b0801ff",
            },
            {
                "label": "empty-particles-preserves-effect-ambience",
                "metadata_hex": "0a11000b0801ff",
            },
            {
                "label": "no-gravity-preserves-effect-ambience",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-effect-ambience",
                "metadata_hex": "0b0800ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-effect-particles",
        "family": "math",
        "operation": "client_world_entity_effect_particles",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-one-effect-particle",
                "metadata_hex": "0a11011c11223344ff",
            },
            {
                "label": "set-two-effect-particles-with-living-flag",
                "metadata_hex": "0800010a11021c112233441c55667788ff",
            },
            {
                "label": "no-gravity-preserves-effect-particles",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-effect-particles",
                "metadata_hex": "0a1100ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-sleeping-position",
        "family": "math",
        "operation": "client_world_entity_sleeping_position",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-positive-sleeping-position",
                "metadata_hex": "0e0b01000003bffffe905aff",
            },
            {
                "label": "combined-living-and-negative-sleeping-position",
                "metadata_hex": "0800010e0b01ffff0000007fffc0ff",
            },
            {
                "label": "no-gravity-preserves-sleeping-position",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-sleeping-position",
                "metadata_hex": "0e0b00ff",
            },
        ],
    }

    yield {
        "case_id": "client-world/entity-stinger-count",
        "family": "math",
        "operation": "client_world_entity_stinger_count",
        "entity_id": 37,
        "entity_type_id": 54,
        "operations": [
            {
                "label": "set-positive-stinger-count",
                "metadata_hex": "0d0102ff",
            },
            {
                "label": "combined-living-and-negative-stinger-count",
                "metadata_hex": "0800010d01ffffffff0fff",
            },
            {
                "label": "no-gravity-preserves-stinger-count",
                "metadata_hex": "050801ff",
            },
            {
                "label": "clear-stinger-count",
                "metadata_hex": "0d0100ff",
            },
        ],
    }
