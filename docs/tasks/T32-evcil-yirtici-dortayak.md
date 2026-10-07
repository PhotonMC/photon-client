# T32 — Evcil ve yırtıcı dört ayaklılar: kurt, kedi, vaşak, tilki, panda, kutup ayısı, hoglin, ravager, strider

**Amaç:** Oturma/uyuma/saldırı dalları olan dört ayaklılar: kurt (öfke, sulu, yakalık rengi), kedi/vaşak (oturma, uzanma), tilki (uyku, çömelme), panda (oturma, yuvarlanma, hapşırma), kutup ayısı (ayağa kalkma), hoglin ve zoglin, ravager, strider.
**Bağımlılık:** T24. **Boy:** L. **Türler:** wolf, cat, ocelot, fox, panda, polar_bear, hoglin, zoglin, ravager, strider.

## Oku
- Java: `client/model/animal/wolf/WolfModel.java`, `client/model/animal/fox/FoxModel.java`, `client/model/animal/panda/PandaModel.java`, `client/model/animal/polarbear/PolarBearModel.java`, `client/model/monster/hoglin/HoglinModel.java`, `client/renderer/entity/WolfRenderer.java`, `client/renderer/entity/FoxRenderer.java`, `client/renderer/entity/PandaRenderer.java`, `client/renderer/entity/AbstractHoglinRenderer.java`, `client/renderer/entity/StriderRenderer.java`.
- Rust: `crates/voxel-render/src/entity/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity/textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Her aile için `setupAnim` dalı (oturma/uyku pozu `Pose` ve bayraklar verisinden).
2. Çeşit kayıtları (kurt/kedi çeşidi) T24b'nin çeşit çözümüyle: kayıt tablosu oracle'dan.
3. Boyun/kuyruk gibi durum kaynaklı pozlar için `entity_snapshot` genişlemesi.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Evcil hayvan etkileşimi (sunucu), yakalık boyama çizimi dışındaki GUI.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Bu görev büyükse kurt+kedi ve diğerleri diye bölünür. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı: yalnız kurt (bölündü). `setup_wolf` (yetişkin + bebek, oturma, silkelenme, kuyruk), `voxel-world/src/wolf.rs` (ilgi eğimi, silkelenme; olay 8/56), evcil/oturma/öfke/tasma verisi, `wolf_variants` ve boya doku rengi tabloları (oracle'dan üretilir), `crates/voxel/src/mobs/animals/wolves.rs` (çeşit dokusu + tasma katmanı). Kayıt: `docs/records/client-wolf-parity-conformance.md`.
- Oracle: `wolf-model` 22, `living-entity-animations` 7 kurt, `wolf-variants`, `variant-fields` +5.
- Sapma: ıslaklık/koyulaşma (suda olma izlenmiyor), zırh katmanı, ses/parçacık.
- Açık: T32b kedi/vaşak, T32c tilki/panda/kutup ayısı, T32d hoglin/ravager/strider.
