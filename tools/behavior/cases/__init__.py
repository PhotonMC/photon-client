"""Original synthetic inputs only. Expected values always come from the Java JAR."""

import itertools

from cases.atlas import atlas_cases
from cases.binary import binary_cases
from cases.block import block_cases
from cases.entity import entity_cases
from cases.gui import gui_generators
from cases.image import image_cases, image_trim_cases
from cases.json import json_cases
from cases.math import math_cases
from cases.model import model_cases
from cases.movement import movement_cases
from cases.nbt import nbt_cases
from cases.resources import resources_cases
from cases.sound import sound_cases
from cases.timer import timer_cases
from cases.utf8 import utf8_cases
from cases.window import window_cases
from cases.wire import wire_cases


def build_cases(families):
    generators = {"wire": wire_cases, "math": math_cases, "timer": timer_cases, "utf8": utf8_cases,
                  "binary": binary_cases, "nbt": nbt_cases, "resources": resources_cases,
                  "window": window_cases,
                  "atlas": atlas_cases,
                  "image": lambda: itertools.chain(image_cases(), image_trim_cases()),
                  "model": model_cases,
                  "json": json_cases,
                  "block": block_cases,
                  "movement": movement_cases,
                  "entity": entity_cases,
                  "sound": sound_cases,
                  "gui": None}
    generators["gui"] = gui_generators
    cases = [dict(schema_version=1, **case)
             for family in sorted(families) for case in generators[family]()]
    ids = [case["case_id"] for case in cases]
    if not cases or len(ids) != len(set(ids)):
        raise ValueError("The corpus must be nonempty with unique case IDs")
    return cases
