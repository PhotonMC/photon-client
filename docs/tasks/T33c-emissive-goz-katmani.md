# T33c — Emissive göz katmanı

**Amaç:** Yeni render türü `RenderTypes.eyes` (ışıksız, harmanlı) ve tüketicileri `SpiderEyesLayer` (örümcek, mağara örümceği) ile `EnderEyesLayer` (enderman). Enerji girdabı ve şarjlı creeper T33d'ye ayrıldı.
**Bağımlılık:** T33, T34c. **Boy:** S–M.

## Oku
- Java: `client/renderer/entity/layers/{SpiderEyesLayer,EyesLayer,EnderEyesLayer,CreeperPowerLayer,EnergySwirlLayer}.java`, `client/renderer/rendertype/RenderTypes.java` (`eyes`, `energySwirl`), `world/entity/monster/Creeper.java` (`isPowered`).
- Rust: `crates/voxel-render/src/vulkan.rs` (entity boru hattı, ışık/örtü kodu), `crates/voxel-render/src/scene.rs` (`Vertex`), `crates/voxel/src/monsters.rs`, `voxel-render/src/mob_model.rs` (`mob_vertices_coloured`, `overlay_code`), `voxel-world/src/animal.rs`.

## Yapılacaklar
Işıksız, harmanlı üst katman çizim yolu ve iki katmanın dokuları (`entity/spider/spider_eyes`, `entity/enderman/enderman_eyes`).

## Oracle
Katman satırları gerçek render türünün ölçülebilir alanlarına (ışık, harman, kayma) karşı; kayma değeri `EnergySwirlLayer.xOffset` ile; `spider_model` kalıbı model için.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`, ekran görüntüsü bir kez.

## Kapsam dışı
Patlama parçacığı (PLAN-M8); parlayan dış hat.

## Belgeler
`docs/index.md` satırı, yeni kayıt; `docs/client-golem-enderman-parity-conformance.md` "enderman gözleri yok" sapması yeni kayda atıfla kalkar.

## Devir notu
- Yapıldı: `voxel-render/src/render_types.rs` (`EYES` hattı olguları, `eyes_texture`), `mob_vertices_glow`, `VulkanBackend::set_entity_glow` (gölge hattı, `glow` tamponu), `MobModels::glow`, `monsters.rs` göz katmanı. Kayıt: `docs/client-eyes-layer-parity-conformance.md`.
- Oracle: `entity/eyes-layers` 2 satır (gerçek `SpiderEyesLayer`/`EnderEyesLayer`, render türünün hattı yansımayla). Canlı: örümcek kırmızı, enderman mor gözler `shot` ile görüldü.
- Sapma: hat yüz elemesi yapmıyor (gözlenebilir fark yok).
- Açık: enerji girdabı ve şarjlı creeper (T33d).

