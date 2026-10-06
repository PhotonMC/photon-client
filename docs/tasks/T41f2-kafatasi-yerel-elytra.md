# T41f2 — Kafatası modelleri, yerel ve mob elytrası, gözde dürbün

**Amaç:** T41f'den kalanlar: kafadaki kafatası modelleri (`SkullBlockRenderer`, `*_head#main` katmanları, `submitSkull`), yerel oyuncunun ve insansı mobların elytrası (`WingsLayer`), süzülen oyuncunun gövde eğimi (`AvatarRenderer.setupRotations`/`fallFlyingScale`), `PlayerItemInHandLayer.renderItemHeldToEye` (dürbün).
**Bağımlılık:** T41f. **Boy:** M.

## Oku
- Java: `client/renderer/blockentity/SkullBlockRenderer.java`, `client/model/object/skull/*.java`, `client/renderer/entity/layers/{CustomHeadLayer,PlayerItemInHandLayer}.java`, `client/renderer/entity/player/AvatarRenderer.java`.
- Rust: `crates/voxel/src/{armor,held_items,overlays,mobs}.rs`, `voxel-world/src/local_player.rs` (elytra durumu yok).

## Oracle
`held_item_pose` kalıbı; kafatası modelleri model tablosundan.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-armor-parity-conformance.md`, `docs/client-held-item-parity-conformance.md`.

## Devir notu
(boş)
