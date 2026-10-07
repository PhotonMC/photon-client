"""The movement family: the player and the world."""

from cases.movement.player.containers import cases as containers
from cases.movement.player.interaction import cases as interaction
from cases.movement.player.light import cases as light
from cases.movement.player.rides import cases as rides
from cases.movement.player.ticks import cases as ticks


def movement_player_cases():
    yield from containers()
    yield from interaction()
    yield from light()
    yield from rides()
    yield from ticks()
