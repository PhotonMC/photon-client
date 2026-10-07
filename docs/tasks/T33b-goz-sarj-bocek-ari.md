# T33b — Gümüş balığı, endermite, arı

**Amaç:** gümüş balığı ve endermite (gövde dalgası, `getFlipDegrees` 180) ve arı (kanat çırpma, bebek, öfke, `rollAmount`). Göz katmanı ve şarj girdabı T33c'ye taşındı.
**Bağımlılık:** T33. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/{SpiderEyesLayer,EyesLayer,CreeperPowerLayer,EnergySwirlLayer}.java`, `client/renderer/rendertype/RenderTypes.java` (`eyes`, `energySwirl`), `client/model/monster/{silverfish/SilverfishModel,endermite/EndermiteModel}.java`, `client/model/animal/bee/BeeModel.java`, `client/renderer/entity/{Silverfish,Endermite,Bee}Renderer.java`, `world/entity/animal/bee/Bee.java` (`getRollAmount`, `isAngry`).
- Rust: `crates/voxel-render/src/vulkan.rs` (entity boru hattı, `overlay_code`), `crates/voxel/src/mobs/monsters.rs`, `voxel-render/src/mob_model.rs` (`setup_spider`, `setup_creeper`), `voxel-world/src/animal.rs`.

## Yapılacaklar
Gümüş balığı, endermite, arı model `setupAnim` kolları ve durumları (`Bee.getRollAmount`, `isAngry` verisi).

## Oracle
`spider_model` kalıbı: gerçek model sınıfı + gerçek `setupAnim`; katman/render türü satırları gerçek render türlerinin ölçülebilir alanlarına (kayma, harman) karşı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Arı sesi (PLAN-M7), patlama parçacığı (PLAN-M8).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `setup_segmented` (gümüş balığı, endermite; `getFlipDegrees` 180), `setup_bee`, arı tiki (yuvarlanma) ve verisi (bayrak, öfke bitişi), sekiz doku; `monsters.rs` ve `farm_animals.rs`. Kayıt: `docs/records/client-insects-parity-conformance.md`.
- Oracle: `segmented-model` 8, `bee-model` 14, `living-entity-animations` 6 arı satırı; `variant-fields` iki satır. Canlı: `shot` ile görüldü (arı ayrıntısı doğrulanamadı, poz oracle'da).
- Sapma: iğne dokusu ve sokma animasyonu yok.

