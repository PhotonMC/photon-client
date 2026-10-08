# T30 — Çiftlik hayvanları 2: koyun, mantar ineği, tavuk, keçi, tavşan

**Amaç:** `QuadrupedModel`'in ötesinde dallar: koyunun yün katmanı ve boyası (`DyeColor` verisi, kırpılmış hâli), tavuğun kanat çırpması ve soğuk çeşidi, keçinin boynuzları, tavşanın zıplaması, mantar ineğinin mantar katmanı.
**Bağımlılık:** T24. **Boy:** M. **Türler:** sheep, mooshroom, chicken, goat, rabbit.

## Oku
- Java: `client/model/animal/sheep/SheepModel.java`, `client/renderer/entity/SheepRenderer.java`, `client/renderer/entity/layers/SheepWoolLayer.java`, `client/model/animal/chicken/ChickenModel.java`, `client/renderer/entity/ChickenRenderer.java`, `client/model/animal/goat/GoatModel.java`, `client/renderer/entity/GoatRenderer.java`, `client/model/animal/rabbit/RabbitModel.java`, `client/renderer/entity/RabbitRenderer.java`, `client/renderer/entity/MushroomCowRenderer.java`.
- Rust: `crates/photon-render/src/entity/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/photon/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/photon-render/src/entity/textures.rs` (`rect`), `crates/photon-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Koyun: `SheepWoolLayer` (renk verisinden), `sheared` bayrağı, jeb_ adı; entity verisi okuyucusu T24b'deki yürüyüşle.
2. Tavuk: `flap`/`flapSpeed` durumu (`Chicken.tick` istemci kolu); soğuk çeşit T24b kalıbıyla.
3. Keçi ve tavşan: ayrı `setup` kolları (zıplama `jumpCompletion`).

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Eyer/binme (T44), yün kırpma etkileşimi sunucu tarafında.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Aile içi çeşit/ölçek tablo satırı. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı (bölündü): koyun (gövde, alt yün, yün; boya, kırpılmış, bebek, yeme ölçekleri; `EntityEvent` 10 ve istemci tik) ve tavuk (üç model, flap tiki, çeşit verisi); entity köşe rengi (`Vertex.tint.w`, shader'da çarpım); `photon-world/animal.rs`, `photon-render/wool.rs`, `photon/sheep.rs`; boya tablosu ve tavuk çeşitleri oracle'dan üretildi. Kayıt: `docs/records/client-sheep-chicken-parity-conformance.md`.
- Oracle: `entity/sheep-model` 25, `chicken-model` 15, `sheep-colors` 27 (`jeb_` dahil), `sheep-eat` 31 satır (gerçek model sınıfları, `ColorLerper`, gerçek `Sheep`); veri indeksleri `variant-fields`'te.
- Sapma: `jeb_` adı çizimde yok, tavuk flap tiki oracle'sız (kaynak + birim test), tavuk düşüş yavaşlaması yok.
- Açık: keçi, tavşan, mantar ineği (T30b).
