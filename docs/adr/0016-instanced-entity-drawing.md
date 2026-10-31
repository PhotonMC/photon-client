# ADR 0016: Instanced entity drawing

- Status: accepted (2026-10-31)

## Context

Every frame the client poses each visible mob on the CPU, builds its vertices quad by quad and uploads them into a dynamic buffer ([issue 25](https://github.com/PhotonMC/photon-client/issues/25)). With many entities of one kind (the `perf-pigs` route) this is the largest cost on the main thread, although most of the work repeats: the posed parts of a pig only depend on its animation state, not on where it stands. The owner approved moving the placement of an entity to the GPU.

## Decision

- **Model-space mesh, per-instance placement.** The posed parts of a model (after the `setup_*` of its renderer) are built once into a mesh in model space under an identity root pose. Each instance carries the placement matrix (position, body yaw and scale, the entity's root transform), the packed light, the overlay code and the colour. The vertex shader applies the placement and rotates the normal; the normal travels in `tint.xyz` as it does today.
- **Cache key.** The posed part transforms plus the model, the baby flag and the texture rectangle. Never entity fields: two entities with the same posed parts share a mesh, and an entity whose animation changed gets a new key without any invalidation logic.
- **Eviction.** The GPU mesh cache is bounded, evicts the least recently used mesh, and is cleared whenever the entity texture atlas or the baked models are rebuilt (resource pack or server pack changes, `reload`).
- **CPU path stays.** The CPU vertex path remains the reference that parity and the oracle compare against. These stay on it at first: glowing entities, invisible or ghost-drawn entities, non-uniform hook scale, and entities with extra layers (saddle, wool, armor, clothing, elytra, heads; see the layering rules). Each later slice may move one more class over.
- **Tolerance.** The GPU applies the placement in f32 in a different order than the JOML-exact CPU path. A vertex may differ by at most 1/256 of a block (about a texel of a model pixel) from the CPU result, with no difference in the choice of faces, winding, uv, light or colour. A unit test builds both paths for the same inputs and enforces this. Pinned shots between the two paths stay inside the noise band between two runs of the same path; otherwise the slice is not kept.
- **Hit rate.** A throwaway count on `perf-pigs` (not committed) found one distinct key per frame for about 98,000 pig builds a second, because the bench pigs stand still. That is the best case: walking pigs differ in limb and head angles, so in play the keys per frame can approach the number of entities. A miss costs one model-space build, no more than today's per-entity build, and the slice's A/B must also be run with moving entities before it is kept. If keys track entity count there, the design falls back to instancing per part (each cuboid built once per model, one instance per part with its world matrix).
- **Order of work.** Quadruped families without layers (pig, cow, sheep body), then the other mob families, armor stands, dropped items, then particles. A slice is kept only if it wins in an interleaved A/B on `perf-pigs` and `perf-busy`.
- **Culling and winding.** Models are mirrored in model space (`scale(-1, -1, 1)`); the instanced pipeline uses the same cull mode and winding as the CPU path, checked by a pinned shot.

## Consequences

- Observable behaviour is unchanged except within the stated tolerance; the CPU path keeps parity testable.
- Two code paths exist for the covered entities until the CPU one is no longer needed for the exclusions above.
- The Vulkan validation layer is not installed on the development machine; instanced-pipeline checks rely on shots and the unit test, and this is reported, not counted as validation.
