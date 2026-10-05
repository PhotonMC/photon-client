# T37h2 — Ender ejderhasının ölümü

**Amaç:** `EnderDragonRenderer`'ın ölüm çizimi: `entityCutoutDissolve` (çözünme maskesi `dragon_exploding.png`, alfa `1 − ölümZamanı/200`), ölüm ışınları (`dragonRays`, tohumu 432 olan rastgele ışın demeti), ölümde `partialTicks = 0`, `dragonDeathTime` (T37h'de sayılıyor), `EXPLOSION` parçacıkları dışında.
**Bağımlılık:** T37h. **Boy:** M.

## Oku
- Java: `client/renderer/entity/EnderDragonRenderer.java` (`submitRays`), `RenderTypes.entityCutoutDissolve`/`dragonRays`, `shaders/core/entity.fsh` (`DISSOLVE`).
- Rust: `crates/voxel/src/ender_dragons.rs`, `crates/voxel-render/src/vulkan.rs` (entity gölgelendiricisi).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
