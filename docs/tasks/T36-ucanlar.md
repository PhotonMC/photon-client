# T36 — Uçanlar ve hayaletler: yarasa, papağan, allay, vex, phantom, blaze, ghast, breeze

**Amaç:** Havada duran/uçan mobların kanat/gövde animasyonları ve parlak (emissive) katmanları: yarasa (asılı/uçan), papağan (omuzda, dans), allay, vex, phantom, blaze (çubuklar), ghast (gözler/ağız), breeze.
**Bağımlılık:** T24. **Boy:** M. **Türler:** bat, parrot, allay, vex, phantom, blaze, ghast, happy_ghast, breeze.

## Oku
- Java: `client/model/ambient/BatModel.java`, `client/model/animal/parrot/ParrotModel.java`, `client/model/animal/allay/AllayModel.java`, `client/model/monster/vex/VexModel.java`, `client/model/monster/phantom/PhantomModel.java`, `client/model/monster/blaze/BlazeModel.java`, `client/model/monster/ghast/GhastModel.java`, `client/renderer/entity/BreezeRenderer.java`, `client/renderer/entity/layers/EyesLayer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Emissive/göz katmanı (`EyesLayer`, `RenderTypes.eyes`): yeni boru hattı ya da ışıksız çizim; tek karar bu görevde.
2. Her aile için `setupAnim` kolu (`ageInTicks`'e bağlı kanat/süzülme).
3. Mutlu ghast (`happy_ghast`) T24b'de metadata ile bağlı; çizimi burada.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Parçacıklar (blaze/ghast ateş topu T39), sesler.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Emissive katmanı ortak altyapıdır; sonraki görevler (T37) kullanır. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
