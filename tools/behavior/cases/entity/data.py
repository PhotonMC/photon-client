"""The `entity` family: data."""

from cases.helpers import *


def entity_type_cases():
    """The entity type registry: every type's name and its id."""
    yield {
        "case_id": "entity/types",
        "family": "entity",
        "operation": "entity_types",
        "rows": [{"label": "all"}],
    }


def entity_data_serializer_cases():
    """The entity data serializers: each one's name and the id the wire
    carries it by (the order `EntityDataSerializers` registers them in)."""
    yield {
        "case_id": "entity/data-serializers",
        "family": "entity",
        "operation": "entity_data_serializers",
        "rows": [{"label": "all"}],
    }


def entity_variant_cases():
    """The pig and cow variants of the game's data pack: the texture an adult
    and a baby wear, and the model layer family each asks for."""
    yield {
        "case_id": "entity/variants",
        "family": "entity",
        "operation": "entity_variants",
        "rows": [{"label": "all"}],
    }


def entity_wolf_variant_cases():
    """The wolf variants of the game's data pack: the textures each wears."""
    yield {
        "case_id": "entity/wolf-variants",
        "family": "entity",
        "operation": "wolf_variants",
        "rows": [{"label": "all"}],
    }


def entity_painting_variant_cases():
    """The painting variants of the game's data pack: the texture each wears and its size."""
    yield {
        "case_id": "entity/painting-variants",
        "family": "entity",
        "operation": "painting_variants",
        "rows": [{"label": "all"}],
    }


def entity_map_data_cases():
    """The map's tables: each packed colour as an ARGB number and the decoration types by id."""
    yield {"case_id": "entity/map-data", "family": "entity", "operation": "map_data", "rows": [{"label": "all"}]}


def entity_variant_field_cases():
    """Where a pig's and a cow's variant sits among their data."""
    yield {
        "case_id": "entity/variant-fields",
        "family": "entity",
        "operation": "entity_variant_fields",
        "rows": [{"label": "all"}],
    }
