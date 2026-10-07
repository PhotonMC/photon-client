"""The movement family: the player, the entities, the sky and the elytra."""

from cases.movement import player, entities, sky, elytra


def movement_cases():
    yield from player.movement_player_cases()
    yield from entities.movement_entity_cases()
    yield from sky.sky_flash_cases()
    yield from elytra.elytra_state_cases()
