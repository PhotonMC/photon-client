# T37c2 — Shulker konumu ve vuruş kutusu

**Amaç:** `Shulker.setPos` (blok merkezine yakalama: `floor(x)+0,5`, `floor(y+0,5)`, `floor(z)+0,5`; blok değişince açıklık verisi 0 ve ışınlanma), `getRenderPosition` (6 tiklik kare ağırlıklı kaydırma, `clientOldAttachPosition`), `startRiding`/`stopRiding` ve açıklığa/yüze bağlı `makeBoundingBox` (`getProgressAabb`).
**Bağımlılık:** T37c. **Boy:** M.

## Oku
- Java: `world/entity/monster/Shulker.java` (`setPos`, `getRenderPosition`, `makeBoundingBox`, `getProgressDeltaAabb`), `world/entity/InterpolationHandler.java`, `client/renderer/entity/ShulkerRenderer.java` (`getRenderOffset`, `shouldRender`).
- Rust: `crates/voxel-world/src/entity.rs` (`tick_position_interpolation`: konum yazılarından sonra yakalama; `set_position`, `apply_entity`), `crates/voxel-world/src/pick.rs` (`dimensions`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
