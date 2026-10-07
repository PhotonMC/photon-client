# Coding standards

> English translation of [`../coding-standards.md`](../coding-standards.md). The Turkish original is authoritative; if they differ, the original wins.

This document defines how Rust code is organised and named, and which complexity limits it follows. Behavioral rules are in [AGENTS.md](../../AGENTS.md) and the [behavior testing document](behavior-testing.md); only the form and structure of the code are covered here. The sources are the Rust API Guidelines, `rustfmt`'s default style, Clippy's default lint set and the YAGNI/KISS/DRY principles.

## Crate layers

The workspace consists of six library crates and the application crate `voxel`, all under `crates/`; the repository root holds no source code. Dependencies flow downward only: a crate must not know about a crate that comes after it, and no cycles are allowed.

| Crate | Responsibility |
| --- | --- |
| `voxel-core` | Basic types and codecs that follow Java semantics: math, AABB, shapes, wire/binary/NBT/JSON/UTF-8 reading and writing, identifiers, randomness, the timer. |
| `voxel-data` | Tables generated from the oracle (`src/generated/`, never edited by hand) and the hand-written types and lookup functions over them. Carries no other logic. |
| `voxel-world` | The client's copy of the world: chunks, block states, light, entities, the local player and movement. |
| `voxel-game` | Game rules: items, components, inventory and creative menus, tooltips, attacks, options. |
| `voxel-net` | Protocol, packets, connection and session (`PlaySession`). |
| `voxel-render` | The Vulkan backend, models/atlas/meshes, sky and weather, the user interface (`gui`). |
| `voxel` | `src/`: the application (the `voxel` binary): game loop, window events, scripts. `examples/`: probes and the behavior adapter. |

New code goes into the lowest crate its responsibility belongs to. When a type moves to another crate, its `pub(crate)` fields become methods or `pub`; since an inherent `impl` cannot be written for another crate's type, a helper becomes a free function.

## Module layout

- A module has a single responsibility and is named after it (`sections`, `controls`, `atmosphere`). Catch-all modules such as `utils` or `helpers` are not created.
- The module doc (`//!`) says in a sentence or two what the module is; if the reference has a counterpart, it names the Java class.
- User interface code lives under `voxel-render::gui`: drawing (`frame`, `atlas`, `font`, `items`), the HUD (`hud`), the text box (`edit_box`) and the screens (`screen`).
- A helper shared by several modules goes into the lowest common crate and the right module there. Before writing a new helper, check whether one exists: fixed-size reads `voxel_core::wire::take`, JSON files `voxel_core::json::read_object`, ARGB `voxel_render::tint`, Java `Math` rules `java_math` in `voxel-world`.

## File size

- **A code file does not exceed 500–600 lines** (Rust, Python, Java; blank lines and comments included). A file approaching the limit is split by responsibility together with the new feature; the split is done before adding, not afterwards.
- Splitting is by responsibility, not by line count: `impl` blocks into submodules (`entity/state.rs`, `controls/events.rs`), tests by topic (`tests/entities.rs`), packet parsers by packet family. A module name says what is in it (no `part` or `misc`).
- Generated tables (`voxel-data/src/generated/`) and data produced by the oracle are outside this limit; when the generator is removed (the D tasks) that exception goes too. Existing files over the limit are reduced through [A06](../tasks/A06-dev-dosyalar.md) and the tasks that depend on it; a new file, or an addition to an existing one, must not leave a file over the limit.

## Naming and comments

- Rust naming rules apply: types `CamelCase`, functions and variables `snake_case`, constants `SCREAMING_SNAKE_CASE`. Full words are used instead of abbreviations.
- The doc of a function that corresponds to a reference method names that method in backticks (`AbstractContainerMenu.doClick`). Original code is not copied; behavior is described in our own sentences.
- A comment does not repeat what the code does; it says why, which reference behavior it preserves, or the units (GUI unit, tick, bit pattern).
- Comments in code are in English; documents and commit messages are in Turkish.

## Complexity limits

The limits are applied to every crate through `clippy.toml` at the root and `[workspace.lints.clippy]` in `Cargo.toml`:

| Measure | Limit | Lint |
| --- | --- | --- |
| Cognitive complexity | 25 | `clippy::cognitive_complexity` |
| Function length | 100 lines | `clippy::too_many_lines` |
| Parameter count | 7 | `clippy::too_many_arguments` |
| Type complexity | 250 | `clippy::type_complexity` |

Code over a limit is split first: a helper function with a single responsibility, a struct that carries the state together (`SectionMeshes`, `Controls`), or a context struct that gathers parameters (`ScreenDraw`, `FrameView`). Where splitting would hurt readability, the exception is written with `#[expect(lint, reason = "...")]`, never `allow`: `expect` warns once the code is fixed, asking for the marker to be removed. Accepted reasons:

- A function kept whole to compare a reference method line by line.
- Vulkan objects created in the order the API demands, or a command buffer recorded in draw order.
- A dispatch `match` with one arm per packet or component id.
- Test, probe and adapter scenarios that read as one flow from start to end (a file-level `#![expect]`).

## Reference data

- When Java **computes** a value from other data, we compute it too (the state cache, values derived from shapes); we do not keep it as a table. When Java **defines** it in code or under `data/`, we keep the definition as a typed, readable record. The status is in the [roadmap](../architecture-roadmap.md).
- Definitions are under `crates/voxel-data/src/generated` and are written by a generator in `tools/generate/`; the doc at the top of the file names the generator. A generated file is never edited by hand.
- The form should read like the reference's code: one `Block::of(...)` chain per block, property constants named after the Java field, records written with field names. To remove repetition, no index lists, bit strings, run-length arrays or raw network bytes are used; on a version jump the diff must show the real change.
- Hand-written code that reads a table (`definition`, `registry`, `shape`) is kept apart from the generated file. Floats are written in the shortest decimal form that returns to the reference bits exactly (`tools/generate/common.py`).
- All definitions are regenerated with `python3 tools/generate/all.py <capture>` and compared from start to end through the behavior adapter in every parity run.

## Preserving behavior

- Numeric code keeps the reference's order of operations and types: `f32`/`f64` conversions happen in the same place, the order of additions does not change; where bit equality is needed, comparison is by bits.
- Java's NaN and `-0.0` rules differ from Rust's. Wherever NaN or a signed zero can occur, the helpers in `java_math` and `voxel_core::math` are used instead of `f64::min`/`max`.
- Integer overflow is checked in every profile; deliberate wrapping is written with `wrapping_*`.
- A refactoring does not change behavior. If it is doubtful whether a change alters behavior, the change is not made and the difference is recorded.

## Error handling

- An operation that can fail returns `Result` and propagates with `?`. Conversion between layers is written with `From` (`wire::ReadError` → `binary::ReadError`); the same mapping function is not repeated module by module.
- `expect` is used only for an invariant that was checked earlier, with a message that says so; `unwrap` is not used in production code.
- Diagnostic output is written with the `log` macros (`error!`, `warn!`, `info!`, `debug!`); the level is chosen with `VOXEL_LOG`, `info` by default. `println!` is only for the `key=value` lines of probe and profile reports. A packet that could not be parsed and is ignored leaves its reason with `debug!`.

## `unsafe`

- Every `unsafe` block has a `// SAFETY:` justification above it and every `unsafe fn` a `# Safety` section; `clippy::undocumented_unsafe_blocks` and `clippy::unwrap_used` are enabled as warnings (`unwrap` is allowed in tests).
- A raw byte view is taken with `as_bytes`; only padding-free `repr(C)` types may be `Plain`, and their size is fixed by a `const` assertion. The shared invariants are in the "Safety conventions" section at the top of `vulkan/mod.rs`.

## UI screens

A container screen implements the `voxel_render::gui::screen::Screen` trait. Layout, highlight, background and tooltip are in `ContainerLayout`; mouse and drag state in `ContainerMouse`; drawing goes through the `ScreenDraw` context. The clicks a screen wants come back as `ScreenAction`, and the application passes them to the session. A new screen uses these parts and does not write its own copies.

## Verification

Every change passes the following steps; the warning count is zero. One command runs all of them: `python3 tools/voxel.py check` (with `--capture` if the oracle sources changed); it prints only errors and one summary line, ending in `CHECK OK`.

1. `cargo fmt --all`
2. `cargo build --release --workspace --all-targets`
3. `VOXEL_ASSETS=$PWD/.data/minecraft/assets/jar/assets cargo test --release --workspace --all-targets`
4. `cargo clippy --release --workspace --all-targets`
5. The parity comparison through the [behavior runner](behavior-runner.md).
