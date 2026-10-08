# T33 — Örümcek, böcek, arı ve creeper

**Amaç:** Çok bacaklı ve küçük yaratıklar: örümcek/mağara örümceği (8 bacak yürüme), gümüş balığı ve endermite (gövde dalgası), arı (kanat, bebek, öfke), creeper (şişme, şarj).
**Bağımlılık:** T24. **Boy:** M. **Türler:** spider, cave_spider, silverfish, endermite, bee, creeper.

## Oku
- Java: `client/model/monster/spider/SpiderModel.java`, `client/model/monster/silverfish/SilverfishModel.java`, `client/model/monster/endermite/EndermiteModel.java`, `client/model/animal/bee/BeeModel.java`, `client/model/monster/creeper/CreeperModel.java`, `client/renderer/entity/SpiderRenderer.java`, `client/renderer/entity/CreeperRenderer.java`, `client/renderer/entity/layers/CreeperPowerLayer.java`, `client/renderer/entity/BeeRenderer.java`.
- Rust: `crates/photon-render/src/entity/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/photon/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/photon-render/src/entity/textures.rs` (`rect`), `crates/photon-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. `SpiderModel.setupAnim` (bacak çiftleri), `getFlipDegrees` 180 (T27 yatışı için `FLIP_DEGREES` tablosuna).
2. Creeper: `swell` (patlamaya yaklaşma ölçeği/beyazlama `getWhiteOverlayProgress`) ve şarj katmanı.
3. Arı: kanat çırpma, `rollAmount`.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Creeper patlama parçacığı (PLAN-M8), arı sesi (PLAN-M7).

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Her biri küçük dal; `getFlipDegrees` 180 olan türler burada tabloya eklenir. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı (bölündü): örümcek, mağara örümceği (`setup_spider`, ölürken 180° yatma), creeper (`setup_creeper`, `creeper_scale` kancası `MobPlacement.hook_scale` ile, `creeper_white_overlay` ve shader'da overlay kodu, `Creeper.tick` fitili istemcide, `DATA_SWELL_DIR`/`DATA_IS_IGNITED` 16/18); `photon/monsters.rs`. Kayıt: `docs/records/client-spider-creeper-parity-conformance.md`.
- Oracle: `entity/spider-model` 10, `creeper-model` 5, `creeper-render` 12, `living-pose` flip 180 satırı, `living-entity-animations`'a creeper fitil satırları (gerçek `Creeper.tick` ve paket).
- Sapma: gözler/şarj girdabı yok (yeni render türleri), `DATA_IS_POWERED` okunmuyor.
- Açık: gümüş balığı, endermite, arı, gözler ve şarj katmanı (T33b).
