# Behavior verification contract

> English translation of [`../behavior-testing.md`](../behavior-testing.md). The Turkish original is authoritative.

This document defines how behavior slices are prepared and verified for Minecraft Java Edition 26.2 / protocol 776 / Java 25. Every slice has its own source review, original scenarios, Java oracle call and combined parity/conformance record.

General documents do not repeat moving corpus totals. The exact scenario inputs are in `tools/behavior/cases.py`, reference ids and record integrity are in the runner reports, and the limits of a behavior are in the relevant combined contract/parity record. A new slice does not require changing old slice documents to add counters.

If the sources or the version change, existing captures are not silently refreshed. A new reference is captured explicitly; earlier records are kept. Once the required local inputs are provided, nobody is expected to play the game by hand to run tests or judge results.

## Reference and comparison

The reference is the pinned Minecraft Java Edition version. For every behavior slice the sources are reviewed; the same controlled scenario is run on Java and on Rust and the observable results are compared. The origin of a test's expected result must be the Java reference. Expectations derived only from reading the source are marked as lacking this until verified against running Java.

Every test record contains the behavior id, the source location (without code quotes), the version ids, the preconditions, the inputs, the observed outputs, the comparison rule and how to run it. Capturing the reference, replaying the scenario, comparing and reporting differences must be automatic. Once the required local sources are provided, the user is not expected to run tests by hand.

No golden that fits Rust is created before Java is verified. Refreshing a reference must be an explicit action; existing expectations must not be changed silently while tests run. The stability of the reference is also judged by repeating the same Java scenario.

## Evidence classes

Every record carries one of two classes and states it in the title of the combined contract/parity document. A record that names no class counts as incomplete.

| Class | Compared | Proves | Does not prove |
| --- | --- | --- | --- |
| `client-parity` | The Java client and Rust on the same input | Client behavior equality | The server side and unmeasured scenarios |
| `server-conformance` | A packet Rust produced and the answer of a live vanilla server | That the server accepts the wire format and protocol flow | That the client would produce the same packet or build the same internal state in the same situation |

Calling the Java oracle only to produce constants (registry ids, component type ids, hashes) does not make a record `client-parity`; if the compared output is not the Java client's behavior, the class is `server-conformance`.

`server-conformance` records are required and advance a roadmap section, but do not close it. A section is not complete without a `client-parity` record in which the relevant client state is compared with Java.

The reasoning behind the distinction, and the period in which the repository lost it, are in the [course correction document](../course-correction-2026-09-12.md).

## Conditions to control

As far as they are relevant, the seed, world/player state, input times, tick, frame time, order of network events, language, resolution, GUI scale, FOV, sound settings and asset id are pinned. Controlling the clock and randomness must not change game semantics. Java's overflow, floating-point, rounding, operation order and event order behaviors are not replaced by Rust assumptions.

For state transitions, numbers, events and protocol fields, the exact equality the contract requires is used. Image and sound comparisons are made over fixed scenarios, camera, animation phase and timing. Variability from platform or GPU is first established by repeated measurements of the same Java reference. No universal epsilon, pixel tolerance or "similar enough" criterion is assumed; if a tolerance is unavoidable, its reason and limit are stated explicitly in the behavior contract. Hiding a game difference behind a tolerance is not acceptable.

Automatic image and sound measurements are by themselves no absolute proof that a human perceives no difference. A verification claim is limited to the measured scenarios and environments; the target is still exact equality.

## Order and completion per slice

1. Extract the behavior and its dependencies from the Java sources; define the controlled scenario.
2. Run the scenario on the local Java reference; record the origin and stability of the output.
3. Prepare the comparison test before the Rust implementation. Verify that the test catches missing or wrong output; do not count a Rust adapter that does not exist yet as passing.
4. Implement the smallest necessary Rust slice and compare with the same inputs.
5. If there is a difference, resolve its cause; do not fit the expectation to Rust. Run the related earlier behavior tests too.
6. Record the verified scope and the open gaps; commit the completed slice separately.

Behavior testing with the same server events fed to the client, and integration verification against a real vanilla 26.2 connection, are evaluated separately. Feeding events in a test does not mean developing a production server. Missing or different behavior of a non-vanilla server is no proof of Java client equality; which side the difference comes from is recorded. Pumpkin runs are kept as separate comparison records.

Every scenario is reported as passed, failed or blocked. A test that does not run for lack of Java, assets, a screen/sound environment or Pumpkin support gives no approval of equality. Failure reports keep the inputs, the first divergence and rerun information locally.

Behavior coverage can be as detailed as needed. For code quality, repeated micro tests and pointless coverage-percentage targets are not added. The test infrastructure grows in small steps as the need arises.

A copy of the same code path with only a constant changed does not create a separate slice, probe or document. In that case the input is added to a table as a row and closed with a single parametric comparison; the record lists all the rows it covers in one document. A separate slice is opened only if there is a branch that runs for the first time in the Java source, or an observed difference.
