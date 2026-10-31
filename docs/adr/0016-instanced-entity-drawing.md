# ADR 0016: Instanced entity drawing

- Status: accepted (2026-10-31)

## Context

Every frame the client poses each visible mob on the CPU, builds its vertices quad by quad and uploads them into a dynamic buffer ([issue 25](https://github.com/PhotonMC/photon-client/issues/25)). With many entities of one kind (the `perf-pigs` route) this is the largest cost on the main thread, although most of the work repeats: the posed parts of a pig only depend on its animation state, not on where it stands. The owner approved moving the placement of an entity to the GPU.

## Decision

- **Per-part meshes, per-part instances.** Each part of a model that has cubes is built once into a mesh in its own space, with no pose. A mob is one instance of each of its drawn parts: the pose of that part from the root (the entity's placement and the parts' poses above it, as the CPU path composes them), the packed light, the overlay code and the colour. The vertex shader applies the matrix and rotates the normal; the normal travels in `tint.xyz` as it does today. A first design shared one mesh per whole pose, keyed by the posed part transforms; walking mobs each have their own pose, so none shared a mesh and the path was slower than the CPU's (walking pigs 5.1 to 7.3 ms). Per part, standing and walking mobs share alike.
- **Cache key.** The model's layer name and the texture rectangle; the part's number in the traversal is the mesh's index. Never entity fields.
- **Eviction.** The meshes are kept on the CPU and the frame uploads only those its instances use, so the GPU holds no cache that could go stale. The CPU cache starts again when it holds more than 4096 meshes. A texture's rectangle is part of the key, so a moved texture (resource or server pack) builds new meshes; a mesh whose model changed under the same layer name would not, which the baked models do not do while the game runs.
- **CPU path stays.** The CPU vertex path remains the reference that parity and the oracle compare against. These stay on it at first: glowing entities, invisible or ghost-drawn entities, non-uniform hook scale, and entities with extra layers (saddle, wool, armor, clothing, elytra, heads; see the layering rules). Each later slice may move one more class over.
- **Tolerance.** The GPU applies the placement in f32 in a different order than the JOML-exact CPU path. A vertex may differ by at most 1/4096 of a block (a sixteenth of a model pixel, which is 1/16 block) from the CPU result, with no difference in the choice of faces, winding, uv, light or colour. A unit test builds both paths for the same inputs and enforces this. Pinned shots between the two paths stay inside the noise band between two runs of the same path; otherwise the slice is not kept.
- **Order of work.** Quadruped families without layers (pig, cow, sheep body), then the other mob families, armor stands, dropped items, then particles. A slice is kept only if it wins in an interleaved A/B on `perf-pigs` and `perf-busy`.
- **Culling and winding.** Models are mirrored in model space (`scale(-1, -1, 1)`); the instanced pipeline uses the same cull mode and winding as the CPU path, checked by a pinned shot.

## Consequences

- Observable behaviour is unchanged except within the stated tolerance; the CPU path keeps parity testable.
- Two code paths exist for the covered entities until the CPU one is no longer needed for the exclusions above.
- The Vulkan validation layer is not installed on the development machine; instanced-pipeline checks rely on shots and the unit test, and this is reported, not counted as validation.
