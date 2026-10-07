"""Entity animation scenarios, by the kind of entity."""

from . import animals
from . import aquatic
from . import bosses
from . import cubes
from . import culling
from . import flyers
from . import objects


def animation_rows_parts():
    rows = []
    rows += animals.rows()
    rows += aquatic.rows()
    rows += bosses.rows()
    rows += cubes.rows()
    rows += culling.rows()
    rows += flyers.rows()
    rows += objects.rows()
    return rows
