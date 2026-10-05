# T39c — Fırlatılan eşyalar

**Amaç:** `ThrownItemRenderer` (kamera yüzlü eşya: kartopu, yumurta, ender incisi, iksirler, büyü şişesi, ender gözü), havai fişek roketi (`FireworkEntityRenderer`).
**Bağımlılık:** T39. **Boy:** M.

## Oku
- Java: `client/renderer/entity/{ThrownItemRenderer,FireworkEntityRenderer}.java`, `client/renderer/entity/state/ThrownItemRenderState.java`.
- Rust: `crates/voxel-render/src/item_entity.rs` (eşya çizimi), `crates/voxel/src/dropped_items.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `ThrownItemRenderer` (scale·orientation) ve `FireworkEntityRenderer` yerleşimi (`thrown_item_pose`, `firework_pose`), `FrameView.rotation` (kamera yönü), thrown/firework verisindeki eşya yığınının alınması (veri 8), roketin `DATA_SHOT_AT_ANGLE`'ı, `thrown_items.rs` (yer bağlamlı eşya katmanları). Kayıt: `docs/client-thrown-item-parity-conformance.md`.
- Ateş topu ve küçük ateş topu (ölçek ve tam parlaklık) bu görevde çözüldü; T39d yalnız ejderha topu ve rüzgâr yükleri kaldı.
- Sapma: eşya modeli ve ışık oracle'sız; canlı görüldü.
