# T35g — Kurbağa ve iribaş

**Amaç:** `FrogModel` (anahtar kare animasyonları: zıplama, dil, şişme, yüzme), `TadpoleModel`, renderer'ları (kurbağa çeşidi dokuları) ve ilgili istemci durumları (`Frog.DATA_VARIANT`, `DATA_TONGUE_TARGET`, animasyon durumları).
**Bağımlılık:** T35d, T43b. **Boy:** M–L.

## Oku
- Java: `client/model/animal/frog/*.java`, `client/renderer/entity/{Frog,Tadpole}Renderer.java`, `world/entity/animal/frog/{Frog,Tadpole}.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_rabbit`), `crates/voxel-world/src/rabbit.rs`.

## Oracle
`rabbit_model` kalıbı; gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, davranışlar; aksolotl (T35f).

## Devir notu
(boş)
