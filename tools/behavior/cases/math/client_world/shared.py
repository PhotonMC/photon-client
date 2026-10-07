"""The `math` family: what several `client_world` cases share."""

from cases.math.common import *
from cases.math.shared import *


def attribute_modifier(identifier, amount, operation):
    return {
        "id": identifier,
        "amount": float_bits(amount, "f64"),
        "operation": operation,
    }
