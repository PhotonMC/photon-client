"""Entity animation scenarios: flyers."""

from cases.helpers import *
from cases.movement.entities.common import *


def rows():
    return [
        # A bat hangs (still, snapped to the block above it, one animation) or flies (the other); a
        # vex reads its charging flag; an allay's arms rise to what it holds, and its dance spins
        # and sways on a cycle of fifty-five ticks.
        # A parrot beats its wings faster in the air (a rider or one on the ground slows them) and
        # sinks slowly while it falls there; its variant is read from its data.
        living("parrot-on-the-ground", "parrot", 30, [on_ground(1, True)], extras=["parrot"]),
        living("parrot-flies", "parrot", 40, [on_ground(1, False)], extras=["parrot"]),
        living("parrot-lands", "parrot", 40, [on_ground(1, False), on_ground(15, True)],
               extras=["parrot"]),
        living("parrot-falls", "parrot", 30,
               [on_ground(1, False), {"tick": 3, "type": "entity_motion", "x": float_bits(0.0, "f64"),
                                      "y": float_bits(-0.5, "f64"), "z": float_bits(0.0, "f64")}],
               extras=["parrot"]),
        living("parrot-variant", "parrot", 10,
               [{"tick": 2, "type": "flier_data", "parrot_variant": 3}], extras=["parrot"]),
        # A breeze runs its idle animation from its first tick; its pose starts the shooting,
        # inhaling and sliding ones (stopping the idle, inhaling, jumping and shooting ones), a
        # long jump runs on its own tick, and ending a slide starts the slide back.
        living("breeze-idles", "breeze", 30, [], extras=["breeze"]),
        living("breeze-shoots", "breeze", 40,
               [{"tick": 5, "type": "frog_data", "pose": "SHOOTING"},
                {"tick": 20, "type": "frog_data", "pose": "STANDING"}], extras=["breeze"]),
        living("breeze-inhales", "breeze", 40,
               [{"tick": 5, "type": "frog_data", "pose": "INHALING"},
                {"tick": 15, "type": "frog_data", "pose": "SHOOTING"}], extras=["breeze"]),
        living("breeze-slides", "breeze", 40,
               [{"tick": 5, "type": "frog_data", "pose": "SLIDING"}], extras=["breeze"]),
        living("breeze-slides-and-stops", "breeze", 50,
               [{"tick": 5, "type": "frog_data", "pose": "SLIDING"},
                {"tick": 20, "type": "frog_data", "pose": "STANDING"}], extras=["breeze"]),
        living("breeze-slides-again", "breeze", 60,
               [{"tick": 5, "type": "frog_data", "pose": "SLIDING"},
                {"tick": 20, "type": "frog_data", "pose": "STANDING"},
                {"tick": 30, "type": "frog_data", "pose": "SLIDING"}], extras=["breeze"]),
        living("breeze-jumps", "breeze", 40,
               [{"tick": 5, "type": "frog_data", "pose": "LONG_JUMPING"},
                {"tick": 18, "type": "frog_data", "pose": "STANDING"}], extras=["breeze"]),
        living("breeze-shoots-again", "breeze", 40,
               [{"tick": 5, "type": "frog_data", "pose": "SHOOTING"},
                {"tick": 12, "type": "frog_data", "pose": "SHOOTING"}], extras=["breeze"]),
        # A wither's side heads turn after the entities its data names (at most 40 degrees up or down
        # and 10 sideways a tick) or after its body; it settles its upward motion and turns to face
        # fast horizontal motion; the invulnerable ticks come from the data.
        living("wither-idle", "wither", 20, [], extras=["wither"], player_x=30.5),
        beside(living("wither-looks-at-a-target", "wither", 40,
                      [{"tick": 2, "type": "wither_data", "target_b": 41, "target_c": 41,
                        "invulnerable": 150}], extras=["wither"], player_x=30.5),
               "pig", 11.5, 66.0, 5.5),
        beside(living("wither-loses-its-target", "wither", 40,
                      [{"tick": 2, "type": "wither_data", "target_b": 41},
                       {"tick": 14, "type": "wither_data", "target_b": 0}],
                      extras=["wither"], player_x=30.5), "pig", -9.5, 60.0, -7.5),
        beside(living("wither-target-above", "wither", 30,
                      [{"tick": 2, "type": "wither_data", "target_c": 41}], extras=["wither"],
                      player_x=30.5), "pig", 0.5, 74.0, 0.5),
        living("wither-turns-with-its-motion", "wither", 30,
               [{"tick": 3, "type": "entity_motion", "x": float_bits(0.4, "f64"),
                 "y": float_bits(0.5, "f64"), "z": float_bits(-0.3, "f64")}],
               extras=["wither"], player_x=30.5),
        living("wither-slow-motion", "wither", 30,
               [{"tick": 3, "type": "entity_motion", "x": float_bits(0.1, "f64"),
                 "y": float_bits(0.5, "f64"), "z": float_bits(0.1, "f64")}],
               extras=["wither"], player_x=30.5),
        # An ender dragon beats its wings faster the slower it moves, steadily while its phase has
        # it sit (the phase it starts in), and records its height and turn each tick into its flight
        # history (sixty-four samples) before its position eases to the last packet's; a dying
        # one does neither.
        # A minecart's hit and damage run down a tick at a time; the block it carries is the one the
        # server named or its type's own (a furnace's lit while it has fuel), lowered by the offset
        # the server said or the type's own; its position and turn ease to the last packet's over three
        # ticks (a move, a move with a turn, a teleport), else its turns are kept under a whole turn;
        # and the rail under a place (and a little ahead and behind it) puts it where the rail leads.
        dict(living("minecart-idle", "minecart", 20, [], extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-hit", "minecart", 20,
               [{"tick": 2, "type": "minecart_data", "hurt": 10, "hurt_dir": -1, "damage": 25.5}],
               extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-chest", "chest_minecart", 6, [], extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-hopper", "hopper_minecart", 6, [], extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-tnt", "tnt_minecart", 6, [], extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-furnace", "furnace_minecart", 12,
               [{"tick": 4, "type": "minecart_data", "fuel": True},
                {"tick": 8, "type": "minecart_data", "fuel": False}], extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-spawner-command", "spawner_minecart", 6, [], extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-tnt-is-lit", "tnt_minecart", 100,
                    [{"tick": 3, "type": "entity_event", "event": 70}], extras=["minecart"], player_x=30.5),
             chunk_radius=1),
        dict(living("minecart-tnt-is-lit-twice", "tnt_minecart", 40,
                    [{"tick": 3, "type": "entity_event", "event": 70},
                     {"tick": 20, "type": "entity_event", "event": 70}], extras=["minecart"], player_x=30.5),
             chunk_radius=1),
        dict(living("minecart-names-its-block", "minecart", 12,
               [{"tick": 2, "type": "minecart_data", "display": 1, "offset": 3},
                {"tick": 7, "type": "minecart_data", "no_display": True}], extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-chest-offset-and-block", "chest_minecart", 8,
               [{"tick": 2, "type": "minecart_data", "display": 1}], extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-moves", "minecart", 24,
               [{"tick": 2, "type": "move", "xa": 4096, "ya": 0, "za": 0},
                {"tick": 8, "type": "move", "xa": 1000, "ya": 2000, "za": -3000},
                {"tick": 9, "type": "move", "xa": 500, "ya": 0, "za": 500}],
               extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-moves-and-turns", "minecart", 30,
               [{"tick": 2, "type": "move_rot", "xa": 4096, "ya": 0, "za": 0, "y_rot": 64, "x_rot": 0},
                {"tick": 8, "type": "move_rot", "xa": 1000, "ya": 0, "za": -3000, "y_rot": 200, "x_rot": 10},
                {"tick": 14, "type": "move_rot", "xa": 0, "ya": 0, "za": 0, "y_rot": 250, "x_rot": -10}],
               extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-is-teleported", "minecart", 12,
               [{"tick": 2, "type": "sync", "x": float_bits(5.5, "f64"), "y": float_bits(65.0, "f64"),
                 "z": float_bits(-2.5, "f64"), "motion_x": float_bits(0.0, "f64"),
                 "motion_y": float_bits(0.0, "f64"), "motion_z": float_bits(0.0, "f64")}],
               extras=["minecart"], player_x=30.5), chunk_radius=1),
        dict(living("minecart-rails", "minecart", 4, [], extras=["minecart"], player_x=30.5), chunk_radius=1,
             blocks=[{"x": x, "y": 63, "z": z, "block": "minecraft:stone"} for x in range(-2, 6) for z in range(-2, 4)]
             + [{"x": 0, "y": 64, "z": 0, "block": "minecraft:rail", "properties": {"shape": "east_west"}},
                {"x": 1, "y": 64, "z": 0, "block": "minecraft:rail", "properties": {"shape": "ascending_east"}},
                {"x": 2, "y": 65, "z": 0, "block": "minecraft:powered_rail", "properties": {"shape": "east_west", "powered": "true"}},
                {"x": 0, "y": 64, "z": 1, "block": "minecraft:detector_rail", "properties": {"shape": "north_south"}},
                {"x": 0, "y": 64, "z": 2, "block": "minecraft:activator_rail", "properties": {"shape": "ascending_south"}},
                {"x": 3, "y": 64, "z": 1, "block": "minecraft:rail", "properties": {"shape": "south_east"}},
                {"x": 4, "y": 64, "z": 1, "block": "minecraft:rail", "properties": {"shape": "north_west"}},
                {"x": 3, "y": 64, "z": 2, "block": "minecraft:rail", "properties": {"shape": "north_east"}},
                {"x": 4, "y": 64, "z": 2, "block": "minecraft:rail", "properties": {"shape": "south_west"}},
                {"x": 4, "y": 64, "z": 0, "block": "minecraft:rail", "properties": {"shape": "ascending_west"}},
                {"x": 5, "y": 64, "z": 0, "block": "minecraft:rail", "properties": {"shape": "ascending_north"}}],
             rail_probes=[[float_bits(x, "f64"), float_bits(y, "f64"), float_bits(z, "f64")]
                          for (x, y, z) in [(0.5, 64.0, 0.5), (0.2, 64.1, 0.5), (0.9, 64.0, 0.5), (1.5, 64.2, 0.5),
                                            (1.1, 64.0, 0.5), (2.5, 65.0, 0.5), (0.5, 64.0, 1.5), (0.5, 64.0, 2.2),
                                            (0.5, 64.0, 2.9), (3.4, 64.0, 1.6), (4.6, 64.0, 1.4), (3.2, 64.0, 2.8),
                                            (4.1, 64.0, 2.2), (4.5, 64.0, 0.5), (5.5, 64.0, 0.4), (0.5, 65.0, 0.5),
                                            (-1.5, 64.0, 0.5), (0.5, 66.5, 0.5), (3.5, 65.0, 1.5)]]),
        # A display entity keeps nothing until some data arrives; then its render state is made from the
        # data (the transformation of the four parts, billboard, brightness, shadow, glow, and the
        # block or item it shows). While the interpolation duration is not zero a change eases from where
        # the last had got to (the turns by the shorter arc, linear where they are the same), over the
        # duration from the start tick (the delay after it was sent); the position and the turn ease over
        # the teleport duration, and are put there at once while it is zero.
        dict(living("display-quiet", "block_display", 6, [], extras=["display"], player_x=30.5), chunk_radius=1),
        dict(living("display-block-static", "block_display", 8,
               [display_data(2, block_state=1, translation=[0.5, 0.25, -1.0], scale=[2.0, 1.0, 0.5],
                             left_rotation=[0.0, 0.25881904, 0.0, 0.9659258],
                             right_rotation=[0.0, 0.0, 0.25881904, 0.9659258], billboard=3,
                             brightness=(15 << 4) | (10 << 20), glow=0xFF8800, shadow_radius=0.75,
                             shadow_strength=0.5, view_range=2.0, width=1.5, height=2.0)],
               extras=["display"], player_x=30.5), chunk_radius=1),
        dict(living("display-eases", "block_display", 30,
               [display_data(2, block_state=1, translation=[0.0, 0.0, 0.0], scale=[1.0, 1.0, 1.0],
                             shadow_radius=0.0),
                display_data(6, duration=10, start_delta=0, translation=[1.0, 2.0, 3.0], scale=[2.0, 2.0, 2.0],
                             left_rotation=[0.0, 0.7071068, 0.0, 0.7071068], shadow_radius=1.0),
                display_data(13, duration=8, start_delta=2, translation=[-1.0, 0.0, 0.5],
                             shadow_radius=0.25, shadow_strength=0.5),
                display_data(24, duration=0, translation=[0.0, 0.0, 0.0])],
               extras=["display"], player_x=30.5), chunk_radius=1),
        dict(living("display-turns-the-other-way", "block_display", 20,
               [display_data(2, block_state=1, left_rotation=[0.0, 0.0, 0.0, 1.0]),
                display_data(5, duration=6, start_delta=0, left_rotation=[0.0, 0.8660254, 0.0, -0.5],
                             right_rotation=[0.0, 0.0, 0.0, 1.0])],
               extras=["display"], player_x=30.5), chunk_radius=1),
        dict(living("display-moves", "block_display", 30,
               [display_data(2, block_state=1),
                {"tick": 3, "type": "sync", "x": float_bits(5.5, "f64"), "y": float_bits(65.0, "f64"),
                 "z": float_bits(-2.5, "f64"), "motion_x": float_bits(0.0, "f64"),
                 "motion_y": float_bits(0.0, "f64"), "motion_z": float_bits(0.0, "f64")},
                display_data(6, pos_rot=6),
                {"tick": 8, "type": "sync", "x": float_bits(10.5, "f64"), "y": float_bits(66.0, "f64"),
                 "z": float_bits(4.5, "f64"), "motion_x": float_bits(0.0, "f64"),
                 "motion_y": float_bits(0.0, "f64"), "motion_z": float_bits(0.0, "f64")},
                {"tick": 12, "type": "move_rot", "xa": 4096, "ya": 0, "za": 0, "y_rot": 64, "x_rot": 0}],
               extras=["display"], player_x=30.5), chunk_radius=1),
        # A text display's text, line width, background, opacity and flags make its render state; the
        # opacity eases in whole numbers and the background by channels, over the same duration.
        dict(living("display-text-static", "text_display", 8,
               [display_data(2, text="Hello\nworld", line_width=120, background=0x40FF8000, text_opacity=-128,
                             style_flags=13)],
               extras=["display"], player_x=30.5), chunk_radius=1),
        dict(living("display-text-eases", "text_display", 24,
               [display_data(2, text="a", background=0x80102030 - (1 << 32), text_opacity=100),
                display_data(5, duration=10, start_delta=0, background=0x00FFFFFF, text_opacity=-1, line_width=40),
                display_data(9, duration=6, start_delta=0, text_opacity=20)],
               extras=["display"], player_x=30.5), chunk_radius=1),
        dict(living("display-item", "item_display", 14,
               [display_data(2, item="minecraft:diamond_sword", item_display=8),
                display_data(8, item_display=5)],
               extras=["display"], player_x=30.5), chunk_radius=1),
        # A minecart of the new behaviour (the server enabled `minecart_improvements`) is not eased to
        # the place a packet gives: a move or a teleport puts it there at once, and it plays back the
        # steps the server sent along the track over three ticks, weighted by how far each went
        # (a batch that arrives while one plays waits; one of no weight is played at once, and its
        # answer is kept for the next as the reference keeps it), turning by the short way round.
        dict(living("minecart-new-idle", "minecart", 6, [], extras=["minecart"], player_x=30.5),
             chunk_radius=1, minecart_improvements=True),
        dict(living("minecart-new-steps", "minecart", 24,
               [{"tick": 2, "type": "minecart_steps", "steps": [
                   cart_step(0.75, 64.1, 0.5, 0.1, 0.0, 0.0, 64, 0, 0.25),
                   cart_step(1.0, 64.1, 0.5, 0.2, 0.0, 0.0, 64, 0, 0.5),
                   cart_step(1.5, 64.6, 0.5, 0.3, 0.1, 0.0, 70, -32, 0.5)]}],
               extras=["minecart"], player_x=30.5), chunk_radius=1, minecart_improvements=True),
        dict(living("minecart-new-batches", "minecart", 30,
               [{"tick": 1, "type": "minecart_steps", "steps": [
                   cart_step(1.0, 64.1, 0.5, 0.25, 0.0, 0.0, 64, 0, 1.0)]},
                {"tick": 2, "type": "minecart_steps", "steps": [
                   cart_step(1.5, 64.1, 0.5, 0.25, 0.0, 0.0, 120, 10, 0.4)]},
                {"tick": 3, "type": "minecart_steps", "steps": [
                   cart_step(2.0, 64.1, 0.5, 0.25, 0.0, 0.0, -120, 100, 0.6)]},
                {"tick": 12, "type": "minecart_steps", "steps": [
                   cart_step(2.0, 64.1, 0.5, 0.0, 0.0, 0.0, -120, 100, 1.0)]}],
               extras=["minecart"], player_x=30.5), chunk_radius=1, minecart_improvements=True),
        dict(living("minecart-new-no-weight", "minecart", 14,
               [{"tick": 2, "type": "minecart_steps", "steps": [
                   cart_step(2.5, 65.1, 0.5, 0.0, 0.0, 0.0, 32, 0, 0.0)]},
                {"tick": 3, "type": "minecart_steps", "steps": [
                   cart_step(3.5, 65.1, 1.5, 0.0, 0.0, 0.0, 96, 8, 0.0)]},
                {"tick": 5, "type": "minecart_steps", "steps": [
                   cart_step(4.5, 65.1, 1.5, 0.0, 0.0, 0.0, 96, 8, 0.0),
                   cart_step(5.5, 65.1, 1.5, 0.0, 0.0, 0.0, 100, 8, 0.0)]}],
               extras=["minecart"], player_x=30.5), chunk_radius=1, minecart_improvements=True),
        # No frame is drawn between the ticks (the cart is out of sight): the place the last tick asked for
        # at no weight is the one the next tick gets.
        dict(living("minecart-new-no-weight-unseen", "minecart", 14,
               [{"tick": 2, "type": "minecart_steps", "steps": [
                   cart_step(2.5, 65.1, 0.5, 0.0, 0.0, 0.0, 32, 0, 0.0)]},
                {"tick": 3, "type": "minecart_steps", "steps": [
                   cart_step(3.5, 65.1, 1.5, 0.0, 0.0, 0.0, 96, 8, 0.0)]},
                {"tick": 4, "type": "minecart_steps", "steps": [
                   cart_step(4.5, 65.1, 1.5, 0.0, 0.0, 0.0, 96, 8, 0.0)]},
                {"tick": 6, "type": "minecart_steps", "steps": [
                   cart_step(5.5, 65.1, 1.5, 0.0, 0.0, 0.0, 96, 8, 1.0)]}],
               extras=["minecart"], player_x=30.5), chunk_radius=1, minecart_improvements=True, sample_from=5),
        dict(living("minecart-new-is-moved", "minecart", 24,
               [{"tick": 2, "type": "move", "xa": 4096, "ya": 0, "za": 0},
                {"tick": 4, "type": "move_rot", "xa": 1000, "ya": 2000, "za": -3000, "y_rot": 200, "x_rot": 10},
                {"tick": 8, "type": "move_rot", "xa": 0, "ya": 0, "za": 0, "y_rot": 250, "x_rot": -100},
                {"tick": 12, "type": "sync", "x": float_bits(5.5, "f64"), "y": float_bits(65.0, "f64"),
                 "z": float_bits(-2.5, "f64"), "motion_x": float_bits(0.0, "f64"),
                 "motion_y": float_bits(0.0, "f64"), "motion_z": float_bits(0.0, "f64")},
                {"tick": 14, "type": "entity_motion", "x": float_bits(0.4, "f64"),
                 "y": float_bits(0.0, "f64"), "z": float_bits(-0.3, "f64")}],
               extras=["minecart"], player_x=30.5), chunk_radius=1, minecart_improvements=True),
        living("dragon-hovers", "ender_dragon", 30, [], extras=["ender_dragon"], player_x=30.5),
        living("dragon-flies", "ender_dragon", 50,
               [{"tick": 2, "type": "dragon_data", "phase": 0},
                {"tick": 3, "type": "entity_motion", "x": float_bits(0.3, "f64"),
                 "y": float_bits(0.2, "f64"), "z": float_bits(-0.4, "f64")}],
               extras=["ender_dragon"], player_x=30.5),
        living("dragon-flies-and-turns", "ender_dragon", 90,
               [{"tick": 2, "type": "dragon_data", "phase": 1},
                {"tick": 3, "type": "entity_motion", "x": float_bits(1.1, "f64"),
                 "y": float_bits(-0.1, "f64"), "z": float_bits(0.2, "f64")}]
               + [{"tick": t, "type": "move_rot", "xa": 300, "ya": 40 if t < 40 else -60, "za": 100,
                   "y_rot": (t * 3) % 256, "x_rot": 0} for t in range(5, 80, 3)],
               extras=["ender_dragon"], player_x=30.5),
        living("dragon-lands", "ender_dragon", 40,
               [{"tick": 2, "type": "dragon_data", "phase": 3}], extras=["ender_dragon"], player_x=30.5),
        living("dragon-dies-hovering", "ender_dragon", 40,
               [{"tick": 6, "type": "death"}], extras=["ender_dragon"], player_x=30.5),
        living("dragon-dies", "ender_dragon", 60,
               [{"tick": 2, "type": "dragon_data", "phase": 0},
                {"tick": 3, "type": "entity_motion", "x": float_bits(0.3, "f64"),
                 "y": float_bits(0.2, "f64"), "z": float_bits(-0.4, "f64")},
                {"tick": 20, "type": "death"}], extras=["ender_dragon"], player_x=30.5),
        living("bat-flies", "bat", 30, [], extras=["bat"]),
        living("bat-hangs", "bat", 30, [{"tick": 2, "type": "flier_data", "bat_flags": 1}],
               extras=["bat"]),
        living("bat-wakes", "bat", 40, [{"tick": 2, "type": "flier_data", "bat_flags": 1},
                                        {"tick": 20, "type": "flier_data", "bat_flags": 0}],
               extras=["bat"]),
        living("vex-charges", "vex", 10, [{"tick": 2, "type": "flier_data", "vex_flags": 1}],
               extras=["vex"]),
        living("allay-hovers", "allay", 20, [], extras=["allay"]),
        living("allay-takes-an-item", "allay", 30,
               [{"tick": 3, "type": "equipment", "item": "minecraft:bamboo"}], extras=["allay"]),
        living("allay-lets-go", "allay", 30,
               [{"tick": 2, "type": "equipment", "item": "minecraft:bamboo"},
                {"tick": 12, "type": "equipment", "item": None}], extras=["allay"]),
        living("allay-dances", "allay", 130,
               [{"tick": 3, "type": "flier_data", "allay_dancing": True}], extras=["allay"]),
        living("allay-stops-dancing", "allay", 100,
               [{"tick": 3, "type": "flier_data", "allay_dancing": True},
                {"tick": 40, "type": "flier_data", "allay_dancing": False}], extras=["allay"]),
        living("bee-quiet", "bee", 6, [], extras=["bee"]),
        living("bee-rolls", "bee", 20, [{"tick": 2, "type": "bee_data", "flags": 2}], extras=["bee"]),
        living("bee-rolls-then-stops", "bee", 30, [{"tick": 1, "type": "bee_data", "flags": 2},
                                                   {"tick": 9, "type": "bee_data", "flags": 0}],
               extras=["bee"]),
        # A bee with nectar drops nectar particles at random, which the headless client cannot draw.
        living("bee-stung", "bee", 6, [{"tick": 1, "type": "bee_data", "flags": 4}], extras=["bee"]),
        living("bee-angry", "bee", 12, [{"tick": 1, "type": "bee_data", "anger_end": 5}], extras=["bee"]),
        living("bee-angry-for-a-long-time", "bee", 12, [{"tick": 1, "type": "bee_data", "anger_end": 500}],
               extras=["bee"]),
        # The idle spin starts 200 to 240 ticks after the wait is drawn (from a source the reference
        # does not seed: fixed), plays for 3.5 seconds and, ten ticks after it started, draws again.
        *[living(f"copper-golem-idle-{ticks}", "copper_golem", ticks,
                 [{"tick": 0, "type": "copper_golem_idle", "seed": 5}], extras=["copper_golem"])
          for ticks in (4, 100, 205, 215, 235, 255, 290, 330, 470, 700)],
    ]
