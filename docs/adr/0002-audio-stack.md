# ADR 0002: `cpal` output with our own mixer, `lewton` decoding

- Status: accepted (2026-10-03)

## Context

The reference plays sound through OpenAL Soft (LWJGL): 30 static plus streaming channels, each an OpenAL source with position, pitch, gain, looping, relative position and linear clamped distance attenuation (reference distance 1, rolloff 0). It decodes Ogg/Vorbis with JOrbis 0.0.17 into 16-bit little-endian PCM. Selection and timing live in `SoundManager`, `SoundEngine`, `ChannelAccess` and `MusicManager`.

Sound is two questions:

1. Which sound plays, when, and with which volume, pitch and position. This is pure logic and can be compared with Java.
2. What is heard. This is output mixing, which can only be bit-identical with the same OpenAL Soft build.

Three output options were considered:

- **Binding OpenAL Soft directly.** These are the same calls as the reference, but OpenAL Soft is missing on Windows, differs on macOS, and is LGPL to ship.
- **`cpal` plus our own mixer.** One dependency, it works everywhere, and the logic stays comparable.
- **A high-level library (`rodio`, `kira`).** Its attenuation and pitch model would replace the reference's channel contract.

## Decision

- **Output:** `cpal` with our own mixer, behind the same channel interface the logic layer drives.
- **Decoding:** `lewton`, compared sample by sample with JOrbis output.
- The logic layer (selection, channel parameters, categories, pausing) is held to `client-parity` with the Java oracle.

## Consequences

- No claim of audible parity is made.
- An OpenAL Soft backend can be added later as a second implementation of the same channel interface.
- If `lewton` diverges from JOrbis, `symphonia` is tried next; a JOrbis port is the last resort.
