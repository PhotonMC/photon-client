"""The `entity` family: player."""

from cases.helpers import *


def entity_camera_hurt_cases():
    """What the view is turned by when the camera's entity flinches or is dying: before and after the
    flinch, across it, from several directions, with the tilt option changed, and while dying."""
    rows = []
    def row(label, hurt_time, duration=10, direction=0.0, dying=False, death=0.0, tilt=1.0):
        rows.append({"label": label, "hurt_time": float_bits(hurt_time), "hurt_duration": duration,
                     "hurt_dir": float_bits(direction), "dying": dying,
                     "death_time": float_bits(death), "tilt": float_bits(tilt, "f64")})
    row("no flinch", -0.5)
    row("flinch ending", 0.0)
    for index, hurt_time in enumerate([0.5, 2.25, 4.0, 6.5, 9.0, 9.9]):
        row(f"flinch {index}", hurt_time, direction=float(index * 57) - 90.0)
    row("weaker tilt", 3.0, direction=33.0, tilt=0.4)
    row("no tilt", 3.0, direction=33.0, tilt=0.0)
    row("longer flinch", 3.0, duration=20, direction=-120.0)
    row("dying", -1.0, dying=True, death=0.5)
    row("dying later", -1.0, dying=True, death=11.25)
    row("dead", -1.0, dying=True, death=33.0)
    row("dying and flinching", 4.0, dying=True, death=2.5, direction=75.0)
    yield {"case_id": "entity/camera-hurt", "family": "entity", "operation": "camera_hurt", "rows": rows}


def entity_elytra_cases():
    """The elytra: its models (grown and baby) posed by the real `ElytraModel.setupAnim` from the wings'
    spread, standing and sneaking; and the real `ElytraAnimationState` ticked through scenarios: standing,
    sneaking, gliding level, diving steeply and shallowly, rising, a glide that ends, sneaking while
    gliding, and no motion."""
    rows = []
    for layer in ("minecraft:elytra#main", "minecraft:elytra_baby#main"):
        for label, spread, crouching in (
                ("rest", (0.2617994, 0.0, -0.2617994), False), ("sneak", (0.6981317, 0.08726646, -0.7853982), True),
                ("glide", (0.34906584, 0.0, -1.5707964), False), ("odd", (0.4, -0.1, -0.9), True)):
            rows.append({"label": f"{layer}-{label}", "layer": layer, "crouching": crouching,
                         "rot_x": float_bits(spread[0]), "rot_y": float_bits(spread[1]),
                         "rot_z": float_bits(spread[2])})
    yield {"case_id": "entity/elytra-model", "family": "entity", "operation": "elytra_model", "rows": rows}


def entity_avatar_pose_cases():
    """Where the real `AvatarRenderer` puts a player: standing, sneaking, turned, dying, swimming (in water
    and crawling), gliding (new, long, past the full tip, with and without a heading, spinning) and
    combinations that prefer one over another."""
    def row(label, body=0.0, pitch=0.0, swim=0.0, water=False, crawling=False, crouching=False, death=0.0,
            glide=False, spin=False, ticks=0.0, heading=None, feet=(10.5, 64.0, -3.5), age=0.0):
        return {"label": label, "body_rot": float_bits(body), "x_rot": float_bits(pitch),
                "swim_amount": float_bits(swim), "in_water": water, "visually_swimming": crawling,
                "crouching": crouching, "death_time": float_bits(death), "fall_flying": glide,
                "spin_attack": spin, "fall_flying_ticks": float_bits(ticks),
                "apply_flying_y_rot": heading is not None,
                "flying_y_rot": float_bits(heading if heading is not None else 0.0),
                "age_in_ticks": float_bits(age), "feet_x": float_bits(feet[0]), "feet_y": float_bits(feet[1]),
                "feet_z": float_bits(feet[2])}
    rows = [
        row("standing"), row("turned", body=123.5), row("negative-turn", body=-77.25),
        row("sneaking", crouching=True, body=40.0), row("far", feet=(-100.25, 70.5, 33.75), body=10.0),
        row("dying-early", death=2.0), row("dying-late", death=18.0), row("dead", death=40.0, body=33.0),
        row("swim-water", swim=0.6, water=True, pitch=-20.0, body=15.0),
        row("swim-water-down", swim=1.0, water=True, pitch=45.0),
        row("swim-air", swim=0.4, pitch=30.0), row("crawl", swim=1.0, crawling=True, pitch=10.0, body=-60.0),
        row("crawl-water", swim=1.0, crawling=True, water=True, pitch=-45.0),
        row("glide-new", glide=True, ticks=0.0, pitch=-30.0), row("glide-early", glide=True, ticks=2.5, pitch=-30.0),
        row("glide-mid", glide=True, ticks=6.0, pitch=15.0, body=75.0),
        row("glide-full", glide=True, ticks=10.0, pitch=40.0), row("glide-long", glide=True, ticks=300.0, pitch=-70.0),
        row("glide-heading-left", glide=True, ticks=12.0, pitch=-10.0, heading=0.7, body=20.0),
        row("glide-heading-right", glide=True, ticks=12.0, pitch=-10.0, heading=-1.2),
        row("glide-heading-partial", glide=True, ticks=4.0, pitch=25.0, heading=2.9, body=-30.0),
        row("glide-spinning", glide=True, spin=True, ticks=12.0, pitch=-10.0, age=7.5),
        row("glide-and-swim", glide=True, swim=0.8, water=True, ticks=12.0, pitch=-10.0),
        row("glide-sneaking", glide=True, crouching=True, ticks=12.0, pitch=-10.0),
        row("glide-dying", glide=True, death=10.0, ticks=12.0, pitch=-10.0, body=5.0),
        row("spin-alone", spin=True, pitch=-20.0, age=9.0), row("spin-dying", spin=True, death=5.0, age=9.0),
    ]
    yield {"case_id": "entity/avatar-pose", "family": "entity", "operation": "avatar_pose", "rows": rows}


def entity_living_pose_cases():
    """A living entity's pose as `LivingEntityRenderer.submit` builds it: the
    facing, the scale and the fall of a dying one (`deathTime` with its part
    of a tick, never below one) which tips over by its sideways flip."""
    rows = []
    for death_time in (0.0, 1.0, 1.25, 2.0, 5.5, 10.25, 15.0, 20.0, 20.9, 25.5):
        for body_rot in (0.0, 135.5):
            rows.append({"label": f"death-{death_time}-facing-{body_rot}",
                         "scale": float_bits(1.0), "body_rot": float_bits(body_rot),
                         "death_time": float_bits(death_time)})
    # A spider falls over by half a turn (`getFlipDegrees`).
    for death_time in (1.0, 5.5, 15.0, 25.5):
        rows.append({"label": f"spider-death-{death_time}", "scale": float_bits(1.0),
                     "body_rot": float_bits(35.5), "death_time": float_bits(death_time),
                     "flip_degrees": float_bits(180.0)})
    rows.append({"label": "scaled", "scale": float_bits(0.5), "body_rot": float_bits(-70.0),
                 "death_time": float_bits(8.5)})
    yield {
        "case_id": "entity/living-pose",
        "family": "entity",
        "operation": "living_pose",
        "rows": rows,
    }


def entity_model_layer_cases():
    """Every model layer the game builds (`LayerDefinitions.createRoots`):
    its texture size and the tree of its parts, each with its pose, cubes
    (origin, size, growth, mirroring, texture offset and scale, faces) and
    children. The reference answers one row for each layer, by name."""
    yield {
        "case_id": "entity/model-layers",
        "family": "entity",
        "operation": "model_layers",
        "rows": [{"label": "all"}],
    }


def entity_player_model_cases():
    """The player model, posed by the reference's `setupAnim` and drawn by its
    part renderer: every quad's corner positions, texture coordinates and
    normals. Both arm widths; standing, walking, crouching, looking about,
    swinging either arm at several points of the swing, swimming through
    each stroke phase, and a spectator's head alone."""
    def pose(label, slim=False, head_y_rot=0.0, x_rot=0.0, walk_position=0.0, walk_speed=0.0,
             age_in_ticks=0.0, attack_time=0.0, attack_arm="right", swim_amount=0.0,
             crouching=False, spectator=False):
        return {
            "label": label,
            "slim": slim,
            "head_y_rot": float_bits(head_y_rot),
            "x_rot": float_bits(x_rot),
            "walk_position": float_bits(walk_position),
            "walk_speed": float_bits(walk_speed),
            "age_in_ticks": float_bits(age_in_ticks),
            "attack_time": float_bits(attack_time),
            "attack_arm": attack_arm,
            "swim_amount": float_bits(swim_amount),
            "crouching": crouching,
            "spectator": spectator,
        }

    rows = [
        pose("standing"),
        pose("standing-slim", slim=True),
        pose("breathing", age_in_ticks=37.25),
        pose("walking", walk_position=3.7, walk_speed=0.83, age_in_ticks=12.5),
        pose("walking-slim", slim=True, walk_position=11.2, walk_speed=0.5, age_in_ticks=4.0),
        pose("looking", head_y_rot=-35.5, x_rot=48.0, age_in_ticks=100.75),
        pose("crouch-walk", crouching=True, walk_position=6.1, walk_speed=0.3, x_rot=20.0),
        pose("swing-early", attack_time=0.1, x_rot=-10.0),
        pose("swing-mid", attack_time=0.5, age_in_ticks=3.3),
        pose("swing-late-left", attack_time=0.85, attack_arm="left", slim=True),
        pose("swim-reach", swim_amount=1.0, walk_position=7.0, walk_speed=0.6, x_rot=30.0),
        pose("swim-pull", swim_amount=0.64, walk_position=17.5, walk_speed=0.9),
        pose("swim-return", swim_amount=1.0, walk_position=24.25, walk_speed=0.4),
        pose("swim-swinging", swim_amount=1.0, walk_position=3.0, attack_time=0.4),
        pose("spectator", spectator=True, head_y_rot=12.0),
    ]
    yield {
        "case_id": "entity/player-model",
        "family": "entity",
        "operation": "player_model",
        "rows": rows,
    }


def entity_walk_animation_cases():
    """The limb animation state: a player walking up to speed and stopping,
    standing, the position scale a baby has, a stop in the middle of a
    walk, and reads between two ticks."""
    def step(target, factor=0.4, scale=1.0, stop=False):
        return {"target": float_bits(target), "factor": float_bits(factor),
                "scale": float_bits(scale), "stop": stop}

    partials = [float_bits(value) for value in (0.0, 0.25, 0.5, 0.75, 1.0)]

    def row(label, steps):
        return {"label": label, "steps": steps, "partials": partials}

    walk = lambda distance: step(min(distance * 4.0, 1.0))
    rows = [
        row("standing", [step(0.0)] * 5),
        row("walk-up-to-speed", [walk(0.2158)] * 12),
        row("walk-then-stop", [walk(0.2158)] * 6 + [walk(0.0)] * 8),
        row("sprint", [walk(0.28)] * 10),
        row("slow-steps", [walk(0.03), walk(0.06), walk(0.0), walk(0.09), walk(0.05)]),
        row("baby-scale", [step(1.0, 0.4, 3.0)] * 6),
        row("stop-midway", [walk(0.2)] * 4 + [step(0.0, stop=True)] + [walk(0.2)] * 3),
        row("factor-one", [step(1.0, 1.0)] * 3),
        row("factor-zero", [step(1.0, 0.0)] * 3),
        row("long-run", [walk(0.2158)] * 200),
    ]
    yield {
        "case_id": "entity/walk-animation",
        "family": "entity",
        "operation": "walk_animation",
        "rows": rows,
    }
