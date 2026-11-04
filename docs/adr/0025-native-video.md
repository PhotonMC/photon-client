# ADR 0025: Native video on game pages

- Status: accepted (2026-11-05); built, audio open

## Context

[ADR 0021](0021-client-ui-technology.md) dropped the web engine and left game page videos as files the client plays itself. The decoder has to build from cargo alone (no assembler or system codec libraries on a player's or contributor's machine), be royalty free, and decode the same file the website plays in a `<video>` element.

## Decision

- **Format.** A WebM file with one 8-bit 4:2:0 AV1 video track. Current Chrome, Edge and Firefox play it natively; Safari plays AV1 only on devices with a hardware decoder (per [caniuse's AV1 table as summarised by Bitmovin](https://bitmovin.com/av1-playback-support/); not checked against Apple's own notes), so on other Safari devices the website shows the poster image instead. AV1 has no royalties, and the container is simple to read. Other codecs, 10-bit and 4:2:2/4:4:4 streams are refused with a log line; the page keeps its placeholder.
- **Decoder.** `rav1d` (BSD-2-Clause, a Rust port of dav1d) without its assembly feature, so it needs no assembler. `webm-iterable` (MIT) reads the container. The new crate is `photon-video`; its single `unsafe` module is the dav1d-shaped call surface and is listed in the lint's allowed places.
- **Measured** (1280x720, 30 fps, 10 s synthetic clip, no assembly): 11.7 ms per frame on one thread (worst packet 89 ms), 3.7 ms per frame on four. The player uses two threads, so a clip decodes well inside its 33 ms frame time.
- **Limits.** The file is at most 32 MiB and each side at most 1920 pixels. The address is `https`, or `http` to this machine; redirects are not followed (an operator names the file itself).
- **Playback.** A thread fetches, demuxes and decodes, shows each picture at its time, drops one that is more than 120 ms late and loops. It runs only while the page is open and stops when the page closes or shows another game. Pictures reach the surface texture of [ADR 0021](0021-client-ui-technology.md) (about 1.9 ms per 1280x720 update); a persistent staged image is the next step if a clip ever plays during the game.
- **Audio** is not played yet.
- **The backend** lists only `https` video addresses; the website plays the same files.

## Consequences

- No browser engine is shipped; a game page costs one decoder thread while it shows a video.
- Operators encode to AV1 WebM (for example `ffmpeg -i in.mp4 -c:v libaom-av1 -crf 32 -pix_fmt yuv420p out.webm`).
- The file is read into memory before playing; streaming and seeking are not built.
