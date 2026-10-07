"""Entity animation scenarios: objects."""

from cases.helpers import *
from cases.movement.entities.common import *


def rows():
    return [
        # A boat's hit (its time and damage) runs down a tick at a time; a bubble column shakes it
        # (not under water) by a sine of its age, building up and dying away; whether it is under
        # water is whether the water reaches a thousandth over its box.
        fish("boat-floats", "oak_boat", 20, [], extras="boat"),
        fish("boat-is-hit", "oak_boat", 30,
             [{"tick": 3, "type": "boat_data", "hurt": 10, "hurt_dir": -1, "damage": 25.0}], extras="boat"),
        fish("boat-shaken-by-bubbles", "oak_boat", 40,
             [{"tick": 3, "type": "boat_data", "bubble_time": 60},
              {"tick": 22, "type": "boat_data", "bubble_time": 0}], water="none", extras="boat"),
        fish("boat-under-water", "oak_chest_boat", 10, [], extras="boat"),
        fish("boat-in-shallow-water", "birch_boat", 10, [], water="shallow", y=63.5, extras="boat"),
        fish("boat-above-the-water", "bamboo_raft", 10, [], water="shallow", y=64.6, extras="boat"),
        fish("boat-on-dry-land", "spruce_boat", 10, [], water="none", extras="boat"),
        # An arrow starts to shake when the server says it is in the ground (not on its first tick,
        # not while it shakes already), and the shake runs down by a tick; a potion's colour above
        # nothing makes it tipped.
        living("arrow-flies", "arrow", 12, [], extras=["arrow"], player_x=30.5),
        living("arrow-sticks", "arrow", 20,
               [{"tick": 3, "type": "arrow_data", "in_ground": True}], extras=["arrow"], player_x=30.5),
        living("arrow-sticks-and-sticks-again", "arrow", 24,
               [{"tick": 3, "type": "arrow_data", "in_ground": True},
                {"tick": 6, "type": "arrow_data", "in_ground": True},
                {"tick": 14, "type": "arrow_data", "in_ground": False},
                {"tick": 16, "type": "arrow_data", "in_ground": True}], extras=["arrow"], player_x=30.5),
        living("arrow-sticks-from-the-start", "arrow", 14,
               [{"tick": 0, "type": "arrow_data", "in_ground": True}], extras=["arrow"], player_x=30.5),
        living("arrow-is-tipped", "arrow", 8,
               [{"tick": 2, "type": "arrow_data", "color": 16253176},
                {"tick": 5, "type": "arrow_data", "color": -1}], extras=["arrow"], player_x=30.5),
        living("spectral-arrow-sticks", "spectral_arrow", 14,
               [{"tick": 3, "type": "arrow_data", "in_ground": True}], extras=["arrow"], player_x=30.5),
        # An end crystal holds the block its beam reaches and whether it shows its base; a dragon
        # looks for the nearest crystal whose box meets its own made 32 blocks wider (one tick in
        # ten, so long enough in these rows), and lets go of one that is gone or out of reach.
        living("crystal-defaults", "end_crystal", 6, [], extras=["end_crystal"], player_x=30.5),
        living("crystal-beam-and-base", "end_crystal", 10,
               [{"tick": 2, "type": "crystal_data", "target": [3, 70, -4], "bottom": False}],
               extras=["end_crystal"], player_x=30.5),
        living("crystal-loses-its-beam", "end_crystal", 10,
               [{"tick": 2, "type": "crystal_data", "target": [3, 70, -4]},
                {"tick": 6, "type": "crystal_data", "no_target": True, "bottom": True}],
               extras=["end_crystal"], player_x=30.5),
        beside(living("dragon-finds-a-crystal", "ender_dragon", 220,
                      [{"tick": 2, "type": "dragon_data", "phase": 0}],
                      extras=["ender_dragon"], player_x=30.5), "end_crystal", 14.5, 64.0, 3.5),
        beside(living("dragon-does-not-find-a-far-crystal", "ender_dragon", 220,
                      [{"tick": 2, "type": "dragon_data", "phase": 0}],
                      extras=["ender_dragon"], player_x=30.5), "end_crystal", 90.5, 64.0, 3.5),
        beside(living("dragon-finds-a-crystal-just-in-reach", "ender_dragon", 220,
                      [], extras=["ender_dragon"], player_x=30.5), "end_crystal", 39.9, 64.0, 0.5),
    ]
