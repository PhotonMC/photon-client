# T33b — Örümcek gözleri, şarjlı creeper, gümüş balığı, endermite, arı

**Amaç:** `SpiderEyesLayer` (parlayan gözler, emissive), `CreeperPowerLayer` (şarjlı creeper: `EnergySwirlLayer`, kayan doku, genişletilmiş model), gümüş balığı ve endermite (gövde dalgası, `getFlipDegrees` 180), arı (kanat çırpma, bebek, öfke, `rollAmount`).
**Bağımlılık:** T33. **Boy:** M–L (emissive ve enerji girdabı yeni render türleri; ayrı bölünebilir).

## Oku
- Java: `client/renderer/entity/layers/{SpiderEyesLayer,EyesLayer,CreeperPowerLayer,EnergySwirlLayer}.java`, `client/renderer/rendertype/RenderTypes.java` (`eyes`, `energySwirl`), `client/model/monster/{silverfish/SilverfishModel,endermite/EndermiteModel}.java`, `client/model/animal/bee/BeeModel.java`, `client/renderer/entity/{Silverfish,Endermite,Bee}Renderer.java`, `world/entity/animal/bee/Bee.java` (`getRollAmount`, `isAngry`).
- Rust: `crates/voxel-render/src/vulkan.rs` (entity boru hattı, `overlay_code`), `crates/voxel/src/monsters.rs`, `voxel-render/src/mob_model.rs` (`setup_spider`, `setup_creeper`), `voxel-world/src/animal.rs`.

## Yapılacaklar
1. Emissive/saydam üst katman render türü (tam ışıklı, harmanlı) ve `SpiderEyesLayer`; sonra `EnergySwirlLayer` (uv kaydırma) ve şarjlı creeper (`Creeper.DATA_IS_POWERED` oku).
2. Gümüş balığı, endermite, arı model `setupAnim` kolları ve durumları.

## Oracle
`spider_model` kalıbı: gerçek model sınıfı + gerçek `setupAnim`; katman/render türü satırları gerçek render türlerinin ölçülebilir alanlarına (kayma, harman) karşı.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Arı sesi (PLAN-M7), patlama parçacığı (PLAN-M8).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
