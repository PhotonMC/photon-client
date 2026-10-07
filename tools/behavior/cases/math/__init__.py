"""The `math` family."""

from cases.math.aabb.construction import cases as aabb_construction
from cases.math.aabb.queries import cases as aabb_queries
from cases.math.client_world.entity_motion import cases as client_world_entity_motion
from cases.math.client_world.entity_state import cases as client_world_entity_state
from cases.math.client_world.entity_ticks import cases as client_world_entity_ticks
from cases.math.client_world.input import cases as client_world_input
from cases.math.client_world.level import cases as client_world_level
from cases.math.client_world.passengers import cases as client_world_passengers
from cases.math.client_world.player import cases as client_world_player
from cases.math.direction import cases as direction
from cases.math.local_player.friction import cases as local_player_friction
from cases.math.local_player.input import cases as local_player_input
from cases.math.local_player.travel import cases as local_player_travel
from cases.math.mth.algorithms import cases as mth_algorithms
from cases.math.mth.angles import cases as mth_angles
from cases.math.mth.interpolation import cases as mth_interpolation
from cases.math.mth.numbers import cases as mth_numbers
from cases.math.options import cases as options
from cases.math.positions.blockpos import cases as positions_blockpos
from cases.math.positions.chunkpos import cases as positions_chunkpos
from cases.math.positions.search import cases as positions_search
from cases.math.positions.sectionpos import cases as positions_sectionpos
from cases.math.shape.border import cases as shape_border
from cases.math.shape.boxes import cases as shape_boxes
from cases.math.shape.collision import cases as shape_collision
from cases.math.shape.dimensions import cases as shape_dimensions
from cases.math.shape.packets import cases as shape_packets
from cases.math.shape.queries import cases as shape_queries
from cases.math.shape.rotation import cases as shape_rotation
from cases.math.shape.world_collision import cases as shape_world_collision
from cases.math.shape.world_index import cases as shape_world_index
from cases.math.vec2 import cases as vec2
from cases.math.vec3.basic import cases as vec3_basic
from cases.math.vec3.distance import cases as vec3_distance
from cases.math.vec3.rotation import cases as vec3_rotation


def math_cases():
    yield from aabb_construction()
    yield from aabb_queries()
    yield from client_world_entity_motion()
    yield from client_world_entity_state()
    yield from client_world_entity_ticks()
    yield from client_world_input()
    yield from client_world_level()
    yield from client_world_passengers()
    yield from client_world_player()
    yield from direction()
    yield from local_player_friction()
    yield from local_player_input()
    yield from local_player_travel()
    yield from mth_algorithms()
    yield from mth_angles()
    yield from mth_interpolation()
    yield from mth_numbers()
    yield from options()
    yield from positions_blockpos()
    yield from positions_chunkpos()
    yield from positions_search()
    yield from positions_sectionpos()
    yield from shape_border()
    yield from shape_boxes()
    yield from shape_collision()
    yield from shape_dimensions()
    yield from shape_packets()
    yield from shape_queries()
    yield from shape_rotation()
    yield from shape_world_collision()
    yield from shape_world_index()
    yield from vec2()
    yield from vec3_basic()
    yield from vec3_distance()
    yield from vec3_rotation()
