"""Entity animation scenarios: bosses."""

from cases.helpers import *
from cases.movement.entities.common import *


def rows():
    return [
        # A warden's pose starts its roar, sniff, emerge and dig animations; its attack event stops
        # the roar and starts the attack, the sonic boom event its boom, the tendril event its
        # pulse; its heart beats every forty ticks, faster as its anger synced to the client grows.
        living("warden-calm", "warden", 100, [], extras=["warden"]),
        living("warden-poses", "warden", 60,
               [{"tick": 4, "type": "frog_data", "pose": "EMERGING"},
                {"tick": 20, "type": "frog_data", "pose": "DIGGING"},
                {"tick": 30, "type": "frog_data", "pose": "ROARING"},
                {"tick": 45, "type": "frog_data", "pose": "SNIFFING"}], extras=["warden"]),
        living("warden-roars-then-attacks", "warden", 50,
               [{"tick": 5, "type": "frog_data", "pose": "ROARING"},
                {"tick": 20, "type": "entity_event", "event": 4}], extras=["warden"]),
        living("warden-booms", "warden", 40, [{"tick": 6, "type": "entity_event", "event": 62}],
               extras=["warden"]),
        living("warden-tendrils", "warden", 20,
               [{"tick": 6, "type": "entity_event", "event": 61}], extras=["warden"]),
        living("warden-agitated", "warden", 90, [{"tick": 1, "type": "warden_data", "anger": 40}],
               extras=["warden"]),
        living("warden-angry", "warden", 40, [{"tick": 1, "type": "warden_data", "anger": 80}],
               extras=["warden"]),
        living("warden-furious", "warden", 40, [{"tick": 1, "type": "warden_data", "anger": 150}],
               extras=["warden"]),
        # A creaking's attack event runs its attack for fifteen ticks and its sway event its
        # invulnerable animation for eight; a torn-down one (heart bound) starts its death animation
        # and counts death time; its stride is quicker and capped at three. The player stands off, as the
        # reference's own player here has no connection to look pushers up in.
        living("creaking-idle", "creaking", 30, [], extras=["creaking"], player_x=30.5),
        living("creaking-attacks", "creaking", 40, [{"tick": 5, "type": "entity_event", "event": 4}],
               extras=["creaking"], player_x=30.5),
        living("creaking-attacks-twice", "creaking", 50,
               [{"tick": 5, "type": "entity_event", "event": 4},
                {"tick": 12, "type": "entity_event", "event": 4}], extras=["creaking"], player_x=30.5),
        living("creaking-sways", "creaking", 30, [{"tick": 5, "type": "entity_event", "event": 66}],
               extras=["creaking"], player_x=30.5),
        living("creaking-walks", "creaking", 20,
               [{"tick": t, "type": "move", "xa": 800, "ya": 0, "za": 0} for t in range(1, 15)],
               extras=["creaking"], player_x=30.5),
        living("creaking-sprints", "creaking", 20,
               [{"tick": t, "type": "move", "xa": 4000, "ya": 0, "za": 0} for t in range(1, 15)],
               extras=["creaking"], player_x=30.5),
        living("creaking-frozen", "creaking", 20,
               [{"tick": 2, "type": "creaking_data", "can_move": False, "active": True}],
               extras=["creaking"], player_x=30.5),
        living("creaking-tears-down", "creaking", 60,
               [{"tick": 2, "type": "creaking_data", "home": True},
                {"tick": 5, "type": "creaking_data", "tearing_down": True},
                {"tick": 6, "type": "death"}], extras=["creaking"], player_x=30.5),
        living("creaking-dies-unbound", "creaking", 40, [{"tick": 3, "type": "death"}],
               extras=["creaking"], player_x=30.5),
        # An armadillo's synced state starts its roll up (rolling), peek (scared, from the
        # middle of it when the state has just come) and roll out (unrolling) animations and
        # decides when it hides in its shell; the peek event restarts the peek; a scared one's head
        # is held to its body.
        living("armadillo-idle", "armadillo", 20, [], extras=["armadillo"]),
        living("armadillo-rolls", "armadillo", 30, [{"tick": 4, "type": "armadillo_data", "state": 1}],
               extras=["armadillo"]),
        living("armadillo-rolls-and-is-scared", "armadillo", 80,
               [{"tick": 4, "type": "armadillo_data", "state": 1},
                {"tick": 14, "type": "armadillo_data", "state": 2}], extras=["armadillo"]),
        living("armadillo-peeks-again", "armadillo", 80,
               [{"tick": 4, "type": "armadillo_data", "state": 2},
                {"tick": 30, "type": "entity_event", "event": 64}], extras=["armadillo"]),
        living("armadillo-unrolls", "armadillo", 60,
               [{"tick": 4, "type": "armadillo_data", "state": 2},
                {"tick": 20, "type": "armadillo_data", "state": 3},
                {"tick": 55, "type": "armadillo_data", "state": 0}], extras=["armadillo"]),
        living("armadillo-state-resent", "armadillo", 40,
               [{"tick": 4, "type": "armadillo_data", "state": 2},
                {"tick": 20, "type": "armadillo_data", "state": 2}], extras=["armadillo"]),
        # A sniffer's synced state stops every animation and starts its own (happy, scenting,
        # sniffing, digging, rising; searching has none) even when it is sent again.
        living("sniffer-idle", "sniffer", 20, [], extras=["sniffer"]),
        living("sniffer-states", "sniffer", 70,
               [{"tick": 4, "type": "sniffer_data", "state": 1},
                {"tick": 12, "type": "sniffer_data", "state": 2},
                {"tick": 20, "type": "sniffer_data", "state": 3},
                {"tick": 28, "type": "sniffer_data", "state": 4},
                {"tick": 36, "type": "sniffer_data", "state": 5},
                {"tick": 44, "type": "sniffer_data", "state": 6},
                {"tick": 52, "type": "sniffer_data", "state": 0}], extras=["sniffer"]),
        living("sniffer-digs", "sniffer", 40, [{"tick": 4, "type": "sniffer_data", "state": 5}],
               extras=["sniffer"]),
        living("sniffer-state-resent", "sniffer", 40,
               [{"tick": 4, "type": "sniffer_data", "state": 5},
                {"tick": 20, "type": "sniffer_data", "state": 5}], extras=["sniffer"]),
        living("sniffer-searches", "sniffer", 30, [{"tick": 4, "type": "sniffer_data", "state": 4}],
               extras=["sniffer"]),
    ]
