# T36e — Breeze

**Amaç:** `BreezeModel` (üç katman: gövde, rüzgâr, gözler; altı keyframe animasyonu: boşta, ateş, kayma, geri kayma, soluma, uzun zıplama), `BreezeWindLayer` (kaydırılan doku, `RenderTypes.breezeWind`), `BreezeEyesLayer` (`RenderTypes.breezeEyes`) ve `Breeze` istemci durumları (poz değişince başlayan animasyon durumları).
**Bağımlılık:** T36. **Boy:** M–L.

## Oku
- Java: `client/model/monster/breeze/BreezeModel.java`, `client/renderer/entity/BreezeRenderer.java`, `client/renderer/entity/layers/Breeze{Wind,Eyes}Layer.java`, `client/renderer/rendertype/RenderTypes.java` (`breezeWind`, `breezeEyes`), `world/entity/monster/breeze/Breeze.java` (`onSyncedDataUpdated`, `tick`).
- Rust: `crates/voxel-world/src/frog.rs` (poza bağlı animasyon durumu kalıbı), `crates/voxel-render/src/{mob_model,render_types}.rs` (`mob_vertices_swirl`: kaydırılan doku), `crates/voxel/src/monsters.rs`.

## Oracle
`frog_model` kalıbı (gerçek model + animasyon durumları); üç katman ayrı satırlar; gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, rüzgâr topu (T39), davranışlar.

## Devir notu
(boş)
