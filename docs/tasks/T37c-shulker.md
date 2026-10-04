# T37c — Shulker

**Amaç:** `ShulkerModel` (kabuk açılma: `peekAmount`, baş dönüşü), `ShulkerRenderer` (renk dokusu, bağlı yüz yönü dönüşü), `Shulker` istemci durumu (`ID_ATTACH_FACE`, `ID_PEEK`, `ID_COLOR`).
**Bağımlılık:** T37. **Boy:** M.

## Oku
- Java: `world/entity/monster/Shulker.java`, `client/model/monster/shulker/ShulkerModel.java`, `client/renderer/entity/{ShulkerRenderer,state/ShulkerRenderState}.java`.
- Rust: `crates/voxel/src/cube_mobs.rs` (kalıp), `crates/voxel-world/src/{entity,metadata}.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
