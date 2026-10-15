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
cargo xtask parity                      # compare every family
cargo xtask parity --family gui         # one family (repeatable)
cargo xtask parity --recapture          # capture Java again even for known keys
```

The pre-push hook runs parity when engine crates or reference sources changed. Requirements: the local reference installed by `cargo xtask setup` under `.data/` and a Java 25 JDK. See [setup](../development/setup.md).

## How it works today

- **Pins.** `tests/behavior/reference.json` pins the version (26.2, protocol 776, Java 25), the client jar SHA-1, its libraries, and each family's oracle sources.
- **Cases.** The `parity-cases` crate (`parity/cases/`) generates the cases as JSONL, one generator per family.
- **Oracle.** `tests/behavior/java/` holds the oracle classes: `Oracle.java` dispatches by family to `<Family>Oracle.run`. They call the game's own classes, built with `Unsafe` and reflection where the game expects a running client. All sources are compiled once into `.data/parity/classes/<digest>/`.
- **Keys.** Every case is keyed by `hash(game jar, libraries, JDK, OS/arch, JVM flags, the family's oracle sources, case input)`. Changing an oracle file or a case invalidates exactly the cases it can affect.
- **Captures.** Java outputs live in `.data/parity/<version>/<family>.jsonl.gz`. Keys the store lacks are captured automatically: each family in its own JVM, twice, families in parallel; a capture is kept only if both runs agree. Rewriting a family drops keys no case uses any more.
- **Comparison.** `target/release/photon-oracle` (`crates/photon-oracle`) answers every selected case through the production Rust code, in parallel, and each answer must equal the capture exactly. JSON numbers are parsed exactly (`float_roundtrip`), so a one-ulp difference is never hidden.
- **Reports.** A passing run writes nothing. A failing run prints the first differences and writes `.data/parity/failures/<time>/failures.json` with the inputs; the newest five reports are kept.
- **Determinism.** Families sharing one JVM in the past hid timing-dependent fixtures. Anything the oracle cannot seed (clock-seeded random sources, `ThreadLocalRandom`) must be pinned by the fixture; a non-repeatable capture fails the run.

### Adapter protocol

- **Input.** Every input line has `schema_version: 1`, `case_id`, `family`, `operation` and operation fields.
- **Output.** Every output line has the same `schema_version` and `case_id` plus a `result` object, in input order. An optional `diagnostics` object is not compared.
- **Logs.** Logs go to stderr, never stdout.
- **Floats** travel as hex bit patterns.

## Families

`atlas`, `binary`, `block`, `entity`, `gui`, `image`, `json`, `math`, `model`, `movement`, `nbt`, `resources`, `sound`, `timer`, `utf8`, `window`, `wire`. `math` NaN bit patterns were recorded on x86-64 Linux and differ on arm64.

The GUI family has its own page: [GUI oracle](gui-oracle.md).

## Legacy records

`docs/records/` holds 670 per-slice contract and parity records from the first phase of the project (in Turkish). They are being converted into case metadata with a generated coverage report and will then be removed.
