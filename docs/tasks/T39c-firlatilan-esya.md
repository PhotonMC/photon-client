# T39c — Fırlatılan eşyalar

**Amaç:** `ThrownItemRenderer` (kamera yüzlü eşya: kartopu, yumurta, ender incisi, iksirler, büyü şişesi, ender gözü), havai fişek roketi (`FireworkEntityRenderer`).
**Bağımlılık:** T39. **Boy:** M.

## Oku
- Java: `client/renderer/entity/{ThrownItemRenderer,FireworkEntityRenderer}.java`, `client/renderer/entity/state/ThrownItemRenderState.java`.
- Rust: `crates/voxel-render/src/item_entity.rs` (eşya çizimi), `crates/voxel/src/dropped_items.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
