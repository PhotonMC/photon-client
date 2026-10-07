"""Entity culling scenarios: how far each kind of entity is drawn and the box its renderer tests."""

from cases.helpers import *
from cases.movement.entities.common import *


def cull(label, entity, events=(), ticks=3):
    return living(f"cull-{label}", entity, ticks, list(events), extras=["cull"], player_x=30.5)


def rows():
    return [
        # The plain rule (the box's mean side times 64 and the view scale), a living renderer's
        # (half a block more round one wearing a dragon head) and the renderers that grow the box
        # (the illusioner sideways, the sniffer, the trident; the happy ghast downwards; a minecart
        # up to the block it shows).
        cull("pig", "pig"),
        cull("zombie-dragon-head", "zombie",
             [{"tick": 1, "type": "equipment", "slot": "head", "item": "minecraft:dragon_head"}]),
        cull("illusioner", "illusioner"),
        cull("sniffer", "sniffer"),
        cull("happy-ghast", "happy_ghast"),
        # One tick: the client does not yet move a trident on its own (T47b).
        cull("trident", "trident", ticks=1),
        cull("minecart", "minecart"),
        cull("chest-minecart", "chest_minecart"),
        cull("minecart-shows-a-block", "minecart",
             [{"tick": 1, "type": "minecart_data", "display": 1, "offset": 10}]),
        # Ten boxes for an arrow; four after the first tick near the camera for what is thrown and
        # for fireballs and wind charges, four from the start for the rest of what is shot; an armor
        # stand; the fixed and scaled reaches.
        cull("arrow", "arrow"),
        cull("snowball", "snowball"),
        cull("eye-of-ender", "eye_of_ender"),
        cull("fireball", "fireball"),
        cull("small-fireball", "small_fireball"),
        cull("wind-charge", "wind_charge"),
        cull("breeze-wind-charge", "breeze_wind_charge"),
        cull("dragon-fireball", "dragon_fireball"),
        cull("wither-skull", "wither_skull"),
        cull("shulker-bullet", "shulker_bullet"),
        cull("armor-stand", "armor_stand"),
        cull("leash-knot", "leash_knot"),
        cull("phantom", "phantom"),
        cull("firework", "firework_rocket"),
        cull("experience-orb", "experience_orb"),
        cull("tnt", "tnt"),
        # An end crystal with a beam is drawn from anywhere; the bolt, the dragon and a display with
        # no size are not culled; a display with a size stands its box on its position.
        cull("end-crystal", "end_crystal"),
        cull("end-crystal-beam", "end_crystal", [{"tick": 1, "type": "crystal_data", "target": [3, 70, -4]}]),
        cull("lightning-bolt", "lightning_bolt"),
        cull("ender-dragon", "ender_dragon"),
        cull("display-no-size", "block_display"),
        cull("display-sized", "block_display",
             [display_data(1, block_state=1, view_range=0.5, width=1.5, height=2.25)]),
    ]
