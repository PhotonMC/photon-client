# Coding standards

Rules that a machine can check are enforced by `cargo xtask lint`, clippy and rustfmt in the git hooks and CI. The rest are reviewed by people. Everything in the repository (code, comments, documents, commit messages) is written in English.

## Enforced limits

| Rule | Limit | Checked by |
| --- | --- | --- |
| Code lines per file (comments, blanks and inline `#[cfg(test)]` modules excluded) | 500 | `xtask lint` |
| Cyclomatic complexity per function (`if`, loops, extra `match` arms and guards, `&&`, `\|\|`, `let ... else`) | 10 | `xtask lint` |
| Code lines per function | 60 | `xtask lint` |
| Parameters per function | 6 | `xtask lint` |
| Nested control flow depth | 4 | `xtask lint` |
| File and directory names | `snake_case`, at most 24 characters, not vague (`utils`, `misc`, `helpers`, `common`), not repeating the parent directory (`gui/gui_button.rs`) | `xtask lint` |
| Crate dependencies | Only the edges in `xtask/lint.toml` | `xtask lint` |
| Source text | ASCII only; use `\u{..}` escapes in strings | `xtask lint` |
| `unsafe` | Only under `crates/photon-render/src/vulkan`, each block with a `// SAFETY:` comment | `xtask lint`, clippy |
| Terminal output in libraries | No `print!`, `println!`, `eprintln!` or `dbg!`; use `log` | `xtask lint` |
| Registry and protocol ids | Never compared with number literals; resolve them by name | `xtask lint` |
| Suppressing size or complexity lints | Not allowed | `xtask lint` |
| Formatting | rustfmt defaults | `cargo fmt --check` |
| Lints | Clippy with `-D warnings`, `unwrap_used` and `undocumented_unsafe_blocks` | clippy |
| Commit messages | Conventional Commits, English ASCII, subject at most 72 characters, no tool attribution | `commit-msg` hook |

Generated code (`photon-data/src/generated/`, files marked as generated) is exempt from the size rules.

### The baseline

Violations that existed when a rule was introduced are listed in `xtask/lint-baseline.toml`, each capped at its recorded value. The cap only moves down:

- When code improves, `cargo xtask lint` lowers or removes the entry. Commit the updated file; CI fails if it is stale.
- A new violation, or a value above its cap, fails the lint.
- Never raise a value or add an entry by hand.

## Structure

- **Lowest crate.** New code goes into the lowest crate its responsibility belongs to. A helper shared by several modules goes into the lowest common crate.
- **Single responsibility.** A module has a single responsibility and is named after it.
- **Splitting.** A file approaching the limit is split by responsibility before more is added, never by numbered parts.
- **Module layout.** Use `foo.rs` with `foo/` beside it; no `mod.rs`.
- **Tests.** Inline tests go at the end of a file as `#[cfg(test)] mod tests`; larger test sets go in a `tests/` folder split by topic.
- **Module docs.** The module doc (`//!`) says what the module is and names the Java class it mirrors, if any.
- **Look first.** Before writing a helper, search for an existing one. Examples: `photon_core::wire::take` for fixed-size reads, `photon_core::json::read_object` for JSON objects, `photon_core::math` and `java_math` for Java float rules.
- **Typed ids.** Entity, entity type, item and block state ids are newtypes in `photon_core::ids`, built once where a number enters (a wire read, a file, a fixture) and never converted with `From<i32>`. Raw registry tables stay `i32` and are compared through `.0`.
- **Clippy beyond the defaults.** `[workspace.lints.clippy]` turns on a pedantic subset: `uninlined_format_args`, `explicit_iter_loop`, `semicolon_if_nothing_returned`, `cloned_instead_of_copied`, `bool_to_int_with_if`, `unnested_or_patterns`, `manual_assert`, `single_match_else` and `if_not_else`. The rest of `pedantic` stays off on purpose: the `cast_*`, `float_cmp`, `manual_midpoint` and `match_same_arms` lints rewrite arithmetic or branches that mirror Java; `wildcard_imports` fights the `use super::*` layout; `doc_markdown` and `similar_names` trip over Java names and ported maths; `too_many_lines` is replaced by the xtask limits; the `must_use` and `*_doc` lints add API noise. `redundant_closure_for_method_calls` was tried and dropped because it produced long paths and wrapped lines. A lint is added only when it ends at zero findings, with no `#[allow]`.
- **Spelling.** `typos` runs from `cargo xtask check` when installed (skipped otherwise, like `cargo-deny`) and in CI. `_typos.toml` excludes the legacy Turkish records and ported hex tables, and lists words that are the reference's own spelling or project terms.

## Following the reference without copying its flaws

- **Observable behaviour.** Parity is about observable behaviour: packets, state, draw output and timing. Internal structure is ours to choose: Java's class hierarchies, singletons, needless copies and wrong assumptions are not carried over.
- **Cached tables.** When the reference caches a table, we keep it as data that is compared with the oracle; the algorithm that produces it may differ, but the output may not.
- **Numeric semantics.** Numeric code keeps the reference's operation order and types: `f32`/`f64` conversions happen in the same place, and the order of additions does not change. Where NaN or signed zero can occur, use the Java-rule helpers instead of `f64::min`/`max`. Integer overflow is checked in every profile; deliberate wrapping uses `wrapping_*`.
- **Refactoring.** A refactoring does not change behaviour. If unsure, measure with parity first.
- **Citing Java.** Original Minecraft code is never copied into this repository. A function that mirrors a reference method names it in its doc (`AbstractContainerMenu.doClick`) and describes the behaviour in our own words.

## Errors and logging

- **`Result` and `?`.** Fallible operations return `Result` and propagate with `?`; conversions between layers use `From`.
- **`expect`, not `unwrap`.** `expect` is only for invariants checked earlier, with a message that says so; `unwrap` is not used outside tests.
- **Logging.** Diagnostics use the `log` macros, and `PHOTON_LOG` selects the level. A packet that is ignored because it could not be parsed leaves its reason at `debug!`.

## `unsafe`

- **Justification.** Every `unsafe` block has a `// SAFETY:` comment and every `unsafe fn` a `# Safety` section.
- **Byte views.** Raw byte views go through `as_bytes`. Only padding-free `repr(C)` types may be `Plain`, with their size fixed by a `const` assertion.
- **Invariants.** The shared invariants are documented at the top of `photon-render/src/vulkan.rs`.
