# Running the local behavior tests

> English translation of [`../behavior-runner.md`](../behavior-runner.md). The Turkish original is authoritative.

It uses the Python 3.10+ standard library and a Java 25 JDK; there is no new Python package or test framework. The Rust JSONL adapter uses the `serde_json` development dependency; the production `flate2` dependency is used for the zlib framing in the real connection probe. The Java adapters call classes in the pinned Minecraft JAR directly. Expected results are never produced from Rust or from algorithms rewritten inside the test.

From the repository root:

```bash
python3 tools/behavior/run.py preflight
python3 tools/behavior/run.py capture
python3 tools/behavior/run.py verify
python3 tools/behavior/test_runner.py
```

- `preflight`: checks the client and related JAR hashes and the Java/javac version. It runs neither the game nor tests.
- `capture`: runs the scenarios in two separate Java processes, compares the outputs and saves the stable reference. It does not change a record with the same id; it fails if an existing record is different or corrupt.
- `verify`: compares the saved reference, without changing it, against two new Java runs. `capture` must have been done before.
- `compare --adapter /absolute/path/program`: feeds the same inputs to the given executable and compares with the saved Java result. A Rust adapter exists for wire/timer/math/utf8/binary/NBT/resources/window/atlas/image/model/json; without an adapter this command exits with **2**. The build and comparison commands are in the [binary slice document](../records/binary-conformance.md).
- `test_runner.py`: synthetic infrastructure checks that only examine false success, corrupt references and faulty process behavior. It needs neither Java nor assets.

To select a single family use `--family wire`, `--family math`, `--family timer`, `--family utf8`, `--family binary`, `--family nbt`, `--family resources`, `--family window`, `--family atlas`, `--family image`, `--family model` or `--family json`; the option can be repeated. If none is selected, all existing families run. The family selection is part of the reference id; for example, before `verify --family timer`, `capture --family timer` must have been run with the same selection. `--runtime /local/runtime` selects an alternative client/`libs` directory, `--jdk-home /jdk/root` selects the Java tools. These paths are read only. If the required references are missing, nothing is downloaded.

## Results and exit codes

| Result | Code | Meaning |
| --- | ---: | --- |
| `preflight_ready` | 0 | The input pre-check passed; behavior is not verified |
| `reference_ready`, `rust_status=not_compared` | 0 | The Java reference is stable/equal to the record; Rust was not compared |
| `parity_passed` | 0 | The given adapter matched Java's output in the selected scenarios only |
| `failed` | 1 | A run/build/format error or a behavior difference |
| `blocked` | 2 | A missing/wrong reference, wrong JDK, missing capture or adapter |

A successful reference command does not show client equality. An adapter passing in a selected family does not mean the other sections pass. No local reference or capture is uploaded to general CI.

## Files and identity

Tracked files: `tools/behavior/` the runner and the original input generator; `tests/behavior/java/` the Java call adapters and the synthetic test bridges they need; `tests/behavior/reference.json` the version and required artifact hashes. For the runtime only the JARs of the selected family are used; assets, the GPU, a server and a decompiled source tree are not preconditions of these first tests.

Every run keeps its input, stdout/stderr, compiled classes, identity and `report.json` under `.data/runs/behavior/<command>-<unique>/`. The `reproduce` command in the report can be rerun from the `invocation_cwd` directory. Every capture keeps `inputs.jsonl`, `outputs.jsonl` and `manifest.json` under `.data/captures/behavior/<id>/`. The reference id covers the client/dependency hashes, the Java and javac version output, OS/architecture, the selected families, the Java adapter/runner code and the scenario inputs. Corruption of an input or output file is rejected by the hash check.

If the sources, the scenarios or the environment change, `capture` is run explicitly for the new id; old records are kept. These records are not a cryptographically signed chain of trust; they track local test inputs and accidental changes. No test run automatically accepts a changed baseline.

## Adapter protocol

The executable takes UTF-8 JSONL from stdin and returns one JSON object to stdout for each input, in the same order. It writes no logs to stdout; logs go to stderr. The time limit is 60 seconds; an error or missing/corrupt output does not count as success. The process is not started through a shell; its working directory is the relevant local run directory.

Every input contains `schema_version: 1`, `case_id`, `family`, `operation` and fields specific to the operation. Every output contains the same `schema_version`/`case_id` and a `result` object. An optional `diagnostics` object (for example the Java exception class) is not compared; all `result` fields are compared exactly in type and value. List/event order is kept; the order in which fields are written in a JSON object does not matter. A missing, extra, repeated or reordered record is a failure.

`wire` family: the `codec` field is `varint` or `varlong`; in a `write` input `value` is a signed decimal **string**, in a `read` input `hex` is a lowercase byte string. A write result has `outcome`, `size`, `hex`, `decoded`, `consumed`, `remaining_hex`; a read result has `outcome`, `value` or `error`, `consumed`, `remaining_hex`. `buffer_underflow` and `too_big` are language-independent categories of the error observed from Java; the class name is only in the diagnostic record.

The wire coverage includes signed min/max, the 7-bit boundaries, negative values, truncated, over-long and non-canonical encodings and the remaining bytes. For every `write` call the original decoder is also called; there are independent synthetic `read` inputs as well. UTF-8 is ready as a Java reference in a separate family; the other P03 formats are not covered yet.

The tool does not prove the programming language of an executable that speaks the JSON protocol. `crates/photon/examples/behavior_adapter/` is the real Rust wire/timer/math adapter; the report records the executable's SHA-256 and the selected scope. Synthetic programs used while trying out the infrastructure are no proof of game equality.

## Family and operation contracts

The runner's common JSONL and exact-comparison rules are defined in the protocol above. The fields specific to a family and an operation are kept, together with the Java source review, in the relevant combined contract/parity record; the single place that produces scenario inputs is `tools/behavior/cases.py`.

| Family | Contract scope |
| --- | --- |
| math | Numeric, geometric, coordinate, shape, random and Mth slices |
| timer | Tick, leftover time, speed and pause/freeze behavior |
| wire | VarInt/VarLong encode/decode and error/consumption behavior |
| utf8 | UTF-8 limits, corrupt sequences and error consumption |
| binary | Vec3/LP Vec3, UUID, position, chunk/section, array and bitset encodings |
| nbt | Compound, list and primitive array reading/writing behavior |
| resources | Asset index path resolution, virtual object links and pack resource priority |
| gui | User interface draw calls (rectangles, sprites, text): chat, suggestion list, title, tab list; details in the [GUI drawing oracle](../gui-oracle.md) |

When a new operation is added, its own source review, oracle branch, scenarios and combined contract/parity record are added. This general runner document keeps no moving scenario total or per-operation counter.

## Currently verified scope

The runner handles seven families with the same JSONL contract: math, timer, wire, utf8, binary, nbt and resources. Every selected capture checks repeatability with two Java runs, and the adapter comparison checks exact equality with the saved Java output.

| Family | Observed scope |
| --- | --- |
| math | Numeric and geometric helpers, coordinate/shape paths, legacy random streams and the related Mth slices |
| timer | Tick production, leftover time, speed, pause/freeze and boundary conditions |
| wire | VarInt/VarLong encode/decode, limits, consumption and errors |
| utf8 | UTF-8/UTF-16 limits, corrupt sequences, titles and error consumption |
| binary | Vec3/LP Vec3, UUID, position, chunk/section, long array and bitset encodings |
| nbt | Compound, list and primitive array reading/writing and boundary behavior |
| resources | Asset index path resolution, virtual object links and pack resource priority |

The exact scenario list is read from the generator, the reference id from the capture manifest, and the limits of a slice from the relevant combined contract/parity records. This section does not repeat a moving corpus total.

This scope is limited to headless helper and data behaviors. The first real vanilla 26.2 session and the Pumpkin comparison are in [subsections of the combined game session record](../records/game-connection-conformance.md). A playable client, window/input, chunk/world state, GPU frame capture, audio capture and full user-facing flows are not verified yet.
