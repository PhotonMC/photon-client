# T33c — Emissive göz katmanı ve enerji girdabı

**Amaç:** Yeni render türleri: `RenderTypes.eyes` (tam ışıklı, harmanlı) ve `energySwirl` (kayan doku). Tüketiciler: `SpiderEyesLayer` (örümcek, mağara örümceği), `EnderEyesLayer` (enderman), `CreeperPowerLayer` (şarjlı creeper: `EnergySwirlLayer`, genişletilmiş model, `DATA_IS_POWERED`).
**Bağımlılık:** T33, T34c. **Boy:** M–L (girdap ayrı bölünebilir).

## Oku
- Java: `client/renderer/entity/layers/{SpiderEyesLayer,EyesLayer,EnderEyesLayer,CreeperPowerLayer,EnergySwirlLayer}.java`, `client/renderer/rendertype/RenderTypes.java` (`eyes`, `energySwirl`), `world/entity/monster/Creeper.java` (`isPowered`).
- Rust: `crates/voxel-render/src/vulkan.rs` (entity boru hattı, ışık/örtü kodu), `crates/voxel-render/src/scene.rs` (`Vertex`), `crates/voxel/src/monsters.rs`, `voxel-render/src/mob_model.rs` (`mob_vertices_coloured`, `overlay_code`), `voxel-world/src/animal.rs`.

## Yapılacaklar
1. Tam ışıklı, harmanlı üst katman çizim yolu; `SpiderEyesLayer` ve `EnderEyesLayer` dokuları (`entity/spider_eyes`, `entity/enderman/enderman_eyes`).
2. `EnergySwirlLayer` (uv kaydırma: `xOffset`, tikle) ve şarjlı creeper; `DATA_IS_POWERED` oku.

## Oracle
Katman satırları gerçek render türünün ölçülebilir alanlarına (ışık, harman, kayma) karşı; kayma değeri `EnergySwirlLayer.xOffset` ile; `spider_model` kalıbı model için.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`, ekran görüntüsü bir kez.

## Kapsam dışı
Patlama parçacığı (PLAN-M8); parlayan dış hat.

## Belgeler
`docs/index.md` satırı, yeni kayıt; `docs/client-golem-enderman-parity-conformance.md` "enderman gözleri yok" sapması yeni kayda atıfla kalkar.

## Devir notu
