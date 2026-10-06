# T41f4 — Kafatası modelleri ve gözde dürbün

**Amaç:** kafadaki kafatası modelleri (`CustomHeadLayer`'ın `wornHeadType` dalı, `SkullBlockRenderer.submitSkull`, `*_head#main`/`*_skull#main` katmanları, `Transforms.skullYOffset`) ve `PlayerItemInHandLayer.renderItemHeldToEye` (dürbün).
**Bağımlılık:** T41f3. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/{CustomHeadLayer,PlayerItemInHandLayer}.java`, `client/renderer/blockentity/SkullBlockRenderer.java`, `client/model/object/skull/*.java`.
- Rust: `crates/voxel/src/held_items.rs` (`Hands`, `HEAD_TRANSFORMS`), `voxel-render/src/held_item.rs`.

## Oracle
`entity/held-item-pose` satırlarına kafatası başı ve dürbün (aynı operation).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-held-item-parity-conformance.md`.

## Devir notu
- `voxel-render::skull` (tür, model, doku, `skull_pose`), `MobModel::setup_skull`, `held_to_eye_pose`/`eye_held_arm`; oyunda `Hands::{skull, eye}`, `Armor::skull_vertices`, `arm_poses::eye_arm`.
- Oracle: `entity/held-item-pose` +84 satır (kafatası 64, dürbün 20), bit bit eşit; canlıda ejderha ve creeper başlı zombiler görüldü.
- Açık: profili olan oyuncu kafası (PLAN-M9), köylü/cadı kafatası ofseti, yerel oyuncunun dürbünü.
