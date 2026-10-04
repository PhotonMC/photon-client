# T36f — Mutlu ghast

**Amaç:** `HappyGhastModel` (+ `HappyGhastHarnessModel` koşum katmanı, ip katmanı), `HappyGhastRenderer` ve ilgili istemci durumları (bebek ve yetişkin, koşum eşyası, `isRidden`, dokunaç salınımı).
**Bağımlılık:** T36, T41. **Boy:** M.

## Oku
- Java: `client/model/animal/ghast/HappyGhast*.java`, `client/renderer/entity/HappyGhastRenderer.java`, `client/renderer/entity/state/HappyGhastRenderState.java`, `world/entity/animal/happyghast/HappyGhast.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_ghast`), `crates/voxel/src/monsters.rs`.

## Oracle
`airborne_model` kalıbı; gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, davranışlar, binme denetimi.

## Devir notu
(boş)
