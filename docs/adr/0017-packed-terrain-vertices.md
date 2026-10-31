# ADR 0017: Packed terrain vertices

- Status: accepted (2026-10-31)

## Context

A section mesh vertex was 28 bytes: three `f32` for the position, two for the texture coordinate, four colour bytes and two light bytes with two bytes of padding. The terrain is the largest part of the scene's GPU time on the `perf-still` route ([issue 24](https://github.com/PhotonMC/photon-client/issues/24)), and the owner brought the issue back into the performance program.

## Decision

- **Layout, 20 bytes.** The position is three unsigned 16-bit steps of 1/2048 block from the mesh's origin corner, the two light bytes follow in the same eight bytes, then the texture coordinate as two `f32` and the four colour bytes. The vertex shader reads the first eight bytes as four 16-bit words and rebuilds the position and the light.
- **The origin corner** is the lowest position of the mesh, floored to a whole number of blocks (from the mesh's own origin, so a small integer). It travels with the draw: the same push constant write that already carries the section's fade carries it, as two words (x and y as 16-bit integers, then z). No extra call per section.
- **Tolerance.** A position is rounded to the nearest step, so it differs from the CPU mesh by at most 1/4096 block (half a step), the same figure as [ADR 0016](0016-instanced-entity-drawing.md). Block models are built in sixteenths of a block and are exact. The corner is a whole number of blocks, so two meshes that share a vertex put it on the same grid and round it alike: no cracks open between sections. The range from the corner is 32 blocks; a position beyond is held at its end (a section is 16 blocks and its models reach a few blocks past it).
- **Texture coordinates stay `f32`.** Their small nudge keeps nearest-sampled texels on their edges; quantising them is not worth the risk for four bytes.
- **The CPU mesh does not change.** Packing happens when the mesh is built (on the worker that builds it) and the upload sends the packed bytes. The face groups, the translucent sort, parity and the oracle read the `f32` mesh. A mesh without packed vertices (a moving block, a test) is packed by the upload.

## Consequences

- Observable behaviour is unchanged within the stated tolerance; a unit test packs and unpacks and checks it, and that a shared vertex rounds alike against two corners.
- Pinned shots of the packed and the unpacked binary differ no more than two runs of the same binary.
- Vertex memory of sections is a third smaller.
- The vertex shader depends on the push constants at bytes 168 to 175 carrying the corner; nothing else may write there during a terrain draw.
