"""The `wire` family: what several `all` cases share."""

from cases.wire.common import *


def server_hello_hex(server_bytes, public_key_hex, challenge_hex, should_authenticate):
    return (varint_hex(len(server_bytes)) + server_bytes.hex()
            + varint_hex(len(bytes.fromhex(public_key_hex))) + public_key_hex
            + varint_hex(len(bytes.fromhex(challenge_hex))) + challenge_hex
            + ("01" if should_authenticate else "00"))

chat_ids = {"full": 0, "system": 1, "hidden": 2}

hand_ids = {"left": 0, "right": 1}

particle_ids = {"all": 0, "decreased": 1, "minimal": 2}
