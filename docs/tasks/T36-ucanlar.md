# T36 — Blaze, ghast ve phantom

**Amaç:** Blaze (çubuk halkaları), ghast (dokunaçlar, yükleme dokusu) ve phantom (kanat, kuyruk, boyut ölçeği, ışıksız gözler). Yarasa, vex ve allay T36b'ye; papağan, breeze ve mutlu ghast T36c'ye ayrıldı.
**Bağımlılık:** T24. **Boy:** M. **Türler:** blaze, ghast, phantom.

## Oku
- Java: `client/model/ambient/BatModel.java`, `client/model/animal/parrot/ParrotModel.java`, `client/model/animal/allay/AllayModel.java`, `client/model/monster/vex/VexModel.java`, `client/model/monster/phantom/PhantomModel.java`, `client/model/monster/blaze/BlazeModel.java`, `client/model/monster/ghast/GhastModel.java`, `client/renderer/entity/BreezeRenderer.java`, `client/renderer/entity/layers/EyesLayer.java`.
- Rust: `crates/photon-render/src/entity/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/photon/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/photon-render/src/entity/textures.rs` (`rect`), `crates/photon-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Emissive/göz katmanı (`EyesLayer`, `RenderTypes.eyes`): yeni boru hattı ya da ışıksız çizim; tek karar bu görevde.
2. Her aile için `setupAnim` kolu (`ageInTicks`'e bağlı kanat/süzülme).
3. Mutlu ghast (`happy_ghast`) T24b'de metadata ile bağlı; çizimi burada.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Parçacıklar (blaze/ghast ateş topu T39), sesler.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Emissive katmanı ortak altyapıdır; sonraki görevler (T37) kullanır. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı: `setup_blaze`, `setup_ghast`, `setup_phantom`; `MobPlacement.hook_translate` (phantom'un kaldırması); `monsters.rs` tablosuna üç tür (blaze'de blok ışığı 15, ghast'ta yükleme dokusu, phantom'da ölçek, eğim ve gözler); veriler 16/16. Kayıt: `docs/records/client-airborne-parity-conformance.md`.
- Karar: ışıksız göz katmanı için yeni boru hattı gerekmedi; T33c'nin `mob_vertices_glow` yolu kullanıldı (`eyes_texture` tablosuna phantom eklendi).
- Bölündü: yarasa/vex/allay T36b'de; papağan/breeze/mutlu ghast T36c'de.
- Sapma: blaze ışığı ve doku yolları oracle'sız; phantom'un kutusu büyümez. Canlı gözlem yapılmadı.
