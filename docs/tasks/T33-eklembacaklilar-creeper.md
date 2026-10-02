# T33 — Örümcek, böcek, arı ve creeper

**Amaç:** Çok bacaklı ve küçük yaratıklar: örümcek/mağara örümceği (8 bacak yürüme), gümüş balığı ve endermite (gövde dalgası), arı (kanat, bebek, öfke), creeper (şişme, şarj).
**Bağımlılık:** T24. **Boy:** M. **Türler:** spider, cave_spider, silverfish, endermite, bee, creeper.

## Oku
- Java: `client/model/monster/spider/SpiderModel.java`, `client/model/monster/silverfish/SilverfishModel.java`, `client/model/monster/endermite/EndermiteModel.java`, `client/model/animal/bee/BeeModel.java`, `client/model/monster/creeper/CreeperModel.java`, `client/renderer/entity/SpiderRenderer.java`, `client/renderer/entity/CreeperRenderer.java`, `client/renderer/entity/layers/CreeperPowerLayer.java`, `client/renderer/entity/BeeRenderer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. `SpiderModel.setupAnim` (bacak çiftleri), `getFlipDegrees` 180 (T27 yatışı için `FLIP_DEGREES` tablosuna).
2. Creeper: `swell` (patlamaya yaklaşma ölçeği/beyazlama `getWhiteOverlayProgress`) ve şarj katmanı.
3. Arı: kanat çırpma, `rollAmount`.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Creeper patlama parçacığı (PLAN-M8), arı sesi (PLAN-M7).

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Her biri küçük dal; `getFlipDegrees` 180 olan türler burada tabloya eklenir. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
