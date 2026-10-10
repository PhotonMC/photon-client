# Parity

Photon proves behaviour by comparing it with the real Java client, never by eye. For each behaviour the Java code runs headless as an **oracle** on a controlled input, its output is captured, and the Rust code must reproduce it exactly on the same input. The system is being rebuilt as described in [ADR 0005](../adr/0005-parity-v2.md); this page describes the contract, which does not change, and how to run it today.

## Rules

- **Expected values come from Java.** No expected value is derived from Rust output or from re-implementing the algorithm inside a test. A capture that cannot be verified against running Java is not a golden.
- **Pinned inputs.** Version, inputs, initial state, seed, clock and tick are pinned; Java's numeric, ordering and timing semantics are checked where they matter (NaN and signed zero, float/double conversions, `Math` intrinsics: the oracle JVM runs with `-XX:-UseLibmIntrinsic` and Rust uses fdlibm ports).
- **Exact equality.** Comparison is exact in type and value. A tolerance needs a stated reason and limit in the case that uses it; none is universal.
- **Missing is not passing.** A missing reference or unsupported environment never counts as passing; it is reported as blocked.
- **Branches, not variants.** A new case set is opened only for a Java branch that runs for the first time or an observed difference. A copy of the same path with a different constant is a table row in an existing parametric case.
- **Tables from Java.** When the reference computes a table once and caches it (UV lock face transforms, the block registry, face occlusion shapes), the table is captured from the oracle, generated into Rust and compared in full on every run instead of being guessed.
- **Headless gaps.** The headless oracle is not the whole reference. Things loaded from assets (colour maps, fonts) can come back empty headless and silently produce wrong values. When results look suspicious, question the environment.
- **Diagnose before blocking.** Find the cause before declaring something impossible in the oracle. Two past misdiagnoses kept slices closed for weeks: `Bootstrap.bootStrap()` swallowing `System.out`, and `SharedConstants.setVersion` rejecting a second version object.

## Evidence classes

| Class | Compared | Proves | Does not prove |
| --- | --- | --- | --- |
| `client-parity` | The Java client and Rust on the same input | Client behaviour equality for the covered cases | Unmeasured scenarios, the server side |
| `server-conformance` | A packet Rust produced and a live vanilla server's answer | Wire format and protocol flow are accepted | That the client would produce the same packet or state |

Calling the oracle only to produce constants (registry ids, hashes) is not `client-parity`. A feature is complete only with `client-parity` coverage of the client state it changes.

## Running

```sh
cargo xtask parity                      # compare every family except math
cargo xtask parity --family gui         # one family (repeatable)
cargo xtask parity --capture            # recapture Java first (after oracle or case changes)
```

The pre-push hook runs parity when engine crates or reference sources changed. Requirements: the local reference installed by `python3 tools/photon.py setup` under `.data/` and a Java 25 JDK. See [setup](../development/setup.md).

## How it works today

- **Pins.** `tests/behavior/reference.json` pins the version (26.2, protocol 776, Java 25), the client jar SHA-1 and its libraries.
- **Cases.** `tools/behavior/cases/` generates cases as JSONL, one generator per family.
- **Oracle.** `tests/behavior/java/` holds the oracle classes: `Oracle.java` dispatches by family to `<Family>Oracle.run`. They call the game's own classes, built with `Unsafe` and reflection where the game expects a running client.
- **Runner.** `tools/behavior/run.py`:
  1. `capture` compiles the oracle, runs it twice, checks the two runs match, and stores the capture under `.data/captures/behavior/<identity>`. The identity hashes the jar and library hashes, JDK, OS/arch, family set, JVM flags, oracle sources, runner and inputs.
  2. `compare` feeds the same inputs to `target/release/photon-oracle` and reports the first difference.
- **Retention.** Passing runs leave no files behind; only the two newest captures per family set are kept.
- **Adapter.** `crates/photon-oracle` reads JSONL from stdin and answers each case through the production Rust code.

Exit codes: `0` passed or reference ready, `1` failed (a difference or an error), `2` blocked (missing reference, capture or adapter).

### Adapter protocol

- **Input.** Every input line has `schema_version: 1`, `case_id`, `family`, `operation` and operation fields.
- **Output.** Every output line has the same `schema_version` and `case_id` plus a `result` object, in input order. An optional `diagnostics` object is not compared.
- **Logs.** Logs go to stderr, never stdout.
- **Floats** travel as hex bit patterns.

## Families

`atlas`, `binary`, `block`, `entity`, `gui`, `image`, `json`, `math`, `model`, `movement`, `nbt`, `resources`, `sound`, `timer`, `utf8`, `window`, `wire`. `math` is captured separately because its NaN bit patterns were recorded on x86-64 Linux.

The GUI family has its own page: [GUI oracle](gui-oracle.md).

## Legacy records

`docs/records/` holds 670 per-slice contract and parity records from the first phase of the project (in Turkish). They are being converted into case metadata with a generated coverage report and will then be removed.
