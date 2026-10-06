# T41f3 — Kafatası modelleri, gözde dürbün ve süzülme eğimi

**Amaç:** kafadaki kafatası modelleri (`SkullBlockRenderer`, `*_head#main` katmanları, `submitSkull`), `PlayerItemInHandLayer.renderItemHeldToEye` (dürbün) ve süzülen oyuncunun gövde eğimi (`AvatarRenderer.setupRotations`, `fallFlyingScale`).
**Bağımlılık:** T41f2. **Boy:** M.

## Oku
- Java: `client/renderer/blockentity/SkullBlockRenderer.java`, `client/model/object/skull/*.java`, `client/renderer/entity/layers/{CustomHeadLayer,PlayerItemInHandLayer}.java`, `client/renderer/entity/player/AvatarRenderer.java`.
- Rust: `crates/voxel/src/{armor,held_items,overlays}.rs`, `voxel-render/src/held_item.rs`.

## Oracle
`held_item_pose` kalıbı; kafatası modelleri model tablosundan.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-held-item-parity-conformance.md`.

## Devir notu
(boş)
