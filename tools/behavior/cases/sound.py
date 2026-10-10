"""Music manager cases, used by the `gui` family until it moves to Rust."""

from cases.helpers import *


def sound_music_cases():
    """The reference's own music manager: the wait before the first song, the wait after one, music that
    replaces the song playing, none to play, the volume fading a song out, and each music frequency."""
    game = {"sound": "minecraft:music.game", "min": 12000, "max": 24000, "replace": False}
    short = {"sound": "minecraft:music.menu", "min": 20, "max": 600, "replace": True}
    other = {"sound": "minecraft:music.creative", "min": 12000, "max": 24000, "replace": False}
    end = {"sound": "minecraft:music.end", "min": 6000, "max": 24000, "replace": True}

    def tick(music, volume=1.0, active=None):
        step = {"music": music, "volume": float_bits(volume)}
        if active is not None:
            step["active"] = active
        return step

    def row(label, ticks, frequency="DEFAULT", seed=7):
        return {"label": label, "frequency": frequency, "seed": seed, "ticks": ticks}

    rows = [
        row("first-song", [tick(game)] * 105),
        row("short-delays-song-ends", [tick(short)] * 40 + [tick(short, active=False)] * 3 + [tick(short)] * 30),
        row("no-music", [tick(None)] * 5),
        row("music-comes-after-none", [tick(None)] * 3 + [tick(game)] * 105),
        row("replace-current", [tick(game)] * 105 + [tick(end), tick(end)] + [tick(end)] * 10, seed=11),
        row("creative-does-not-replace", [tick(game)] * 105 + [tick(other)] * 3),
        row("constant", [tick(game)] * 105 + [tick(game, active=False)] * 3 + [tick(game)] * 105, frequency="CONSTANT"),
        row("frequent", [tick(game)] * 105 + [tick(game, active=False)] * 3 + [tick(game)] * 3, frequency="FREQUENT", seed=3),
        row("fade-out", [tick(game)] * 105 + [tick(game, volume=0.0)] * 200),
        row("fade-down-and-back", [tick(game)] * 105 + [tick(game, volume=0.4)] * 60 + [tick(game, volume=1.0)] * 400),
        row("silent-biome-after-song", [tick(game)] * 105 + [tick(None, active=False)] * 3 + [tick(None)] * 3),
    ]
    yield {"case_id": "gui/music-manager", "family": "gui", "operation": "music_manager", "rows": rows}
