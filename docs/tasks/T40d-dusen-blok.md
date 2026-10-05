# T40d — Düşen blok

**Amaç:** `FallingBlockEntity` (`FallingBlockRenderer`): `AddEntity` verisinden blok durumu, `shouldRender` (yerindeki blok aynıysa çizilmez), `MovingBlockFeatureRenderer`'ın tek bloklu mesh'i (biyom rengi, ışık motoru, kardinal ışık, çevre hava).
**Bağımlılık:** T40. **Boy:** M.

## Oku
- Java: `client/renderer/entity/FallingBlockRenderer.java`, `client/renderer/feature/MovingBlockFeatureRenderer.java`, `client/renderer/block/MovingBlockRenderState.java`, `world/entity/item/FallingBlockEntity.java`.
- Rust: `crates/voxel-render/src/world_mesh.rs` (blok mesh'i), `crates/voxel/src/overlays.rs` (`cracks`, blok quad'ları).

## Oracle
Gerçek renderer/model sınıfı ve durumu (örnek: `docs/client-arrow-parity-conformance.md`, `docs/client-minecart-parity-conformance.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, partiküller, dış çizgi.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
(boş)
