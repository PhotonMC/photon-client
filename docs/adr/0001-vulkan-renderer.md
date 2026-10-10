# ADR 0001: GLFW and raw Vulkan 1.2 through `ash`

- Status: accepted (2026-09-15)

## Context

The 26.2 client ships two graphics paths. `PreferredGraphicsApi` has `DEFAULT`, `OPENGL` and `VULKAN`; `DEFAULT` normalises to OpenGL at startup, and Vulkan is a production alternative, not an experiment. The Vulkan path uses GLFW for windows and input, Vulkan 1.2, the Vulkan Memory Allocator, shaderc for GLSL to SPIR-V (Vulkan 1.2 target, `auto_bind_uniforms` and `auto_map_locations`), SPIRV-Cross for reflection, and MoltenVK with the portability extensions on macOS.

## Decision

Photon renders with GLFW, raw Vulkan 1.2 through `ash`, and shaderc at build time. This mirrors the reference's Vulkan backend: same window library, API version, shader compiler and compile options.

`wgpu` and `winit` were rejected because they insert their own layer between our code and the API:

- `wgpu` compiles shaders through naga, which yields different SPIR-V and floating-point behaviour from the same GLSL.
- It normalises pipeline state across backends, so the reference's blaze3d pipelines can only be approximated.
- When a pixel differs, the cause can no longer be pinned to the reference, our code or the abstraction.
- Window focus, resize, key repeat and event order are library-specific, which is why GLFW is used instead of `winit`.

## Consequences

- Parity claims about pixels are made against the Java client configured with `preferredGraphicsBackend=vulkan`. A measured fixed scene differs from Java's default OpenGL path in 22 pixels at grazing angles (texel boundary rounding), deterministically per backend.
- Rendering parity is checked on geometry, draw lists and pipeline state rather than on screenshots (see [parity](../parity/README.md)).
- The renderer owns its GPU-driven optimisations (see [ADR 0005](0005-parity-v2.md) and [rendering](../architecture/rendering.md)); they must not change what is drawn.
