"""The `wire` family."""

from cases.wire.configuration import cases as configuration
from cases.wire.framing import cases as framing
from cases.wire.login import cases as login
from cases.wire.status import cases as status


def wire_cases():
    yield from configuration()
    yield from framing()
    yield from login()
    yield from status()
