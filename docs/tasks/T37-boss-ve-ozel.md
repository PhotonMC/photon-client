# T37 — Boss ve özel mobların çizimi: wither, ender ejderhası, warden, shulker, creaking, balçıklar, armadillo, sniffer

**Amaç:** Tek başına özel kolları olan büyük/karmaşık mobların çizimi: wither (üç baş, zırh kabuğu), ender ejderhası (parçalı gövde, kanat), warden (`KeyframeAnimation`), shulker (kabuk açılma), creaking, slime/magma/sulfur küpü (boyut ölçeği), armadillo (top), sniffer.
**Bağımlılık:** T43, T36. **Boy:** L. **Türler:** wither, ender_dragon, warden, shulker, creaking, slime, magma_cube, sulfur_cube, armadillo, sniffer.

## Oku
- Java: `client/model/monster/wither/WitherBossModel.java`, `client/model/monster/warden/WardenModel.java`, `client/model/monster/shulker/ShulkerModel.java`, `client/model/monster/slime/SlimeModel.java`, `client/model/animal/sniffer/SnifferModel.java`, `client/renderer/entity/EnderDragonRenderer.java`, `client/renderer/entity/WitherBossRenderer.java`, `client/renderer/entity/WardenRenderer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Her tür ayrı dal; en çok kullanılandan (slime/shulker/warden) başla, ender ejderhası en sona (parçalar `EnderDragonPart` entity'leri; T26 notu: seçilebilirlik).
2. Hepsi `KeyframeAnimation` ve emissive katmanına dayanır (T43, T36).
3. Her tür için küçük görevlere böl: bu dosya yalnız sıralamadır.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Ender ejderhası savaş yapıları, boss çubuğu (var), sesler.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Büyük; uygulamaya başlarken tür başına alt görev dosyası aç. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
