# T35f — Aksolotl

**Amaç:** `AxolotlModel` (yetişkin ve bebek), `AxolotlRenderer` (renk çeşidi dokuları), anahtar kare animasyonları (yüzme, yürüme, suda durma, ölü taklidi) ve ilgili istemci durumları (`Axolotl.DATA_VARIANT`, `DATA_PLAYING_DEAD`, `FROM_BUCKET`).
**Bağımlılık:** T35d, T43b. **Boy:** M.

## Oku
- Java: `client/model/animal/axolotl/*.java`, `client/renderer/entity/AxolotlRenderer.java`, `client/renderer/entity/state/AxolotlRenderState.java`, `world/entity/animal/axolotl/Axolotl.java` (animasyon durumları, `DATA_*`, `AxolotlAnimation`).
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_turtle`, `setup_rabbit`: animasyon uygulama kalıbı), `crates/voxel-world/src/{rabbit,fish}.rs`, `crates/voxel/src/{turtles,fish}.rs`.

## Oracle
`sea_animal_model` ve `rabbit_model` kalıbı (gerçek model + animasyon durumları), gerçek varlık tiki için `living-entity-animations` (`axolotl` ekstrası).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Kova ile tutma, sesler, parçacıklar, davranışlar; kurbağa ve iribaş (T35g).

## Devir notu
(boş)
